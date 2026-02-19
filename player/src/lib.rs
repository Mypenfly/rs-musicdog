use std::path::PathBuf;

use walkdir::WalkDir;

pub mod core;
pub mod music;
pub use music::Song;

//扫描本地文件
pub fn scan(dir: &std::path::PathBuf) -> Vec<Song> {
    let mut songs = Vec::new();

    let song_exts = ["mp3", "flac", "wav", "ogg", "m4a"];
    let lyc_exts = ["lrc", "txt"];

    let song_files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| song_exts.contains(&ext))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
        .collect();
    let mut lyc_files = std::collections::HashMap::new();
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| lyc_exts.contains(&ext))
                .unwrap_or(false)
        })
        .map(|e| e.into_path())
    {
        if let Some(stem) = entry.file_stem().and_then(|s| s.to_str()) {
            let key = stem.to_lowercase();
            lyc_files.entry(key).or_insert(entry);
        }
    }
    if song_files.is_empty() {
        println!("未检索出任何文件！！！")
    }

    for song_path in song_files {
        let mut song = Song::from_path(&song_path);
        if let Some(stem) = song_path.file_stem().and_then(|s| s.to_str()) {
            let key = stem.to_lowercase();
            if let Some(lyric_path) = lyc_files.get(&key) {
                song.lyc_path = Some(lyric_path.clone());
                song.parse_lyc();
            }
        }
        songs.push(song);
    }
    songs
}
