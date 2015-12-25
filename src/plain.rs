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
    pub path_color: Rgb<u8>,
    pub wall_color: Rgb<u8>,
}
impl Rendering for RendererPlain {
    fn tile_size(&self) -> usize {
        4
    }
    fn draw_cell(&self, _maze: &Maze, img: &mut RgbImage, c: &Coord,
                 cell_kind: CellKind) {
        match cell_kind {
            CellKind::PathKind(_) => {
                draw_cell_plain(self, img, c, &self.path_color);
            },
            CellKind::WallKind => {
                draw_cell_plain(self, img, c, &self.wall_color);
            },
            _ => {
            }
        }
    }
}
