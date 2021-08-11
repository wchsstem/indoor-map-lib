use crate::svg_path_parser::SimpleSvgPath;
use std::convert::TryFrom;
use svg::events::Event;
use svg::node::element::path;

#[derive(Debug)]
pub enum SvgRoomShape {
    Rect {
        width: f32,
        height: f32,
        x: f32,
        y: f32,
    },
    Path(path::Data),
}

fn transform_svg_coords(coords: (f32, f32), offsets: (f32, f32)) -> (f32, f32) {
    (coords.0 - offsets.0, -coords.1 + offsets.1)
}

#[derive(Debug)]
pub struct SvgRoom {
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
            .map(|coords| transform_svg_coords(coords, offsets))
            .collect(),
            SvgRoomShape::Path(path_data) => SimpleSvgPath::from(path_data)
                .into_iter()
                // TODO: Integrate interfaces to avoid destructuring:   \/
                .map(|coords| transform_svg_coords((coords.0, coords.1), offsets))
                .collect(),
        }
    }

    pub fn get_number(&self) -> &str {
        &self.number
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
