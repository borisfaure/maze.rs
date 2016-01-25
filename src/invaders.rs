use image::{
    RgbImage,
    Rgb,
};
use std;
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
use plain::draw_cell_plain;

const TILE_SIZE : u32 = 7;

pub struct RendererInvaders {
    pub invader_color: Rgb<u8>,
    pub wall_color: Rgb<u8>,
}

impl RendererInvaders {
    fn draw_invader(&self, img: &mut RgbImage, c: &Coord) {
        let mut rng : ThreadRng = rand::thread_rng();
        let invader_range: Range<u16> = Range::new(0, std::u16::MAX);
        let invader_nbr = invader_range.ind_sample(&mut rng);

        /* draw background */
        for i in 0..TILE_SIZE {
            for j in 0..TILE_SIZE {
                img.put_pixel(c.x as u32 * TILE_SIZE + i,
                              c.y as u32 * TILE_SIZE + j,
                              self.wall_color);
            }
        }

        for i in 1..4 {
            for j in 1..6 {
                if invader_nbr & ((1 << (i*j)) as u16) > 0 {
                    img.put_pixel(c.x as u32 * TILE_SIZE + i,
                                  c.y as u32 * TILE_SIZE + j,
                                  self.invader_color);
                    img.put_pixel(c.x as u32 * TILE_SIZE + (6-i),
                                  c.y as u32 * TILE_SIZE + j,
                                  self.invader_color);
                }
            }
        }
    }
}

impl Rendering for RendererInvaders{
    fn tile_size(&self) -> usize {
        TILE_SIZE as usize
    }
    fn draw_cell(&self, _maze: &Maze, img: &mut RgbImage, c: &Coord,
                 cell_kind: CellKind) {
        match cell_kind {
            CellKind::PathKind(_) => {
                self.draw_invader(img, c)
            },
            CellKind::WallKind => {
                draw_cell_plain(self, img, c, &self.wall_color);
            },
            _ => {
            }
        }
    }
    fn draw_cell_gif(&self, _maze: &Maze, _img_geom: &super::Geometry,
                     _buffer: &mut Vec<u8>, _c: &Coord, _cell_kind: CellKind) {
        panic!("unimplemented");
    }
    fn get_gif_palette(&self) -> Vec<u8> {
        panic!("unimplemented");
    }
}
