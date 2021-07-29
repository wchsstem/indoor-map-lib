use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::fs;

use serde::Deserialize;

use crate::map_data::{compiled, Edge, Floor, RoomTag, Vertex};
use crate::svg_room::SvgRoom;
use crate::util::{centroid, shoelace_area, undefined, unique};
use std::path::Path;

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

#[derive(Deserialize, Debug, PartialEq)]
pub struct MapData {
    pub floors: Vec<Floor>,
    pub vertices: HashMap<String, Vertex>,
    pub edges: Vec<Edge>,
    pub rooms: HashMap<String, Room>,
}

impl MapData {
    fn verify(self) -> Result<Self, MapDataError> {
        // Get floor numbers and check that all are unique
        let floor_numbers = unique(self.floors.iter().map(|f| &f.number))
            .map_err(|floor_number| MapDataError::RepeatedFloorNumber(floor_number.to_owned()))?;

        // Check that there are no undefined floor numbers
        undefined(
            self.vertices.iter().map(|(_id, v)| &v.floor),
            &floor_numbers,
        )
        .map_err(|floor_number: &String| {
            MapDataError::UndefinedFloorNumber(floor_number.clone())
        })?;

        // Check that there are no undefined vertices in the rooms
        let room_vertex_ids = self.rooms.values().map(|r| &r.vertices).flatten();
        undefined(room_vertex_ids, &self.vertices.keys().collect())
            .map_err(|vertex_id| MapDataError::UndefinedVertexId(vertex_id.clone()))?;

        // Check that there are no undefined vertices in the edges
        let edge_vertex_ids = self.edges.iter().map(|e| vec![&e.from, &e.to]).flatten();
        undefined(edge_vertex_ids, &self.vertices.keys().collect())
            .map_err(|vertex_id| MapDataError::UndefinedVertexId(vertex_id.clone()))?;

        Ok(self)
    }

    pub fn new(json_data: &str) -> Result<Self, MapDataDeserializeError> {
        Ok(serde_json::from_str::<Self>(json_data)?.verify()?)
    }

    fn get_floor_images(&self, base_path: &Path) -> Vec<(String, (f32, f32))> {
        self.floors
            .iter()
            .map(|floor| (floor.get_image(), floor.get_offsets()))
            .map(|(image_rel_path, o)| (base_path.join(image_rel_path), o))
            .map(|(image_path, o)| {
                (
                    fs::read_to_string(image_path).expect("Image file doesn't exist"),
                    o,
                )
            })
            .collect()
    }

    fn svg_rooms(image_content: &str) -> impl Iterator<Item = SvgRoom> + '_ {
        let svg_reader = svg::read(&image_content).expect("SVG must be valid");
        svg_reader.filter_map(|event| SvgRoom::try_from(event).ok())
    }

    pub fn compile(mut self, base_path: &Path) -> compiled::MapData {
        let mut compiled_rooms = HashMap::with_capacity(self.rooms.len());

        for (image_content, offsets) in self.get_floor_images(base_path) {
            for svg_room in Self::svg_rooms(&image_content) {
                let outline = svg_room.outline(offsets);
                let uncompiled_room = match self.rooms.remove(svg_room.get_number()) {
                    Some(old_room) => old_room,
                    None => {
                        println!("Room does not exist: {}", svg_room.get_number());
                        continue;
                    }
                };

                let compiled_room = uncompiled_room.compile(outline);
                compiled_rooms.insert(svg_room.get_number().to_owned(), compiled_room);
            }
        }

        compiled::MapData {
            floors: self.floors,
            vertices: self.vertices,
            rooms: compiled_rooms,
            edges: self.edges,
        }
    }
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Room {
    pub vertices: HashSet<String>,
    #[serde(default)]
    pub names: Vec<String>,
    #[serde(default)]
    pub center: Option<(f32, f32)>,
    #[serde(default)]
    pub tags: HashSet<RoomTag>,
}

impl Room {
    pub fn compile(self, outline: Vec<(f32, f32)>) -> compiled::Room {
        let center = match self.center {
            Some(center) => center,
            None => centroid(&outline),
        };
        let area = shoelace_area(&outline).abs();

        compiled::Room {
            vertices: self.vertices,
            names: self.names,
            center,
            outline,
            area,
            tags: self.tags,
        }
    }
}
