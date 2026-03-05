use futures_util::StreamExt;
use reqwest::{Client, Response, Url};
use std::{fs, path::{Path, PathBuf}};
use tempfile::TempDir;

///下载错误类型
#[derive(Debug)]
pub enum DownloadError {
    HttpError(reqwest::Error),
    IoError(std::io::Error),
    UrlPaserError(url::ParseError),
    InvalidResponse(String),
}

pub type DownloadResult<T> = Result<T, DownloadError>;

///临时文件持有，做url下载和临时文件存储的作用
pub struct Tempf {
    //持有目录
    pub temp_dir: TempDir,

    //持有路径
    pub song_path: PathBuf,
    pub lyr_path:PathBuf,

    //代理
    client: Client,
}

impl Tempf {
    pub fn new() -> DownloadResult<Self> {
        let cached_dir = std::env::temp_dir().join("musicdog");
        std::fs::create_dir_all(&cached_dir).ok();

        let temp_dir = tempfile::Builder::new()
            .prefix("musicdog_")
            .tempdir_in(&cached_dir)
            .map_err(DownloadError::IoError)?;

        let song_path = PathBuf::new();
        let lyr_path =PathBuf::new() ;

        let client = Client::new();

        Ok(Self {
            temp_dir,
            song_path,
            lyr_path,
            client,
        })
    }

    pub async fn fetch_source(&mut self, url: String, name: &str) -> DownloadResult<()> {
        //解析Url
        let url = Url::parse(&url).map_err(DownloadError::UrlPaserError)?;

        //文件建立
        let ext = get_ext(&url);

        let file_name = format!("{}.{}", name, ext);
        let file_path = self.temp_dir.path().join(file_name);

        //请求
        let reponse = self
            .client
            .get(url.clone())
            .send()
            .await
            .map_err(DownloadError::HttpError)?;

        if !reponse.status().is_success() {
            return Err(DownloadError::InvalidResponse(format!(
                "HTTP {}: {}",
                reponse.status(),
                reponse.status().canonical_reason().unwrap_or("unknow")
            )));
        }
        self.song_path = file_path.clone();
        
        //文件写入
        write_into(&file_path, reponse).await;
        Ok(())

    }

    pub fn get_lyc_file(&mut self,lyric:String,name: &str) ->DownloadResult<()> {
        let file_name = format!("{}.lrc",name);
        let file_path = self.temp_dir.path().join(file_name);
        self.lyr_path = file_path.clone();

        fs::write(file_path, lyric).map_err(DownloadError::IoError)
        
    }
}

fn get_ext(url: &Url) -> String {
    let path = url.path();

    if path.ends_with(".mp3") {
        "mp3".to_string()
    } else if path.ends_with(".wav") {
        "wav".to_string()
    } else if path.ends_with(".flac") {
        "flac".to_string()
    } else if path.ends_with(".ogg") {
        "ogg".to_string()
    } else if path.ends_with(".m4a") {
        "m4a".to_string()
    } else if path.ends_with(".txt") {
        "txt".to_string()
    } else if path.ends_with(".lrc") {
        "lrc".to_string()
    } else {
        "mp3".to_string()
    }
}

///写入
async fn write_into(file_path: &Path, response: Response) -> DownloadResult<()> {
    let mut file = tokio::fs::File::create(file_path)
        .await
        .map_err(DownloadError::IoError)?;

    let mut stream = response.bytes_stream();
    //stream writtting
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(DownloadError::HttpError)?;
        tokio::io::copy(&mut chunk.as_ref(), &mut file)
            .await
            .map_err(DownloadError::IoError)?;
    }
    Ok(())
}
