extern crate docopt;
extern crate image;
extern crate rand;

use docopt::Docopt;
use std::path;
use image::{
    RgbImage,
    Rgb
};
use rand::{
    random,
    Rng
};

#[derive(Debug)]
struct Coord {
    x: u32,
    y: u32,
}
type Path = Coord;
type Wall = Coord;

fn pop_random_wall(walls: &mut Vec<Wall>) -> Wall {
    let pos: usize = rand::random::<usize>() % walls.len();
    walls.swap_remove(pos)
}

struct Maze {
    img: RgbImage,
    pixel_size: u32,
    width: u32,
    height: u32,
    grid_width: u32,
    grid_height: u32,
    path_color: Rgb<u8>,
    wall_color: Rgb<u8>,
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

#[derive(Debug)]
enum CellKind {
    Wall,
    Path,
    Undefined
}

fn add_walls(walls: &mut Vec<Wall>, new_walls: Vec<Coord>) {
    for w in new_walls {
        walls.push(w);
    }
}

impl Maze {
    fn new(width: u32, height: u32, pixel_size: u32) -> Maze {
        let mut m = Maze {
            width: width,
            height: height,
            grid_width: width / pixel_size,
            grid_height: height / pixel_size,
            pixel_size: pixel_size,
            img: RgbImage::new(width, height),
            path_color: Rgb{data:[253, 246, 227]},
            wall_color: Rgb{data:[  7,  54,  66]},
        };
        /* draw right/bottom walls if needed */
        let d = m.width - m.grid_width * m.pixel_size;
        for i in 0..d {
            let x = m.grid_width * m.pixel_size + i;
            for y in 0..m.height {
                m.img.put_pixel(x, y, m.wall_color);
            }
        }
        let d = m.height - m.grid_height * m.pixel_size;
        for i in 0..d {
            let y = m.grid_height * m.pixel_size + i;
            for x in 0..m.width {
                m.img.put_pixel(x, y, m.wall_color);
            }
        }
        m
    }

    fn cell_kind(&self, c: &Coord) -> CellKind {
        if c.x >= self.grid_width || c.y >= self.grid_height {
            return CellKind::Undefined;
        }
        let p = self.img.get_pixel(c.x * self.pixel_size,
                               c.y * self.pixel_size);
        if *p == self.wall_color {
            CellKind::Wall
        } else if *p == self.path_color {
            CellKind::Path
        } else {
            CellKind::Undefined
        }
    }

    fn draw_path(&mut self, c: &Coord) {
        if let CellKind::Path = self.cell_kind(&c) {
            return;
        }
        for i in 0..self.pixel_size {
            for j in 0..self.pixel_size {
                self.img.put_pixel(c.x * self.pixel_size + i,
                                   c.y * self.pixel_size + j,
                                   self.path_color);
            }
        }
    }

    fn draw_wall(&mut self, c: &Coord) {
        if let CellKind::Wall = self.cell_kind(&c) {
            return;
        }
        for i in 0..self.pixel_size {
            for j in 0..self.pixel_size {
                self.img.put_pixel(c.x * self.pixel_size + i,
                                   c.y * self.pixel_size + j,
                                   self.wall_color);
            }
        }
    }

    fn draw_walls(&mut self, walls: &Vec<Coord>) {
        for w in walls {
            self.draw_wall(&w as &Wall);
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
        if c.y >= self.grid_height - 1 {
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
        if c.x >= self.grid_width - 1 {
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
        let d;
        let mut rng = rand::thread_rng();
        if w.x % 2 == 0 && w.y % 2 == 0 {
            d = rng.next_u32() % 4;
        } else if w.x % 2 == 0 {
            d = rng.next_u32() % 2;
        } else if w.y % 2 == 0 {
            d = 2 + rng.next_u32() % 2;
        } else {
            d = 4;
        }

        match d {
            0 => Some(Direction::Up),
            1 => Some(Direction::Down),
            2 => Some(Direction::Left),
            3 => Some(Direction::Right),
            _ => None,
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
        let mut walls : Vec<Wall> = Vec::new();
        let start = Coord{x:0, y:0};
        self.draw_path(&start);
        let new_walls = self.get_undefined_cells_around(&start);
        self.draw_walls(&new_walls);
        add_walls(&mut walls, new_walls);

        while !walls.is_empty() {
            /* Pick a random wall from the list */
            let w = pop_random_wall(&mut walls);
            if let Some(dir) = self.get_random_wall_direction(&w) {
                let o1 = self.get_coord_next(&w as &Coord, &dir);
                let o2 = self.get_coord_next(&w as &Coord, &opposite(&dir));
                match (o1, o2) {
                    (Some(c1), Some(c2)) => {
                        if let (CellKind::Path, CellKind::Path)
                            = (self.cell_kind(&c1), self.cell_kind(&c2)) {
                                continue;
                        }
                        self.draw_path(&w);
                        self.draw_path(&c2);
                        self.draw_path(&c1);
                        let new_walls = self.get_undefined_cells_around(&w);
                        self.draw_walls(&new_walls);
                        let new_walls = self.get_undefined_cells_around(&c1);
                        self.draw_walls(&new_walls);
                        add_walls(&mut walls, new_walls);
                        let new_walls = self.get_undefined_cells_around(&c2);
                        self.draw_walls(&new_walls);
                        add_walls(&mut walls, new_walls);
                    },
                    (Some(c), _) => {
                        self.draw_path(&w);
                        self.draw_path(&c);
                        let new_walls = self.get_undefined_cells_around(&w);
                        self.draw_walls(&new_walls);
                        let new_walls = self.get_undefined_cells_around(&c);
                        self.draw_walls(&new_walls);
                        add_walls(&mut walls, new_walls);
                    },
                    (_, Some(c)) => {
                        self.draw_path(&w);
                        self.draw_path(&c);
                        let new_walls = self.get_undefined_cells_around(&w);
                        self.draw_walls(&new_walls);
                        let new_walls = self.get_undefined_cells_around(&c);
                        self.draw_walls(&new_walls);
                        add_walls(&mut walls, new_walls);
                    },
                    (_, _) => {
                    }
                }
            }
        }
    }
    fn save(&mut self, path: &path::Path) {
        let _ = self.img.save(path);
    }
}


fn generate_image(path: &path::Path, width: u32, height: u32) {
    let mut maze = Maze::new(width, height, 4);

    maze.randomized_prim();
    maze.save(path);
}

const USAGE: &'static str = "
Maze background generator.

Usage: maze [options] FILE
       maze -g GEOM <kind> FILE
       maze --geometry GEOM <kind> FILE
       maze -h | --help
       maze -v | --version

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
    let path = path::Path::new(path);

    generate_image(path, geometry.0, geometry.1);
}
