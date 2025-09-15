use crate::maze::{CellKind, Coord, Maze, Rendering};
use color_scaling::scale_rgb;
use image::{Rgb, RgbImage};

pub fn draw_cell_plain<T: Rendering>(renderer: &T, img: &mut RgbImage, c: &Coord, p: &Rgb<u8>) {
    let tile_size = renderer.tile_size();
    for i in 0..tile_size {
        for j in 0..tile_size {
            img.put_pixel(
                (c.x * tile_size + i) as u32,
                (c.y * tile_size + j) as u32,
                *p,
            );
        }
    }
}

pub fn draw_cell_plain_gif<T: Rendering>(
    renderer: &T,
    width: usize,
    buffer: &mut [u8],
    c: &Coord,
    color_idx: u8,
) {
    let tile_size = renderer.tile_size();
    for i in 0..tile_size {
        for j in 0..tile_size {
            buffer[(c.x * tile_size + i) + (c.y * tile_size + j) * width] = color_idx;
        }
    }
}

pub struct RendererPlain {
    pub path_color_start: Rgb<u8>,
    pub path_color_end: Rgb<u8>,
    pub wall_color: Rgb<u8>,
}
impl Rendering for RendererPlain {
    fn tile_size(&self) -> usize {
        4
    }
    fn draw_cell(&self, _maze: &Maze, img: &mut RgbImage, c: &Coord, cell_kind: CellKind) {
        match cell_kind {
            CellKind::PathKind(f) => {
                let color = scale_rgb(&self.path_color_start, &self.path_color_end, f).unwrap();
                draw_cell_plain(self, img, c, &color);
            }
            _ => {
                draw_cell_plain(self, img, c, &self.wall_color);
            }
        }
    }
    fn draw_cell_gif(
        &self,
        _maze: &Maze,
        img_geom: &super::Geometry,
        buffer: &mut Vec<u8>,
        c: &Coord,
        cell_kind: CellKind,
    ) {
        match cell_kind {
            CellKind::PathKind(f) => {
                if self.path_color_start != self.path_color_end
                    && !f.is_nan()
                    && (0f64..1f64).contains(&f)
                {
                    panic!("not implemented");
                } else {
                    draw_cell_plain_gif(self, img_geom.width, buffer, c, 1);
                }
            }
            _ => {
                draw_cell_plain_gif(self, img_geom.width, buffer, c, 0);
            }
        }
    }
    fn get_gif_palette(&self) -> Vec<u8> {
        if self.path_color_start != self.path_color_end {
            panic!("not implemented");
        } else {
            vec![
                self.wall_color[0],
                self.wall_color[1],
                self.wall_color[2],
                self.path_color_start[0],
                self.path_color_start[1],
                self.path_color_start[2],
            ]
        }
    }
}
