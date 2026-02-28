use std::{path::PathBuf, time::Duration};

use player::Song;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph, Wrap},
};
use status::{Player, PlayerList};

const LOGO: &str = include_str!("./assets/logo.txt");
const COVER: &str = include_str!("./assets/default.txt");

pub fn render_page(frame: &mut Frame, area: Rect, items: &[&str], selected: usize, subtitle: &str) {
    //tui名称
    let top_title = Paragraph::new(LOGO)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    //副标题名称
    let sub_title = Paragraph::new(subtitle)
        .style(Style::default().fg(Color::LightGreen))
        .block(Block::default().padding(Padding::horizontal(2)));

    //创建一个高亮列表
    let list_items: Vec<ListItem> = items.iter().map(|item| ListItem::new(*item)).collect();
    let list = List::new(list_items)
        .block(Block::default().padding(Padding::vertical(2)))
        .highlight_style(Style::default().fg(Color::Red))
        .highlight_symbol("> ");

    //# 绘制tui
    //## 布局
    let chunks = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .split(area);

    frame.render_widget(top_title, chunks[0]);
    frame.render_widget(sub_title, chunks[1]);
    frame.render_stateful_widget(
        list,
        chunks[2],
        &mut ratatui::widgets::ListState::default().with_selected(Some(selected)),
    );
}

pub fn render_playing(frame: &mut Frame, area: Rect, player: &Player) {
    let song = match player.curent_song.clone() {
        Some(s) => s,
        None => Song {
        song_path: PathBuf::new(),
            lyc_path: None,
            title: String::new(),
            artist: String::new(),
            lyc_lines: None,
            totle_time: None,
            album:"None".to_string()
        },
    };
    //tui名称
    let top_title = Paragraph::new(LOGO)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    //分区
    let main_chunks = Layout::vertical([Constraint::Length(8), Constraint::Min(0)]).split(area);

    let music_chunks = Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)])
        .split(main_chunks[1]);

    let info_chunks = Layout::vertical([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(music_chunks[0]);
    //填入

    let cover_default = Paragraph::new(COVER)
        .block(Block::default().padding(Padding::horizontal(2)))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::LightBlue));

    let info_text = format!(
        "歌曲：{}\n艺人：{}\n路径：{}",
        song.title,
        song.artist,
        song.album,
    );
    let info_para = Paragraph::new(info_text)
        .block(Block::default().padding(Padding::horizontal(2)))
        .style(Style::default().fg(Color::Blue))
        .wrap(Wrap { trim: true });

    //渲染
    frame.render_widget(top_title, main_chunks[0]);
    frame.render_widget(cover_default, info_chunks[0]);
    frame.render_widget(info_para, info_chunks[1]);
    render_lyrics(frame, music_chunks[1], player);
}
pub fn render_lyrics(frame: &mut Frame, area: Rect, player: &Player) {
    let elapsed = player.elapsed();
    let song = match player.curent_song.clone() {
        Some(s) => s,
        None => Song {
            song_path: PathBuf,
            lyc_path: None,
            title: String::new(),
            artist: String::new(),
            lyc_lines: None,
            totle_time: None,
            album:"None".to_string()
        },
    };
    let index = song.lyc_lines.as_ref().and_then(|lyrics| {
        lyrics
            .iter()
            .enumerate()
            .rfind(|(_, line)| line.time <= elapsed)
            .map(|(i, _)| i)
    });
    let remaining_style = Style::default().fg(Color::DarkGray);
    let current_line_style = Style::default().fg(Color::Red).add_modifier(Modifier::BOLD);

    let (lyrics, idx) = match (song.lyc_lines.as_ref(), index) {
        (Some(lyrics), Some(idx)) => (lyrics, idx),
        _ => {
            let para = Paragraph::new(Text::raw("暂无歌词")).alignment(Alignment::Center);
            frame.render_widget(para, area);
            return;
        }
    };

    //为双语歌词高亮做准备
    let mut group_start = idx;
    let mut group_end = idx;
    while group_start > 0 && lyrics[group_start - 1].time == lyrics[idx].time {
        group_start -= 1;
    }
    while group_end + 1 < lyrics.len() && lyrics[group_end + 1].time == lyrics[idx].time {
        group_end += 1;
    }

    let mut lines = Vec::new();

    let context_lines = 15;
    let display_start = group_start.saturating_sub(context_lines);
    let display_end = (group_end + 1 + context_lines).min(lyrics.len());

    for _ in 0..context_lines.saturating_sub(group_start) {
        lines.push(Line::default());
    }

    for i in display_start..display_end {
        let line = &lyrics[i];

        if i >= group_start && i <= group_end {
            if i != 0 {
                for _ in 0..2 {
                    lines.push(Line::default());
                }
            }
            lines.push(Line::from(Span::styled(
                line.text.clone(),
                current_line_style,
            )));
            if i != lyrics.len() {
                for _ in 0..2 {
                    lines.push(Line::default());
                }
            }
        } else {
            lines.push(Line::from(Span::styled(line.text.clone(), remaining_style)));
        }
    }
    for _ in 0..10 {
        lines.push(Line::default());
    }

    let para = Paragraph::new(Text::from(lines))
        .alignment(Alignment::Center)
        .block(Block::default().padding(Padding::horizontal(2)));

    frame.render_widget(para, area);
}

