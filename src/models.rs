use serde::Serialize;

pub struct Count {
    pub count: i64
}

#[derive(Debug, Serialize)]
pub struct PlayerStats {
    pub position: Option<f64>,
    pub soldiername: Option<String>,
    pub score: Option<String>,
    pub global_rank: u16,
    pub kills: u32,
    pub deaths: u32,
    pub teamkills: u32,
    pub suicides: u32,
    pub kdr: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServerRankTemplate {
    pub base_path: String,
    pub players: Vec<PlayerStats>
}
