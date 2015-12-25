use image::{
    RgbImage,
    Rgb,
};
use rand;
use rand::ThreadRng;
use rand::distributions::{
    IndependentSample,
    Range
};
use maze::{
    CellKind,
    Coord,
    Maze,
    Rendering,
};


const TILE_SIZE : u32 = 5;

const TILES_DARK: [ [Rgb<u8>; 3]; 5] = [
[ Rgb{data: [31, 31, 31]}, Rgb{data: [112, 112, 122]}, Rgb{data: [92, 92, 92]}],
[ Rgb{data: [31, 31, 31]}, Rgb{data: [ 95,  95,  95]}, Rgb{data: [79, 79, 79]}],
[ Rgb{data: [31, 31, 31]}, Rgb{data: [ 85,  85,  85]}, Rgb{data: [71, 71, 71]}],
[ Rgb{data: [31, 31, 31]}, Rgb{data: [ 63,  63,  63]}, Rgb{data: [55, 55, 55]}],
[ Rgb{data: [31, 31, 31]}, Rgb{data: [ 49,  49,  49]}, Rgb{data: [44, 44, 44]}]
];

const TILES_LIGHT: [ [Rgb<u8>; 3]; 5] = [
[ Rgb{data: [254, 254, 254]}, Rgb{data: [199, 199, 199]}, Rgb{data: [210, 210, 210]}],
[ Rgb{data: [254, 254, 254]}, Rgb{data: [206, 206, 206]}, Rgb{data: [220, 220, 220]}],
[ Rgb{data: [254, 254, 254]}, Rgb{data: [216, 216, 216]}, Rgb{data: [230, 230, 230]}],
[ Rgb{data: [254, 254, 254]}, Rgb{data: [225, 225, 225]}, Rgb{data: [240, 240, 240]}],
[ Rgb{data: [254, 254, 254]}, Rgb{data: [240, 240, 240]}, Rgb{data: [245, 245, 245]}]
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

fn draw_tile_rand(img: &mut RgbImage, x: u32, y: u32, tiles: &[[Rgb<u8>; 3]; 5]) {
    let between : Range<u8> = Range::new(0, tiles.len() as u8);
    let mut rng : ThreadRng = rand::thread_rng();
    let idx = between.ind_sample(&mut rng) as usize;
    draw_tile(img, x, y, &tiles[idx]);
}


pub struct RendererMosaic {
    pub is_inverted: bool,
}

impl Rendering for RendererMosaic {
    fn tile_size(&self) -> usize {
        TILE_SIZE as usize
    }
    fn draw_cell(&self, _maze: &Maze, img: &mut RgbImage, c: &Coord,
                 cell_kind: CellKind) {
        match cell_kind {
            CellKind::PathKind(_) => {
                if self.is_inverted {
                    draw_tile_rand(img,
                                   c.x as u32 * TILE_SIZE,
                                   c.y as u32 * TILE_SIZE,
                                   &TILES_DARK);
                } else {
                    draw_tile_rand(img,
                                   c.x as u32 * TILE_SIZE,
                                   c.y as u32 * TILE_SIZE,
                                   &TILES_LIGHT);
                }
            },
            CellKind::WallKind => {
                if self.is_inverted {
                    draw_tile_rand(img,
                                   c.x as u32 * TILE_SIZE,
                                   c.y as u32 * TILE_SIZE,
                                   &TILES_LIGHT);
                } else {
                    draw_tile_rand(img,
                                   c.x as u32 * TILE_SIZE,
                                   c.y as u32 * TILE_SIZE,
                                   &TILES_DARK);
                }
            },
            _ => {
            }
        }
    }
}
