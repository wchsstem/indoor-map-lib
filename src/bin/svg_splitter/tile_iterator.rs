use crate::tile::TileCoords;
use nalgebra::Vector2;

pub struct TileIterator {
    coords: Option<TileCoords>,
}

impl TileIterator {
    pub fn new(zoom_level: u32) -> Self {
        let initial_coords = TileCoords::new(Vector2::new(0, 0), zoom_level);
        Self {
            coords: Some(initial_coords),
        }
    }

    fn max_coords_for_zoom_level(zoom_level: u32) -> u32 {
        2_u32.pow(zoom_level) - 1
    }
}

impl Iterator for TileIterator {
    type Item = TileCoords;

    fn next(&mut self) -> Option<Self::Item> {
        let coords = self.coords.clone();

        if let Some(coords) = &mut self.coords {
            let max_coords = Self::max_coords_for_zoom_level(coords.zoom);
            if coords.location[0] < max_coords {
                coords.location[0] += 1;
            } else {
                coords.location[0] = 0;
                if coords.location[1] < max_coords {
                    coords.location[1] += 1;
                } else {
                    self.coords = None;
                }
            }
        }

        coords
    }
}
