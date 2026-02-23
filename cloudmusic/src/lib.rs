use ncmapi::NcmApi;
use status::Page;
use status::messege::{self, Messenge};

#[derive(Default)]
pub struct CloudCore {
    pub api: NcmApi,
    pub messenge: messege::Messenge,
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

        Self { api, messenge }
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

    pub async fn recommand(&self) -> (&str, Vec<u8>) {
        let recommmand_songs = match self.api.recommend_songs().await {
            Ok(apiresponse) => apiresponse,
            _ => {
                self.messenge.warning("获取推荐歌曲失败！！！");
                return ("", Vec::new());
            }
        };
        let songs = recommmand_songs.data().clone();
        ("推荐歌曲", songs)
    }

    pub async fn back_items(&self, page: &Page) -> Option<(&str, Vec<String>)> {
        let (sub_title, items_u8) = match page {
            Page::EveryDaySingle => self.recommand().await,
            _ => return None,
        };
        let items: Vec<String> = match std::str::from_utf8(&items_u8) {
            Ok(s) => s.lines().map(|s| s.to_string()).collect(),
            Err(_) => {
                self.messenge.warning("列表解析失败");
                return None;
            }
        };

        Some((sub_title, items))
    }
}

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
                        let id = song.get("id").and_then(|s| s.as_u64()).unwrap_or(0);

                        println!("name:{}\nid:{}", name, id);

                        if let Some(artist_ar) = song.get("ar").unwrap().as_array() {
                            // println!("aritist:{:#?}",artist_ar);
                            for artists in artist_ar {
                                if let Some(artist) = artists.as_object() {
                                    let ar_name = artist
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unkown");
                                    let ar_id =
                                        artist.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                                    println!("artist:{}({})", ar_name, ar_id);
                                }
                            }
                        }
                        let (al_name, al_id) = song
                            .get("al")
                            .and_then(|v| v.as_object())
                            .map(|obj| {
                                let name =
                                    obj.get("name").and_then(|v| v.as_str()).unwrap_or("unkown");
                                let id = obj.get("id").and_then(|v| v.as_u64()).unwrap_or(0);
                                (name, id)
                            })
                            .unwrap_or(("unkown", 0));
                        println!("album:{}({})", al_name, al_id);
                    }
                }
            }
        }
        // println!("{:#?}",songs);
    }
}
