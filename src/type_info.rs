use serde::{Deserialize, Serialize};

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
pub struct Playlist{
    pub playlistTitle: String,
    pub songs: Vec<Songs>,
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