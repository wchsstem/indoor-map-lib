use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use structopt::StructOpt;
use svg::node::element::path;
use svg::parser::Event;

use indoor_map_lib::MapData;
use svg_path_parser::Path;

mod svg_path_parser;

#[derive(StructOpt, Debug)]
#[structopt(name = "compile_map_json")]
struct Opt {
    #[structopt(name = "INPUT JSON", parse(from_os_str))]
    input: PathBuf,
    #[structopt(name = "OUTPUT JSON", parse(from_os_str))]
    output: PathBuf,
}

fn transformed(coords: (f32, f32), offsets: (f32, f32)) -> (f32, f32) {
    (coords.0 - offsets.0, -coords.1 + offsets.1)
}

fn shoelace_area(points: &[(f32, f32)]) -> f32 {
    let this = points.iter();
    let next = points.iter().cycle().skip(1);
    let double_area: f32 = this
        .zip(next)
        .map(|((this_x, this_y), (next_x, next_y))| this_x * next_y - (next_x * this_y))
        .sum();
    0.5 * double_area
}

fn centroid(points: &[(f32, f32)]) -> (f32, f32) {
    let this = points.iter();
    let next = points.iter().cycle().skip(1);
    let (center_x, center_y) = this
        .zip(next)
        .map(|((this_x, this_y), (next_x, next_y))| {
            let diff = (this_x * next_y) - (next_x * this_y);
            let x = (this_x + next_x) * diff;
            let y = (this_y + next_y) * diff;
            (x, y)
        })
        .fold((0.0, 0.0), |(acc_x, acc_y), (x, y)| (acc_x + x, acc_y + y));

    let coefficient = 1.0 / (6.0 * shoelace_area(points));

    (coefficient * center_x, coefficient * center_y)
}

#[derive(Debug)]
enum SvgRoomShape {
    Rect {
        width: f32,
        height: f32,
        x: f32,
        y: f32,
    },
    Path(path::Data),
}

#[derive(Debug)]
struct SvgRoom {
    number: String,
    shape: SvgRoomShape,
}

impl SvgRoom {
    pub fn outline(&self, offsets: (f32, f32)) -> Vec<(f32, f32)> {
        match &self.shape {
            SvgRoomShape::Rect {
                x,
                y,
                width,
                height,
            } => vec![
                (*x, *y),
                (*x, y + height),
                (x + width, y + height),
                (x + width, *y),
            ]
                .into_iter()
                .map(|coords| transformed(coords, offsets))
                .collect(),
            SvgRoomShape::Path(path_data) => Path::from(path_data)
                .into_iter()
                // TODO: Integrate interfaces to avoid this:         \/
                .map(|coords| transformed((coords.0, coords.1), offsets))
                .collect(),
        }
    }
}

impl<'a> TryFrom<Event<'a>> for SvgRoom {
    type Error = ();

    fn try_from(event: Event<'a>) -> Result<Self, Self::Error> {
        match event {
            Event::Tag("rect", _, attr) => {
                let number = attr
                    .get("id")
                    .ok_or(())?
                    .strip_prefix("room")
                    .ok_or(())?
                    .to_owned();

                let width = attr.get("width").ok_or(())?.parse().map_err(|_| ())?;
                let height = attr.get("height").ok_or(())?.parse().map_err(|_| ())?;
                let x = attr.get("x").ok_or(())?.parse().map_err(|_| ())?;
                let y = attr.get("y").ok_or(())?.parse().map_err(|_| ())?;

                Ok(Self {
                    number,
                    shape: SvgRoomShape::Rect {
                        width,
                        height,
                        x,
                        y,
                    },
                })
            }
            Event::Tag("path", _, attr) => {
                let number = attr
                    .get("id")
                    .ok_or(())?
                    .strip_prefix("room")
                    .ok_or(())?
                    .to_owned();

                let d = attr.get("d").ok_or(())?;
                let path_data = path::Data::parse(d).map_err(|_| ())?;

                Ok(Self {
                    number,
                    shape: SvgRoomShape::Path(path_data),
                })
            }
            _ => Err(()),
        }
    }
}

fn main() {
    let opt: Opt = Opt::from_args();

    let input_json = fs::read_to_string(&opt.input).expect("Error reading input file");

    let base_path = opt.input.parent().expect("Input path should be a file");

    let mut map_data = MapData::new(&input_json).expect("Error in the JSON file");

    let floors = map_data.get_floors();
    let floor_images: Vec<_> = floors
        .iter()
        .map(|floor| (floor.get_image(), floor.get_offsets()))
        .map(|(image_rel_path, o)| (base_path.join(image_rel_path), o))
        .map(|(image_path, o)| {
            (
                fs::read_to_string(image_path).expect("Image file doesn't exist"),
                o,
            )
        })
        .collect();

    for (image_content, offsets) in floor_images {
        let svg_reader = svg::read(&image_content).expect("SVG must be valid");

        let rooms = svg_reader.filter_map(|event| SvgRoom::try_from(event).ok());
        for room in rooms {
            let outline = room.outline(offsets);

            let old_room = match map_data.room_mut(&room.number) {
                Some(old_room) => old_room,
                None => {
                    println!("Room does not exist: {}", room.number);
                    continue;
                }
            };

            if old_room.center().is_none() {
                let center = centroid(&outline);
                old_room.set_center(Some(center));
            }

            if old_room.area().is_none() {
                let area = shoelace_area(&outline).abs();
                old_room.set_area(Some(area));
            }

            old_room.set_outline(outline);
        }
    }

    let output_data = serde_json::to_string(&map_data).expect("Error serializing map data");
    let mut output = File::create(opt.output).expect("Error before writing to output file");
    write!(output, "{}", output_data).expect("Error while writing to output file");
}
