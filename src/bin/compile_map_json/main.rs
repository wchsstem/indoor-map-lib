use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use structopt::StructOpt;

use indoor_map_lib::map_data::uncompiled;

#[derive(StructOpt, Debug)]
#[structopt(name = "compile_map_json")]
struct Opt {
    #[structopt(name = "INPUT JSON", parse(from_os_str))]
    input: PathBuf,
    #[structopt(name = "OUTPUT JSON", parse(from_os_str))]
    output: PathBuf,
}

fn main() {
    let opt: Opt = Opt::from_args();

    let input_json = fs::read_to_string(&opt.input).expect("Error reading input file");

    let base_path = opt.input.parent().expect("Input path should be a file");

    let map_data = uncompiled::MapData::new(&input_json).expect("Error in the JSON file");

    let output_data = serde_json::to_string(
        &map_data
            .compile(base_path)
            .expect("Error compiling map data"),
    )
    .expect("Error serializing map data");
    let mut output = File::create(opt.output).expect("Error before writing to output file");
    write!(output, "{}", output_data).expect("Error while writing to output file");
}
