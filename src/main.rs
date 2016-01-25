extern crate docopt;
extern crate image;
extern crate gif;
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
use gif::{
    Frame,
    ExtensionData,
    DisposalMethod,
    Encoder,
    Repeat,
};

use rand::{
    random,
};
use std::fs::File;
use std::borrow::Cow;

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

pub enum Gradient {
    Length,
    Solution,
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

pub trait Algorithm<'a> {
    fn next(&mut self) -> Option<&Maze>;
}

/* Kruskal {{{ */

/*
 * 1. Create a list of all walls, and create a set for each cell,
 *    each containing just that one cell.
 * 2. For each wall, in some random order:
 *    If the cells divided by this wall belong to distinct sets:
 *      1. Remove the current wall.
 *      2. Join the sets of the formerly divided cells.
 */
struct Kruskal<'a> {
    maze: &'a mut Maze,
    vwalls: Vec<Coord>,
    hwalls: Vec<Coord>,
    f: f64,
}

impl<'a> Kruskal<'a> {
    fn init(maze: &'a mut Maze) -> Kruskal<'a> {
        /* Create list of walls */
        let nb_walls = ((maze.geometry.width + 1) / 2 ) * ((maze.geometry.height + 1) / 2) / 2;
        let mut vwalls : Vec<Coord> = Vec::with_capacity(nb_walls);
        let mut hwalls : Vec<Coord> = Vec::with_capacity(nb_walls);
        for y in 0..maze.geometry.height {
            for x in 0..maze.geometry.width {
                match ((x & 1), (y & 1)) {
                    (0, 1) => {
                        vwalls.push(Coord{x: x, y: y});
                    },
                    (1, 0) => {
                        hwalls.push(Coord{x: x, y: y});
                    },
                    (_, _) => {}
                }
            }
        }

        let f = 0_f64;

        let origin = maze.origin.clone();
        maze.set_path(&origin, f);
        Kruskal {
            maze: maze,
            vwalls: vwalls,
            hwalls: hwalls,
            f:f ,
        }
    }
}

impl<'a> Algorithm<'a> for Kruskal<'a> {
    fn next(&mut self) -> Option<&Maze> {
        if self.vwalls.is_empty() || self.hwalls.is_empty() {
            /* Find end */
            self.maze.find_end();
            /* mark unvisited as walls */
            for y in 0..self.maze.geometry.height {
                for x in 0..self.maze.geometry.width {
                    let c = Coord{x:x, y:y};
                    match self.maze.cell_kind(&c) {
                        CellKind::Undefined => {
                            self.maze.set_wall(&c);
                        },
                        CellKind::PathKind(f) => {
                            self.maze.grid[y * self.maze.geometry.width + x] =
                                CellKind::PathKind(f / self.maze.len);
                        },
                        _ => {
                        }
                    }
                }
            }
            return None
        }
        let w = pop_random_wall(&mut self.vwalls, &mut self.hwalls,
                                self.maze.vertical_bias);
        if let Some(dir) = self.maze.get_random_wall_direction(&w) {
            let o1 = self.maze.get_coord_next(&w as &Coord, &dir);
            let o2 = self.maze.get_coord_next(&w as &Coord, &opposite(&dir));
            match (o1, o2) {
                (Some(c1), Some(c2)) => {
                    match (self.maze.cell_kind(&c1), self.maze.cell_kind(&c2)) {
                        (CellKind::PathKind(d1), CellKind::PathKind(d2)) => {
                            if d1 != d2 {
                                /* merge paths */
                                self.maze.set_path(&w, d1);
                                self.maze.set_path_value(d1, &c2);
                            }
                        },
                        (CellKind::PathKind(d), CellKind::Undefined) |
                            (CellKind::Undefined, CellKind::PathKind(d)) => {
                                self.maze.set_path(&w, d);
                                self.maze.set_path(&c1, d);
                                self.maze.set_path(&c2, d);
                            },
                            (CellKind::Undefined, CellKind::Undefined) => {
                                self.f += 1_f64;
                                self.maze.set_path(&w,  self.f);
                                self.maze.set_path(&c1, self.f);
                                self.maze.set_path(&c2, self.f);
                            }
                        (_, _) => {
                        }
                    }
                },
                (Some(c), _) | (_, Some(c)) => {
                    if let CellKind::PathKind(d) = self.maze.cell_kind(&c) {
                        self.maze.set_path(&w, d);
                    } else if let CellKind::Undefined = self.maze.cell_kind(&c) {
                        self.f += 1_f64;
                        self.maze.set_path(&w, self.f);
                        self.maze.set_path(&c, self.f);
                    }
                },
                (_, _) => {
                }
            }
        }
        Some(self.maze)
    }
}

