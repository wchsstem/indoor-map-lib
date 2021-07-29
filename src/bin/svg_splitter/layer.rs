use indoor_map_lib::bounding_box::BoundingSquare;
use indoor_map_lib::svg_parser::SvgElement;

use crate::tile::{Tile, TileCoords};

#[derive(Debug)]
pub struct Layer<'a> {
    root_element: SvgElement<'a>,
    bounds: BoundingSquare,
}

impl<'a> Layer<'a> {
    pub fn new(svg_data: &'a str, bounds: BoundingSquare) -> anyhow::Result<Self> {
        let root_element = SvgElement::from_svg_data(svg_data)?;
        Ok(Self {
            root_element,
            bounds,
        })
    }

    fn bounds_for_tile_coords(&self, coords: &TileCoords) -> BoundingSquare {
        let edge_length = self.bounds.edge_length() * (1. / (2_i32.pow(coords.zoom) as f64));

        let top_left = edge_length * coords.location.map(|x| x as f64);

        BoundingSquare::new(top_left, edge_length)
    }

    pub fn tile(&self, coords: &TileCoords) -> Tile {
        let bounds = self.bounds_for_tile_coords(coords).as_bounding_box();
        let view_box = bounds.as_view_box();
        let mut svg = self
            .root_element
            .select_with(&bounds)
            .unwrap_or_else(|| SvgElement::empty_root(bounds));
        svg.set_attr("viewBox", view_box.into());
        svg.delete_attr("height");
        svg.delete_attr("width");
        Tile::new(svg)
    }
}
