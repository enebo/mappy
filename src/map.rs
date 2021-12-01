use std::fmt;
use std::fmt::{Display, Formatter};
use ndarray::{Array, Axis, Ix2};
use pathfinding::prelude::astar;
use pathfinding::utils::absdiff;
use rand::{Rng, thread_rng};
use crate::{Overlay, Tile};


pub struct Map {
    pub width: usize,
    pub height: usize,
    map: Array<Tile, Ix2>,
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
    type Item = ((usize, usize), &'a Tile);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.map.map.len() {
            return None;
        }

        let loc = self.map.point_for(self.index);
        let tile = self.map.get(&loc).unwrap();
        self.index += 1;

        Some((loc, tile))
    }
}


// FIXME: I had wanted loc to be reference but life time woes once I hit calling astar in shortest path.
struct CoordIterator<'a> {
    map: &'a Map,
    loc_x: usize,
    loc_y: usize,
    // Current index in POINTS
    index: usize,
    available: &'a dyn Fn(&Tile) -> bool
}

impl<'a> CoordIterator<'a> {
    fn new(map: &'a Map, loc: &(usize, usize), available: &'a (dyn Fn(&Tile) -> bool + 'a)) -> Self {
        Self {
            map,
            loc_x: loc.0,
            loc_y: loc.1,
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
    type Item = ((usize, usize), usize);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < POINTS.len() {
            let (dx, dy) = POINTS[self.index];
            self.index += 1;

            if let Some(nx) = math_is_hard(self.loc_x, dx) {
                if let Some(ny) = math_is_hard(self.loc_y, dy) {
                    if let Some(tile) = self.map.get(&(nx, ny)) {
                        if (self.available)(&tile) {
                            return Some(((nx as usize, ny as usize), tile.weight))
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
            map: Array::<Tile, Ix2>::from_elem((width, height), Tile::new(default_char, default_weight)),
        }
    }

    pub fn create_light_map(&self) -> Overlay<bool> {
        Overlay::new(self.width, self.height, false)
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
                map.set(&(x, y), Tile::new(tile, 1));
            }
        }

        Ok(map)
    }

    #[inline]
    pub(crate) fn get(&self, loc: &(usize, usize)) -> Option<&Tile> {
        self.map.get(*loc)
    }

    #[inline]
    pub(crate) fn is_valid_loc(&self, loc: &(usize, usize)) -> bool {
        loc.0 < self.width && loc.1 < self.height
    }

    #[inline]
    pub fn set(&mut self, loc: &(usize, usize), tile: Tile) -> bool {
        let spot = self.map.get_mut(*loc);
        let found = spot.is_some();

        if found {
            *spot.unwrap() = tile;
        }

        found
    }

    /// Note: Assumes all index accesses will get an index from a method which will prepare
    /// a safe index.
    fn point_for(&self, index: usize) -> (usize, usize) {
        (index % self.width, index / self.width)
    }

    // Assumes valid point
    #[inline]
    fn adjacent_ats<'a>(&'a self, loc: &(usize, usize), available: &'a (dyn Fn(&Tile) -> bool + 'a)) -> impl Iterator<Item=((usize, usize), usize)> + 'a {
        CoordIterator::new(self, loc, available)
    }

    #[inline]
    fn distance(p1: &(usize, usize), p2: &(usize, usize)) -> usize {
        absdiff(p1.0, p2.0) + absdiff(p1.1, p2.1)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=((usize, usize), &'a Tile)> + 'a {
        MapIterator::new(self)
    }

    pub fn shortest_path(&self, start: &(usize, usize), end: &(usize, usize), available: &dyn Fn(&Tile) -> bool) -> Option<(Vec<(usize, usize)>, usize)> {
        astar(&start,
              |i| self.adjacent_ats(i, available),
              |i| Self::distance(i, end),
              |i| i == end)
    }

    pub fn find_random_tile_loc(&self, tile_id: char) -> (usize, usize) {
        loop { // FIXME: This is a really scary method since it is non-deterministic and not even guaranteed to have an answer.
            let x = thread_rng().gen_range(0, self.width);
            let y = thread_rng().gen_range(0, self.height);
            if self.map.get((x, y)).unwrap().id == tile_id {
                return (x, y)
            }
        }
    }
}

impl Display for Map {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for line in self.map.axis_iter(Axis(1)) {
            let line: String = line.iter().map(|tile| tile.id).collect();
            let _ = writeln!(f, "{}", line);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Map, Tile};

    #[test]
    fn test_is_valid_loc() {
        let width = 5;
        let map = Map::new(width, 10, '.', 1);

        assert!(map.is_valid_loc(&(0, 0)));
        assert!(map.is_valid_loc(&(1, 0)));
        assert!(map.is_valid_loc(&(0, 1)));
        assert!(!map.is_valid_loc(&(6, 0)));
        assert!(!map.is_valid_loc(&(0, 10)));
    }

    #[test]
    fn test_point_for() {
        let width = 5;
        let map = Map::new(width, 10, '.', 1);

        assert_eq!(map.point_for(0), (0, 0));
        assert_eq!(map.point_for(1), (1, 0));
        assert_eq!(map.point_for(5), (0, 1));
    }

    #[test]
    fn test_get_and_set() {
        let width = 5;
        let mut map = Map::new(width, 10, '.', 1);

        let loc = (0, 0);
        assert_eq!(map.get(&loc).unwrap().id, '.');
        map.set(&loc, Tile::new('=', 1));
        assert_eq!(map.get(&loc).unwrap().id, '=');
    }

    #[test]
    fn test_adjacent_ats() {
        let width = 5;
        let map = Map::new(width, 10, '.', 1);
        let available = |tile: &Tile| tile.id == '.';

        //  +--
        //  |xo
        //  |oo
        let ats = map.adjacent_ats(&(0, 0), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(loc, _)| loc ).collect();
        assert_eq!(ats, vec![(1, 0), (0, 1), (1, 1)]);

        //  +---
        //  |oxo
        //  |ooo
        let ats = map.adjacent_ats(&(1, 0), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(loc, _)| loc ).collect();
        assert_eq!(ats, vec![(0, 0), (2, 0), (0, 1), (1, 1), (2, 1)]);

        //  +---
        //  |ooo
        //  |oxo
        //  |ooo
        let ats = map.adjacent_ats(&(1, 1), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(loc, _)| loc ).collect();
        assert_eq!(ats, vec![(0, 0), (1, 0), (2, 0), (0, 1), (2, 1), (0, 2), (1, 2), (2, 2)]);

        // --+
        // ox|
        // oo|
        let ats = map.adjacent_ats(&(4, 0), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(loc, _)| loc ).collect();
        assert_eq!(ats, vec![(3, 0), (3, 1), (4, 1)]);

        // oo|
        // ox|
        // --+
        let ats = map.adjacent_ats(&(4, 9), &available);
        let ats: Vec<(usize, usize)> = ats.map(|(loc, _)| loc ).collect();
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
        let path = map.shortest_path(&(1, 1), &(12, 1), &available);
        if let Some(path) = path {
            let (path, distance) = path;
            println!("distance {}", distance);
            let route: Vec<_> = path.iter().collect();
            println!("Path {:?}", route);
            for i in &path {
                let tile = Tile::new('x', 1);
                map.set(i, tile);
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