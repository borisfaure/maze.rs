extern crate docopt;
extern crate image;
extern crate rand;

mod invaders;
mod mosaic;

#[derive(Debug,Clone)]
pub struct Geometry{
    width: usize,
    height: usize,
}

use docopt::Docopt;
use std::path;
use std::str::FromStr;

mod maze {

use std::path;
use image::{
    RgbImage,
    Rgb
};
use rand::{
    random,
};

#[derive(Debug)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}
type Path = Coord;
type Wall = Coord;

#[derive(Debug,Clone)]
pub enum CellKind {
    WallKind,
    PathKind,
    Undefined
}

struct Maze {
    geometry: super::Geometry,
    grid: Vec<CellKind>,
    vertical_bias: f64,
}

#[derive(Debug)]
enum Direction {
    Up,
    Down,
    Left,
    Right
}
fn opposite(dir: &Direction) -> Direction {
    match *dir {
        Direction::Up    => Direction::Down,
        Direction::Down  => Direction::Up,
        Direction::Left  => Direction::Right,
        Direction::Right => Direction::Left,
    }
}

fn add_walls(vwalls: &mut Vec<Wall>, hwalls: &mut Vec<Wall>, new_walls: Vec<Coord>) {
    for w in new_walls {
        if w.x % 2 == 0 {
            vwalls.push(w);
        } else if w.y % 2 == 0 {
            hwalls.push(w);
        }
    }
}

fn pop_random_wall(vwalls: &mut Vec<Wall>,
                   hwalls: &mut Vec<Wall>,
                   vertical_bias: f64) -> Wall {
    let r : usize = random::<usize>();
    match (vwalls.len(), hwalls.len()) {
        (0, len) => {
            let pos = r % len;
            hwalls.swap_remove(pos)
        },
        (len, 0) => {
            let pos = r % len;
            vwalls.swap_remove(pos)
        },
        (vlen, hlen) => {
            let f = random::<f64>();
            if f < vertical_bias {
                let pos = r % vlen;
                vwalls.swap_remove(pos)
            } else {
                let pos = r % hlen;
                hwalls.swap_remove(pos)
            }
        }
    }
}


impl Maze {
    fn new(g: &super::Geometry, vertical_bias: f64) -> Maze {
        let mut m = Maze {
            geometry: g.clone(),
            grid: Vec::new(),
            vertical_bias: vertical_bias,
        };
        m.grid.reserve(g.width * g.height);
        for _ in 0..(g.width * g.height) {
            m.grid.push(CellKind::Undefined);
        }
        m
    }

    fn cell_kind(&self, c: &Coord) -> CellKind {
        if c.x >= self.geometry.width || c.y >= self.geometry.height {
            CellKind::Undefined
        } else {
            self.grid[c.y * self.geometry.width + c.x].clone()
        }
    }

    fn set_path(&mut self, c: &Coord) {
        if let CellKind::PathKind = self.cell_kind(&c) {
            return;
        }
        self.grid[c.y * self.geometry.width + c.x] = CellKind::PathKind;
    }

    fn set_wall(&mut self, c: &Coord) {
        if let CellKind::WallKind = self.cell_kind(&c) {
            return;
        }
        self.grid[c.y * self.geometry.width + c.x] = CellKind::WallKind;
    }

    fn set_walls(&mut self, walls: &Vec<Coord>) {
        for w in walls {
            self.set_wall(&w as &Wall);
        }
    }


    fn get_undefined_cells_around(&mut self, c: &Coord) -> Vec<Coord> {
        let dirs = vec![Direction::Up, Direction::Down,
                        Direction::Left, Direction::Right];
        let mut v : Vec<Coord> = Vec::new();
        for d in dirs {
            let o = self.get_coord_next(&c, &d);
            if let Some(c) = o {
                if let CellKind::Undefined = self.cell_kind(&c) {
                    v.push(c);
                }
            }
        }
        v
    }

    fn get_coord_up(&self, c: &Coord) -> Option<Coord> {
        if c.y == 0 {
            None
        } else {
            Some(Coord{x: c.x, y: c.y - 1})
        }
    }
    fn get_coord_down(&self, c: &Coord) -> Option<Coord> {
        if c.y >= self.geometry.height - 1 {
            None
        } else {
            Some(Coord{x: c.x, y: c.y + 1})
        }
    }
    fn get_coord_left(&self, c: &Coord) -> Option<Coord> {
        if c.x == 0 {
            None
        } else {
            Some(Coord{x: c.x - 1, y: c.y})
        }
    }
    fn get_coord_right(&self, c: &Coord) -> Option<Coord> {
        if c.x >= self.geometry.width - 1 {
            None
        } else {
            Some(Coord{x: c.x + 1, y: c.y})
        }
    }

