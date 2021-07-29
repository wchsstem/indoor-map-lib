use std::collections::{HashMap, HashSet};

use crate::map_data::{Edge, Floor, RoomTag, Vertex};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MapData {
    pub floors: Vec<Floor>,
    pub vertices: HashMap<String, Vertex>,
    pub edges: Vec<Edge>,
    pub rooms: HashMap<String, Room>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Room {
    pub vertices: HashSet<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub names: Vec<String>,
    pub center: (f32, f32),
    pub outline: Vec<(f32, f32)>,
    pub area: f32,
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    pub tags: HashSet<RoomTag>,
}
