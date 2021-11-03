use gif::{DisposalMethod, Encoder, ExtensionData, Frame, Repeat};
use image::RgbImage;
use std::path;

use rand::random;
use std::borrow::Cow;
use std::fs::File;

#[derive(Debug, Clone)]
pub struct Coord {
    pub x: usize,
    pub y: usize,
}
type Wall = Coord;

#[derive(Debug, Clone)]
pub enum CellKind {
    WallKind,
    PathKind(f64),
    Undefined,
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
    Right,
}
fn opposite(dir: &Direction) -> Direction {
    match *dir {
        Direction::Up => Direction::Down,
        Direction::Down => Direction::Up,
        Direction::Left => Direction::Right,
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

fn pop_random_wall(vwalls: &mut Vec<Wall>, hwalls: &mut Vec<Wall>, vertical_bias: f64) -> Wall {
    let r: usize = random::<usize>();
    match (vwalls.len(), hwalls.len()) {
        (0, len) => {
            let pos = r % len;
            hwalls.swap_remove(pos)
        }
        (len, 0) => {
            let pos = r % len;
            vwalls.swap_remove(pos)
        }
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
        let nb_walls = ((maze.geometry.width + 1) / 2) * ((maze.geometry.height + 1) / 2) / 2;
        let mut vwalls: Vec<Coord> = Vec::with_capacity(nb_walls);
        let mut hwalls: Vec<Coord> = Vec::with_capacity(nb_walls);
        for y in 0..maze.geometry.height {
            for x in 0..maze.geometry.width {
                match ((x & 1), (y & 1)) {
                    (0, 1) => {
                        vwalls.push(Coord { x: x, y: y });
                    }
                    (1, 0) => {
                        hwalls.push(Coord { x: x, y: y });
                    }
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
            f: f,
        }
    }

    fn set_path_value(&mut self, f: f64, o: &Coord) {
        /* Walk c2 and put f in the path */
        let mut stack: Vec<Coord> = Vec::new();
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
            self.maze.grid[c.y * self.maze.geometry.width + c.x] = CellKind::PathKind(f);
            match self.maze.walk(&c, &is_visited) {
                Some((next, _)) => {
                    self.maze.grid[next.y * self.maze.geometry.width + next.x] =
                        CellKind::PathKind(f);
                    stack.push(c);
                    c = next;
                }
                None => match stack.pop() {
                    Some(next) => {
                        c = next.clone();
                    }
                    None => {
                        break;
                    }
                },
            }
        }
    }

    fn find_end(&mut self) {
        self.maze.clear_path();
        let mut stack: Vec<(Coord, Direction, f64)> = Vec::new();
        let mut c = self.maze.origin.clone();
        let mut f = 0_f64;
        let is_visited = |m: &Maze, c: &Coord| m.is_visited(c);
        self.maze.set_path(&c, 0_f64);
        loop {
            match self.maze.walk(&c, &is_visited) {
                Some((next, direction)) => {
                    self.maze.set_path(&next, 0_f64);
                    f += 1_f64;
                    if f > self.maze.len {
                        self.maze.len = f;
                        self.maze.end = next.clone();
                    }
                    stack.push((c, direction, f));
                    c = next;
                }
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
}

impl<'a> Algorithm<'a> for Kruskal<'a> {
    fn next(&mut self) -> Option<&Maze> {
        if self.vwalls.is_empty() || self.hwalls.is_empty() {
            /* Find end */
            self.find_end();
            /* mark unvisited as walls */
            for y in 0..self.maze.geometry.height {
                for x in 0..self.maze.geometry.width {
                    let c = Coord { x: x, y: y };
                    match self.maze.cell_kind(&c) {
                        CellKind::Undefined => {
                            self.maze.set_wall(&c);
                        }
                        CellKind::PathKind(f) => {
                            self.maze.grid[y * self.maze.geometry.width + x] =
                                CellKind::PathKind(f / self.maze.len);
                        }
                        _ => {}
                    }
                }
            }
            return None;
        }
        let w = pop_random_wall(&mut self.vwalls, &mut self.hwalls, self.maze.vertical_bias);
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
                                self.set_path_value(d1, &c2);
                            }
                        }
                        (CellKind::PathKind(d), CellKind::Undefined)
                        | (CellKind::Undefined, CellKind::PathKind(d)) => {
                            self.maze.set_path(&w, d);
                            self.maze.set_path(&c1, d);
                            self.maze.set_path(&c2, d);
                        }
                        (CellKind::Undefined, CellKind::Undefined) => {
                            self.f += 1_f64;
                            self.maze.set_path(&w, self.f);
                            self.maze.set_path(&c1, self.f);
                            self.maze.set_path(&c2, self.f);
                        }
                        (_, _) => {}
                    }
                }
                (Some(c), _) | (_, Some(c)) => {
                    if let CellKind::PathKind(d) = self.maze.cell_kind(&c) {
                        self.maze.set_path(&w, d);
                    } else if let CellKind::Undefined = self.maze.cell_kind(&c) {
                        self.f += 1_f64;
                        self.maze.set_path(&w, self.f);
                        self.maze.set_path(&c, self.f);
                    }
                }
                (_, _) => {}
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
        let vwalls: Vec<Wall> = Vec::new();
        let hwalls: Vec<Wall> = Vec::new();

        let start = maze.origin();
        maze.set_path(&start, 0.0_f64);
        let mut p = Prim {
            maze: maze,
            vwalls: vwalls,
            hwalls: hwalls,
        };
        let new_walls = p.get_undefined_cells_around(&start);
        p.set_walls(&new_walls);
        add_walls(&mut p.vwalls, &mut p.hwalls, new_walls);
        p
    }

    fn set_walls(&mut self, walls: &Vec<Coord>) {
        for w in walls {
            self.maze.set_wall(&w as &Wall);
        }
    }

    fn get_undefined_cells_around(&mut self, c: &Coord) -> Vec<Coord> {
        let dirs = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let mut v: Vec<Coord> = Vec::new();
        for d in dirs {
            let o = self.maze.get_coord_next(&c, &d);
            if let Some(c) = o {
                if let CellKind::Undefined = self.maze.cell_kind(&c) {
                    v.push(c);
                }
            }
        }
        v
    }
}

impl<'a> Algorithm<'a> for Prim<'a> {
    fn next(&mut self) -> Option<&Maze> {
        if self.vwalls.is_empty() && self.hwalls.is_empty() {
            for y in (0..self.maze.geometry.height).filter(|&v| v % 2 == 1) {
                for x in (0..self.maze.geometry.width).filter(|&v| v % 2 == 1) {
                    if let CellKind::Undefined = self.maze.cell_kind(&Coord { x: x, y: y }) {
                        self.maze.grid[y * self.maze.geometry.width + x] = CellKind::WallKind;
                    }
                }
            }
            for y in 0..self.maze.geometry.height {
                for x in 0..self.maze.geometry.width {
                    if let CellKind::PathKind(f) = self.maze.cell_kind(&Coord { x: x, y: y }) {
                        self.maze.grid[y * self.maze.geometry.width + x] =
                            CellKind::PathKind(f / self.maze.len);
                    }
                }
            }
            return None;
        }
        /* Pick a random wall from the list */
        let w = pop_random_wall(&mut self.vwalls, &mut self.hwalls, self.maze.vertical_bias);
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

                            let walls = self.get_undefined_cells_around(&c2);
                            self.set_walls(&walls);
                            add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                            if self.maze.len < d + 2_f64 {
                                self.maze.len = d + 2_f64;
                                self.maze.end = c2;
                            }
                        }
                        let walls = self.get_undefined_cells_around(&w);
                        self.set_walls(&walls);
                        add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                        if self.maze.len < d + 1_f64 {
                            self.maze.len = d + 1_f64;
                            self.maze.end = w;
                        }
                    } else if let CellKind::PathKind(d) = self.maze.cell_kind(&c2) {
                        self.maze.set_path(&w, d + 1_f64);
                        if let CellKind::Undefined = self.maze.cell_kind(&c1) {
                            self.maze.set_path(&c1, d + 2_f64);

                            let walls = self.get_undefined_cells_around(&c1);
                            self.set_walls(&walls);
                            add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                            if self.maze.len < d + 2_f64 {
                                self.maze.len = d + 2_f64;
                                self.maze.end = c1;
                            }
                        }
                        let walls = self.get_undefined_cells_around(&w);
                        self.set_walls(&walls);
                        add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                        if self.maze.len < d + 1_f64 {
                            self.maze.len = d + 1_f64;
                            self.maze.end = w;
                        }
                    }
                }
                (Some(c), _) | (_, Some(c)) => {
                    if let CellKind::PathKind(d) = self.maze.cell_kind(&c) {
                        self.maze.set_path(&w, d + 1_f64);

                        let walls = self.get_undefined_cells_around(&w);
                        self.set_walls(&walls);
                        add_walls(&mut self.vwalls, &mut self.hwalls, walls);

                        if self.maze.len < d + 1_f64 {
                            self.maze.len = d + 1_f64;
                            self.maze.end = w;
                        }
                    }
                }
                (_, _) => {}
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
    to_finish: bool,
}

