use std::cmp::{max, min};
use rand::{Rng, thread_rng};
use crate::{Map, Rectangle, RectangleIteratorType, Spot};

pub struct RoomBuilder<'a, T: PartialEq, I: Default + PartialEq> {
    map: &'a mut Map<T, I>,
    floor_fn: &'a dyn Fn((usize, usize)) -> T,
    wall_fn: &'a dyn Fn((usize, usize)) -> T,
}

impl<'a, T: PartialEq, I: Default + PartialEq> RoomBuilder<'a, T, I> {
    pub fn new(map: &'a mut Map<T, I>, floor_fn: &'a dyn Fn((usize, usize)) -> T, wall_fn: &'a dyn Fn((usize, usize)) -> T) -> Self {
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
                self.render_room(&new_room);
                rooms.push(new_room);
            }
        }

        for (i, room) in rooms.iter().skip(1).enumerate() {
            let new_center = room.center();
            let old_center = rooms[i].center();

            if rng.gen_range(0, 2) == 1 {
                self.render_horizontal_tunnel(old_center.0, new_center.0, old_center.1);
                self.render_vertical_tunnel(old_center.1, new_center.1, new_center.0);
            } else {
                self.render_vertical_tunnel(old_center.1, new_center.1, old_center.0);
                self.render_horizontal_tunnel(old_center.0, new_center.0, new_center.1);
            }
        }

        for room in rooms.drain(0..) {
            self.map.add_room(room);
        }

        Ok(())
    }

    fn render_room(&mut self, rect: &Rectangle) {
        for (point, point_type) in rect.iter() {
            let tile_fn = match point_type {
                RectangleIteratorType::BORDER => self.wall_fn,
                RectangleIteratorType::BODY => self.floor_fn
            };

            self.map.set(&point, Spot::new(tile_fn(point), None));
        }
    }

    fn render_horizontal_tunnel(&mut self, start_x: usize, end_x: usize, y: usize) {
        for x in min(start_x, end_x) ..= max(start_x, end_x) {
            let loc = (x, y);
            self.map.set(&loc, Spot::new((self.floor_fn)(loc), None));
        }
    }

    fn render_vertical_tunnel(&mut self, start_y: usize, end_y: usize, x: usize) {
        for y in min(start_y, end_y) ..= max(start_y, end_y) {
            let loc = (x, y);
            self.map.set(&loc, Spot::new((self.floor_fn)(loc), None));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::builders::RoomBuilder;
    use crate::Map;

    #[test]
    fn test_runs() {
        let mut map: Map<char, char> = Map::new("map", 50, 50, &|_| '#');
        let mut builder = RoomBuilder::new(&mut map, &|_| '.', &|_| '#');

        builder.create(7, 4, 10).unwrap();
    }
}
