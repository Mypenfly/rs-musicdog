use crate::music::Song;
use rodio::{self, Sink};
use tokio;

#[derive(Debug, Clone)]
pub enum PlayerCommand {
    Play(Song),
    Pause,
    Resume,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PlayerState {
    #[default]
    Playing,
    Stopped,
    Paused,
    Ended,
}

pub struct PlayerCore {
    rx: tokio::sync::mpsc::Receiver<PlayerCommand>,
    tx: std::sync::mpsc::Sender<PlayerCommand>,
}

impl PlayerCore {
    pub async fn new(
        rx: tokio::sync::mpsc::Receiver<PlayerCommand>,
        tx: tokio::sync::mpsc::Sender<PlayerState>,
    ) -> Self {
        let (audio_tx, audio_rx) = std::sync::mpsc::channel();

        std::thread::spawn(move || {
            audio(audio_rx, tx);
        });
        Self { rx, tx: audio_tx }
    }

    pub async fn run_player(mut self) {
        while let Some(cmd) = self.rx.recv().await {
            let audio_cmd = cmd.clone();
            let _ = self.tx.send(audio_cmd);
        }
    }
}

pub fn audio(
    rx: std::sync::mpsc::Receiver<PlayerCommand>,
    tx: tokio::sync::mpsc::Sender<PlayerState>,
) {
    let (_stream, stream_handle) = rodio::OutputStream::try_default().expect("无法初始化音频");
    let mut sink: Option<Sink> = None;
    let mut state = PlayerState::Stopped;

    loop {
        if let Ok(cmd) = rx.recv() {
            match cmd {
                PlayerCommand::Play(song) => {
                    if let Some(s) = sink.take() {
                        s.stop();
                    }
                    match std::fs::File::open(&song.song_path) {
                        Ok(file) => match rodio::Decoder::new(std::io::BufReader::new(file)) {
                            Ok(decoder) => {
                                let new_sink = Sink::try_new(&stream_handle).unwrap();
                                new_sink.append(decoder);
                                sink = Some(new_sink);
                                state = PlayerState::Playing;
                                let _ = tx.blocking_send(state);
                            }
                            Err(e) => eprintln!("解码失败:{}", e),
                        },
                        Err(e) => eprintln!("文件打开失败：{}", e),
                    }
                }
                PlayerCommand::Pause => {
                    if let Some(s) = &sink {
                        s.pause();
                        state = PlayerState::Paused;
                        let _ = tx.blocking_send(state);
                    }
                }
                PlayerCommand::Resume => {
                    if let Some(s) = &sink {
                        s.play();
                        state = PlayerState::Playing;
                        let _ = tx.blocking_send(state);
                    }
                }
                PlayerCommand::Stop => {
                    if let Some(s) = &sink {
                        s.stop();
                        state = PlayerState::Stopped;
                        let _ = tx.blocking_send(state);
                    }
                }
            }
        }

        if let Some(s) = &sink
            && s.empty()
            && state == PlayerState::Playing
        {
            sink = None;
            state = PlayerState::Stopped;
            let _ = tx.blocking_send(state);
        };

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
