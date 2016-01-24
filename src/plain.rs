use image::{
    RgbImage,
    Rgb,
};
use maze::{
    CellKind,
    Coord,
    Maze,
    Rendering,
};
use color_scaling::scale_rgb;

pub fn draw_cell_plain<T: Rendering>(renderer: &T, img: &mut RgbImage,
                                     c: &Coord, p: &Rgb<u8>) {
    let tile_size = renderer.tile_size();
    for i in 0..tile_size {
        for j in 0..tile_size {
            img.put_pixel((c.x * tile_size + i) as u32,
                          (c.y * tile_size + j) as u32,
                          *p);
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
    fn draw_cell(&self, _maze: &Maze, img: &mut RgbImage, c: &Coord,
                 cell_kind: CellKind) {
        match cell_kind {
            CellKind::PathKind(f) => {
                let color = scale_rgb(&self.path_color_start,
                                      &self.path_color_end,
                                      f).unwrap();
                draw_cell_plain(self, img, c, &color);
            },
            _ => {
                draw_cell_plain(self, img, c, &self.wall_color);
            },
        }
    }
}
