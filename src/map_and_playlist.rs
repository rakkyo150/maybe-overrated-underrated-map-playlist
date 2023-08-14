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
    pub bpm: f64,
    pub duration: f64,
    songName: String,
    songSubName: String,
    songAuthorName: String,
    levelAuthorName: String,
    plays: u32,
    dailyPlays: u32,
    downloads: u32,
    upvotes: u32,
    downvotes: u32,
    upvotesRatio: f64,
    uploatedAt: String,
    createdAt: String,
    updatedAt: String,
    lastPublishedAt: String,
    automapper: String,
    qualified: String,
    loved: String,
    pub difficulty: String,
    // sageScoreはempty stringの場合があるので
    pub sageScore: String,
    pub njs: f64,
    pub offset: f64,
    pub notes: u32,
    pub bombs: u32,
    pub obstacles: u32,
    pub nps: f64,
    length: f64,
    pub characteristic: String,
    pub events: f64,
    pub chroma: String,
    me: String,
    ne: String,
    cinema: String,
    seconds: f64,
    pub errors: u64,
    pub warns: u64,
    pub resets: u64,
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
    pub overrated_playlist: [PlaylistSet; 15],
    pub underrated_playlist: [PlaylistSet; 15],
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

    pub fn search_playlist_set(&mut self, rank: &f64) -> Result<(&mut PlaylistSet, &mut PlaylistSet), String> {
        let rank_i32 = rank.floor() as i32;
        if 0 <= rank_i32 && rank_i32 < 15 {
            Ok((&mut self.overrated_playlist[*rank as usize], &mut self.underrated_playlist[*rank as usize]))
        } else {
            Err(String::from("No rank number"))
        }
    }
}

fn make_playlist_array(playlist_base_name: &str) -> [PlaylistSet; 15] {
    // https://tyfkda.github.io/blog/2020/03/19/rust-init-array.html
    // PlaylistにCopyトレイトが実装されていないので[要素; 要素数]が使えない
    const LEN: usize = 15;
    // 未初期化の領域を確保
    let mut unsafe_playlist: [MaybeUninit<PlaylistSet>; LEN] = unsafe { MaybeUninit::uninit().assume_init() };
    for (i, slot) in unsafe_playlist.iter_mut().enumerate() {
        // 初期化する
        *slot = MaybeUninit::new(PlaylistSet::new(format!("{} {}★", playlist_base_name, i)));
    }
    let playlists = unsafe{ std::mem::transmute::<_, [PlaylistSet; LEN]>(unsafe_playlist) };
    playlists
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct PlaylistSet{
    pub a_little_version: Playlist,
    pub fairly_version: Playlist,
    pub very_version: Playlist
}

impl PlaylistSet{
    pub fn new(playlist_base_title: String) -> PlaylistSet{
        let a_little_version = Playlist { playlistTitle: format!("A Little {}", playlist_base_title), songs: vec![]};
        let fairly_version = Playlist { playlistTitle: format!("Fairly {}", playlist_base_title), songs: vec![]};
        let very_version = Playlist { playlistTitle: format!("Very {}", playlist_base_title), songs: vec![]};

        PlaylistSet { a_little_version, fairly_version, very_version }
    }
}

#[derive(Debug, Serialize)]
#[allow(non_snake_case)]
pub struct Playlist{
    pub playlistTitle: String,
    pub songs: Vec<Songs>,
}

impl Playlist{
    pub fn search_songs(&mut self, song_name: &str, hash: &str) -> Option<&mut Songs>{
        for value in &mut self.songs {
            if value.songName == song_name.to_string() && value.hash == hash.to_string(){
                return Some(value);
            }
        }
        None
    }

    pub fn sort(&mut self){
        if self.songs.len() == 0 { return };
        if self.songs[0].difficulties[0].diff > 0.0 {
            for song in &mut self.songs{
                song.difficulties.sort_by(|a, b| b.diff.total_cmp(&a.diff))
            }

            self.songs.sort_by(|a,b| b.difficulties[0].diff.total_cmp(&a.difficulties[0].diff))
        }
        else{
            for song in &mut self.songs{
                song.difficulties.sort_by(|a, b| a.diff.total_cmp(&b.diff))
            }

            self.songs.sort_by(|a,b| a.difficulties[0].diff.total_cmp(&b.difficulties[0].diff))
        }
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