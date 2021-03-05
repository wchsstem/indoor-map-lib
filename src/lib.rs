use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::hash::Hash;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct MapData {
    floors: Vec<Floor>,
    vertices: HashMap<String, Vertex>,
    edges: Vec<Edge>,
    rooms: HashMap<String, Room>,
}

impl MapData {
    fn unique<'a, T: Eq + Hash>(
        items: impl Iterator<Item = &'a T>,
    ) -> Result<HashSet<&'a T>, &'a T> {
        items.fold(Ok(HashSet::new()), |acc, element| match acc {
            Ok(mut acc) => {
                if !acc.contains(&element) {
                    acc.insert(element);
                    Ok(acc)
                } else {
                    Err(element)
                }
            }
            Err(_) => acc,
        })
    }

    fn undefined<'a, T: Eq + Hash>(
        mut items: impl Iterator<Item = &'a T>,
        defined: &HashSet<&T>,
    ) -> Result<(), &'a T> {
        if let Some(undefined_item) = items.find(|item| !defined.contains(item)) {
            Err(undefined_item)
        } else {
            Ok(())
        }
    }

    pub fn new(json_data: &str) -> Result<Self, MapDataDeserializeError> {
        Ok(serde_json::from_str::<Self>(json_data)?.verify()?)
    }

    pub fn room_mut(&mut self, number: &str) -> Option<&mut Room> {
        self.rooms.get_mut(number)
    }

    fn verify(self) -> Result<Self, MapDataError> {
        // Get floor numbers and check that all are unique
        let floor_numbers = Self::unique(self.floors.iter().map(|f| &f.number))
            .map_err(|repeat| MapDataError::RepeatedFloorNumber(repeat.to_owned()))?;

        // Check that there are no undefined floor numbers
        Self::undefined(
            self.vertices.iter().map(|(_id, v)| &v.floor),
            &floor_numbers,
        )
        .map_err(|floor_number| MapDataError::UndefinedFloorNumber(floor_number.clone()))?;

        // Check that there are no undefined vertices in the rooms
        let room_vertex_ids = self.rooms.values().map(|r| &r.vertices).flatten();
        Self::undefined(room_vertex_ids, &self.vertices.keys().collect())
            .map_err(|vertex_id| MapDataError::UndefinedVertexId(vertex_id.clone()))?;

        // Check that there are no undefined vertices in the edges
        let edge_vertex_ids = self.edges.iter().map(|e| vec![&e.from, &e.to]).flatten();
        Self::undefined(edge_vertex_ids, &self.vertices.keys().collect())
            .map_err(|vertex_id| MapDataError::UndefinedVertexId(vertex_id.clone()))?;

        Ok(self)
    }

    pub fn get_floors(&self) -> &Vec<Floor> {
        &self.floors
    }
}

#[derive(thiserror::Error, Debug)]
pub enum MapDataDeserializeError {
    #[error("JSON error while deserializing: {0}")]
    InvalidJson(#[from] serde_json::Error),
    #[error(transparent)]
    MapDataError(#[from] MapDataError),
}

#[derive(thiserror::Error, Debug)]
pub enum MapDataError {
    #[error("The floor number `{0}` was repeated")]
    RepeatedFloorNumber(String),
    #[error("The vertex ID `{0}` was repeated")]
    RepeatedVertexId(String),
    #[error("The floor number `{0}` is undefined")]
    UndefinedFloorNumber(String),
    #[error("The vertex ID `{0}` is undefined")]
    UndefinedVertexId(String),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Floor {
    number: String,
    image: PathBuf,
    offsets: (f32, f32),
}

impl Floor {
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
    tags: HashSet<VertexTag>,
}

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq, Hash)]
pub enum VertexTag {
    #[serde(rename = "stairs")]
    Stairs,
    #[serde(rename = "elevator")]
    Elevator,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Room {
    vertices: HashSet<String>,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    center: Option<(f32, f32)>,
    #[serde(default)]
    outline: Vec<(f32, f32)>,
}

impl Room {
    pub fn set_outline(&mut self, outline: Vec<(f32, f32)>) {
        self.outline = outline;
    }

    pub fn center(&self) -> &Option<(f32, f32)> {
        &self.center
    }

    pub fn set_center(&mut self, center: Option<(f32, f32)>) {
        self.center = center;
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
struct Edge {
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

    fn file(path: &str) -> String {
        use std::fs;
        fs::read_to_string(path).unwrap()
    }

    #[test]
    fn construct_simple_data() {
        let json = file("tests/json/simple.json");
        let map_data = MapData::new(&json).unwrap();
        let actual_map_data = MapData {
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
                "106".to_string() => Room {
                    vertices: hash_set!["a".to_string()],
                    center: None,
                    names: hash_set![],
                },
                "107".to_string() => Room {
                    vertices: hash_set!["b".to_string(), "c".to_string()],
                    center: Some((489.9375, 36.9375)),
                    names: hash_set![
                        "guidance".to_string(),
                        "guidance office".to_string(),
                        "counselors".to_string(),
                        "counseling office".to_string(),
                    ],
                },
            },
        };
        assert_eq!(actual_map_data, map_data);
    }

    #[test]
    fn reject_repeat_floor_number() {
        let json = file("tests/json/repeat_floor_number.json");
        let map_data = MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataError::RepeatedFloorNumber(number) => assert_eq!("1", &number),
                _ => panic!("Should be repeated floor number 1"),
            },
            Ok(_) => panic!("Should be error"),
        }
    }

    #[test]
    fn reject_repeat_vertex_id() {
        let json = file("tests/json/repeat_vertex_id.json");
        let map_data = MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataError::RepeatedVertexId(id) => assert_eq!("a", &id),
                _ => panic!("Should be repeated vertex id"),
            },
            Ok(_) => panic!("Should be error"),
        }
    }

    #[test]
    fn reject_undefined_floor_number_vertex() {
        let json = file("tests/json/undefined_floor_number_vertex.json");
        let map_data = MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataError::UndefinedFloorNumber(floor_number) => {
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
        let map_data = MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataError::UndefinedVertexId(vertex_id) => {
                    assert_eq!("a".to_owned(), vertex_id);
                }
                _ => panic!("Should be undefined vertex id"),
            },
            Ok(_) => panic!("Should be error"),
        }
    }

    #[test]
    fn reject_undefined_vertex_id_edge() {
        let json = file("tests/json/undefined_vertex_id_edge.json");
        let map_data = MapData::new(&json);
        match map_data {
            Err(error) => match error {
                MapDataError::UndefinedVertexId(vertex_id) => {
                    assert_eq!("b".to_owned(), vertex_id);
                }
                _ => panic!("Should be undefined vertex id"),
            },
            Ok(_) => panic!("Should be error"),
        }
    }
}
