use std::path::Path;
use image::{
    //ImageBuffer,
    RgbImage,
    Rgb,
};
use rand;
use rand::ThreadRng;
use rand::distributions::{
    IndependentSample,
    Range
};

const TILE_SIZE : u32 = 5;

const TILES: [ [Rgb<u8>; 3]; 5] = [
[ Rgb{data: [31, 31, 31]}, Rgb{data: [112, 112, 122]}, Rgb{data: [92, 92, 92]}],
[ Rgb{data: [31, 31, 31]}, Rgb{data: [ 95,  95,  95]}, Rgb{data: [79, 79, 79]}],
[ Rgb{data: [31, 31, 31]}, Rgb{data: [ 85,  85,  85]}, Rgb{data: [71, 71, 71]}],
[ Rgb{data: [31, 31, 31]}, Rgb{data: [ 63,  63,  63]}, Rgb{data: [55, 55, 55]}],
[ Rgb{data: [31, 31, 31]}, Rgb{data: [ 49,  49,  49]}, Rgb{data: [44, 44, 44]}]
];


fn draw_tile(img: &mut RgbImage, x: u32, y: u32, tile: &[Rgb<u8>;3]) {
    img.put_pixel(x, y, tile[0]);
    img.put_pixel(x, y+1, tile[0]);
    img.put_pixel(x, y+2, tile[0]);
    img.put_pixel(x, y+3, tile[0]);
    img.put_pixel(x, y+4, tile[0]);
    img.put_pixel(x+1, y, tile[0]);
    img.put_pixel(x+2, y, tile[0]);
    img.put_pixel(x+3, y, tile[0]);
    img.put_pixel(x+4, y, tile[0]);

    img.put_pixel(x+1, y+1, tile[1]);
    img.put_pixel(x+1, y+2, tile[1]);
    img.put_pixel(x+1, y+3, tile[1]);
    img.put_pixel(x+1, y+4, tile[1]);
    img.put_pixel(x+2, y+1, tile[1]);
    img.put_pixel(x+3, y+1, tile[1]);
    img.put_pixel(x+4, y+1, tile[1]);
    img.put_pixel(x+2, y+4, tile[1]);
    img.put_pixel(x+3, y+4, tile[1]);
    img.put_pixel(x+4, y+2, tile[1]);
    img.put_pixel(x+4, y+3, tile[1]);
    img.put_pixel(x+4, y+4, tile[1]);

    img.put_pixel(x+2, y+2, tile[1]);
    img.put_pixel(x+2, y+3, tile[1]);
    img.put_pixel(x+3, y+2, tile[1]);
    img.put_pixel(x+3, y+3, tile[1]);
}

fn draw_tile_rand(img: &mut RgbImage, x: u32, y: u32) {
    let between : Range<u8> = Range::new(0, TILES.len() as u8);
    let mut rng : ThreadRng = rand::thread_rng();
    let idx = between.ind_sample(&mut rng) as usize;
    draw_tile(img, x, y, &TILES[idx]);
}

pub fn generate_image(path: &Path, width: u32, height: u32) {

    let mut img = RgbImage::new(width, height);

    for y in 0..(height/TILE_SIZE) {
        for x in 0..(width/TILE_SIZE) {
            draw_tile_rand(&mut img, x*TILE_SIZE ,y*TILE_SIZE);
        }
    }
    let _ = img.save(path);
}
