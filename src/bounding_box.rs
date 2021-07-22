use nalgebra::Vector2;

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

    pub fn get_bottom_right(&self) -> Vector2<f64> {
        self.top_left + self.size
    }
}

#[derive(Clone, Debug)]
pub struct BoundingSquare {
    top_left: Vector2<f64>,
    size: f64,
}

impl BoundingSquare {
    pub fn new(top_left: Vector2<f64>, size: f64) -> Self {
        Self {
            top_left,
            size,
        }
    }

    pub fn contain_bounding_box(bounding_box: &BoundingBox) -> Self {
        Self {

        }
    }
}
