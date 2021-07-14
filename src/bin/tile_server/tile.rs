use indoor_map_lib::svg_parser::SvgElement;
use nalgebra::Vector2;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TileCoords {
    location: Vector2<i32>,
    zoom: i32,
}

#[derive(Debug)]
pub struct Tile {
    image: SvgElement,
}
