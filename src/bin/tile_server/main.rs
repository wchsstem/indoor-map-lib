use crate::layer::Layer;
use std::error::Error;
use std::fs;

mod layer;
mod tile;

fn main() -> Result<(), Box<dyn Error>> {
    let svg_data = fs::read_to_string("test.svg")?;
    let layer = Layer::new(&svg_data)?;
    Ok(())
}