    fn get_coord_next(&self, c: &Coord, dir: &Direction) -> Option<Coord> {
        match *dir {
            Direction::Up => self.get_coord_up(c),
            Direction::Down => self.get_coord_down(c),
            Direction::Left => self.get_coord_left(c),
            Direction::Right => self.get_coord_right(c),
        }
    }

    fn get_random_wall_direction(&self, w: &Wall) -> Option<Direction> {
        if w.x % 2 == 0 {
            match random::<u8>() % 2 {
                0 => Some(Direction::Up),
                _ => Some(Direction::Down),
            }
        } else if w.y % 2 == 0 {
            match random::<u8>() % 2 {
                0 => Some(Direction::Left),
                _ => Some(Direction::Right),
            }
        } else {
            None
        }

    }

/* Randomized Prim's algorithm
 *
 * This algorithm is a randomized version of Prim's algorithm.
 *
 *  Start with a grid full of walls.
 *  Pick a cell, mark it as part of the maze. Add the walls of the cell to the
 *  wall list.
 *  While there are walls in the list:
 *      Pick a random wall from the list and a random direction. If the cell
 *      in that direction isn't in the maze yet:
 *          Make the wall a passage and mark the cell on the opposite side as
 *          part of the maze.
 *          Add the neighboring walls of the cell to the wall list.
 *      Remove the wall from the list.
 *
 * It will usually be relatively easy to find the way to the starting cell,
 * but hard to find the way anywhere else.
 *
 * Note that simply running classical Prim's on a graph with random edge
 * weights would create mazes stylistically identical to Kruskal's, because
 * they are both minimal spanning tree algorithms. Instead, this algorithm
 * introduces stylistic variation because the edges closer to the starting
 * point have a lower effective weight.
 *
 * Modified version
 * Although the classical Prim's algorithm keeps a list of edges, for maze
 * generation we could instead maintain a list of adjacent cells. If the
 * randomly chosen cell has multiple edges that connect it to the existing
 * maze, select one of these edges at random. This will tend to branch
 * slightly more than the edge-based version above.
 */

    fn randomized_prim(&mut self) {
        let mut vwalls : Vec<Wall> = Vec::new();
        let mut hwalls : Vec<Wall> = Vec::new();
        let start = Coord{x:0, y:0};
        self.set_path(&start);
        let new_walls = self.get_undefined_cells_around(&start);
        self.set_walls(&new_walls);
        add_walls(&mut vwalls, &mut hwalls, new_walls);

        while !vwalls.is_empty() || !hwalls.is_empty() {
            /* Pick a random wall from the list */
            let w = pop_random_wall(&mut vwalls, &mut hwalls, self.vertical_bias);
            if let Some(dir) = self.get_random_wall_direction(&w) {
                let o1 = self.get_coord_next(&w as &Coord, &dir);
                let o2 = self.get_coord_next(&w as &Coord, &opposite(&dir));
                match (o1, o2) {
                    (Some(c1), Some(c2)) => {
                        if let (CellKind::PathKind, CellKind::PathKind)
                            = (self.cell_kind(&c1), self.cell_kind(&c2)) {
                                continue;
                        }
                        self.set_path(&w);
                        self.set_path(&c2);
                        self.set_path(&c1);

                        let walls = self.get_undefined_cells_around(&w);
                        self.set_walls(&walls);
                        add_walls(&mut vwalls, &mut hwalls, walls);

                        let walls = self.get_undefined_cells_around(&c1);
                        self.set_walls(&walls);
                        add_walls(&mut vwalls, &mut hwalls, walls);

                        let walls = self.get_undefined_cells_around(&c2);
                        self.set_walls(&walls);
                        add_walls(&mut vwalls, &mut hwalls, walls);
                    },
                    (Some(c), _) => {
                        self.set_path(&w);
                        self.set_path(&c);

                        let  walls = self.get_undefined_cells_around(&w);
                        self.set_walls(&walls);
                        add_walls(&mut vwalls, &mut hwalls, walls);

                        let walls = self.get_undefined_cells_around(&c);
                        self.set_walls(&walls);
                        add_walls(&mut vwalls, &mut hwalls, walls);

                    },
                    (_, Some(c)) => {
                        self.set_path(&w);
                        self.set_path(&c);

                        let walls = self.get_undefined_cells_around(&w);
                        self.set_walls(&walls);
                        add_walls(&mut vwalls, &mut hwalls, walls);

                        let walls = self.get_undefined_cells_around(&c);
                        self.set_walls(&walls);
                        add_walls(&mut vwalls, &mut hwalls, walls);
                    },
                    (_, _) => {
                    }
                }
            }
        }
        for y in (0..self.geometry.height).filter(|&v| v % 2 == 1) {
            for x in (0..self.geometry.width).filter(|&v| v % 2 == 1) {
                if let CellKind::Undefined= self.cell_kind(&Coord{x:x, y:y}) {
                    self.grid[y * self.geometry.width + x] = CellKind::WallKind;
                }
            }
        }
    }