/* }}} */
/* Prim {{{ */
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

struct Prim<'a> {
    maze: &'a mut Maze,
    vwalls: Vec<Wall>,
    hwalls: Vec<Wall>,
}

impl<'a> Prim<'a> {
    fn init(maze: &'a mut Maze) -> Prim<'a> {
        let mut vwalls : Vec<Wall> = Vec::new();
        let mut hwalls : Vec<Wall> = Vec::new();

        let start = maze.origin();
        maze.set_path(&start, 0.0_f64);
        let new_walls = maze.get_undefined_cells_around(&start);
        maze.set_walls(&new_walls);
        add_walls(&mut vwalls, &mut hwalls, new_walls);
        Prim {
            maze: maze,
            vwalls: vwalls,
            hwalls: hwalls,
        }
    }
}

impl<'a> Algorithm<'a> for Prim<'a> {
    fn next(&mut self) -> Option<&Maze> {
        if self.vwalls.is_empty() || self.hwalls.is_empty() {
            for y in (0..self.maze.geometry.height).filter(|&v| v % 2 == 1) {
                for x in (0..self.maze.geometry.width).filter(|&v| v % 2 == 1) {
                    if let CellKind::Undefined= self.maze.cell_kind(&Coord{x:x, y:y}) {
                        self.maze.grid[y * self.maze.geometry.width + x] =
                            CellKind::WallKind;
                    }
                }
            }
            for y in 0..self.maze.geometry.height {
                for x in 0..self.maze.geometry.width {
                    if let CellKind::PathKind(f) = self.maze.cell_kind(&Coord{x:x, y:y}) {
                        self.maze.grid[y * self.maze.geometry.width + x] =
                            CellKind::PathKind(f / self.maze.len);
                    }
                }
            }
            return None
        }
        /* Pick a random wall from the list */
        let w = pop_random_wall(&mut self.vwalls, &mut self.hwalls,
                                self.maze.vertical_bias);
        if let Some(dir) = self.maze.get_random_wall_direction(&w) {
            let o1 = self.maze.get_coord_next(&w as &Coord, &dir);
            let o2 = self.maze.get_coord_next(&w as &Coord, &opposite(&dir));
            match (o1, o2) {
                (Some(c1), Some(c2)) => {
                    if let CellKind::PathKind(d) = self.maze.cell_kind(&c1) {
                        if let CellKind::PathKind(_) = self.maze.cell_kind(&c2) {
                            return Some(self.maze);
                        }
                        self.maze.set_path(&w, d + 1_f64);
                        if let CellKind::Undefined = self.maze.cell_kind(&c2) {
                            self.maze.set_path(&c2, d + 2_f64);

                            let walls = self.maze.get_undefined_cells_around(&c2);
                            self.maze.set_walls(&walls);
                            add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                            if self.maze.len < d + 2_f64 {
                                self.maze.len = d + 2_f64;
                                self.maze.end = c2;
                            }
                        }
                        let walls = self.maze.get_undefined_cells_around(&w);
                        self.maze.set_walls(&walls);
                        add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                        if self.maze.len < d + 1_f64 {
                            self.maze.len = d + 1_f64;
                            self.maze.end = w;
                        }
                    } else if let CellKind::PathKind(d) = self.maze.cell_kind(&c2) {
                        self.maze.set_path(&w, d + 1_f64);
                        if let CellKind::Undefined = self.maze.cell_kind(&c1) {
                            self.maze.set_path(&c1, d + 2_f64);

                            let walls = self.maze.get_undefined_cells_around(&c1);
                            self.maze.set_walls(&walls);
                            add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                            if self.maze.len < d + 2_f64 {
                                self.maze.len = d + 2_f64;
                                self.maze.end = c1;
                            }
                        }
                        let walls = self.maze.get_undefined_cells_around(&w);
                        self.maze.set_walls(&walls);
                        add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                        if self.maze.len < d + 1_f64 {
                            self.maze.len = d + 1_f64;
                            self.maze.end = w;
                        }
                    }
                },
                (Some(c), _) | (_, Some(c)) => {
                    if let CellKind::PathKind(d) = self.maze.cell_kind(&c) {
                        self.maze.set_path(&w, d + 1_f64);

                        let walls = self.maze.get_undefined_cells_around(&w);
                        self.maze.set_walls(&walls);
                        add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                        if self.maze.len < d + 1_f64 {
                            self.maze.len = d + 1_f64;
                            self.maze.end = w;
                        }
                    }
                },
                (_, _) => {
                }
            }
        }
        Some(self.maze)
    }
}

