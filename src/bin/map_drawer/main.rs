use std::fs;
use std::path::PathBuf;

use structopt::StructOpt;
use svg::node::element::Group;
use svg::node::element::Path;
use svg::Document;

use indoor_map_lib::map_data::compiled;
use indoor_map_lib::map_data::compiled::Room;
use std::collections::HashMap;
use svg::node::element::path::Data;

#[derive(StructOpt, Debug)]
#[structopt(name = "map_drawer")]
struct Opt {
    #[structopt(
        name = "INPUT JSON",
        parse(from_os_str),
        help = "path to compiled JSON to use for the drawing"
    )]
    input_compiled_json: PathBuf,
    #[structopt(
        name = "OUTPUT DIRECTORY",
        parse(from_os_str),
        help = "directory to write drawn SVGs to"
    )]
    output_directory: PathBuf,
    #[structopt(name = "FLOOR", help = "floor number to draw maps of")]
    floor: String,
    #[structopt(
        short = "m",
        long,
        default_value = "0",
        help = "minimum zoom level to create tiles for (no less than 0)"
    )]
    min_zoom_level: u32,
}

fn get_compiled_map_data(opt: &Opt) -> compiled::MapData {
    let input_compiled_json =
        fs::read_to_string(&opt.input_compiled_json).expect("Error reading input file");
    serde_json::from_str::<compiled::MapData>(&input_compiled_json).expect("Error in the JSON file")
}

fn get_input_svg_path(opt: &Opt, compiled_map_data: &compiled::MapData) -> PathBuf {
    let relative_input_svg_path = compiled_map_data
        .floors
        .iter()
        .find(|room| room.get_number() == opt.floor)
        .unwrap()
        .get_image();
    opt.input_compiled_json
        .parent()
        .unwrap()
        .join(relative_input_svg_path)
}

fn get_input_svg_document<'a>(
    opt: &Opt,
    compiled_map_data: &compiled::MapData,
    contents_owner: &'a mut String,
) -> Document<'a> {
    let svg_path = get_input_svg_path(opt, compiled_map_data);
    let svg_parser = svg::open(svg_path, contents_owner).expect("Error parsing SVG");
    Document::from_event_parser(svg_parser).unwrap()
}

fn get_floors_for_vertices(compiled_map_data: &compiled::MapData) -> HashMap<&str, &str> {
    compiled_map_data
        .vertices
        .iter()
        .map(|(name, vertex)| (name.as_str(), vertex.get_floor()))
        .collect()
}

fn get_output_file_path(opt: &Opt) -> PathBuf {
    let mut output_file = opt.output_directory.clone();
    output_file.push("base.svg");
    output_file
}

fn room_on_floor(room: &Room, floor: &str, vertex_floors: &HashMap<&str, &str>) -> bool {
    vertex_floors
        .get(room.vertices.iter().next().unwrap().as_str())
        .map(|room_floor| *room_floor == floor)
        .unwrap_or(false)
}

fn main() {
    let opt: Opt = Opt::from_args();

    let compiled_map_data = get_compiled_map_data(&opt);

    let mut svg_contents = String::new();
    let mut document = get_input_svg_document(&opt, &compiled_map_data, &mut svg_contents);

    let vertex_floors = get_floors_for_vertices(&compiled_map_data);

    let outlines = compiled_map_data
        .rooms
        .values()
        .filter(|room| room_on_floor(room, &opt.floor, &vertex_floors))
        .map(|room| &room.outline);

    let mut outlines_element =
        Group::new().set("transform", "scale(1, -1) translate(-4.5, -465.5)");
    for outline in outlines {
        let mut points = outline.iter();
        let mut data = Data::new().move_to(*points.next().unwrap());
        for point in points {
            data = data.line_to(*point);
        }
        let data = data.close();
        let path = Path::new()
            .set("fill", "rgb(125, 181, 52)")
            .set("fill-opacity", "0.2")
            .set("d", data);
        outlines_element = outlines_element.add(path);
    }
    let children = document.get_mut_svg().get_mut_children();
    children.push(outlines_element.into());

    svg::save(get_output_file_path(&opt), &document).unwrap();
}
