use std::cmp::{max, min};
use rand::{Rng, thread_rng};
use crate::map::Map;
use crate::rectangle::{Rectangle, RectangleIteratorType};
use crate::tile::Tile;


pub struct RoomBuilder<'a> {
    map: &'a mut Map,
    floor_tile: &'a Tile,
    wall_tile: &'a Tile,
}

impl<'a> RoomBuilder<'a> {
    pub fn new(map: &'a mut Map, floor_tile: &'a Tile, wall_tile: &'a Tile) -> Self {
        Self {
            map,
            floor_tile,
            wall_tile,
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
                self.add_horizontal_tunnel(old_center.x, new_center.x, old_center.y);
                self.add_vertical_tunnel(old_center.y, new_center.y, new_center.x);
            } else {
                self.add_vertical_tunnel(old_center.y, new_center.y, old_center.x);
                self.add_horizontal_tunnel(old_center.x, new_center.x, new_center.y);
            }
        }

        Ok(())
    }

    fn add_room(&mut self, rect: &Rectangle) {
        for (point, point_type) in rect.iter() {
            let tile = match point_type {
                RectangleIteratorType::BORDER => self.wall_tile,
                RectangleIteratorType::BODY => self.floor_tile
            };

            let point = (point.x, point.y);
            // FIXME: This tile cloning is driving me mad
            self.map.set(point, tile.clone());
        }
    }

    fn add_horizontal_tunnel(&mut self, start_x: usize, end_x: usize, y: usize) {
        for x in min(start_x, end_x) ..= max(start_x, end_x) {
            self.map.set((x, y), self.floor_tile.clone());
        }
    }

    fn add_vertical_tunnel(&mut self, start_y: usize, end_y: usize, x: usize) {
        for y in min(start_y, end_y) ..= max(start_y, end_y) {
            self.map.set((x, y), self.floor_tile.clone());
        }
    }
}

