extern crate docopt;
extern crate image;
extern crate gif;
extern crate rand;
#[macro_use]
extern crate debug_macros;
extern crate color_scaling;
extern crate read_color;

mod maze;
mod plain;
mod invaders;
mod mosaic;

#[derive(Debug,Clone)]
pub struct Geometry {
    width: usize,
    height: usize,
}

use docopt::Docopt;
use std::path;
use std::str::FromStr;

#[derive(Debug)]
pub struct Origin {
    x: f64,
    y: f64,
}

use image::{
    Rgb,
};

/* CLI {{{ */

const USAGE: &'static str = "
Maze background generator.

Usage: maze [options] FILE
       maze --geometry GEOM FILE
       maze -g GEOM FILE
       maze --rendering=RENDERING FILE
       maze -r RENDERING FILE
       maze --vertical-bias BIAS FILE
       maze -b BIAS FILE
       maze -o ORIGIN FILE
       maze --origin ORIGIN FILE
       maze --foreground COLOR FILE
       maze --background COLOR FILE
       maze --gradient GRADIENT FILE
       maze --algorithm ALGORITHM FILE
       maze --animation
       maze -h | --help
       maze -v | --version

Options:
    -h, --help                                    Show this message
    -v, --version                                 Show the version
    -g=<WIDTHxHEIGHT>, --geometry=<WIDTHxHEIGHT>  Geometry of the image to generate [default: 100x100]
    -r=RENDERING, --rendering=RENDERING           Rendering mode. Valid values are: plain, invaders, mosaic. [default: plain]
    -b=BIAS, --vertical-bias=BIAS                 Vertical Bias. Larger than 0.5, the maze will then to be more vertical. Lower than 0.5, will tend to be more horizontal. [default: 0.5]
    -o=ORIGIN, --origin=ORIGIN                    Relative origin of the maze in floating point coordinates. Middle is 0.5x0.5. [default: 0.0x0.0]
    --background=COLOR                            Background color. [default: #073642]
    --foreground=COLOR                            Foreground color(s). Either one or two colors (\"#ffffff\" or \"#ffffff #ff00ff\"). [default: #d70000 #ffffd7]
    --gradient=GRADIENT                           If 2 foreground colors, define how to do the gradient. Valid values are: length, solution. [default: length]
    --algorithm=ALGORITHM                         Algorithm used to generate the maze. Valid values are: prim, kruskal. [default: prim]
    --animation                                   Render an animation as the maze is being generated
";



fn geometry_parse(geometry: &str) -> Geometry {
    let geometry : Vec<&str> = geometry.split('x').collect();
    if geometry.len() != 2 {
        panic!("invalid geometry");
    }
    let width : usize = geometry[0].parse()
        .ok()
        .expect("invalid geometry");
    let height : usize = geometry[1].parse()
        .ok()
        .expect("invalid geometry");
    Geometry{width: width, height: height}
}

fn rendering_parse(rendering: &str, bg: Rgb<u8>, fg: [Rgb<u8>; 2])
    -> Box<maze::Rendering> {
    match rendering {
        "plain" => {
            Box::new(plain::RendererPlain {
                path_color_start: fg[0],
                path_color_end: fg[1],
                wall_color: bg,
            })
        },
        "invaders" => {
            Box::new(invaders::RendererInvaders{
                invader_color: fg[0],
                wall_color: bg,
            })
        },
        "mosaic" => {
            Box::new(mosaic::RendererMosaic{
                is_inverted: false,
            })
        },
        _ => {
            panic!("invalid rendering mode")
        }
    }
}

fn vertical_bias_parse(vertical_bias: &str) -> f64 {
    let d = f64::from_str(vertical_bias)
        .ok()
        .expect("vertical_bias is not a floating number");

    if d >= 1.0 {
        panic!("vertical_bias is too large");
    } else if d <= 0.0 {
        panic!("vertical_bias is too small");
    }
    d
}

fn origin_parse(origin: &str) -> Origin {
    let origin: Vec<&str> = origin.split('x').collect();
    if origin.len() != 2 {
        panic!("invalid geometry");
    }
    let x : f64 = origin[0].parse()
        .ok()
        .expect("invalid origin");
    let y : f64 = origin[1].parse()
        .ok()
        .expect("invalid origin");
    Origin{x: x, y: y}
}

fn color_parse(color: &str) -> Rgb<u8> {
    let c = &mut color.chars();
    match c.next() {
        None => {
            panic!("invalid color {}", color);
        },
        Some(x) if x != '#' => {
            panic!("invalid color {}", color);
        },
        _ => {
        }
    }
    match read_color::rgb(c) {
        None => {
            panic!("invalid color {}", color);
        },
        Some(d) => {
            Rgb{data: d}
        }
    }
}

fn gradient_parse(g: &str) -> Option<maze::Gradient> {
    match g {
        "length" => {
            Some(maze::Gradient::Length)
        },
        "solution" => {
            Some(maze::Gradient::Solution)
        },
        _ => {
            None
        }
    }
}

fn algorithm_parse(s: &str) -> maze::AlgorithmKind {
    match s {
        "prim" => {
            maze::AlgorithmKind::Prim
        },
        "kruskal" => {
            maze::AlgorithmKind::Kruskal
        },
        "backtracker" => {
            maze::AlgorithmKind::Backtracker
        }
        _ => {
            panic!("invalid algorithm {}", s);
        }
    }
}

fn colors_parse(bg: &str, fg: &str) -> (Rgb<u8>, [Rgb<u8>; 2]) {
    let bg = color_parse(bg);

    let vec_str : Vec<&str> = fg.split(' ').collect();
    if vec_str.len() > 2 {
        panic!("invalid colors '{}'", fg);
    }
    let fg1 = color_parse(vec_str[0]);
    let fg2 = {
        if vec_str.len() > 1 {
            color_parse(vec_str[1])
        } else {
            fg1.clone() as Rgb<u8>
        }
    };

    (bg, [fg1, fg2])
}

fn main() {
    let version = env!("CARGO_PKG_VERSION").to_owned();
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.version(Some(version)).parse())
                      .unwrap_or_else(|e| e.exit());
    let geometry = args.get_str("--geometry");
    let geometry = geometry_parse(&geometry);

    let (bg, fg) = colors_parse(args.get_str("--background"),
                                args.get_str("--foreground"));

    let rendering = args.get_str("--rendering");
    let rendering = rendering_parse(&rendering, bg, fg);

    let path = args.get_str("FILE");
    let path = path::Path::new(path);

    let vertical_bias = args.get_str("--vertical-bias");
    let vertical_bias = vertical_bias_parse(&vertical_bias);

    let origin = args.get_str("--origin");
    let origin = origin_parse(&origin);

    let gradient = args.get_str("--gradient");
    let gradient = gradient_parse(&gradient);

    let algorithm = args.get_str("--algorithm");
    let algorithm = algorithm_parse(&algorithm);

    let animation = args.get_bool("--animation");

    maze::generate_image(path, geometry, &*rendering, vertical_bias,
                         origin, gradient, algorithm, animation);
}

/* }}} */
