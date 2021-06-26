use indoor_map_lib::svg_parser::SvgElement;

pub struct Layer {}

impl Layer {
    pub fn new(svg_data: &str) -> anyhow::Result<Self> {
        let element = SvgElement::from_svg_data(svg_data)?;
        println!("{:#?}", element);
        Ok(Layer {})
    }
}
