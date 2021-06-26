use indoor_map_lib::bounding_box::BoundingBox;
use indoor_map_lib::svg_parser::SvgElement;
use nalgebra::Vector2;

pub struct Layer {
    root_element: SvgElement,
}

impl Layer {
    pub fn new(svg_data: &str) -> anyhow::Result<Self> {
        let root_element = SvgElement::from_svg_data(svg_data)?;
        let selection = root_element.select_with(&BoundingBox::new(
            Vector2::new(20.0, 0.0),
            Vector2::new(20.0, 20.0),
        ));
        println!("selection: {:#?}", selection);
        Ok(Layer { root_element })
    }
}
