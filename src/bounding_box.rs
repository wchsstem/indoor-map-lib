use nalgebra::Vector2;
use svg::node::element::path::Data;

use crate::svg_path_parser::Path;

#[derive(Clone, Debug)]
pub struct BoundingBox {
    top_left: Vector2<f64>,
    size: Vector2<f64>,
}

impl BoundingBox {
    pub fn new(top_left: Vector2<f64>, size: Vector2<f64>) -> Self {
        Self { top_left, size }
    }

    pub fn intersects(&self, other: &Self) -> bool {
        let self_left_of_other = self.top_left[0] + self.size[0] < other.top_left[0];
        let other_left_of_self = other.top_left[0] + other.size[0] < self.top_left[0];
        let self_below_other = self.top_left[1] + self.size[1] < other.top_left[1];
        let other_below_self = other.top_left[1] + other.size[1] < self.top_left[1];
        !(self_left_of_other || other_left_of_self || self_below_other || other_below_self)
    }

    pub fn get_top_left(&self) -> Vector2<f64> {
        self.top_left.clone()
    }

    pub fn get_size(&self) -> Vector2<f64> {
        self.size.clone()
    }

    pub fn get_bottom_right(&self) -> Vector2<f64> {
        self.top_left + self.size
    }

    pub fn as_view_box(&self) -> String {
        format!(
            "{} {} {} {}",
            self.top_left[0], self.top_left[1], self.size[0], self.size[1]
        )
    }
}

impl From<&Data> for BoundingBox {
    fn from(data: &Data) -> Self {
        let path = Path::from(data);

        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        let mut min_y = f32::MAX;
        let mut max_y = f32::MIN;

        for command in path.into_iter() {
            if command.0 < min_x {
                min_x = command.0;
            }
            if command.0 > max_x {
                max_x = command.0;
            }
            if command.1 < min_y {
                min_y = command.1;
            }
            if command.1 > max_y {
                max_y = command.1;
            }
        }

        let top_left = Vector2::new(min_x as f64, min_y as f64);
        let bottom_right = Vector2::new(max_x as f64, max_y as f64);
        let size = bottom_right - &top_left;

        Self::new(top_left, size)
    }
}

#[derive(Clone, Debug)]
pub struct BoundingSquare {
    top_left: Vector2<f64>,
    size: f64,
}

impl BoundingSquare {
    pub fn new(top_left: Vector2<f64>, size: f64) -> Self {
        Self { top_left, size }
    }

    pub fn contain_bounding_box(bounding_box: &BoundingBox) -> Self {
        Self {
            top_left: bounding_box.top_left,
            size: bounding_box.size.max(),
        }
    }

    pub fn edge_length(&self) -> f64 {
        self.size
    }

    pub fn as_bounding_box(&self) -> BoundingBox {
        BoundingBox::new(self.top_left, Vector2::new(self.size, self.size))
    }
}
