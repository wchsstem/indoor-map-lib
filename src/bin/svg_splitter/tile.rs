use indoor_map_lib::svg_parser::SvgElement;
use nalgebra::Vector2;
use svg::node::element::GenericElement;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TileCoords {
    pub location: Vector2<u32>,
    pub zoom: u32,
}

impl TileCoords {
    pub fn new(location: Vector2<u32>, zoom: u32) -> Self {
        Self { location, zoom }
    }
}

#[derive(Debug)]
pub struct Tile<'a> {
    image: SvgElement<'a>,
}

impl<'a> Tile<'a> {
    pub fn new(image: SvgElement<'a>) -> Self {
        Self { image }
    }

    pub fn as_element(&self) -> GenericElement {
        self.image.as_element()
    }
}
