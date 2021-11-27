use std::fmt::{Display, Formatter};
use core::fmt;
use nalgebra::{Point2};
use pathfinding::prelude::astar;
use pathfinding::utils::absdiff;
use rand::{Rng, thread_rng};

pub type Point = Point2<usize>;

pub mod rectangle;
pub mod builders;

#[derive(Debug)]
pub struct MyError {}

#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub id: char,
    weight: usize,
}

impl Tile {
    pub fn new(id: char, weight: usize) -> Tile {
        Tile {
            id,
            weight,
        }
    }
}

// FIXME: dynamic maps vs pre-compiled sizes and using a primitive array.
// This also makes me wonder how much you could compile a map so more actions with a map are precomputed.
pub struct Map {
    pub width: usize,
    pub height: usize,
    map: Vec<Tile>
}

struct MapIterator<'a> {
    map: &'a Map,
    index: usize,
}

impl<'a> MapIterator<'a> {
    fn new(map: &'a Map) -> Self {
        Self {
            map,
            index: 0,
        }
    }
}

impl<'a> Iterator for MapIterator<'a> {
    type Item = (Point, &'a Tile);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.map.map.len() {
            return None;
        }

        let point = self.map.point_for(self.index);
        let tile = self.map.at(&point).unwrap();
        self.index += 1;

        Some((point, tile))
    }
}


// FIXME: I had wanted loc to be reference but life time woes once I hit calling astar in shortest path.
struct CoordIterator<'a> {
    map: &'a Map,
    loc: Point,
    // Current index in POINTS
    index: usize,
    available: &'a dyn Fn(&Tile) -> bool
}

impl<'a> CoordIterator<'a> {
    fn new(map: &'a Map, loc: Point, available: &'a (dyn Fn(&Tile) -> bool + 'a)) -> Self {
        Self {
            map,
            loc,
            index: 0,
            available
        }
    }
}

const POINTS: [(isize, isize); 8] = [
    (-1, -1),  // upper left
    (0, -1),   // up
    (1, -1),   // upper right
    (-1, 0),   // left
    (1, 0),    // right
    (-1, 1),   // lower left
    (0, 1),    // down
    (1, 1)     // lower right
];

#[inline]
pub const fn math_is_hard(x: usize, d: isize) -> Option<usize> {
    let result = x as isize + d;

    if result.is_negative() {
        None
    } else {
        Some(result as usize)
    }
}

impl<'a> Iterator for CoordIterator<'a> {
    type Item = (Point, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < POINTS.len() {
            let (dx, dy) = POINTS[self.index];
            self.index += 1;

            if let Some(nx) = math_is_hard(self.loc.x, dx) {
                if let Some(ny) = math_is_hard(self.loc.y, dy) {
                    if let Some(tile) = self.map.at_raw(nx as usize, ny as usize) {
                        if (self.available)(&tile) {
                            return Some((Point::new(nx as usize, ny as usize), tile.weight))
                        }
                    }
                }
            }
        }

        None
    }
}

impl Map {
    pub fn new(width: usize, height: usize, default_char: char, default_weight: usize) -> Self {
        Self {
            width,
            height,
            map: vec![Tile::new(default_char, default_weight); width * height],
        }
    }

    pub fn generate_ascii_map(ascii_map: &str) -> Result<Self, ()> {
        let rows: Vec<&str> = ascii_map.split_terminator('\n').collect();
        let height = rows.len();

        if height == 0 {
            return Err(())
        }

        let width = rows[0].len();

        // verify all lines are same length;
        if let Some(_) = rows.iter().find(|e| e.len() != width) {
            return Err(())
        }

        let mut map = Map::new(width, height, '.', 1);

        for (y, row) in rows.iter().enumerate() {
            for (x, tile) in row.chars().enumerate() {
                // FIXME: All tiles will be immutable so share them all.
                let tile = Tile::new(tile, 1);
                let point = Point::new(x, y);

                map.set_at(&point, tile).unwrap();
            }
        }

        Ok(map)
    }

    /// Note: Assumes all index accesses will get an index from a method which will prepare
    /// a safe index.
    pub fn at(&self, loc: &Point) -> Option<&Tile> {
        self.is_valid_loc(loc).map(|index| &self.map[index])
    }

    fn at_raw(&self, x: usize, y: usize) -> Option<&Tile> {
        if x >= self.width || y >= self.height {
            None
        } else {
            Some(&self.map[y * self.width + x])
        }
    }

    fn is_valid_loc(&self, loc: &Point) -> Option<usize> {
        if loc.x >= self.width || loc.y >= self.height {
            None
        } else {
            Some(self.at_xy_raw(loc))
        }
    }

    pub fn set_at(&mut self, loc: &Point, tile: Tile) -> Result<(), MyError>{
        if let Some(index) = self.is_valid_loc(loc) {
            self.map[index] = tile;
            Ok(())
        } else {
            Err(MyError{})
        }
    }

    #[inline]
    fn at_xy_raw(&self, loc: &Point) -> usize {
        loc.y * self.width + loc.x
    }

    /// Note: Assumes all index accesses will get an index from a method which will prepare
    /// a safe index.
    fn point_for(&self, index: usize) -> Point {
        Point::new(index % self.width, index / self.width)
    }

