use std::borrow::{Borrow, Cow};
use std::collections::HashMap;
use std::iter::Peekable;
use std::num::ParseFloatError;

use anyhow::{anyhow, Context};
use nalgebra::{Matrix3, Vector2, Vector3};
use svg::node::element::tag::Type;
use svg::node::element::Element;
use svg::node::{Attributes, Value};
use svg::parser::Event;
use svg::{Node, Parser};

use crate::bounding_box::BoundingBox;
use crate::transform;
use crate::util::max_f64;
use svg::node::element::path::Data;

#[derive(Debug)]
pub struct SvgElement<'a> {
    bounding_box: BoundingBox,
    children: Vec<SvgElement<'a>>,
    tag_name: &'a str,
    attributes: Attributes,
}

impl<'a> SvgElement<'a> {
    pub fn empty_root(bounding_box: BoundingBox) -> Self {
        Self {
            bounding_box,
            children: vec![],
            tag_name: "svg",
            attributes: HashMap::with_capacity(0),
        }
    }

    pub fn from_svg_data(svg_data: &'a str) -> anyhow::Result<Self> {
        let mut parser = svg::read(svg_data)?.peekable();
        let initial_transformation_matrix =
            Matrix3::new(1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0);

        // Allow skipping over `<?xml version="1.0" encoding="UTF-8" standalone="no"?>`, which is
        // ignored
        match Self::parse_event(&initial_transformation_matrix, &mut parser)? {
            Some(element) => Ok(element),
            None => Self::parse_event(&initial_transformation_matrix, &mut parser)?
                .ok_or_else(|| anyhow!("Expected SVG data but did not find any")),
        }
    }

    pub fn get_bottom_right(&self) -> Vector2<f64> {
        self.bounding_box.get_bottom_right()
    }

    pub fn get_bounding_box(&self) -> BoundingBox {
        self.bounding_box.clone()
    }

    pub fn set_attr(&mut self, name: &str, value: Value) {
        self.attributes.insert(name.to_owned(), value);
    }

    pub fn delete_attr(&mut self, name: &str) {
        self.attributes.remove(name);
    }

    /// Returns `Some` if this element overlaps the given bounding box. The returned element only
    /// has the children of this element which overlap the bounding box, the children only keep
    /// their children which overlap, and so on.
    pub fn select_with(&self, bounding_box: &BoundingBox) -> Option<Self> {
        if self.bounding_box.intersects(bounding_box) {
            let selected_children = self
                .children
                .iter()
                .filter_map(|child| child.select_with(bounding_box))
                .collect::<Vec<_>>();
            Some(Self {
                bounding_box: self.bounding_box.clone(),
                children: selected_children,
                tag_name: self.tag_name,
                attributes: self.attributes.clone(),
            })
        } else {
            None
        }
    }

    fn num_from_attr(attributes: &Attributes, key: &str) -> Result<Option<f64>, ParseFloatError> {
        attributes
            .get(key)
            .map(|value| value.trim_end_matches("mm").parse())
            .transpose()
    }

    fn parse_matrix_transform(matrix: &str) -> anyhow::Result<Matrix3<f64>> {
        let data_str = matrix.trim_start_matches("matrix(").trim_end_matches(')');
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
            .trim_end_matches(')');
        let data = data_str
            .split(&[' ', ','][..])
            .map(|num| num.parse())
            .collect::<Result<Vec<f64>, _>>()?;
        if data.is_empty() || data.len() > 2 {
            Err(anyhow!(
                "Wrong number of arguments to translate transform: {:?}",
                translate
            ))
        } else {
            let x = data[0];
            let y = data.get(1).copied().unwrap_or(0.0);
            Ok(transform::translate(Vector2::new(x, y)))
        }
    }

    fn parse_rotate_transform(rotate: &str) -> anyhow::Result<Matrix3<f64>> {
        let data_str = rotate.trim_start_matches("rotate(").trim_end_matches(')');
        let data = data_str
            .split(&[' ', ','][..])
            .map(|num| num.parse())
            .collect::<Result<Vec<f64>, _>>()?;

        match data.len() {
            1 => Ok(transform::rotate_deg(data[0])),
            3 => Ok(transform::rotate_deg_about(
                data[0],
                Vector2::new(data[1], data[2]),
            )),
            0 => Err(anyhow!(
                "Not enough arguments to rotate transform: {:?}",
                rotate
            )),
            2 => Err(anyhow!("Expected a y-coordinate to rotate about")),
            _ => Err(anyhow!(
                "Too many arguments to rotate transform: {:?}",
                rotate
            )),
        }
    }