/* }}} */
/* BackTracker {{{ */
    /*
     * 1. Make the initial cell the current cell and mark it as visited
     * 2. While there are unvisited cells
     *    1. If the current cell has any neighbours which have not been
     *       visited
     *       1. Choose randomly one of the unvisited neighbours
     *       2. Push the current cell to the stack
     *       3. Remove the wall between the current cell and the chosen cell
     *       4. Make the chosen cell the current cell and mark it as visited
     *     2. Else if stack is not empty
     *        1. Pop a cell from the stack
     *        2. Make it the current cell
     */
struct Backtracker<'a> {
    maze: &'a mut Maze,
    c: Coord,
    stack: Vec<Coord>,
    f: f64,
    done: bool,
    to_finish: bool
}

impl<'a> Backtracker<'a> {
    fn init(maze: &'a mut Maze) -> Backtracker<'a> {
        let c = maze.origin().clone();
        maze.set_path(&c, 0.0_f64);
        let stack : Vec<Coord> = Vec::new();

        maze.len = 0_f64;
        Backtracker {
            maze: maze,
            c: c,
            stack: stack,
            f: 0_f64,
            done: false,
            to_finish: false
        }
    }
}

impl<'a> Algorithm<'a> for Backtracker<'a> {
    fn next(&mut self) -> Option<&Maze> {
        if self.to_finish {
            /* mark unvisited as walls */
            for y in 0..self.maze.geometry.height {
                for x in 0..self.maze.geometry.width {
                    let c = Coord{x:x, y:y};
                    match self.maze.cell_kind(&c) {
                        CellKind::Undefined => {
                            self.maze.set_wall(&c);
                        },
                        CellKind::PathKind(f) => {
                            self.maze.grid[y * self.maze.geometry.width + x] =
                                CellKind::PathKind(f / self.maze.len);
                        },
                        _ => {
                        }
                    }
                }
            }
            self.done = true;
        }
        match self.maze.get_random_unvisited_cell_neighbour(&self.c) {
            None => {
                match self.stack.pop() {
                    None => {
                        self.to_finish = true;
                        return Some(self.maze);
                    },
                    Some(n) => {
                        if let CellKind::PathKind(d) = self.maze.cell_kind(&n) {
                            self.f = d;
                        }
                        self.c = n;
                    }
                }
            },
            Some(n) => {
                let w = Coord{x: (n.x + self.c.x) / 2, y: (n.y + self.c.y) / 2};
                self.f += 1_f64;
                self.maze.set_path(&w, self.f);
                self.c = n.clone();
                self.f += 1_f64;
                self.maze.set_path(&n, self.f);
                if self.maze.len < self.f {
                    self.maze.len = self.f;
                    self.maze.end = n.clone();
                }
                self.stack.push(n);
            }
        }
        Some(self.maze)
    }
}

/* }}} */

pub enum AlgorithmKind {
    Prim,
    Kruskal,
    Backtracker,
}