impl<'a> Backtracker<'a> {
    fn init(maze: &'a mut Maze) -> Backtracker<'a> {
        let c = maze.origin().clone();
        maze.set_path(&c, 0.0_f64);
        let stack: Vec<Coord> = Vec::new();

        maze.len = 0_f64;
        Backtracker {
            maze: maze,
            c: c,
            stack: stack,
            f: 0_f64,
            to_finish: false,
        }
    }

    fn get_coord_twice(&self, c: &Coord, dir: &Direction) -> Option<Coord> {
        match *dir {
            Direction::Up => self
                .maze
                .get_coord_up(c)
                .and_then(|n| self.maze.get_coord_up(&n)),
            Direction::Down => self
                .maze
                .get_coord_down(c)
                .and_then(|n| self.maze.get_coord_down(&n)),
            Direction::Left => self
                .maze
                .get_coord_left(c)
                .and_then(|n| self.maze.get_coord_left(&n)),
            Direction::Right => self
                .maze
                .get_coord_right(c)
                .and_then(|n| self.maze.get_coord_right(&n)),
        }
    }

    fn get_random_unvisited_cell_neighbour(&mut self) -> Option<Coord> {
        let dirs = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        let mut vec: Vec<Coord> = Vec::with_capacity(4);
        for d in dirs {
            if let Some(n) = self.get_coord_twice(&self.c, &d) {
                if let CellKind::Undefined = self.maze.cell_kind(&n) {
                    vec.push(n);
                }
            }
        }
        if vec.len() == 0 {
            None
        } else {
            let r: usize = random::<usize>();
            let len = vec.len();
            Some(vec.swap_remove(r % len))
        }
    }
}

impl<'a> Algorithm<'a> for Backtracker<'a> {
    fn next(&mut self) -> Option<&Maze> {
        if self.to_finish {
            /* mark unvisited as walls */
            for y in 0..self.maze.geometry.height {
                for x in 0..self.maze.geometry.width {
                    let c = Coord { x: x, y: y };
                    match self.maze.cell_kind(&c) {
                        CellKind::Undefined => {
                            self.maze.set_wall(&c);
                        }
                        CellKind::PathKind(f) => {
                            self.maze.grid[y * self.maze.geometry.width + x] =
                                CellKind::PathKind(f / self.maze.len);
                        }
                        _ => {}
                    }
                }
            }
            return None;
        }
        loop {
            match self.get_random_unvisited_cell_neighbour() {
                None => match self.stack.pop() {
                    None => {
                        self.to_finish = true;
                        return Some(self.maze);
                    }
                    Some(n) => {
                        if let CellKind::PathKind(d) = self.maze.cell_kind(&n) {
                            self.f = d;
                        }
                        self.c = n;
                    }
                },
                Some(n) => {
                    let w = Coord {
                        x: (n.x + self.c.x) / 2,
                        y: (n.y + self.c.y) / 2,
                    };
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
                    return Some(self.maze);
                }
            }
        }
    }
}

