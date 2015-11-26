extern crate docopt;
extern crate image;
extern crate rand;

use docopt::Docopt;
use std::path::Path;

mod mosaic;

const USAGE: &'static str = "
Background generator.

Usage: bg_gen mosaic FILE [options]
       bg_gen fall FILE [options]
       bg_gen -g GEOM <kind> FILE
       bg_gen --geometry GEOM <kind> FILE
       bg_gen -h | --help
       bg_gen -v | --version

Kinds:
   mosaic      a random mosaic filling all the image
   fall        a random mosaic that falls from the top of the image

Options:
    -h, --help                            Show this message
    -v, --version                         Show the version
    -g=<WIDTHxHEIGHT>, --geometry=<WIDTHxHEIGHT>  Geometry of the image to generate [default: 100x100]
";



fn geometry_parse(geometry: &str) -> (u32, u32) {
    let geometry : Vec<&str> = geometry.split('x').collect();
    if geometry.len() != 2 {
        panic!("invalid geometry");
    }
    let width : u32 = geometry[0].parse()
        .ok()
        .expect("invalid geometry");
    let height : u32 = geometry[1].parse()
        .ok()
        .expect("invalid geometry");
    (width, height)
}

fn main() {
    let version = "0.0.1".to_owned();
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.version(Some(version)).parse())
                      .unwrap_or_else(|e| e.exit());
    let geometry = args.get_str("--geometry");
    let geometry = geometry_parse(&geometry);

    let path = args.get_str("FILE");
    let path = Path::new(path);

    if args.get_bool("mosaic") {
        mosaic::generate_mosaic(path, geometry.0, geometry.1);
    } else if args.get_bool("fall") {
        mosaic::generate_falling_mosaic(path, geometry.0, geometry.1);
    }
}
