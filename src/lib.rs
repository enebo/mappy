use std::fmt::{Display, Formatter};
use core::fmt;
use nalgebra::{Point2};
use pathfinding::prelude::astar;
use pathfinding::utils::absdiff;

pub type Point = Point2<usize>;

#[derive(Debug)]
pub struct MyError {}

#[derive(Clone, Debug)]
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

pub struct Map {
    width: usize,
    height: usize,
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
    type Item = (Point, Tile);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.map.map.len() {
            return None;
        }

        let point = self.map.point_for(self.index);
        let tile = self.map.at(&point).unwrap();
        self.index += 1;

        // FIXME: It bugs me that I am having to clone immutable data here.
        Some((point, tile.clone()))
    }
}


// FIXME: I had wanted loc to be reference but life time woes once I hit calling astar in shortest path.
struct CoordIterator<'a> {
    map: &'a Map,
    loc: Point,
    // Current index in POINTS
    index: usize,
    available: &'a dyn Fn(&Map, &Point, usize) -> bool
}

impl<'a> CoordIterator<'a> {
    fn new(map: &'a Map, loc: Point, available: &'a (dyn Fn(&Map, &Point, usize) -> bool + 'a)) -> Self {
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


impl<'a> Iterator for CoordIterator<'a> {
    type Item = (Point, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < POINTS.len() {
            let (dx, dy) = POINTS[self.index];
            self.index += 1;

            let nx = (self.loc.x as isize) + dx;
            if !nx.is_negative() {
                let ny = (self.loc.y as isize) + dy;
                if !ny.is_negative() {
                    let new_loc = Point::new(nx as usize, ny as usize);
                    if let Some(tile) = self.map.at(&new_loc) {
                        if (self.available)(&self.map, &new_loc, tile.weight) {
                            return Some((new_loc, tile.weight))
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

    /// Note: Assumes all index accesses will get an index from a method which will prepare
    /// a safe index.
    pub fn at(&self, loc: &Point) -> Option<&Tile> {
        if let Some(index) = self.is_valid_loc(loc) {
            return Some(&self.map[index]);
        }

        None
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

    fn at_xy_raw(&self, loc: &Point) -> usize {
        loc.y * self.width + loc.x
    }

    /// Note: Assumes all index accesses will get an index from a method which will prepare
    /// a safe index.
    fn point_for(&self, index: usize) -> Point {
        Point::new(index % self.width, index / self.width)
    }

    // Assumes valid point
    fn adjacent_ats<'a>(&'a self, loc: Point, available: &'a (dyn Fn(&Map, &Point, usize) -> bool + 'a)) -> impl Iterator<Item=(Point, usize)> + 'a {
        CoordIterator::new(self, loc, available)
    }

    fn distance(p1: &Point, p2: &Point) -> usize {
        absdiff(p1.x, p2.x) + absdiff(p1.y, p2.y)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=(Point, Tile)> + 'a {
        MapIterator::new(self)
    }

    pub fn shortest_path(&self, start: &Point, end: &Point, available: &dyn Fn(&Map, &Point, usize) -> bool) -> Option<(Vec<Point>, usize)> {
        astar(&start,
              |i| self.adjacent_ats(i.clone(), available),
              |i| Self::distance(i, end),
              |i| i == end)
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

    pub fn generate_ascii_map(ascii_map: &str) -> Option<Map> {
        let rows: Vec<&str> = ascii_map.split_terminator('\n').collect();
        let height = rows.len();

        if height == 0 {
            return None;
        }

        let width = rows[0].len();

        // verify all lines are same length;
        if let Some(_) = rows.iter().find(|e| e.len() != width) {
            return None;
        }

        println!("Making map of size: {}x{}", width, height);
        let mut map = Map::new(width, height, '.', 1);

        for (y, row) in rows.iter().enumerate() {
            for (x, tile) in row.chars().enumerate() {
                // FIXME: All tiles will be immutable so share them all.
                let tile = Tile::new(tile, 1);
                let point = Point::new(x, y);

                map.set_at(&point, tile).unwrap();
            }
        }

        Some(map)
    }

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
        let available = |map: &Map, point: &Point, weight| map.at(point).unwrap().id == '.';

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


        let mut map = generate_ascii_map(map_string).unwrap();
        /*        assert_eq!(map.width, 14);
                assert_eq!(map.height, 4);
                assert_eq!(map.at(map.at_xy(0, 0).unwrap()), TileType::Wall);
                assert_eq!(map.at(map.at_xy(1, 1).unwrap()), TileType::Floor);*/
        println!("{}", map);

        let available = |map: &Map, point: &Point, weight| map.at(point).unwrap().id == '.';
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
        let map = generate_ascii_map(map_string).unwrap();

        let string: String = map.iter().map(|(_, tile)| tile.id).collect();

        assert_eq!(string, "123#.####");
    }
}