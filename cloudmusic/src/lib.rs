use std::{collections::HashMap,  path::PathBuf};

use ncmapi::NcmApi;
use player::Song;
use serde_json::Value;
use status::messege::{self, Messenge};

use crate::tempfiles::DownloadError;
mod cloudmusic;
mod tempfiles;

#[derive(Debug)]
pub enum CloudCoreErr{
    InitError(DownloadError)
}

pub type CloudCoreResult<T> = Result<T,CloudCoreErr>;

pub struct CloudCore {
    pub api: NcmApi,
    pub messenge: messege::Messenge,
    pub music: cloudmusic::CloudSong,
    pub temp:tempfiles::Tempf
}

#[derive(Debug, Default)]
pub enum User {
    #[default]
    Login,
    Logout,
}

impl CloudCore {
    pub fn new() -> CloudCoreResult<Self> {
        let api = NcmApi::default();
        let messenge = Messenge::default();
        let music = cloudmusic::CloudSong::new();
        let temp = tempfiles::Tempf::new()
            .map_err(CloudCoreErr::InitError)?;

        Ok(Self { api, messenge, music, temp })

    }
    pub async fn login(&self, phone: &str, password: &str) -> Result<(), ()> {
        match self.api.login_phone(phone, password).await {
            Ok(_) => {
                self.messenge.info("登陆成功！！！");
                let _ =self.api.login_refresh().await;
                Ok(())
            }
            _ => {
                self.messenge.warning("登陆失败了！！！");
                Err(())
            }
        }
    }

    pub async fn recommand_songs(&mut self) -> Vec<Song> {
        let recommmand_songs = match self.api.recommend_songs().await {
            Ok(apiresponse) => apiresponse,
            _ => {
                return Vec::new();
            }
        };

        let mut list = Vec::new();
        let resp = recommmand_songs.deserialize_to_implict();
        if let Some(data_arry) = resp.data.as_object() {
            for (class, value) in data_arry {
                if class != "dailySongs" {
                    continue;
                }
                if let Some(datas) = value.as_array() {
                    for data in datas {
                        let (song_name, song_id) = get_song(data);
                        let artists = get_artist(data);
                        let (album_name, album_id, album_pic) = get_album(data);

                        //存储数据到map中，方便后面检索
                        self.music.song.insert(song_name.to_string(), song_id);
                        self.music.artists = artists.clone();
                        self.music.album.insert(album_name.to_string(), album_id);
                        self.music.pic.insert(album_name.to_string(), album_pic);

                        let song =back_item(song_name, artists, album_name) ;
                        list.push(song);
                    }
                }
            }
        }
        list
    }

    ///得到音频文件的url
    async fn get_url(&self,id:u64) -> Option<String> {
    
        let mut url:Option<String> = None;
        
        let list = vec![id as usize];
        let url_res = self.api.song_url(&list).await.ok();
        let response = url_res.unwrap().deserialize_to_implict();
        if let Some(url_data) = response.data.as_array() {
            for urls in url_data {
                if let Some(value) = urls.as_object() {
                    url = value.get("url").map(|v|v.to_string());
                }
            }
        }
        url
    }

    ///转化成SONG格式
   pub async fn turn_song (&mut self,current_song:&Song,song_id:u64) -> Song{
        let url = match self.get_url(song_id).await {
            Some(url) => url.trim_matches('"').to_string(),
            None => {self.messenge.warning("获取歌曲url失败！！");return Song::default();},
        };

        let name = &current_song.title.replace(" ", "_");
        
        // println!("URL:{:?}",&url);
        //获取资源
        let _ =self.temp.fetch_source(url, name).await;

        //解析歌词
        let raw = self.api.lyric(song_id as usize).await.unwrap();

        let v: serde_json::Value = serde_json::from_slice(raw.data()).unwrap();
        let lyric = v["lrc"]["lyric"].as_str().unwrap().to_string();
        let _ = self.temp.get_lyc_file(lyric, name);

        //转化成Song格式
        let mut final_song = Song::from_path(&self.temp.song_path);
        final_song.lyc_path = Some(self.temp.lyr_path.clone());
        final_song.parse_lyc();

        //重新填入数据（避免解析错误）
        final_song.title = current_song.title.clone();
        final_song.artist = current_song.artist.clone();
        final_song.album = current_song.album.clone();
        
        final_song
    }
}
fn get_song(data: &Value) -> (&str, u64) {
    let name = data
        .get("name")
        .and_then(|s| s.as_str())
        .unwrap_or("unkown");
    let id = data.get("id").and_then(|s| s.as_u64()).unwrap_or(0);
    (name, id)
}

fn get_artist(data: &Value) -> Vec<HashMap<String, u64>> {
    let artists: Vec<HashMap<String, u64>> = data
        .get("ar")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|arrist| {
                    let name = arrist.get("name")?.as_str()?;
                    let id = arrist.get("id")?.as_u64()?;
                    let mut map = HashMap::new();
                    map.insert(name.to_string(), id);
                    Some(map)
                })
                .collect()
        })
        .unwrap_or_default();
    artists
}

