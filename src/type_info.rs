use serde::{Deserialize, Serialize};
use std::mem::MaybeUninit;

#[derive(Debug, Deserialize)]
#[allow(dead_code, non_snake_case)]
pub struct MapData{
    pub index: u32,
    id: String,
    leaderboardId: u32,
    pub hash: String,
    pub name: String,
    description: String,
    uploaderId: u32,
    uploaderName: String,
    uploaderHash: String,
    uploaderAvatar: String,
    uploaderLoginType: String,
    uploaderCurator: String,
    uploaderVerifiedMapper: String,
    bpm: f32,
    duration: f32,
    songName: String,
    songSubName: String,
    songAuthorName: String,
    levelAuthorName: String,
    plays: u32,
    dailyPlays: u32,
    downloads: u32,
    upvotes: u32,
    downvotes: u32,
    upvotesRatio: f32,
    uploatedAt: String,
    createdAt: String,
    updatedAt: String,
    lastPublishedAt: String,
    automapper: String,
    qualified: String,
    loved: String,
    pub difficulty: String,
    sageScore: String,
    njs: f32,
    offset: f32,
    notes: u32,
    bombs: u32,
    obstacles: u32,
    nps: f32,
    length: f32,
    pub characteristic: String,
    events: f32,
    chroma: String,
    me: String,
    ne: String,
    cinema: String,
    seconds: f32,
    errors: u32,
    warns: u32,
    resets: u32,
    positiveModifiers: String,
    pub stars: f64,
    maxScore: u32,
    downloadUrl: String,
    coverUrl: String,
    previewUrl: String,
    tag: String
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct Playlists{
    pub overrated_playlist: [Playlist; 15],
    pub underrated_playlist: [Playlist; 15],
}

impl Playlists {
    pub fn new() -> Playlists{
        let overrated_playlist = make_playlist_array("Overrated Playlist");
        let underrated_playlist = make_playlist_array("Underrated Playlist");

        let playlists = Playlists {
            overrated_playlist,
            underrated_playlist,
        };

        playlists
    }

    pub fn search_playlist(&mut self, rank: &f64) -> Result<(&mut Playlist, &mut Playlist), String> {
        let rank_i32 = rank.floor() as i32;
        if 0 <= rank_i32 && rank_i32 < 15 {
            Ok((&mut self.overrated_playlist[*rank as usize], &mut self.underrated_playlist[*rank as usize]))
        } else {
            Err(String::from("No rank number"))
        }
    }
}

fn make_playlist_array(playlist_base_name: &str) -> [Playlist; 15] {
    // https://tyfkda.github.io/blog/2020/03/19/rust-init-array.html
    // PlaylistにCopyトレイトが実装されていないので[要素; 要素数]が使えない
    const LEN: usize = 15;
    // 未初期化の領域を確保
    let mut unsafe_overrated_playlist: [MaybeUninit<Playlist>; LEN] = unsafe { MaybeUninit::uninit().assume_init() };
    for (i, slot) in unsafe_overrated_playlist.iter_mut().enumerate() {
        // 初期化する
        *slot = MaybeUninit::new(Playlist {playlistTitle: format!("{} {}★", playlist_base_name, i), songs: vec![] });
    }
    let overrated_playlists = unsafe{ std::mem::transmute::<_, [Playlist; LEN]>(unsafe_overrated_playlist) };
    overrated_playlists
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct Playlist{
    pub playlistTitle: String,
    pub songs: Vec<Songs>,
}

impl Playlist{
    pub fn search_songs(&mut self, songName: &str, hash: &str) -> Option<&mut Songs>{
        for value in &mut self.songs {
            if value.songName == songName.to_string() && value.hash == hash.to_string(){
                return Some(value);
            }
        }
        None
    }
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct Songs{
    pub songName: String,
    pub difficulties: Vec<Difficulties>,
    pub hash: String,
}

#[derive(Debug, Serialize, Clone)]
#[allow(non_snake_case)]
pub struct Difficulties{
    pub name: String,
    pub characteristic: String,
    pub diff: f64,
}