    fn draw<T: ?Sized + Rendering>(&mut self, renderer: &T) -> RgbImage {
        let g = image_geometry(renderer, &self.geometry);
        let mut img = RgbImage::new(g.width as u32, g.height as u32);

        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                let c = Coord{x: x, y: y};
                renderer.draw_cell(&mut img, &c, self.cell_kind(&c));
            }
        }
        img
    }
}


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

pub trait Rendering {
    fn tile_size(&self) -> usize;
    fn draw_cell(&self, &mut RgbImage, &Coord, CellKind);
}


pub struct RendererPlain{
    pub path_color: Rgb<u8>,
    pub wall_color: Rgb<u8>,
}
impl Rendering for RendererPlain{
    fn tile_size(&self) -> usize {
        4
    }
    fn draw_cell(&self, img: &mut RgbImage, c: &Coord,
                        cell_kind: CellKind) {
        match cell_kind {
            CellKind::PathKind => {
                draw_cell_plain(self, img, c, &self.path_color)
            },
            CellKind::WallKind => {
                draw_cell_plain(self, img, c, &self.wall_color);
            },
            _ => {
            }
        }
    }
}

fn grid_geometry<T: ?Sized + Rendering>(renderer: &T, g: &super::Geometry) -> super::Geometry {
    let tile_size = renderer.tile_size();
    super::Geometry{
        width: g.width / tile_size,
        height: g.height / tile_size,
    }
}

fn image_geometry<T: ?Sized + Rendering>(renderer: &T, g: &super::Geometry) -> super::Geometry {
    let tile_size = renderer.tile_size();
    super::Geometry{
        width:  g.width * tile_size,
        height: g.height * tile_size,
    }
}

pub fn generate_image<T: ?Sized + Rendering>(path: &path::Path,
                                             g: super::Geometry,
                                             renderer: &T,
                                             vertical_bias: f64) {
    let grid_geometry = grid_geometry(renderer, &g);

    let mut maze = Maze::new(&grid_geometry, vertical_bias);

    maze.randomized_prim();

    let img = maze.draw(renderer);
    let _ = img.save(path);
}
}


const USAGE: &'static str = "
Maze background generator.

Usage: maze [options] FILE
       maze --geometry GEOM FILE
       maze -g GEOM FILE
       maze --rendering=RENDERING FILE
       maze -r RENDERING FILE
       maze --vertical-bias BIAS
       maze -b BIAS
       maze -h | --help
       maze -v | --version

Options:
    -h, --help                                    Show this message
    -v, --version                                 Show the version
    -g=<WIDTHxHEIGHT>, --geometry=<WIDTHxHEIGHT>  Geometry of the image to generate [default: 100x100]
    -r=RENDERING, --rendering=RENDERING           Rendering mode. Valid values are: plain, invaders, mosaic. [default: plain]
    -b=BIAS, --vertical-bias=BIAS                 Vertical Bias. Larger than 0.5, the maze will then to be more vertical. Lower than 0.5, will tend to be more horizontal. [default: 0.5]
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

fn rendering_parse(rendering: &str) -> Box<maze::Rendering> {
    match rendering {
        "plain" => {
            Box::new(maze::RendererPlain {
                path_color: image::Rgb{data:[253, 246, 227]},
                wall_color: image::Rgb{data:[  7,  54,  66]},
            })
        },
        "invaders" => {
            Box::new(invaders::RendererInvaders{
                invader_color:    image::Rgb{data:[253, 246, 227]},
                wall_color: image::Rgb{data:[  7,  54,  66]},
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

fn main() {
    let version = env!("CARGO_PKG_VERSION").to_owned();
    let args = Docopt::new(USAGE)
                      .and_then(|dopt| dopt.version(Some(version)).parse())
                      .unwrap_or_else(|e| e.exit());
    let geometry = args.get_str("--geometry");
    let geometry = geometry_parse(&geometry);

    let rendering = args.get_str("--rendering");
    let rendering = rendering_parse(&rendering);

    let path = args.get_str("FILE");
    let path = path::Path::new(path);

    let vertical_bias = args.get_str("--vertical-bias");
    let vertical_bias = vertical_bias_parse(&vertical_bias);

    maze::generate_image(path, geometry, &*rendering, vertical_bias);
}
