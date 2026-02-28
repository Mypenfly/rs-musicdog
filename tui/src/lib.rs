use crossterm::event::{self, Event, KeyCode};

use cloudmusic::{CloudCore, User};
use player::{
    Song,
    core::{PlayerCommand, PlayerCore, PlayerState},
    scan,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
};
use status::{Input, Page, Player, Status, messege};
use std::{io, path::PathBuf};
use tokio::select;
mod ui;

#[derive(Debug, Default)]
pub struct LocalSongs {
    pub list: Vec<String>,
    pub input_path: String,
    pub songs: Vec<Song>,
}
#[derive(Default)]
pub struct Info {
    phone: String,
    passward: String,
    localpath: String,
    state: Input,
}

#[derive(Default)]
pub struct App {
    pub state: Status,
    pub page: Page,
    pub page_stack: Vec<Page>,
    pub selected: usize,
    pub input: String,
    pub keyword_lock: bool,
    pub player: Player,
    pub messenge: messege::Messenge,
    pub user: User,
    info: Info,
}

impl App {
    pub fn new() -> Self {
        //状态机启动
        let state = Status::Running;
        let page = Page::Main;
        let page_stack = Vec::new();

        let selected = 0;
        let input = String::new();
        let keyword_lock = false;

        let player = Player::new();
        let messenge = messege::Messenge::default();
        let user = User::Logout;
        let info = Info {
            phone: String::new(),
            passward: String::new(),
            localpath: String::new(),
            state: Input::LocalFind,
        };

        Self {
            state,
            page,
            page_stack,
            selected,
            input,
            keyword_lock,
            player,
            messenge,
            user,
            info,
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        //播放线程准备
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::channel::<PlayerCommand>(32);
        let (std_tx, mut std_rx) = tokio::sync::mpsc::channel::<PlayerState>(32);
        tokio::spawn(async move {
            let player_core = PlayerCore::new(cmd_rx, std_tx).await;
            player_core.run_player().await
        });

        //监听按键
        let (key_tx, mut key_rx) = tokio::sync::mpsc::channel::<KeyCode>(32);
        tokio::spawn(async move {
            loop {
                let event = tokio::task::spawn_blocking(event::read).await;
                match event {
                    Ok(Ok(Event::Key(key))) => {
                        if key_tx.send(key.code).await.is_err() {
                            break;
                        }
                    }
                    _ => break,
                }
            }
        });

        //网易云核心启动
        let mut cloudcore = CloudCore::new();

        while self.state != Status::Stopped {
            let list = self.player.list.items.clone();
            let title_list = list.iter().map(|s| s.title.to_string()).collect();
            let (subtitle, items) = match self.page {
                Page::Main => (
                    "首页",
                    ["0.本地音乐播放", "1.网易云", "2.退出"]
                        .iter()
                        .map(|s| s.to_string())
                        .collect(),
                ),
                Page::LocalPlay => ("本地播放", title_list),
                Page::Input => ("输入", Vec::new()),
                Page::Playing => ("播放中ing", Vec::new()),
                Page::PlayList => ("", Vec::new()),
                Page::CloudMusic => (
                    "网易云☁️",
                    [
                        "0.每日推荐单曲",
                        "1.每日推荐歌单",
                        "2.排行榜",
                        "3.精选歌单",
                        "4.我的",
                        "5.搜索",
                        "6.设置",
                    ]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                ),
                Page::Exit => (
                    "退出",
                    ["0.返回", "1.退出"].iter().map(|s| s.to_string()).collect(),
                ),
                Page::EveryDaySingle => ("每日推荐",title_list),
                _=>("",Vec::new())
            };
            select! {
                Some(key_code) = key_rx.recv() => {
                    self.handle_key(key_code, &items, &cmd_tx,&mut cloudcore).await;
                }
                Some(state) = std_rx.recv() => {
                    self.player.state = state;
                    match state {
                        PlayerState::Stopped => {
                            self.player.list.current_index = (self.player.list.current_index + 1 ) % self.player.list.items.len();
                            let next_song = self.player.follow_index();
                            self.player.play(next_song.clone());
                            self.messenge.song_changed(&next_song);
                            let _ = cmd_tx.send(PlayerCommand::Play(next_song)).await;

                        },
                        PlayerState::Ended => {
                            self.player.stop();
                            self.player.state = PlayerState::Ended;
                        }
                        _ => (),
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(16)) =>{
                    terminal.draw(|frame| self.draw(frame, &items, subtitle))?;
                }
            }
            terminal.draw(|frame| self.draw(frame, &items, subtitle))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, items_list: &[String], subtitle: &str) {
        let items: Vec<&str> = items_list.iter().map(|s| s.as_str()).collect();
        match self.state {
            Status::Playing => {
                let chunks = Layout::vertical([Constraint::Min(0), Constraint::Length(3)])
                    .split(frame.area());
                match self.page {
                    Page::Input => {
                        self.draw_input(frame, chunks[0]);
                    }
                    Page::Playing => ui::render_playing(frame, chunks[0], &self.player),
                    Page::PlayList => {
                        ui::render_playlist(frame, chunks[0], self.selected, &self.player.list);
                    }
                    _ => ui::render_page(frame, chunks[0], &items, self.selected, subtitle),
                };
                ui::render_bar(frame, chunks[1], &self.player);
            }
            _ => match self.page {
                Page::Input => {
                    self.draw_input(frame, frame.area());
                }
                Page::Playing => ui::render_playing(frame, frame.area(), &self.player),
                Page::PlayList => {
                    ui::render_playlist(frame, frame.area(), self.selected, &self.player.list);
                }
                _ => ui::render_page(frame, frame.area(), &items, self.selected, subtitle),
            },
        }
    }

    fn draw_input(&mut self, frame: &mut Frame, area: Rect) {
        match self.info.state {
            Input::LocalFind => ui::render_input(frame, area, self.input.as_str(), "输入地址"),
            Input::Phone => ui::render_input(frame, area, self.input.as_str(), "登陆：电话号码"),
            Input::Passward => ui::render_input(frame, area, self.input.as_str(), "登陆：输入密码"),
        }
    }
    async fn operate(
        &mut self,
        player_tx: &tokio::sync::mpsc::Sender<PlayerCommand>,
        cloudcore: &mut CloudCore
    ) -> Option<()> {
        match self.page {
            Page::Main => {
                if self.selected == 0 {
                    self.selected = 0;
                    if self.player.locallist.items.is_empty() {
                        self.page = Page::Input;
                        self.info.state = Input::LocalFind;
                        self.keyword_lock = true;
                    } else {
                        self.page = Page::LocalPlay;
                    }
                    Some(())
                } else if self.selected == 1 {
                    self.selected = 0;
                    match self.user {
                        User::Login => self.page = Page::CloudMusic,
                        User::Logout=> {
                            self.keyword_lock = true;
                            self.page = Page::Input;
                            self.info.state = Input::Phone;
                        }
                    }
                    Some(())
                } else {
                    self.selected = 0;
                    self.page = Page::Exit;
                    Some(())
                }
            }
            Page::LocalPlay => {
                //此处selected不归0,是为了能指定出是第几首歌
                self.jump_playing(player_tx).await;
                Some(())
            }
            Page::Input => {
                self.page = Page::LocalPlay;
                self.keyword_lock = false;
                Some(())
            }
            Page::Playing => {
                self.selected = 0;
                Some(())
            }
            Page::PlayList => {
                if self.selected == self.player.list.current_index {
                    self.page = Page::Playing;
                    self.state = Status::Playing;
                    Some(())
                } else {
                    self.player.list.current_index = self.selected;
                    let song = self.player.follow_index();
                    self.selected = 0;

                    self.page = Page::Playing;
                    self.state = Status::Playing;
                    let _ = player_tx.send(PlayerCommand::Play(song.clone())).await;
                    self.messenge.song_changed(&song);
                    self.player.play(song);
                    Some(())
                }
            }
            Page::CloudMusic => {
                self.player.cloudlist.items=cloudcore.recommand_songs().await;
                self.player.list.items = self.player.cloudlist.items.clone();
                self.page = Page::EveryDaySingle;
                Some(())
            },
            Page::Exit => {
                if self.selected == 0 {
                    self.selected = 0;
                    self.page = Page::Main;
                    Some(())
                } else {
                    self.selected = 0;
                    None
                }
            }
            _ => {
                self.jump_playing(player_tx).await;
                Some(())
            },
        }
    }

    async fn handle_key(
        &mut self,
        key: KeyCode,
        items: &[String],
        player_tx: &tokio::sync::mpsc::Sender<PlayerCommand>,
        cloudcore: &mut CloudCore,
    ) {
        match key {
            KeyCode::Down => match self.page {
                Page::Playing => (),
                Page::PlayList => {
                    let list_len = self.player.list.items.len();
                    if list_len > 0 {
                        self.selected = (self.selected + 1) % list_len;
                    }
                }
                _ => self.selected = (self.selected + 1) % items.len(),
            },
            KeyCode::Up => match self.page {
                Page::Playing => (),
                Page::PlayList => {
                    let list_len = self.player.list.items.len();
                    if list_len > 0 {
                        self.selected = self.selected.wrapping_sub(1) % list_len;
                    }
                }
                _ => self.selected = self.selected.wrapping_sub(1) % items.len(),
            },
            KeyCode::Enter => match self.page {
                Page::Input => self.handle_input(cloudcore).await,
                _ => {
                    self.push_stack();
                    match self.operate(player_tx,cloudcore).await {
                        Some(_) => (),
                        None => self.state = Status::Stopped,
                    }
                }
            },
            KeyCode::Char('q') => {
                if !self.keyword_lock {
                    self.state = Status::Stopped;
                } else {
                    self.input.push('q');
                }
            }
            KeyCode::Char(c) => {
                if self.keyword_lock {
                    self.page = Page::Input;
                    self.input.push(c);
                }
            }
            KeyCode::Backspace => {
                if self.keyword_lock {
                    self.page = Page::Input;
                    self.input.pop();
                }
            }
            KeyCode::Tab => {
                self.page_stack.push(self.page);
                self.page = Page::PlayList;
            }
            KeyCode::Esc => {
                self.keyword_lock = false;
                if let Some(back_page) = self.page_stack.last() {
                    self.page = *back_page
                } else {
                    self.page = Page::Main
                }
                self.page_stack.pop();
            }
            _ => {}
        }

        //播放状态下的操作
        if let Status::Playing = self.state {
            match key {
                KeyCode::Char(' ') => match self.player.state {
                    PlayerState::Playing => {
                        let _ = player_tx.send(PlayerCommand::Pause).await;
                        self.player.pause();
                    }
                    PlayerState::Paused => {
                        let _ = player_tx.send(PlayerCommand::Resume).await;
                        self.player.resume();
                    }
                    _ => {
                        if let Some(song) = &self.player.curent_song {
                            let _ = player_tx.send(PlayerCommand::Play(song.clone())).await;
                            self.player.play(song.clone());
                        }
                    }
                },
                KeyCode::Char('s') => {
                    let _ = player_tx.send(PlayerCommand::Stop).await;
                    self.player.stop();
                    self.state = Status::Running;
                }
                KeyCode::Char('b') => {
                    self.page_stack.push(self.page);
                    self.page = Page::Playing;
                }
                _ => (),
            }
        }
    }
    fn push_stack(&mut self) {
        if self.page != Page::PlayList && self.page != Page::Input {
            self.page_stack.push(self.page);
        }
    }

    async fn handle_input(&mut self, cloudcore: &CloudCore) {
        self.keyword_lock = true;
        match self.info.state {
            Input::Phone => {
                self.info.phone = self.input.clone();
                self.info.state = Input::Passward;
                self.page = Page::Input;
                self.input = String::new();
            }
            Input::Passward => {
                self.info.passward = self.input.clone();
                match cloudcore.login(&self.info.phone, &self.info.passward).await {
                    Ok(_) => {
                        self.page = Page::CloudMusic;
                        self.user = User::Login;
                        self.keyword_lock = false;
                    }
                    Err(_) => {
                        self.page = Page::Input;
                        self.info.state = Input::Phone;
                    }
                };
                self.input = String::new();
            }
            Input::LocalFind => {
                self.info.localpath = self.input.clone();
                self.player.locallist.items = scan(&PathBuf::from(&self.info.localpath));
                self.page = Page::LocalPlay;
                self.player.switch_local();
                self.input = String::new();
                self.keyword_lock = false;
            }
        }
    }
    async fn jump_playing(&mut self,player_tx: &tokio::sync::mpsc::Sender<PlayerCommand>){
        
        self.player.list.current_index = self.selected;
        let selected_song = self.player.follow_index();
        self.selected = 0;

        self.page = Page::Playing;
        let _ = player_tx
            .send(PlayerCommand::Play(selected_song.clone()))
            .await;
        self.state = Status::Playing;
        self.messenge.song_changed(&selected_song);
        self.player.play(selected_song);
    }
}
