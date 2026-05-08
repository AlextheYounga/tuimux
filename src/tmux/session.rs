use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Window {
    pub index: String,
    pub name: String,
    pub layout: String,
    pub active_pane_path: String,
    pub activity: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Session {
    pub name: String,
    pub work_dir: String,
    pub windows: Vec<Window>,
    pub activity: u64,
}
