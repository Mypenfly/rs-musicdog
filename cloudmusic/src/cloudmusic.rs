use std::collections::HashMap;

#[derive(Default)]
pub struct CloudSong{
   pub  song:HashMap<String,u64>,
    pub artists:Vec<HashMap<String,u64>>,
    pub album:HashMap<String,u64>,
    pub pic:HashMap<String,Option<String>>
} 

impl CloudSong {
    pub fn new()->Self {
        let song = HashMap::new();
        let artists = Vec::new();
        let album = HashMap::new();
        let pic = HashMap::new();
        Self { song, artists, album, pic }
    }
    pub fn song_id(&self,index:&str) -> u64{
        if let Some(id) = self.song.get(index)  {
            *id
        }else {
            0
        }
    }
}
