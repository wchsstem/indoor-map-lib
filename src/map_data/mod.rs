use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::Hash;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub mod compiled;
pub mod uncompiled;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum VertexTag {
    #[serde(rename = "stairs")]
    Stairs,
    #[serde(rename = "elevator")]
    Elevator,
    #[serde(rename = "up")]
    Up,
    #[serde(rename = "down")]
    Down,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum RoomTag {
    #[serde(rename = "closed")]
    Closed,
    #[serde(rename = "women-bathroom")]
    WomenBathroom,
    #[serde(rename = "men-bathroom")]
    MenBathroom,
    #[serde(rename = "staff-women-bathroom")]
    StaffWomenBathroom,
    #[serde(rename = "staff-men-bathroom")]
    StaffMenBathroom,
    #[serde(rename = "unknown-bathroom")]
    UnknownBathroom,
    #[serde(rename = "bsc")]
    Bsc,
    #[serde(rename = "ec")]
    Ec,
    #[serde(rename = "wf")]
    Wf,
    #[serde(rename = "hs")]
    Hs,
    #[serde(rename = "bleed-control")]
    BleedControl,
    #[serde(rename = "aed")]
    Aed,
    #[serde(rename = "ahu")]
    Ahu,
    #[serde(rename = "idf")]
    Idf,
    #[serde(rename = "mdf")]
    Mdf,
    #[serde(rename = "eru")]
    Eru,
    #[serde(rename = "cp")]
    Cp,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Floor {
    number: String,
    image: PathBuf,
    offsets: (f32, f32),
}

impl Floor {
    pub fn get_number(&self) -> &str {
        &self.number
    }

    pub fn get_image(&self) -> &PathBuf {
        &self.image
    }

    pub fn get_offsets(&self) -> (f32, f32) {
        self.offsets
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Vertex {
    floor: String,
    location: (f32, f32),
    #[serde(default)]
    #[serde(skip_serializing_if = "HashSet::is_empty")]
    tags: HashSet<VertexTag>,
}

impl Vertex {
    pub fn get_floor(&self) -> &str {
        &self.floor
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct EdgeJson(Vec<Value>);

impl From<Edge> for EdgeJson {
    fn from(edge: Edge) -> Self {
        if edge.directed {
            Self(vec![
                Value::String(edge.from),
                Value::String(edge.to),
                Value::Bool(true),
            ])
        } else {
            Self(vec![Value::String(edge.from), Value::String(edge.to)])
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Clone)]
#[serde(try_from = "EdgeJson")]
#[serde(into = "EdgeJson")]
pub struct Edge {
    from: String,
    to: String,
    directed: bool,
}

impl TryFrom<EdgeJson> for Edge {
    type Error = &'static str;

    fn try_from(value: EdgeJson) -> Result<Self, Self::Error> {
        let mut values = value.0.into_iter();

        let from = values
            .next()
            .ok_or("Missing from")?
            .as_str()
            .ok_or("From not a string")?
            .to_owned();
        let to = values
            .next()
            .ok_or("Missing to")?
            .as_str()
            .ok_or("To not a string")?
            .to_owned();
        let directed = match values.next() {
            Some(value) => value.as_bool().ok_or("Directed not a bool")?,
            None => false,
        };

        if values.len() != 0 {
            return Err("Array too long");
        }

        Ok(Self { from, to, directed })
    }
}

#[cfg(test)]
mod test {
    use common_macros::{hash_map, hash_set};

    use super::*;
    use crate::map_data::uncompiled::{MapDataDeserializeError, MapDataError};

    fn file(path: &str) -> String {
        use std::fs;
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn construct_simple_data() {
        let json = file("tests/json/simple.json");
        let map_data = uncompiled::MapData::new(&json).unwrap();
        let actual_map_data = uncompiled::MapData {
            floors: vec![Floor {
                number: "1".to_string(),
                image: "assets/map/1st_floor.svg".into(),
                offsets: (0.0, 0.0),
            }],
            vertices: hash_map![
                "a".to_string() => Vertex {
                    floor: "1".to_string(),
                    location: (434.875, 288.0),
                    tags: hash_set![VertexTag::Stairs],
                },
                "b".to_string() => Vertex {
                    floor: "1".to_string(),
                    location: (0.0, 0.0),
                    tags: hash_set![],
                },
                "c".to_string() => Vertex {
                    floor: "1".to_string(),
                    location: (0.0, 1.0),
                    tags: hash_set![],
                },
            ],
            edges: vec![
                Edge {
                    from: "c".to_string(),
                    to: "b".to_string(),
                    directed: false,
                },
                Edge {
                    from: "a".to_string(),
                    to: "b".to_string(),
                    directed: true,
                },
            ],
            rooms: hash_map! {
                "106".to_string() => uncompiled::Room {
                    vertices: hash_set!["a".to_string()],
                    center: None,
                    names: vec![],
                    tags: hash_set![],
                },
                "107".to_string() => uncompiled::Room {
                    vertices: hash_set!["b".to_string(), "c".to_string()],
                    center: Some((489.9375, 36.9375)),
                    names: vec![
                        "guidance".to_string(),
                        "guidance office".to_string(),
                        "counselors".to_string(),
                        "counseling office".to_string(),
                    ],
                    tags: hash_set![],
                },
            },
        };
        assert_eq!(actual_map_data, map_data);
    }

    #[test]
    fn reject_repeat_floor_number() {
        let json = file("tests/json/repeat_floor_number.json");
        let map_data = uncompiled::MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataDeserializeError::MapDataError(MapDataError::RepeatedFloorNumber(
                    number,
                )) => assert_eq!("1", &number),
                _ => panic!("Should be repeated floor number 1, was {:?}", error),
            },
            Ok(_) => panic!("Should be error"),
        }
    }

    #[test]
    fn reject_undefined_floor_number_vertex() {
        let json = file("tests/json/undefined_floor_number_vertex.json");
        let map_data = uncompiled::MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataDeserializeError::MapDataError(MapDataError::UndefinedFloorNumber(
                    floor_number,
                )) => {
                    assert_eq!("2".to_owned(), floor_number);
                }
                _ => panic!("Should be undefined floor numbers"),
            },
            Ok(_) => panic!("Should be error"),
        }
    }

    #[test]
    fn reject_undefined_vertex_id_room() {
        let json = file("tests/json/undefined_vertex_id_room.json");
        let map_data = uncompiled::MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataDeserializeError::MapDataError(MapDataError::UndefinedVertexId(
                    vertex_id,
                )) => {
                    assert_eq!("a".to_owned(), vertex_id);
                }
                _ => panic!("Should be undefined vertex id, was {:?}", error),
            },
            Ok(_) => panic!("Should be error"),
        }
    }

    #[test]
    fn reject_undefined_vertex_id_edge() {
        let json = file("tests/json/undefined_vertex_id_edge.json");
        let map_data = uncompiled::MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataDeserializeError::MapDataError(MapDataError::UndefinedVertexId(
                    vertex_id,
                )) => {
                    assert_eq!("b".to_owned(), vertex_id);
                }
                _ => panic!("Should be undefined vertex id"),
            },
            Ok(_) => panic!("Should be error"),
        }
    }
}
