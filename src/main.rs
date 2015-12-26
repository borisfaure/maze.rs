extern crate docopt;
extern crate image;
extern crate rand;
#[macro_use]
extern crate debug_macros;
extern crate color_scaling;
extern crate read_color;

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

mod maze {

use std::path;
use image::{
    RgbImage,
};
use rand::{
    random,
};

#[derive(Debug,Clone)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}
type Path = Coord;
type Wall = Coord;

#[derive(Debug,Clone)]
pub enum CellKind {
    WallKind,
    PathKind(f64),
    Undefined
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


pub struct Maze {
    geometry: super::Geometry,
    grid: Vec<CellKind>,
    vertical_bias: f64,
    origin: Coord,
    end: Coord,
    len: f64,
}

impl Maze {
    fn new(g: &super::Geometry, vertical_bias: f64, origin: &super::Origin) -> Maze {
        let mut m = Maze {
            geometry: g.clone(),
            grid: Vec::new(),
            vertical_bias: vertical_bias,
            origin: Coord{x: 0, y: 0},
            len: 0_f64,
            end: Coord{x: g.width-1 , y: g.height-1},
        };
        m.grid.reserve(g.width * g.height);
        for _ in 0..(g.width * g.height) {
            m.grid.push(CellKind::Undefined);
        }
        m.origin = m.origin_to_coord(origin);
        m
    }

    pub fn origin(&self) -> Coord {
        self.origin.clone()
    }
    pub fn end(&self) -> Coord {
        self.end.clone()
    }
    pub fn len(&self) -> f64 {
        self.len
    }

    fn origin_to_coord(&self, origin: &super::Origin) -> Coord {
        let x = origin.x * (self.geometry.width as f64);
        let y = origin.y * (self.geometry.height as f64);
        Coord {
            x: (x as usize) & !1,
            y: (y as usize) & !1,
        }
    }

    fn cell_kind(&self, c: &Coord) -> CellKind {
        if c.x >= self.geometry.width || c.y >= self.geometry.height {
            CellKind::Undefined
        } else {
            self.grid[c.y * self.geometry.width + c.x].clone()
        }
    }

    fn set_path(&mut self, c: &Coord, d: f64) {
        if let CellKind::PathKind(_) = self.cell_kind(&c) {
            return;
        }
        self.grid[c.y * self.geometry.width + c.x] = CellKind::PathKind(d);
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

        let start = self.origin();
        self.set_path(&start, 0.0_f64);
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
                        if let CellKind::PathKind(d) = self.cell_kind(&c1) {
                            if let CellKind::PathKind(_) = self.cell_kind(&c2) {
                                continue;
                            }
                            self.set_path(&w, d + 1_f64);
                            if let CellKind::Undefined = self.cell_kind(&c2) {
                                self.set_path(&c2, d + 2_f64);

                                let walls = self.get_undefined_cells_around(&c2);
                                self.set_walls(&walls);
                                add_walls(&mut vwalls, &mut hwalls, walls);

                                if self.len < d + 2_f64 {
                                    self.len = d + 2_f64;
                                    self.end = c2;
                                }
                            }
                            let walls = self.get_undefined_cells_around(&w);
                            self.set_walls(&walls);
                            add_walls(&mut vwalls, &mut hwalls, walls);

                            if self.len < d + 1_f64 {
                                self.len = d + 1_f64;
                                self.end = w;
                            }
                        } else if let CellKind::PathKind(d) = self.cell_kind(&c2) {
                            self.set_path(&w, d + 1_f64);
                            if let CellKind::Undefined = self.cell_kind(&c1) {
                                self.set_path(&c1, d + 2_f64);

                                let walls = self.get_undefined_cells_around(&c1);
                                self.set_walls(&walls);
                                add_walls(&mut vwalls, &mut hwalls, walls);

                                if self.len < d + 2_f64 {
                                    self.len = d + 2_f64;
                                    self.end = c1;
                                }
                            }
                            let walls = self.get_undefined_cells_around(&w);
                            self.set_walls(&walls);
                            add_walls(&mut vwalls, &mut hwalls, walls);

                            if self.len < d + 1_f64 {
                                self.len = d + 1_f64;
                                self.end = w;
                            }
                        }
                    },
                    (Some(c), _) => {
                        if let CellKind::PathKind(d) = self.cell_kind(&c) {
                            self.set_path(&w, d + 1_f64);

                            let walls = self.get_undefined_cells_around(&w);
                            self.set_walls(&walls);
                            add_walls(&mut vwalls, &mut hwalls, walls);

                            if self.len < d + 1_f64 {
                                self.len = d + 1_f64;
                                self.end = w;
                            }
                        }
                    },
                    (_, Some(c)) => {
                        if let CellKind::PathKind(d) = self.cell_kind(&c) {
                            self.set_path(&w, d + 1_f64);

                            let walls = self.get_undefined_cells_around(&w);
                            self.set_walls(&walls);
                            add_walls(&mut vwalls, &mut hwalls, walls);

                            if self.len < d + 1_f64 {
                                self.len = d + 1_f64;
                                self.end = w;
                            }
                        }
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
        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                if let CellKind::PathKind(f) = self.cell_kind(&Coord{x:x, y:y}) {
                    self.grid[y * self.geometry.width + x] =
                        CellKind::PathKind(f / self.len);
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
                renderer.draw_cell(&self, &mut img, &c, self.cell_kind(&c));
            }
        }
        img
    }
}


pub trait Rendering {
    fn tile_size(&self) -> usize;
    fn draw_cell(&self, &Maze, &mut RgbImage, &Coord, CellKind);
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
                                             vertical_bias: f64,
                                             origin: super::Origin) {
    let grid_geometry = grid_geometry(renderer, &g);

    let mut maze = Maze::new(&grid_geometry, vertical_bias, &origin);

    maze.randomized_prim();

    dbg!("On grid {:?}, from {:?} to {:?} (len: {})",
        grid_geometry, maze.origin(), maze.end(), maze.len().ceil());

    let img = maze.draw(renderer);
    let _ = img.save(path);
}
}

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

    maze::generate_image(path, geometry, &*rendering, vertical_bias,
                         origin);
}

/* }}} */
