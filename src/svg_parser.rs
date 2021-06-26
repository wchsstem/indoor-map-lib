use std::iter::Peekable;

use crate::util::max_f64;
use anyhow::anyhow;
use nalgebra::{Matrix3, Vector2, Vector3};
use std::borrow::{Borrow, Cow};
use std::num::ParseFloatError;
use svg::node::element::tag::Type;
use svg::node::Attributes;
use svg::parser::Event;
use svg::Parser;

#[derive(Debug)]
pub struct SvgElement {
    global_top_left: Vector2<f64>,
    size: Vector2<f64>,
    children: Vec<SvgElement>,
}

impl SvgElement {
    pub fn from_svg_data(svg_data: &str) -> anyhow::Result<Self> {
        let mut parser = svg::read(svg_data)?.peekable();
        let initial_transformation_matrix =
            Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);
        Self::parse_event(&initial_transformation_matrix, &mut parser)
    }

    pub fn get_bottom_right(&self) -> Vector2<f64> {
        self.global_top_left + self.size
    }

    fn get_attr_num(attributes: &Attributes, key: &str) -> Result<Option<f64>, ParseFloatError> {
        attributes
            .get(key)
            .map(|value| value.trim_end_matches("mm").parse())
            .transpose()
    }

    fn parse_matrix_transform(matrix: &str) -> anyhow::Result<Matrix3<f64>> {
        let data_str = matrix.trim_start_matches("matrix(").trim_end_matches(")");
        let data = data_str
            .split_whitespace()
            .map(|num| num.parse())
            .collect::<Result<Vec<f64>, _>>()?;
        if data.len() != 6 {
            Err(anyhow!(
                "Wrong number of arguments to matrix transform: {:?}",
                matrix
            ))
        } else {
            Ok(Matrix3::new(
                data[0], data[2], data[4], data[1], data[3], data[5], 0.0, 0.0, 1.0,
            ))
        }
    }

    fn parse_translate_transform(translate: &str) -> anyhow::Result<Matrix3<f64>> {
        let data_str = translate
            .trim_start_matches("translate(")
            .trim_end_matches(")");
        let data = data_str
            .split_whitespace()
            .map(|num| num.parse())
            .collect::<Result<Vec<f64>, _>>()?;
        if data.len() < 1 || data.len() > 2 {
            Err(anyhow!(
                "Wrong number of arguments to translate transform: {:?}",
                translate
            ))
        } else {
            let x = data[0];
            let y = data.get(1).copied().unwrap_or(0.0);
            Ok(Matrix3::new(1.0, 0.0, x, 0.0, 1.0, y, 0.0, 0.0, 1.0))
        }
    }

    fn parse_transform(transformation: &str) -> anyhow::Result<Matrix3<f64>> {
        match transformation {
            matrix if matrix.starts_with("matrix") => Self::parse_matrix_transform(matrix),
            translate if translate.starts_with("translate") => {
                Self::parse_translate_transform(translate)
            }
            other => panic!("Unimplemented transformation {}", other),
        }
    }

    fn parse_children(
        parser: &mut Peekable<Parser>,
        current_transformation_matrix: &Matrix3<f64>,
    ) -> anyhow::Result<Vec<Self>> {
        let mut children = Vec::new();
        while let Some(event) = parser.peek() {
            if let Event::Tag(_name, Type::End, _attributes) = event {
                break;
            }
            children.push(Self::parse_event(current_transformation_matrix, parser)?);
        }
        // Consume ending tag
        parser.next();
        Ok(children)
    }

    fn parse_tag(
        current_transformation_matrix: &Matrix3<f64>,
        name: &str,
        children_type: Type,
        attributes: Attributes,
        parser: &mut Peekable<Parser>,
    ) -> anyhow::Result<Self> {
        let min_width: f64 = Self::get_attr_num(&attributes, "width")?.unwrap_or(0.0);
        let min_height: f64 = Self::get_attr_num(&attributes, "height")?.unwrap_or(0.0);
        let size = Vector2::new(min_width, min_height);

        let local_top_left_homogenous = match name {
            "rect" | _ => {
                let x: f64 = Self::get_attr_num(&attributes, "x")?.unwrap_or(0.0);
                let y: f64 = Self::get_attr_num(&attributes, "y")?.unwrap_or(0.0);
                Vector3::new(x, y, 1.0)
            }
        };

        let current_transformation_matrix = match attributes.get("transform") {
            Some(transformation) => {
                Cow::Owned(current_transformation_matrix * Self::parse_transform(transformation)?)
            }
            None => Cow::Borrowed(current_transformation_matrix),
        };

        let global_top_left_homogenous: Vector3<f64> =
            <Cow<Matrix3<f64>> as Borrow<Matrix3<f64>>>::borrow(&current_transformation_matrix)
                * local_top_left_homogenous;
        let global_top_left =
            Vector2::new(global_top_left_homogenous[0], global_top_left_homogenous[1]);

        match children_type {
            Type::End => Err(anyhow!(
                "Unexpected end tag: {}, {:?}, {:?}",
                name,
                children_type,
                attributes
            )),
            Type::Empty => Ok(Self {
                global_top_left,
                size,
                children: vec![],
            }),
            Type::Start => {
                let bottom_right = global_top_left + size;
                let right = bottom_right[0];
                let bottom = bottom_right[1];

                let children = Self::parse_children(parser, &current_transformation_matrix)?;
                let (rights, bottoms): (Vec<f64>, Vec<f64>) = children
                    .iter()
                    .map(|child| child.get_bottom_right())
                    .map(|bottom_right| (bottom_right[0], bottom_right[1]))
                    .unzip();

                let max_right = max_f64(rights.into_iter())
                    .map_or(right, |child_max_right| child_max_right.max(right));
                let max_bottom = max_f64(bottoms.into_iter())
                    .map_or(bottom, |child_max_bottom| child_max_bottom.max(bottom));

                let actual_bottom_right = Vector2::new(max_right, max_bottom);
                let actual_size = actual_bottom_right - global_top_left;

                Ok(Self {
                    global_top_left,
                    size: actual_size,
                    children,
                })
            }
        }
    }

    fn parse_event(
        current_transformation_matrix: &Matrix3<f64>,
        parser: &mut Peekable<Parser>,
    ) -> anyhow::Result<Self> {
        match parser.next() {
            None => Err(anyhow!("Unexpected end of SVG")),
            Some(event) => match event {
                Event::Error(error) => Err(error.into()),
                // Nothing we need to do with these, so skip them
                Event::Text(_) | Event::Comment | Event::Declaration | Event::Instruction => {
                    Self::parse_event(current_transformation_matrix, parser)
                }
                Event::Tag(name, children_type, attributes) => Self::parse_tag(
                    current_transformation_matrix,
                    name,
                    children_type,
                    attributes,
                    parser,
                ),
            },
        }
    }
}