/* }}} */

pub enum AlgorithmKind {
    Prim,
    Kruskal,
    Backtracker,
}

/* Maze {{{ */
#[derive(Debug, Clone)]
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
            origin: Coord { x: 0, y: 0 },
            len: 0_f64,
            end: Coord {
                x: g.width - 1,
                y: g.height - 1,
            },
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

    fn get_coord_up(&self, c: &Coord) -> Option<Coord> {
        if c.y == 0 {
            None
        } else {
            Some(Coord { x: c.x, y: c.y - 1 })
        }
    }
    fn get_coord_down(&self, c: &Coord) -> Option<Coord> {
        if c.y >= self.geometry.height - 1 {
            None
        } else {
            Some(Coord { x: c.x, y: c.y + 1 })
        }
    }
    fn get_coord_left(&self, c: &Coord) -> Option<Coord> {
        if c.x == 0 {
            None
        } else {
            Some(Coord { x: c.x - 1, y: c.y })
        }
    }
    fn get_coord_right(&self, c: &Coord) -> Option<Coord> {
        if c.x >= self.geometry.width - 1 {
            None
        } else {
            Some(Coord { x: c.x + 1, y: c.y })
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
        match (w.x % 2, w.y % 2) {
            (0, 1) => match random::<u8>() % 2 {
                0 => Some(Direction::Up),
                _ => Some(Direction::Down),
            },
            (1, 0) => match random::<u8>() % 2 {
                0 => Some(Direction::Left),
                _ => Some(Direction::Right),
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
            }
            (_, _) => {
                /* (0, 0) */
                None
            }
        }
    }

    fn draw<T: ?Sized + Rendering>(&self, renderer: &T) -> RgbImage {
        let g = image_geometry(renderer, &self.geometry);
        let mut img = RgbImage::new(g.width as u32, g.height as u32);

        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                let c = Coord { x: x, y: y };
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

        let mut buffer: Vec<u8> = Vec::with_capacity(g.width * g.height);
        for _ in 0..(g.width * g.height) {
            buffer.push(0);
        }

        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                let c = Coord { x: x, y: y };
                renderer.draw_cell_gif(&self, &g, &mut buffer, &c, self.cell_kind(&c));
            }
        }
        frame.buffer = Cow::Owned(buffer);
        frame
    }

    fn clear_path(&mut self) {
        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                if let CellKind::PathKind(_) = self.cell_kind(&Coord { x: x, y: y }) {
                    self.grid[y * self.geometry.width + x] = CellKind::PathKind(-1_f64);
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

    fn walk<F>(&self, start: &Coord, is_visited: F) -> Option<(Coord, Direction)>
    where
        F: Fn(&Maze, &Coord) -> Option<bool>,
    {
        let dirs = vec![
            Direction::Up,
            Direction::Down,
            Direction::Left,
            Direction::Right,
        ];
        for d in dirs {
            if let Some(c) = self.get_coord_next(start, &d) {
                match is_visited(self, &c) {
                    Some(b) if !b => {
                        return Some((c, d));
                    }
                    _ => {}
                }
            }
        }
        None
    }

    fn compute_solution(&mut self) {
        self.clear_path();
        let mut sol: Vec<(Coord, Direction)> = Vec::new();
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
                }
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
            let mut stack: Vec<Coord> = Vec::new();
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
                    }
                    None => match stack.pop() {
                        Some(next) => {
                            c = next.clone();
                            if let CellKind::PathKind(f) = self.cell_kind(&c) {
                                d = f;
                            }
                        }
                        None => {
                            break;
                        }
                    },
                }
            }
        }
        len = len.log10();
        for y in 0..self.geometry.height {
            for x in 0..self.geometry.width {
                if let CellKind::PathKind(f) = self.cell_kind(&Coord { x: x, y: y }) {
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

fn generate_algorithm<'a>(
    maze: &'a mut Maze,
    algorithm: AlgorithmKind,
) -> Box<dyn Algorithm<'a> + 'a> {
    match algorithm {
        AlgorithmKind::Prim => Box::new(Prim::init(maze)),
        AlgorithmKind::Kruskal => Box::new(Kruskal::init(maze)),
        AlgorithmKind::Backtracker => Box::new(Backtracker::init(maze)),
    }
}

pub trait Rendering {
    fn tile_size(&self) -> usize;
    fn draw_cell(&self, maze: &Maze, img: &mut RgbImage, c: &Coord, cell_kind: CellKind);
    fn draw_cell_gif(
        &self,
        maze: &Maze,
        img_geom: &super::Geometry,
        buffer: &mut Vec<u8>,
        c: &Coord,
        cell_kind: CellKind,
    );
    fn get_gif_palette(&self) -> Vec<u8>;
}

fn grid_geometry<T: ?Sized + Rendering>(renderer: &T, g: &super::Geometry) -> super::Geometry {
    let tile_size = renderer.tile_size();
    super::Geometry {
        width: g.width / tile_size,
        height: g.height / tile_size,
    }
}

fn image_geometry<T: ?Sized + Rendering>(renderer: &T, g: &super::Geometry) -> super::Geometry {
    let tile_size = renderer.tile_size();
    super::Geometry {
        width: g.width * tile_size,
        height: g.height * tile_size,
    }
}

pub fn generate_image<T: ?Sized + Rendering>(
    path: &path::Path,
    g: super::Geometry,
    renderer: &T,
    vertical_bias: f64,
    origin: super::Origin,
    gradient: Option<Gradient>,
    algorithm: AlgorithmKind,
    animation: bool,
) {
    let grid_geometry = grid_geometry(renderer, &g);

    let mut maze = Maze::new(&grid_geometry, vertical_bias, &origin);
    let mut nb_iterations = 0u32;
    let g = image_geometry(renderer, &maze.geometry);
    let width: u16 = g.width as u16;
    let height: u16 = g.height as u16;

    let mut image = match animation {
        true => Some(File::create(path).unwrap()),
        false => None,
    };
    let mut encoder = match animation {
        true => {
            let palette = renderer.get_gif_palette();
            Some(Encoder::new(image.as_mut().unwrap(), width, height, &palette).unwrap())
        }
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
                }
                None => {
                    break;
                }
            }
        }
    }

    if animation {
        if let Some(e) = encoder.as_mut() {
            e.write_extension(ExtensionData::new_control_ext(
                100u16,
                DisposalMethod::Any,
                false,
                None,
            ))
            .unwrap();
            draw_animated_frame(&maze, e);
            e.write_extension(ExtensionData::Repetitions(Repeat::Infinite))
                .unwrap();
        }
    }

    dbg!(
        "On grid {:?}, from {:?} to {:?} (len: {}). Generated on {} iterations",
        grid_geometry,
        maze.origin(),
        maze.end(),
        maze.len().ceil(),
        nb_iterations
    );

    if !animation {
        if let Some(Gradient::Solution) = gradient {
            maze.compute_solution();
        }

        let img = maze.draw(renderer);
        let _ = img.save(path);
    }
}
