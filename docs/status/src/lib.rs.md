#文件：lib.rs
路径：status/src/lib.rs
代码如下:

```rust
use player::Song;
use player::core::PlayerState;
use std::time::{Duration, Instant};
pub mod messege;

//应用状态
#[derive(Debug, PartialEq, Default)]
pub enum Status {
    #[default]
    Running,
    Playing,
    Stopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Page {
    #[default]
    Main,
    LocalPlay,
    Input,
    Playing,
    PlayList,
    CloudMusic,
    Exit,
}

//播放列表
#[derive(Debug, Default)]
pub struct PlayerList {
    pub items: Vec<Song>,
    pub current_index: usize,
}
impl PlayerList {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current_index: 0,
        }
    }
}

//播放状态
#[derive(Debug, Default)]
pub struct Player {
    pub state: PlayerState,
    pub curent_song: Option<Song>,
    pub start_time: Option<Instant>,
    pub pause_time: Option<Instant>,
    pub pause_duration: Duration,
    pub list: PlayerList,
}
impl Player {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Playing,
            curent_song: None,
            start_time: None,
            pause_time: None,
            pause_duration: Duration::ZERO,
            list: PlayerList::new(),
        }
    }
    pub fn play(&mut self, song: Song) {
        self.curent_song = Some(song);
        self.start_time = Some(Instant::now());
        self.pause_time = None;
        self.pause_duration = Duration::ZERO;
        self.state = PlayerState::Playing;
    }
    pub fn pause(&mut self) {
        if self.pause_time.is_none() && self.state == PlayerState::Playing {
            self.pause_time = Some(Instant::now());
            self.state = PlayerState::Paused;
        }
    }
    pub fn resume(&mut self) {
        if let Some(t) = self.pause_time.take() {
            self.pause_duration += t.elapsed();
            self.state = PlayerState::Playing;
        }
    }
    pub fn stop(&mut self) {
        self.state = PlayerState::Stopped;
        self.start_time = None;
        self.pause_time = None;
        self.pause_duration = Duration::ZERO;
    }
    pub fn elapsed(&self) -> Duration {
        let start = match self.start_time.as_ref() {
            Some(s) => s.elapsed(),
            None => Duration::ZERO,
        };
        let base = start.saturating_sub(self.pause_duration);

        match self.pause_time.as_ref() {
            Some(paused_at) => base.saturating_sub(paused_at.elapsed()),
            None => base,
        }
    }

    pub fn total_duration(&self) -> Duration {
        self.curent_song
            .as_ref()
            .and_then(|s| s.totle_time)
            .unwrap_or(Duration::ZERO)
    }
    pub fn follow_index(&mut self) -> Song {
        let song = self.list.items[self.list.current_index].clone();
        self.curent_song = Some(song.clone());
        song
    }
}
```