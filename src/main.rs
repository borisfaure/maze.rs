extern crate docopt;
extern crate image;
extern crate rand;

use docopt::Docopt;
use std::path::Path;

mod mosaic;

const USAGE: &'static str = "
Background generator.

Usage: bg_gen [options] FILE
       bg_gen -g GEOM FILE
       bg_gen --geometry GEOM FILE
       bg_gen -h | --help
       bg_gen -v | --version

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
    println!("geometry='{}'", geometry);

    let path = args.get_str("FILE");

    let geometry = geometry_parse(&geometry);

    println!("path='{}'", path);
    println!("geometry={:?}", geometry);

    let path = Path::new(path);
    /* TODO: kind: mosaic, fall */
    mosaic::generate_image(path, geometry.0, geometry.1);
}
