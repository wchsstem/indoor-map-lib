use crate::tile::{Tile, TileCoords};
use indoor_map_lib::bounding_box::BoundingBox;
use indoor_map_lib::svg_parser::SvgElement;
use nalgebra::Vector2;
use std::collections::HashMap;

pub struct Layer {
    root_element: SvgElement,
    bounding_box: BoundingBox,
    tile_cache: HashMap<TileCoords, Tile>,
}

impl Layer {
    pub fn new(svg_data: &str) -> anyhow::Result<Self> {
        let root_element = SvgElement::from_svg_data(svg_data)?;
        Ok(Self {
            root_element,
            bounding_box: root_element.get_bounding_box(),
            tile_cache: HashMap::new(),
        })
    }

    fn bounding_box_for_tile_coords(coords: TileCoords) -> BoundingBox {}
}
