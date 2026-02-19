use notify_rust::{Notification, Timeout};
use player::Song;
use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Messenge {
    pub level: MessengeLevel,
    pub text: String,
    pub time: Instant,
    pub enable: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum MessengeLevel {
    #[default]
    Info,
    Warning,
    Error,
}

impl Default for Messenge {
    fn default() -> Self {
        Self {
            level: MessengeLevel::Info,
            text: String::new(),
            time: Instant::now(),
            enable: true,
        }
    }
}
impl Messenge {
    pub fn disable(&mut self) {
        self.enable = false;
    }

    pub fn push(&self, messenge: String, title: &str) {
        if !self.enable {
            return;
        }

        match Notification::new()
            .summary(title)
            .body(messenge.as_str())
            .icon("./messenge_icon.png")
            .timeout(Timeout::Milliseconds(2))
            .show()
        {
            Ok(_) => (),
            Err(e) => eprintln!("notification failed:{}", e),
        }
    }

    pub fn song_changed(&mut self, song: &Song) {
        self.level = MessengeLevel::Info;

        let title = "Info - Now Playing";
        let messenge = format!("{} - {}", song.title, song.artist);
        self.push(messenge, title);
    }
}