    fn parse_scale_transform(scale: &str) -> anyhow::Result<Matrix3<f64>> {
        let data_str = scale.trim_start_matches("scale(").trim_end_matches(')');
        let data = data_str
            .split(&[' ', ','][..])
            .map(|num| num.parse())
            .collect::<Result<Vec<f64>, _>>()?;

        match data.len() {
            1 => Ok(transform::scale(Vector2::new(data[0], data[0]))),
            2 => Ok(transform::scale(Vector2::new(data[0], data[1]))),
            0 => Err(anyhow!(
                "Not enough arguments to scale transform: {:?}",
                scale
            )),
            _ => Err(anyhow!(
                "Too many arguments to scale transform: {:?}",
                scale
            )),
        }
    }

    fn parse_transform(transformation: &str) -> anyhow::Result<Matrix3<f64>> {
        match transformation {
            matrix if matrix.starts_with("matrix") => Self::parse_matrix_transform(matrix),
            translate if translate.starts_with("translate") => {
                Self::parse_translate_transform(translate)
            }
            rotate if rotate.starts_with("rotate") => Self::parse_rotate_transform(rotate),
            scale if scale.starts_with("scale") => Self::parse_scale_transform(scale),
            other => panic!("Unimplemented transformation {}", other),
        }
    }

    fn parse_children<'b>(
        parser: &'b mut Peekable<Parser<'a>>,
        current_transformation_matrix: &Matrix3<f64>,
    ) -> anyhow::Result<Vec<Self>> {
        let mut children = Vec::new();
        while let Some(event) = parser.peek() {
            if let Event::Tag(_name, Type::End, _attributes) = event {
                break;
            }
            if let Some(element) = Self::parse_event(current_transformation_matrix, parser)? {
                children.push(element);
            }
        }
        // Consume ending tag
        parser.next();
        Ok(children)
    }

    fn parse_tag<'b>(
        current_transformation_matrix: &Matrix3<f64>,
        name: &'a str,
        children_type: Type,
        attributes: Attributes,
        parser: &'b mut Peekable<Parser<'a>>,
    ) -> anyhow::Result<Self> {
        let (size, local_top_left_homogenous) = match name {
            "path" => {
                let d = attributes.get("d").context("Missing path data")?;
                let data = Data::parse(d)?;
                let bounds = BoundingBox::from(&data);

                let top_left = bounds.get_top_left();
                let homogenous_top_left = Vector3::new(top_left[0], top_left[1], 1.);

                (bounds.get_size(), homogenous_top_left)
            }
            "rect" | _ => {
                let min_width: f64 = Self::num_from_attr(&attributes, "width")?.unwrap_or(0.0);
                let min_height: f64 = Self::num_from_attr(&attributes, "height")?.unwrap_or(0.0);
                let size = Vector2::new(min_width, min_height);

                let x: f64 = Self::num_from_attr(&attributes, "x")?.unwrap_or(0.0);
                let y: f64 = Self::num_from_attr(&attributes, "y")?.unwrap_or(0.0);
                let top_left = Vector3::new(x, y, 1.);

                (size, top_left)
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
                bounding_box: BoundingBox::new(global_top_left, size),
                children: vec![],
                tag_name: name,
                attributes,
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
                    bounding_box: BoundingBox::new(global_top_left, actual_size),
                    children,
                    tag_name: name,
                    attributes,
                })
            }
        }
    }

    fn parse_event<'b>(
        current_transformation_matrix: &Matrix3<f64>,
        parser: &'b mut Peekable<Parser<'a>>,
    ) -> anyhow::Result<Option<Self>> {
        match parser.next() {
            None => Err(anyhow!("Unexpected end of SVG")),
            Some(event) => match event {
                Event::Error(error) => Err(error.into()),
                // Nothing we need to do with these, so skip them
                Event::Text(_) | Event::Comment | Event::Declaration | Event::Instruction => {
                    Ok(None)
                }
                Event::Tag(name, children_type, attributes) => Some(Self::parse_tag(
                    current_transformation_matrix,
                    name,
                    children_type,
                    attributes,
                    parser,
                ))
                .transpose(),
            },
        }
    }

    pub fn as_element(&self) -> Element {
        let mut element = Element::new(self.tag_name);
        for (name, value) in &self.attributes {
            element.assign(name, value.clone());
        }
        for child in &self.children {
            element.append(child.as_element());
        }
        element
    }
}
