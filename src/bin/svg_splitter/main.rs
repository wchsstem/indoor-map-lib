use crate::layer::Layer;
use crate::tile::TileCoords;
use crate::tile_iterator::TileIterator;
use indoor_map_lib::bounding_box::BoundingSquare;
use nalgebra::Vector2;
use std::error::Error;
use std::fs;

mod layer;
mod tile;
mod tile_iterator;

fn main() -> Result<(), Box<dyn Error>> {
    let svg_data = fs::read_to_string("test.svg")?;
    let layer_bounds = BoundingSquare::new(Vector2::new(0., 0.), 609.68834);
    let layer = Layer::new(&svg_data, layer_bounds)?;

    for coords in TileIterator::new(0, 3) {
        let tile = layer.tile(&coords);
        svg::save(format!("out/{:?}.svg", coords), &tile.as_element())?;
    }

    Ok(())
}
