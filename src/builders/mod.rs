use std::cmp::{max, min};
use rand::{Rng, thread_rng};
use crate::map::Map;
use crate::rectangle::{Rectangle, RectangleIteratorType};


pub struct RoomBuilder<'a, T: Clone + PartialEq> {
    map: &'a mut Map<T>,
    floor_fn: &'a dyn Fn((usize, usize)) -> T,
    wall_fn: &'a dyn Fn((usize, usize)) -> T,
}

impl<'a, T: Clone + PartialEq> RoomBuilder<'a, T> {
    pub fn new(map: &'a mut Map<T>, floor_fn: &'a dyn Fn((usize, usize)) -> T, wall_fn: &'a dyn Fn((usize, usize)) -> T) -> Self {
        Self {
            map,
            floor_fn,
            wall_fn,
        }
    }

    pub fn create(&mut self, max_rooms: usize, min_size: usize, max_size: usize) -> Result<(), String>{
        let mut rng = thread_rng();

        if min_size < 3 {
            return Err("min_size too small (must be >3".to_string())
        }

        if max_size > self.map.width || max_size > self.map.height {
            return Err("max_size too large".to_string())
        }

        let mut rooms: Vec<Rectangle> = Vec::with_capacity(max_rooms);

        for _ in 0..max_rooms {
            let width = rng.gen_range(min_size, max_size);
            let height = rng.gen_range(min_size, max_size);
            let x = rng.gen_range(0, self.map.width - width - 1);
            let y = rng.gen_range(0, self.map.height - height - 1);
            let new_room = Rectangle::new(x, y, width, height).unwrap();

            if rooms.iter().find(|room| new_room.intersect(room)).is_none() {
                self.add_room(&new_room);
                rooms.push(new_room);
            }
        }

        for (i, room) in rooms.iter().skip(1).enumerate() {
            let new_center = room.center();
            let old_center = rooms[i].center();

            if rng.gen_range(0, 2) == 1 {
                self.add_horizontal_tunnel(old_center.0, new_center.0, old_center.1);
                self.add_vertical_tunnel(old_center.1, new_center.1, new_center.0);
            } else {
                self.add_vertical_tunnel(old_center.1, new_center.1, old_center.0);
                self.add_horizontal_tunnel(old_center.0, new_center.0, new_center.1);
            }
        }

        Ok(())
    }

    fn add_room(&mut self, rect: &Rectangle) {
        for (point, point_type) in rect.iter() {
            let tile_fn = match point_type {
                RectangleIteratorType::BORDER => self.wall_fn,
                RectangleIteratorType::BODY => self.floor_fn
            };

            // FIXME: This tile cloning is driving me mad
            self.map.set(&point, tile_fn(point));
        }
    }

    fn add_horizontal_tunnel(&mut self, start_x: usize, end_x: usize, y: usize) {
        for x in min(start_x, end_x) ..= max(start_x, end_x) {
            let loc = (x, y);
            self.map.set(&loc, (self.floor_fn)(loc));
        }
    }

    fn add_vertical_tunnel(&mut self, start_y: usize, end_y: usize, x: usize) {
        for y in min(start_y, end_y) ..= max(start_y, end_y) {
            let loc = (x, y);
            self.map.set(&loc, (self.floor_fn)(loc));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::builders::RoomBuilder;
    use crate::Map;

    #[test]
    fn test_runs() {
        let mut map = Map::new("map", 50, 50, &|_| '#');
        let mut builder = RoomBuilder::new(&mut map, &|_| '.', &|_| '#');

        builder.create(7, 4, 10).unwrap();
    }
}