pub fn render_input(frame: &mut Frame, area: Rect, input: &str, title: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Length(3), Constraint::Min(1)].as_ref())
        .split(area);
    let input_para = Paragraph::new(input)
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title_top(title));
    frame.render_widget(input_para, chunks[0]);
}

pub fn render_bar(frame: &mut Frame, area: Rect, player: &Player) {
    let chunks = Layout::horizontal([Constraint::Min(20), Constraint::Fill(1)]).split(area);

    let (icon, state_color) = match player.state {
        player::core::PlayerState::Playing => ("||", Color::Green),
        player::core::PlayerState::Stopped => ("▶", Color::Gray),
        player::core::PlayerState::Ended => ("▶️", Color::Gray),
        player::core::PlayerState::Paused => ("..z.Z", Color::Yellow),
    };
    let text = if let Some(song) = &player.curent_song {
        format!("{} {} - {}", icon, song.title, song.artist)
    } else {
        format!("{} nothing", icon)
    };
    let left = Paragraph::new(text)
        .style(Style::default().fg(state_color))
        .scroll((0, 0));

    let mut elapsed = player.elapsed();
    let total = player.total_duration();
    if elapsed > total {
        elapsed = total;
    }

    let fmt_time = |d: Duration| -> String {
        let secs = d.as_secs();
        format!("{:02}:{:02}", secs / 60, secs % 60)
    };

    let time = if total.as_secs() > 0 {
        format!("{} / {}", fmt_time(elapsed), fmt_time(total))
    } else {
        "0:00 / 0:00".to_string()
    };

    let progress_width = chunks[1].width.saturating_sub(time.len() as u16 + 2);
    let process_percent = if total.as_secs() > 0 {
        elapsed.as_secs_f32() / total.as_secs_f32()
    } else {
        0.0
    };
    let filled_chars = (process_percent * progress_width as f32).round() as usize;
    let bar: String = "█".repeat(filled_chars)
        + &"░".repeat((progress_width as usize).saturating_sub(filled_chars));
    let right_text = format!("{} {}", bar, time);
    let right = Paragraph::new(right_text)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Right);

    frame.render_widget(left, chunks[0]);
    frame.render_widget(right, chunks[1]);
}

pub fn render_playlist(frame: &mut Frame, area: Rect, selected: usize, queue: &PlayerList) {
    let popup_width = (area.width as f32 * 0.7) as u16;
    let popup_height = (queue.items.len() as u16 + 4).min(12);

    let popup_area = Rect::new(
        area.x + (area.width - popup_width) / 2,
        area.y + (area.height - popup_height) / 2,
        popup_width,
        popup_height,
    );

    let block = Block::default()
        .title_top("正在播放的列表")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let items: Vec<ListItem> = queue
        .items
        .iter()
        .enumerate()
        .map(|(i, song)| {
            let line = if i == queue.current_index {
                Line::from(vec![
                    Span::styled("▶", Style::default().fg(Color::LightYellow)),
                    Span::raw(format!("{} - {}", song.title, song.artist)),
                ])
            } else {
                Line::from(format!("{} - {}", song.title, song.artist))
            };
            ListItem::new(line).style(if i == selected {
                Style::default().bg(Color::Red)
            } else {
                Style::default()
            })
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_symbol(">  ")
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    frame.render_stateful_widget(
        list,
        popup_area,
        &mut ListState::default().with_selected(Some(selected)),
    );
}