fn get_album(data: &Value) -> (&str, u64, Option<String>) {
    let (al_name, al_id, pic) = data
        .get("al")
        .and_then(|v| v.as_object())
        .map(|obj| {
            let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("unkown");
            let id = obj.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
            let pic = obj.get("picUrl").map(|v| v.to_string());
            (name, id, pic)
        })
        .unwrap_or(("unkown", 0, None));
    (al_name, al_id, pic)
}


fn back_item(song_name:&str,artists:Vec<HashMap<String,u64>>,album_name:&str) -> Song {
    let song_path =PathBuf::new() ;
    let album = album_name.to_string();
    let title = song_name.to_string();
    let mut artist = String::new();
    for map in artists.iter(){
        for key in map.keys(){
            artist.push_str(key);
        }
    }
    Song { song_path, lyc_path: None, title, artist, lyc_lines: None, totle_time: None, album }
}

#[cfg(test)]
mod test {


    use ncmapi::NcmApi;

    use crate::CloudCore;

    #[tokio::test]
    async fn test(){
        let cloudcore = CloudCore::new().unwrap();

        let phone = "18942185404";
        let password = "lpf20061205155X";

        let response = cloudcore.api
            .login_phone(phone, password)
            .await.unwrap();

        println!("login: {:#?}",&response);
        
        let status = cloudcore.api.login_status().await.unwrap();
        println!("status:{:#?}",status);
        // let list = cloudcore.recommand_songs().await;
        // // println!("\nList:{:?}\n",list);

        // let song = list[5].clone();
        // // println!("\nSong:{:#?}\n",song);

        // let id = cloudcore.music.song_id(&song.title);
        // // println!("\nid:{}\n",id);

        // let now_song = cloudcore.turn_song(&song, id).await ;
        // println!("\nnow_song:{:#?}",now_song);

        // let raw = cloudcore.api.lyric(id as usize).await.unwrap();
        // println!("raw:{:#?}",raw);

        // let v: serde_json::Value = serde_json::from_slice(raw.data()).unwrap();
        // let lyric_text = v["lrc"]["lyric"].as_str().unwrap().to_string();

        // println!("lyric:{}",lyric_text);
        recommand_test(cloudcore.api).await;
    }

//     #[tokio::test]
    async fn recommand_test(api:NcmApi) {
        // let api = NcmApi::default();
        let recommmand_songs = match api.recommend_resource().await {
            Ok(apiresponse) => apiresponse,
            _ => {
                return;
            }
        };

        println!("albums:{:#?}",&recommmand_songs);
        let resp = recommmand_songs.deserialize_to_implict();
        println!("test:{:#?}", resp.data);
//         if let Some(data_arry) = resp.data.as_object() {
//             for (class, value) in data_arry {
//                 if class != "dailySongs" {
//                     continue;
//                 }
//                 if let Some(songs) = value.as_array() {
//                     for song in songs {
//                         let name = song
//                             .get("name")
//                             .and_then(|s| s.as_str())
//                             .unwrap_or("unkown");
//                         let id = song.get("id").and_then(|s| s.as_u64()).unwrap_or(0) as usize;

                        
//                         let list = vec![id];
//                         let url_res = api.song_url(&list).await.ok();
//                         let response = url_res.unwrap().deserialize_to_implict();
//                         if let Some(url_data) = response.data.as_array() {
//                             for urls in url_data {
//                                 if let Some(value) = urls.as_object() {
//                                     let url = value.get("url").and_then(|v| v.as_str());
//                                     println!("url:{:#?}", url);
//                                 }
//                             }
//                         }

//                         println!("\nname:{}\nid:{}", name, id);

//                         let artists: Vec<(&str, u64)> = song
//                             .get("ar")
//                             .and_then(|v| v.as_array())
//                             .map(|arr| {
//                                 arr.iter()
//                                     .filter_map(|arrist| {
//                                         let name = arrist.get("name")?.as_str()?;
//                                         let id = arrist.get("id")?.as_u64()?;
//                                         Some((name, id))
//                                     })
//                                     .collect()
//                             })
//                             .unwrap_or_default();
//                         println!("artist{:?}", artists);

//                         let (al_name, al_id, pic) = song
//                             .get("al")
//                             .and_then(|v| v.as_object())
//                             .map(|obj| {
//                                 let name =
//                                     obj.get("name").and_then(|v| v.as_str()).unwrap_or("unkown");
//                                 let id = obj.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
//                                 let pic = obj.get("picUrl").and_then(|v| v.as_str());
//                                 (name, id, pic)
//                             })
//                             .unwrap_or(("unkown", 0, None));
//                         println!("album:{}({})\npic:{:?}", al_name, al_id, pic);
//                     }
//                 }
//             }
//         }
//         // println!("{:#?}",songs);
    }
}
