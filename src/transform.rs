use nalgebra::{Matrix3, Vector2};

pub fn translate(translation: Vector2<f64>) -> Matrix3<f64> {
    Matrix3::new(1., 0., translation[0], 0., 1., translation[1], 0., 0., 1.)
}

pub fn rotate_deg(rotation: f64) -> Matrix3<f64> {
    let rotation = rotation.to_radians();
    Matrix3::new(
        rotation.cos(),
        -rotation.sin(),
        0.,
        rotation.sin(),
        rotation.cos(),
        0.,
        0.,
        0.,
        1.,
    )
}

pub fn rotate_deg_about(rotation: f64, about: Vector2<f64>) -> Matrix3<f64> {
    translate(about) * rotate_deg(rotation) * translate(-about)
}

pub fn scale(factor: Vector2<f64>) -> Matrix3<f64> {
    Matrix3::new(factor[0], 0., 0., 0., factor[1], 0., 0., 0., 1.)
}