/* Maze {{{ */
#[derive(Debug,Clone)]
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

    fn get_coord_twice(&self, c: &Coord, dir: &Direction) -> Option<Coord> {
        match *dir {
            Direction::Up => {
                self.get_coord_up(c).and_then(|n| self.get_coord_up(&n))
            },
            Direction::Down => {
                self.get_coord_down(c).and_then(|n| self.get_coord_down(&n))
            },
            Direction::Left => {
                self.get_coord_left(c).and_then(|n| self.get_coord_left(&n))
            },
            Direction::Right => {
                self.get_coord_right(c).and_then(|n| self.get_coord_right(&n))
            }
        }
    }

    fn get_random_wall_direction(&self, w: &Wall) -> Option<Direction> {
        match (w.x % 2, w.y % 2) {
            (0, 1) => {
                match random::<u8>() % 2 {
                    0 => Some(Direction::Up),
                    _ => Some(Direction::Down),
                }
            },
            (1, 0) => {
                match random::<u8>() % 2 {
                    0 => Some(Direction::Left),
                    _ => Some(Direction::Right),
                }
            },
            (1, 1) => {
                let f = random::<f64>();
                if f < self.vertical_bias {
                    match random::<u8>() % 2 {
                        0 => Some(Direction::Left),
                        _ => Some(Direction::Right),
                    }
                } else {
                    match random::<u8>() % 2 {
                        0 => Some(Direction::Up),
                        _ => Some(Direction::Down),
                    }
                }
            },
            (_, _) => { /* (0, 0) */
                None
            }
        }
    }

    fn set_path_value(&mut self, f: f64, o: &Coord) {
        /* Walk c2 and put f in the path */
        let mut stack : Vec<Coord> = Vec::new();
        let mut c = o.clone();
        let is_visited = |m: &Maze, c: &Coord| {
            if let CellKind::PathKind(d) = m.cell_kind(c) {
                if d == f {
                    Some(true)
                } else {
                    Some(false)
                }
            } else {
                None
            }
        };
        stack.push(c.clone());
        loop {
            self.grid[c.y * self.geometry.width + c.x] =
                CellKind::PathKind(f);
            match self.walk(&c, &is_visited) {
                Some((next, _)) => {
                    self.grid[next.y * self.geometry.width + next.x] =
                        CellKind::PathKind(f);
                    stack.push(c);
                    c = next;
                },
                None => {
                    match stack.pop() {
                        Some(next) => {
                            c = next.clone();
                        },
                        None => {
                            break;
                        }
                    }
                }
            }
        }
    }


    fn get_random_unvisited_cell_neighbour(&mut self, c: &Coord) -> Option<Coord> {
        let dirs = vec![Direction::Up, Direction::Down,
                        Direction::Left, Direction::Right];
        let mut vec : Vec<Coord> = Vec::with_capacity(4);
        for d in dirs {
            if let Some(n) = self.get_coord_twice(c, &d) {
                if let CellKind::Undefined = self.cell_kind(&n) {
                    vec.push(n);
                }
            }
        }
        if vec.len() == 0 {
            None
        } else {
            let r : usize = random::<usize>();
            let len = vec.len();
            Some(vec.swap_remove(r % len))
        }
    }

    fn draw<T: ?Sized + Rendering>(&self, renderer: &T) -> RgbImage {
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
    fn draw_gif<T: ?Sized + Rendering>(&self, renderer: &T) -> Frame {
        let g = image_geometry(renderer, &self.geometry);
        let mut frame = Frame::default();

        frame.width = g.width as u16;
        frame.height = g.height as u16;
        frame.palette = None;
        frame.transparent = None;

        let mut buffer : Vec<u8> = Vec::with_capacity(g.width * g.height);
        for _ in 0..(g.width*g.height) {
            buffer.push(0);
        }

        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                let c = Coord{x: x, y: y};
                renderer.draw_cell_gif(&self, &g, &mut buffer, &c, self.cell_kind(&c));
            }
        }
        frame.buffer = Cow::Owned(buffer);
        frame
    }

    fn clear_path(&mut self) {
        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                if let CellKind::PathKind(_) = self.cell_kind(&Coord{x:x, y:y}) {
                    self.grid[y * self.geometry.width + x] =
                        CellKind::PathKind(-1_f64);
                }
            }
        }
    }

    fn is_visited(&self, c: &Coord) -> Option<bool> {
        if let CellKind::PathKind(f) = self.cell_kind(c) {
            if f == -1_f64 {
                Some(false)
            } else {
                Some(true)
            }
        } else {
            None
        }
    }

    fn walk<F>(&self, start: &Coord, is_visited: F)
      -> Option<(Coord, Direction)>
      where F: Fn(&Maze, &Coord) -> Option<bool> {
        let dirs = vec![Direction::Up, Direction::Down,
                        Direction::Left, Direction::Right];
        for d in dirs {
            if let Some(c) = self.get_coord_next(start, &d) {
                match is_visited(self, &c) {
                    Some(b) if !b => {
                        return Some((c, d));
                    },
                    _ => {
                    }
                }
            }
        }
        None
    }

    fn find_end(&mut self) {
        self.clear_path();
        let mut stack : Vec<(Coord, Direction, f64)> = Vec::new();
        let mut c = self.origin.clone();
        let mut f = 0_f64;
        let is_visited = |m: &Maze, c: &Coord| m.is_visited(c);
        self.set_path(&c, 0_f64);
        loop {
            match self.walk(&c, &is_visited) {
                Some((next, direction)) => {
                    self.set_path(&next, 0_f64);
                    f += 1_f64;
                    if f > self.len {
                        self.len = f;
                        self.end = next.clone();
                    }
                    stack.push((c, direction, f));
                    c = next;
                },
                None => {
                    if let Some((next, _, distance)) = stack.pop() {
                        c = next;
                        f = distance;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn compute_solution(&mut self) {
        self.clear_path();
        let mut sol : Vec<(Coord, Direction)> = Vec::new();
        let mut c = self.origin.clone();
        let is_visited = |m: &Maze, c: &Coord| m.is_visited(c);
        self.set_path(&c, 0_f64);
        loop {
            match self.walk(&c, &is_visited) {
                Some((next, direction)) => {
                    self.set_path(&next, 0_f64);
                    if (next.x, next.y) == (self.end.x, self.end.y) {
                        break;
                    }
                    sol.push((c, direction));
                    c = next;
                },
                None => {
                    if let Some((next, _)) = sol.pop() {
                        c = next;
                    }
                }
            }
        }
        /* compute lengths */
        self.clear_path();
        /* mark solution as 0 */
        for v in &sol {
            let c = &v.0;
            self.grid[c.y * self.geometry.width + c.x] = CellKind::PathKind(0_f64);
        }
        let mut len = 0_f64;
        for v in &sol {
            let mut stack : Vec<Coord> = Vec::new();
            let mut d = 0_f64;
            c = v.0.clone();
            loop {
                match self.walk(&c, &is_visited) {
                    Some((next, _)) => {
                        d = d + 1_f64;
                        if d > len {
                            len = d;
                        }
                        self.set_path(&next, d);
                        stack.push(c);
                        c = next;
                    },
                    None => {
                        match stack.pop() {
                            Some(next) => {
                                c = next.clone();
                                if let CellKind::PathKind(f) = self.cell_kind(&c) {
                                    d = f;
                                }
                            },
                            None => {
                                break;
                            }
                        }
                    }
                }
            }
        }
        len = len.log10();
        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                if let CellKind::PathKind(f) = self.cell_kind(&Coord{x:x, y:y}) {
                    if f > 0_f64 {
                        self.grid[y * self.geometry.width + x] =
                            CellKind::PathKind(f.log10() / len);
                    }
                }
            }
        }
    }
}

/* }}} */

fn generate_algorithm<'a>(maze: &'a mut Maze, algorithm: AlgorithmKind) -> Box<Algorithm<'a> + 'a> {
    match algorithm {
        AlgorithmKind::Prim => {
            Box::new(Prim::init(maze))
        },
        AlgorithmKind::Kruskal => {
            Box::new(Kruskal::init(maze))
        },
        AlgorithmKind::Backtracker => {
            Box::new(Backtracker::init(maze))
        },
    }
}



pub trait Rendering {
    fn tile_size(&self) -> usize;
    fn draw_cell(&self, &Maze, &mut RgbImage, &Coord, CellKind);
    fn draw_cell_gif(&self, &Maze, &super::Geometry, &mut Vec<u8>, &Coord, CellKind);
    fn get_gif_palette(&self) -> Vec<u8>;
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
                                             origin: super::Origin,
                                             gradient: Option<Gradient>,
                                             algorithm: AlgorithmKind,
                                             animation: bool) {
    let grid_geometry = grid_geometry(renderer, &g);

    let mut maze = Maze::new(&grid_geometry, vertical_bias, &origin);
    let mut nb_iterations = 0u32;
    let g = image_geometry(renderer, &maze.geometry);
    let width : u16 = g.width as u16;
    let height : u16 = g.height as u16;

    let mut image = match animation {
        true => { Some(File::create(path).unwrap()) },
        false => None,
    };
    let mut encoder = match animation {
        true => {
            let palette = renderer.get_gif_palette();
            Some(Encoder::new(image.as_mut().unwrap(), width, height, &palette).unwrap())
        },
        false => None,
    };

    let draw_animated_frame = |m: &Maze, enc: &mut Encoder<&mut File>| {
        let f = m.draw_gif(renderer);
        enc.write_frame(&f).unwrap();
    };

    {
        let mut a = generate_algorithm(&mut maze, algorithm);
        loop {
            match a.next() {
                Some(m) => {
                    nb_iterations += 1u32;
                    if !animation {
                        continue;
                    }
                    dbg!("{} frames generated in gif", nb_iterations);
                    draw_animated_frame(&m, encoder.as_mut().unwrap());
                },
                None => {
                    break;
                }
            }
        }
    }

    if animation {
        if let Some(e) = encoder.as_mut() {
            e.write_extension(ExtensionData::new_control_ext(100u16,
                                                             DisposalMethod::Any,
                                                             false, None)).unwrap();
            draw_animated_frame(&maze, e);
            e.write_extension(ExtensionData::Repetitions(Repeat::Infinite)).unwrap();
        }
    }

    dbg!("On grid {:?}, from {:?} to {:?} (len: {}). Generated on {} iterations",
        grid_geometry, maze.origin(), maze.end(), maze.len().ceil(),
        nb_iterations);

    if !animation {
        if let Some(Gradient::Solution) = gradient {
            maze.compute_solution();
        }

        let img = maze.draw(renderer);
        let _ = img.save(path);
    }
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
       maze --gradient GRADIENT FILE
       maze --algorithm ALGORITHM FILE
       maze --animation
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
    --gradient=GRADIENT                           If 2 foreground colors, define how to do the gradient. Valid values are: length, solution. [default: length]
    --algorithm=ALGORITHM                         Algorithm used to generate the maze. Valid values are: prim, kruskal. [default: prim]
    --animation                                   Render an animation as the maze is being generated
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

fn gradient_parse(g: &str) -> Option<maze::Gradient> {
    match g {
        "length" => {
            Some(maze::Gradient::Length)
        },
        "solution" => {
            Some(maze::Gradient::Solution)
        },
        _ => {
            None
        }
    }
}

fn algorithm_parse(s: &str) -> maze::AlgorithmKind {
    match s {
        "prim" => {
            maze::AlgorithmKind::Prim
        },
        "kruskal" => {
            maze::AlgorithmKind::Kruskal
        },
        "backtracker" => {
            maze::AlgorithmKind::Backtracker
        }
        _ => {
            panic!("invalid algorithm {}", s);
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

    let gradient = args.get_str("--gradient");
    let gradient = gradient_parse(&gradient);

    let algorithm = args.get_str("--algorithm");
    let algorithm = algorithm_parse(&algorithm);

    let animation = args.get_bool("--animation");

    maze::generate_image(path, geometry, &*rendering, vertical_bias,
                         origin, gradient, algorithm, animation);
}

/* }}} */
