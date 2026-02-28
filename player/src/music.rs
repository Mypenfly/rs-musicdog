use std::{path::PathBuf, time::Duration};

use lofty::{Accessor, AudioFile, TaggedFileExt};

//歌曲的相关数据
#[derive(Debug, Default, Clone)]
pub struct Song {
    pub song_path: PathBuf,
    pub lyc_path: Option<PathBuf>,
    pub title: String,
    pub artist: String,
    pub lyc_lines: Option<Vec<LyricLine>>,
    pub totle_time: Option<Duration>,
    pub album:String,
}
#[derive(Debug, Default, Clone)]
pub struct LyricLine {
    pub time: std::time::Duration,
    pub text: String,
}

impl Song {
    pub fn from_path(path: &PathBuf) -> Self {
        let song_path = path.clone();
        let lyc_path = None;
        let mut title = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();
        let mut artist = "Unknown".to_string();
        let album = "Local".to_string();
        let lyc_lines = None;
        let mut totle_time = None;

        if let Ok(tf) = lofty::read_from_path(path) {
            let (new_title, new_artist) = tf
                .primary_tag()
                .map(|tag| {
                    let t = tag
                        .title()
                        .map(|t| t.to_string())
                        .unwrap_or_else(|| title.clone());
                    let a = tag
                        .artist()
                        .map(|a| a.to_string())
                        .unwrap_or_else(|| artist.clone());
                    (t, a)
                })
                .unwrap_or_else(|| (title, artist));

            title = new_title;
            artist = new_artist;

            totle_time = Some(Duration::from_secs(tf.properties().duration().as_secs()));
        }

        Self {
            song_path,
            lyc_path,
            title,
            artist,
            lyc_lines,
            totle_time,
            album
        }
    }
    pub fn parse_lyc(&mut self) {
        let mut lines = Vec::new();
        let re = regex::Regex::new(r"\[(\d{2}):(\d{2})\.(\d{2,3})\](.*)").unwrap();
        if let Some(content) = self
            .lyc_path
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
        {
            for cap in re.captures_iter(&content) {
                let min: u64 = cap[1].parse().unwrap_or(0);
                let sec: u64 = cap[2].parse().unwrap_or(0);
                let ms_str = &cap[3];
                let ms = if ms_str.len() == 2 {
                    ms_str.parse().unwrap_or(0) * 10
                } else {
                    ms_str.parse().unwrap_or(0)
                };
                let total_millis = min * 60 * 1000 + sec * 1000 + ms;
                let time = std::time::Duration::from_millis(total_millis);
                let text = cap[4].trim().to_string();
                if !text.is_empty() {
                    lines.push(LyricLine { time, text });
                }
            }
        }

        self.lyc_lines = Some(lines);
    }
}
