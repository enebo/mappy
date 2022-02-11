use ndarray::{Array, Ix2};
use pathfinding::prelude::astar;
use pathfinding::utils::absdiff;
use rand::{Rng, thread_rng};
use crate::{add_delta, Overlay};
use crate::rectangle::Rectangle;


pub struct Map<T: PartialEq> {
    pub name: String,
    pub width: usize,
    pub height: usize,
    // FIXME: A trait for different shape rooms is desired here but until I understand what the needs are we will use one struct
    pub rooms: Vec<Rectangle>,
    map: Array<T, Ix2>,
}

struct MapIterator<'a, T: PartialEq> {
    map: &'a Map<T>,
    index: usize,
}

impl<'a, T: PartialEq> MapIterator<'a, T> {
    fn new(map: &'a Map<T>) -> Self {
        Self {
            map,
            index: 0,
        }
    }
}

impl<'a, T: PartialEq> Iterator for MapIterator<'a, T> {
    type Item = ((usize, usize), &'a T);

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
struct CoordIterator<'a, T: PartialEq, U: PartialEq> {
    map: &'a Map<T>,
    loc: (usize, usize),
    // Current index in POINTS
    index: usize,
    available: &'a dyn Fn(&T) -> U,
    invalid: U,
    include_diagonals: bool,
}

impl<'a, T: PartialEq, U: PartialEq> CoordIterator<'a, T, U> {
    fn new(map: &'a Map<T>, loc: &(usize, usize), available: &'a (dyn Fn(&T) -> U + 'a), invalid: U, include_diagonals: bool) -> Self {
        Self {
            map,
            loc: *loc,
            index: 0,
            available,
            invalid,
            include_diagonals,
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

const SIMPLE_POINTS: [(isize, isize); 4] = [
    (0, -1),   // up
    (-1, 0),   // left
    (1, 0),    // right
    (0, 1),    // down
];

impl<'a, T: PartialEq, U: PartialEq> Iterator for CoordIterator<'a, T, U> {
    type Item = ((usize, usize), U);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.include_diagonals {
            assert!(POINTS.len() >= 8);
            while self.index < POINTS.len() {
                let delta = POINTS[self.index];
                self.index += 1;

                if let Some(loc) = add_delta(&self.loc, &delta) {
                    if let Some(tile) = self.map.get(&loc) {
                        let test = (self.available)(&tile);
                        if test != self.invalid {
                            return Some((loc, test))
                        }
                    }
                }
            }
        } else {
            assert!(POINTS.len() >= 4);
            while self.index < SIMPLE_POINTS.len() {
                let delta = SIMPLE_POINTS[self.index];
                self.index += 1;

                if let Some(loc) = add_delta(&self.loc, &delta) {
                    if let Some(tile) = self.map.get(&loc) {
                        let test = (self.available)(&tile);
                        if test != self.invalid {
                            return Some((loc, test))
                        }
                    }
                }
            }

        }

        None
    }
}

pub fn generate_ascii_map<S: Into<String>>(name: S, ascii_map: &str) -> Result<Map<char>, ()> {
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

    let default_fn = |_| '.';
    let mut map: Map<char> = Map::new(name.into(), width, height, &default_fn);


    for (y, row) in rows.iter().enumerate() {
        for (x, tile) in row.chars().enumerate() {
            map.set(&(x, y), tile);
        }
    }

    Ok(map)
}

impl<T: PartialEq> Map<T> {
    pub fn new<S: Into<String>>(name: S, width: usize, height: usize, default_fn: &dyn Fn((usize, usize)) -> T) -> Self {
        Self {
            name: name.into(),
            width,
            height,
            rooms: vec![],
            map: Array::<T, Ix2>::from_shape_fn((width, height), default_fn),
        }
    }

    pub fn add_room(&mut self, rect: Rectangle) {
        self.rooms.push(rect);
    }

    pub fn create_overlay(&self) -> Overlay<bool> {
        Overlay::new(self.width, self.height, false)
    }

    #[inline]
    pub fn get(&self, loc: &(usize, usize)) -> Option<&T> {
        self.map.get(*loc)
    }

    #[inline]
    pub fn is_valid_loc(&self, loc: &(usize, usize)) -> bool {
        loc.0 < self.width && loc.1 < self.height
    }

    #[inline]
    pub fn set(&mut self, loc: &(usize, usize), tile: T) -> bool {
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
    fn adjacent_ats<'a>(&'a self, loc: &(usize, usize), available: &'a (dyn Fn(&T) -> usize + 'a)) -> impl Iterator<Item=((usize, usize), usize)> + 'a {
        CoordIterator::new(self, loc, available, 0, true)
    }

    #[inline]
    fn distance(p1: &(usize, usize), p2: &(usize, usize)) -> usize {
        absdiff(p1.0, p2.0) + absdiff(p1.1, p2.1)
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=((usize, usize), &'a T)> + 'a {
        MapIterator::new(self)
    }

    /// Search all paths around the current loc and return bit pattern representing all the
    /// matching locations.  The bit pattern itself is 3 bits per line and will contain a 1
    /// if there is an adjacent matching loc (test fn returns true).  For example:
    ///
    /// ###
    /// # #
    /// ###
    ///
    /// (0,0) -> 0b_000_001_010
    /// (1,0) -> 0b_000_101_000
    /// (2,0) -> 0b_000_100_010
    ///
    /// For sprites in a game you may use this function to exclude diagonals to get the limited
    /// number of sprites for representing a fence where adjacent other fence types should connect
    /// together.
    ///
    /// For combat you could use this to look for adjacent monsters?
    pub fn adjacent_paths(&self, loc: &(usize, usize), test: &dyn Fn(&T) -> bool, include_diagonals: bool) -> usize {
        let iter = CoordIterator::new(self, loc, &test, false, include_diagonals);

        let mut result = 0;
        for ((x, y), _) in iter {
            let (dx, dy) = (x as isize - loc.0 as isize, y as isize - loc.1 as isize);
            let bits = 1 << ((dx*-1 + 1) + ((dy*-1 + 1) * 3));

            result = result | bits
        }
        result
    }

    pub fn shortest_path(&self, start: &(usize, usize), end: &(usize, usize), available: &dyn Fn(&T) -> usize) -> Option<(Vec<(usize, usize)>, usize)> {
        astar(&start,
              |i| self.adjacent_ats(i, available),
              |i| Self::distance(i, end),
              |i| i == end)
    }

    pub fn find_random_tile_loc(&self, available: &dyn Fn(&T) -> bool) -> Result<(usize, usize), ()> {
        let room_count = self.rooms.len();
        let room_index = thread_rng().gen_range(0, room_count);
        let room = self.rooms.get(room_index).unwrap();

        for _ in 0..100 {
            let loc = (room.random_x(false), room.random_y(false));
            if (available)(&self.map.get(loc).unwrap()) {
                return Ok(loc)
            }
        }

        Err(())
    }
}

/*
impl<T: Clone + PartialEq> Display for Map<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for line in self.map.axis_iter(Axis(1)) {
            let line: String = line.iter().map(|tile| tile.id).collect();
            let _ = writeln!(f, "{}", line);
        }
        Ok(())
    }
}*/

#[cfg(test)]
mod tests {
    use std::collections::{HashMap};
    use crate::Map;
    use crate::map::generate_ascii_map;
    use crate::rectangle::Rectangle;

    #[test]
    fn test_random_tile_loc() {
        //  +-+
        //  |.|
        //  +-+
        let mut map = Map::new("map", 3, 3, &|_| '.');
        let room = Rectangle::new(0, 0, 3, 3).unwrap();
        map.add_room(room);
        assert!(map.find_random_tile_loc(&|c| *c == '.').is_ok());

        assert!(map.find_random_tile_loc(&|c| *c != '.').is_err());
    }

    #[test]
    fn test_is_valid_loc() {
        let width = 5;
        let map = Map::new("map", width, 10, &|_| '.');

        assert!(map.is_valid_loc(&(0, 0)));
        assert!(map.is_valid_loc(&(1, 0)));
        assert!(map.is_valid_loc(&(0, 1)));
        assert!(!map.is_valid_loc(&(6, 0)));
        assert!(!map.is_valid_loc(&(0, 10)));
    }

    #[test]
    fn test_point_for() {
        let width = 5;
        let map = Map::new("map", width, 10, &|_| '.');

        assert_eq!(map.point_for(0), (0, 0));
        assert_eq!(map.point_for(1), (1, 0));
        assert_eq!(map.point_for(5), (0, 1));
    }

    #[test]
    fn test_get_and_set() {
        let width = 5;
        let mut map = Map::new("map", width, 10, &|_| '.');

        let loc = (0, 0);
        assert_eq!(map.get(&loc).unwrap(), &'.');
        map.set(&loc, '=');
        assert_eq!(map.get(&loc).unwrap(), &'=');
    }

    #[test]
    fn test_adjacent_ats() {
        let width = 5;
        let map = Map::new("map", width, 10, &|_| '.');
        let available = |tile: &char| if tile == &'.' { 1 } else { 0 } ;

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
    fn test_adjacent_paths() {
        let map_string = "##############\n\
                          #..#......#..#\n\
                          #..###.#.....#\n\
                          #..###..#.#..#\n\
                          #..######.#..#\n\
                          #............#\n\
                          ##############";
        let map = generate_ascii_map("map", map_string).unwrap();

        let mut pattern = map.adjacent_paths(&(0, 0), &|c| *c == '#', false);
        println!("Upper left corner");
        println!("Pattern: {:03b}", ((pattern >> 6) & 7));
        println!("Pattern: {:03b}", ((pattern >> 3) & 7));
        println!("Pattern: {:03b}", (pattern & 7));
        assert_eq!(pattern, 0b_000_001_010);
        pattern = map.adjacent_paths(&(13, 0), &|c| *c == '#', false);
        println!("Upper right corner");
        println!("Pattern: {:03b}", ((pattern >> 6) & 7));
        println!("Pattern: {:03b}", ((pattern >> 3) & 7));
        println!("Pattern: {:03b}", (pattern & 7));
        assert_eq!(pattern, 0b_000_100_010);
        pattern = map.adjacent_paths(&(4, 3), &|c| *c == '#', true);
        println!("Surrounded");
        println!("Pattern: {:03b}", ((pattern >> 6) & 7));
        println!("Pattern: {:03b}", ((pattern >> 3) & 7));
        println!("Pattern: {:03b}", (pattern & 7));
        assert_eq!(pattern, 0b_111_101_111);

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


        let mut map = generate_ascii_map("map", map_string).unwrap();
        let mut weights = HashMap::new();
        weights.insert('.', 1 as usize);
        let available = |tile: &char| *weights.get(tile).unwrap_or(&0);
        let path = map.shortest_path(&(1, 1), &(12, 1), &available);
        if let Some(path) = path {
            let (path, distance) = path;
            println!("distance {}", distance);
            let route: Vec<_> = path.iter().collect();
            println!("Path {:?}", route);
            for i in &path {
                map.set(i, 'x');
            }

            // FIXME: Add weighted test (use 123 as tiles which will just be their weight.
            // FIXME: WOT!
            for line in map.iter().map(|(_, tile)| *tile).collect::<Vec<char>>().chunks(map.width) {
                for c in line {
                    print!("{}", *c);
                }
                println!("");

            }
        }
    }

    #[test]
    fn test_map_iterator() {
        let map_string = "123\n\
                                #.#\n\
                                ###";
        let map = generate_ascii_map("map", map_string).unwrap();

        let string: String = map.iter().map(|(_, tile)| tile).collect();

        assert_eq!(string, "123#.####");
    }
}