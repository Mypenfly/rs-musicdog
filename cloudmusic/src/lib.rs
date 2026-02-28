use std::collections::HashMap;

use ncmapi::NcmApi;
use player::Song;
use serde_json::Value;
use status::messege::{self, Messenge};
mod cloudmusic;

#[derive(Default)]
pub struct CloudCore {
    pub api: NcmApi,
    pub messenge: messege::Messenge,
    pub music: cloudmusic::CloudSong,
}

#[derive(Debug, Default)]
pub enum User {
    #[default]
    Login,
    Logout,
}

impl CloudCore {
    pub fn new() -> Self {
        let api = NcmApi::default();
        let messenge = Messenge::default();
        let music = cloudmusic::CloudSong::new();

        Self {
            api,
            messenge,
            music,
        }
    }
    pub async fn login(&self, phone: &str, password: &str) -> Result<(), ()> {
        match self.api.login_phone(phone, password).await {
            Ok(_) => {
                self.messenge.info("登陆成功！！！");
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
                        self.music.song.insert(song_name.to_string(), song_id);
                        self.music.artists = artists.clone();
                        self.music.album.insert(album_name.to_string(), album_id);
                        self.music.pic.insert(album_name.to_string(), album_pic);

                        let song = turn_song(song_name, artists, album_name);
                        list.push(song);
                    }
                }
            }
        }
        list
    }

    // pub async fn back_items(&self, page: &Page) -> Option<(&str, Vec<String>)> {
    //     let (sub_title, items_u8) = match page {
    //         Page::EveryDaySingle => self.recommand().await,
    //         _ => return None,
    //     };
    //     let items: Vec<String> = match std::str::from_utf8(&items_u8) {
    //         Ok(s) => s.lines().map(|s| s.to_string()).collect(),
    //         Err(_) => {
    //             self.messenge.warning("列表解析失败");
    //             return None;
    //         }
    //     };

    //     Some((sub_title, items))
    // }
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

// fn turn_song(song_name:&str,artists:Vec<HashMap<String,u64>>,album_name:&str) -> Song {
//     let song_path = album_name.to_string();
//     let title = song_name.to_string();
//     let mut artist = String::new();
//     for map in artists.iter(){
//         for key in map.keys(){
//             artist.push_str(key);
//         }
//     }
//     Song { song_path, lyc_path:None, title, artist, lyc_lines: None, totle_time: None }
// }

#[cfg(test)]
mod test {

    use ncmapi::NcmApi;

    #[tokio::test]
    async fn recommand_test() {
        let api = NcmApi::default();
        let recommmand_songs = match api.recommend_songs().await {
            Ok(apiresponse) => apiresponse,
            _ => {
                return;
            }
        };
        let resp = recommmand_songs.deserialize_to_implict();
        println!("test:{:#?}", resp.data.as_object().unwrap());
        if let Some(data_arry) = resp.data.as_object() {
            for (class, value) in data_arry {
                if class != "dailySongs" {
                    continue;
                }
                if let Some(songs) = value.as_array() {
                    for song in songs {
                        let name = song
                            .get("name")
                            .and_then(|s| s.as_str())
                            .unwrap_or("unkown");
                        let id = song.get("id").and_then(|s| s.as_u64()).unwrap_or(0) as usize;
                        let list = vec![id];
                        let url_res = api.song_url(&list).await.ok();
                        let response = url_res.unwrap().deserialize_to_implict();
                        if let Some(url_data) = response.data.as_array() {
                            for urls in url_data {
                                if let Some(value) = urls.as_object() {
                                    let url = value.get("url").and_then(|v| v.as_str());
                                    println!("url:{:#?}", url);
                                }
                            }
                        }

                        println!("\nname:{}\nid:{}", name, id);

                        let artists: Vec<(&str, u64)> = song
                            .get("ar")
                            .and_then(|v| v.as_array())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|arrist| {
                                        let name = arrist.get("name")?.as_str()?;
                                        let id = arrist.get("id")?.as_u64()?;
                                        Some((name, id))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        println!("artist{:?}", artists);

                        let (al_name, al_id, pic) = song
                            .get("al")
                            .and_then(|v| v.as_object())
                            .map(|obj| {
                                let name =
                                    obj.get("name").and_then(|v| v.as_str()).unwrap_or("unkown");
                                let id = obj.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                                let pic = obj.get("picUrl").and_then(|v| v.as_str());
                                (name, id, pic)
                            })
                            .unwrap_or(("unkown", 0, None));
                        println!("album:{}({})\npic:{:?}", al_name, al_id, pic);
                    }
                }
            }
        }
        // println!("{:#?}",songs);
    }
}