    // Assumes valid point
    #[inline]
    fn adjacent_ats<'a>(&'a self, loc: Point, available: &'a (dyn Fn(&Tile) -> bool + 'a)) -> impl Iterator<Item=(Point, usize)> + 'a {
        CoordIterator::new(self, loc, available)
    }

    #[inline]
    fn distance(p1: &Point, p2: &Point) -> usize {
        absdiff(p1.x, p2.x) + absdiff(p1.y, p2.y)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=(Point, &'a Tile)> + 'a {
        MapIterator::new(self)
    }

    pub fn shortest_path(&self, start: &Point, end: &Point, available: &dyn Fn(&Tile) -> bool) -> Option<(Vec<Point>, usize)> {
        astar(&start,
              |i| self.adjacent_ats(i.clone(), available),
              |i| Self::distance(i, end),
              |i| i == end)
    }

    pub fn find_random_tile_loc(&self, tile_id: char) -> Point {
        loop { // FIXME: This is a really scary method since it is non-deterministic and not even guaranteed to have an answer.
            let index = thread_rng().gen_range(0, self.map.len());
            if self.map.get(index).unwrap().id == tile_id {
                return self.point_for(index);
            }
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let stream: Vec<char> = self.map.iter().map(|e| e.id).collect();
        let split = &stream.chunks(self.width).map(|c| c.iter().collect::<String>()).collect::<Vec<_>>();
        for line in split {
            let _ = writeln!(f, "{}", line);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Map, Point, Tile};

    #[test]
    fn test_is_valid_loc() {
        let width = 5;
        let map = Map::new(width, 10, '.', 1);

        assert_eq!(map.is_valid_loc(&Point::new(0, 0)), Some(0));
        assert_eq!(map.is_valid_loc(&Point::new(1, 0)), Some(1));
        assert_eq!(map.is_valid_loc(&Point::new(0, 1)), Some(5));
        assert_eq!(map.is_valid_loc(&Point::new(6, 0)), None);
        assert_eq!(map.is_valid_loc(&Point::new(0, 10)), None);
    }

    #[test]
    fn test_point_for() {
        let width = 5;
        let map = Map::new(width, 10, '.', 1);

        assert_eq!(map.point_for(0), Point::new(0, 0));
        assert_eq!(map.point_for(1), Point::new(1, 0));
        assert_eq!(map.point_for(5), Point::new(0, 1));
    }

    #[test]
    fn test_at_and_set_at() {
        let width = 5;
        let mut map = Map::new(width, 10, '.', 1);

        let point = &Point::new(0, 0);
        assert_eq!(map.at(point).unwrap().id, '.');
        map.set_at(point, Tile::new('=', 1)).unwrap();
        assert_eq!(map.at(point).unwrap().id, '=');
    }

    #[test]
    fn test_adjacent_ats() {
        let width = 5;
        let map = Map::new(width, 10, '.', 1);
        let available = |tile: &Tile| tile.id == '.';

        //  +--
        //  |xo
        //  |oo
        let ats = map.adjacent_ats(Point::new(0, 0), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(i, _)| (i.x, i.y)).collect();
        assert_eq!(ats, vec![(1, 0), (0, 1), (1, 1)]);

        //  +---
        //  |oxo
        //  |ooo
        let ats = map.adjacent_ats(Point::new(1, 0), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(i, _)| (i.x, i.y)).collect();
        assert_eq!(ats, vec![(0, 0), (2, 0), (0, 1), (1, 1), (2, 1)]);

        //  +---
        //  |ooo
        //  |oxo
        //  |ooo
        let ats = map.adjacent_ats(Point::new(1, 1), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(i, _)| (i.x, i.y)).collect();
        assert_eq!(ats, vec![(0, 0), (1, 0), (2, 0), (0, 1), (2, 1), (0, 2), (1, 2), (2, 2)]);

        // --+
        // ox|
        // oo|
        let ats = map.adjacent_ats(Point::new(4, 0), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(i, _)| (i.x, i.y)).collect();
        assert_eq!(ats, vec![(3, 0), (3, 1), (4, 1)]);

        // oo|
        // ox|
        // --+
        let ats = map.adjacent_ats(Point::new(4, 9), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(i, _)| (i.x, i.y)).collect();
        assert_eq!(ats, vec![(3, 8), (4, 8), (3, 9)]);
    }

    #[test]
    fn test_generate_ascii_map() {
        let map_string = "##############\n\
                                #..#......#..#\n\
                                #...##.#.....#\n\
                                #..##...#.#..#\n\
                                #..######.#..#\n\
                                #............#\n\
                                ##############";


        let mut map = Map::generate_ascii_map(map_string).unwrap();
        /*        assert_eq!(map.width, 14);
                assert_eq!(map.height, 4);
                assert_eq!(map.at(map.at_xy(0, 0).unwrap()), TileType::Wall);
                assert_eq!(map.at(map.at_xy(1, 1).unwrap()), TileType::Floor);*/
        println!("{}", map);

        let available = |tile: &Tile| tile.id == '.';
        let start = Point::new(1, 1);
        let end = Point::new(12, 1);
        let path = map.shortest_path(&start, &end, &available);
        if let Some(path) = path {
            let (path, distance) = path;
            println!("distance {}", distance);
            let route: Vec<_> = path.iter().collect();
            println!("Path {:?}", route);
            for i in &path {
                let tile = Tile::new('x', 1);
                map.set_at(i, tile).unwrap();
            }
            println!("{}", map);
        }
    }

    #[test]
    fn test_map_iterator() {
        let map_string = "123\n\
                                #.#\n\
                                ###";
        let map = Map::generate_ascii_map(map_string).unwrap();

        let string: String = map.iter().map(|(_, tile)| tile.id).collect();

        assert_eq!(string, "123#.####");
    }
}