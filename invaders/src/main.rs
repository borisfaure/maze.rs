extern crate docopt;
extern crate image;
extern crate rand;

use docopt::Docopt;
use std::path::Path;
use std::cmp;
use std::u16;
use image::{
    RgbImage,
    Rgb
};
use rand::ThreadRng;
use rand::distributions::{
    IndependentSample,
    Range
};

fn draw_pixel_factor(img: &mut RgbImage, x: u32, y: u32, d: u32) {
    for i in 0..d {
        for j in 0..d {
            img.put_pixel(x+i, y+j, Rgb{data: [255, 255, 255]});
        }
    }
}
fn draw_invader_factor(img: &mut RgbImage, x: u32, y: u32, d: u32) {
    let mut rng : ThreadRng = rand::thread_rng();
    let invader_range: Range<u16> = Range::new(0, u16::MAX);
    let invader_nbr = invader_range.ind_sample(&mut rng);
    let d = d / 7;

    for i in 1..4 {
        for j in 1..6 {
            if invader_nbr & ((1 << (i*j)) as u16) > 0 {
                draw_pixel_factor(img,
                                  x + i*d,
                                  y + j*d,
                                  d);
                draw_pixel_factor(img,
                                  x + (6-i)*d,
                                  y + j*d,
                                  d);
            }
        }
    }
}

fn square_ceil_7(n: u32) -> u32 {
    let mut r = 1;
    let mut n = n / 7;
    n = n >> 1;
    while n > 0 {
        r = r << 1;
        n = n >> 1;
    }
    r * 7
}

fn draw_invaders(img: &mut RgbImage, x: u32, y: u32, w: u32, h: u32) {
    if w < 7 || h < 7 {
        return;
    }

    let d = square_ceil_7(cmp::min(w, h));

    let mut rng : ThreadRng = rand::thread_rng();
    let corner_range: Range<u8> = Range::new(0, 4);

    match corner_range.ind_sample(&mut rng) {
        0 => {
            /* top left */
            draw_invader_factor(img, x, y, d);
            draw_invaders(img, x+d, y, w-d, h); /* right */
            draw_invaders(img, x, y+d, d, h-d); /* bottom */
        }
        1 => {
            /* top-right */
            draw_invader_factor(img, x+w-d, y, d);
            draw_invaders(img, x, y+d, w, h-d); /* bottom */
            draw_invaders(img, x, y, w-d, d); /* left */
        }
        2 => {
            /* bottom-right */
            draw_invader_factor(img, x+w-d, y+h-d, d);
            draw_invaders(img, x, y, w-d, h); /* left */
            draw_invaders(img, x+w-d, y, d, h-d); /* top */
        }
        _ => {
            /* bottom-left */
            draw_invader_factor(img, x, y+h-d, d);
            draw_invaders(img, x, y, w, h-d); /* top */
            draw_invaders(img, x+d, y+h-d, w-d, d); /* right */
        }
    }
}

fn generate_image(path: &Path, width: u32, height: u32) {

    let mut img = RgbImage::new(width, height);

    if width > 3 && height > 3 {
        draw_invaders(&mut img, 3, 3, width - 3, height - 3);
    }

    let _ = img.save(path);
}

const USAGE: &'static str = "
Invaders background generator.

Usage: invaders [options] FILE
       invaders -g GEOM <kind> FILE
       invaders --geometry GEOM <kind> FILE
       invaders -h | --help
       invaders -v | --version

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
    let version = env!("CARGO_PKG_VERSION").to_owned();
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.version(Some(version)).parse())
                      .unwrap_or_else(|e| e.exit());
    let geometry = args.get_str("--geometry");
    let geometry = geometry_parse(&geometry);

    let path = args.get_str("FILE");
    let path = Path::new(path);

    generate_image(path, geometry.0, geometry.1);
}
