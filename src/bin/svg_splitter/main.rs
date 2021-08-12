use std::error::Error;
use std::fs;
use std::path::PathBuf;

use nalgebra::Vector2;
use structopt::StructOpt;

use indoor_map_lib::bounding_box::BoundingSquare;

use crate::layer::Layer;
use crate::tile_iterator::TileIterator;
use svg::Document;

mod layer;
mod tile;
mod tile_iterator;

#[derive(StructOpt, Debug)]
#[structopt(name = "svg_splitter")]
struct Opt {
    #[structopt(name = "INPUT SVG", parse(from_os_str), help = "path to SVG to split")]
    input: PathBuf,
    #[structopt(
        name = "OUTPUT DIRECTORY",
        parse(from_os_str),
        help = "directory to write split SVGs to"
    )]
    output: PathBuf,
    #[structopt(
        short = "m",
        long,
        default_value = "0",
        help = "zoom level to create tiles for (no less than 0)"
    )]
    zoom_level: u32,
    #[structopt(
        short = "x",
        long,
        default_value = "0",
        help = "x-coordinate of the top left of the zoom level 0 tile"
    )]
    top_left_x: f64,
    #[structopt(
        short = "y",
        long,
        default_value = "0",
        help = "y-coordinate of the top left of the zoom level 0 tile"
    )]
    top_left_y: f64,
    #[structopt(
        short = "s",
        long,
        default_value = "100",
        help = "length of the edge of the zoom level 0 tile"
    )]
    size: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let opt: Opt = Opt::from_args();

    let svg_data = fs::read_to_string(opt.input)?;
    let layer_bounds = BoundingSquare::new(Vector2::new(opt.top_left_x, opt.top_left_y), opt.size);
    let layer = Layer::new(&svg_data, layer_bounds)?;

    for coords in TileIterator::new(opt.zoom_level) {
        let tile = layer.tile(&coords);
        let mut file_path = opt.output.clone();
        file_path.push(format!(
            "{}.{}.{}.svg",
            coords.zoom, coords.location[0], coords.location[1]
        ));
        let document = Document::new().add(tile.as_element());
        svg::save(file_path, document)?;
    }

    Ok(())
}
