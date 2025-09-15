use crate::maze::{CellKind, Coord, Maze, Rendering};
use image::{Rgb, RgbImage};
use rand::distr::{Distribution, Uniform};

const TILE_SIZE: u32 = 5;

const TILES_DARK: [[Rgb<u8>; 3]; 5] = [
    [Rgb([31, 31, 31]), Rgb([112, 112, 122]), Rgb([92, 92, 92])],
    [Rgb([31, 31, 31]), Rgb([95, 95, 95]), Rgb([79, 79, 79])],
    [Rgb([31, 31, 31]), Rgb([85, 85, 85]), Rgb([71, 71, 71])],
    [Rgb([31, 31, 31]), Rgb([63, 63, 63]), Rgb([55, 55, 55])],
    [Rgb([31, 31, 31]), Rgb([49, 49, 49]), Rgb([44, 44, 44])],
];

const TILES_LIGHT: [[Rgb<u8>; 3]; 5] = [
    [
        Rgb([254, 254, 254]),
        Rgb([199, 199, 199]),
        Rgb([210, 210, 210]),
    ],
    [
        Rgb([254, 254, 254]),
        Rgb([206, 206, 206]),
        Rgb([220, 220, 220]),
    ],
    [
        Rgb([254, 254, 254]),
        Rgb([216, 216, 216]),
        Rgb([230, 230, 230]),
    ],
    [
        Rgb([254, 254, 254]),
        Rgb([225, 225, 225]),
        Rgb([240, 240, 240]),
    ],
    [
        Rgb([254, 254, 254]),
        Rgb([240, 240, 240]),
        Rgb([245, 245, 245]),
    ],
];

fn draw_tile(img: &mut RgbImage, x: u32, y: u32, tile: &[Rgb<u8>; 3]) {
    img.put_pixel(x, y, tile[0]);
    img.put_pixel(x, y + 1, tile[0]);
    img.put_pixel(x, y + 2, tile[0]);
    img.put_pixel(x, y + 3, tile[0]);
    img.put_pixel(x, y + 4, tile[0]);
    img.put_pixel(x + 1, y, tile[0]);
    img.put_pixel(x + 2, y, tile[0]);
    img.put_pixel(x + 3, y, tile[0]);
    img.put_pixel(x + 4, y, tile[0]);

    img.put_pixel(x + 1, y + 1, tile[1]);
    img.put_pixel(x + 1, y + 2, tile[1]);
    img.put_pixel(x + 1, y + 3, tile[1]);
    img.put_pixel(x + 1, y + 4, tile[1]);
    img.put_pixel(x + 2, y + 1, tile[1]);
    img.put_pixel(x + 3, y + 1, tile[1]);
    img.put_pixel(x + 4, y + 1, tile[1]);
    img.put_pixel(x + 2, y + 4, tile[1]);
    img.put_pixel(x + 3, y + 4, tile[1]);
    img.put_pixel(x + 4, y + 2, tile[1]);
    img.put_pixel(x + 4, y + 3, tile[1]);
    img.put_pixel(x + 4, y + 4, tile[1]);

    img.put_pixel(x + 2, y + 2, tile[1]);
    img.put_pixel(x + 2, y + 3, tile[1]);
    img.put_pixel(x + 3, y + 2, tile[1]);
    img.put_pixel(x + 3, y + 3, tile[1]);
}

fn draw_tile_rand(img: &mut RgbImage, x: u32, y: u32, tiles: &[[Rgb<u8>; 3]; 5]) {
    let between: Uniform<u8> =
        Uniform::new(0, tiles.len() as u8).expect("cannot create uniform random distribution");
    let mut rng = rand::rng();
    let idx = between.sample(&mut rng) as usize;
    draw_tile(img, x, y, &tiles[idx]);
}

pub struct RendererMosaic {
    pub is_inverted: bool,
}

impl Rendering for RendererMosaic {
    fn tile_size(&self) -> usize {
        TILE_SIZE as usize
    }
    fn draw_cell(&self, _maze: &Maze, img: &mut RgbImage, c: &Coord, cell_kind: CellKind) {
        match cell_kind {
            CellKind::PathKind(_) => {
                if self.is_inverted {
                    draw_tile_rand(
                        img,
                        c.x as u32 * TILE_SIZE,
                        c.y as u32 * TILE_SIZE,
                        &TILES_DARK,
                    );
                } else {
                    draw_tile_rand(
                        img,
                        c.x as u32 * TILE_SIZE,
                        c.y as u32 * TILE_SIZE,
                        &TILES_LIGHT,
                    );
                }
            }
            CellKind::WallKind => {
                if self.is_inverted {
                    draw_tile_rand(
                        img,
                        c.x as u32 * TILE_SIZE,
                        c.y as u32 * TILE_SIZE,
                        &TILES_LIGHT,
                    );
                } else {
                    draw_tile_rand(
                        img,
                        c.x as u32 * TILE_SIZE,
                        c.y as u32 * TILE_SIZE,
                        &TILES_DARK,
                    );
                }
            }
            _ => {}
        }
    }
    fn draw_cell_gif(
        &self,
        _maze: &Maze,
        _img_geom: &super::Geometry,
        _buffer: &mut Vec<u8>,
        _c: &Coord,
        _cell_kind: CellKind,
    ) {
        panic!("unimplemented");
    }
    fn get_gif_palette(&self) -> Vec<u8> {
        panic!("unimplemented");
    }
}
