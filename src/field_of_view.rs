use crate::{Map, Point, Tile};

const MULTIPLIERS: [(isize, isize, isize, isize); 8] = [
    (1, 0, 0, 1),
    (0, 1, 1, 0),
    (0, -1, 1, 0),
    (-1, 0, 0, 1),
    (-1, 0, 0, -1),
    (0, -1, -1, 0),
    (0, 1, -1, 0),
    (1, 0, 0, -1),
];

// FIXME: Replace with some bit datastructure
// FIXME: probably want a more features FOV map which can be merged with actual map for at least debugging.

// http://www.roguebasin.com/index.php/FOV_using_recursive_shadowcasting
pub fn calculate_field_of_view(map: &mut Map, start: &Point, radius: usize,
                               light_map: &mut Vec<bool>, visible: &dyn Fn(&Tile) -> bool) {
    for point in &mut light_map.iter_mut() {
        *point = false;
    }

    let index = map.width * start.y + start.x;
    let _ = std::mem::replace(&mut light_map[index], true);

    for multipliers in MULTIPLIERS {
        shadow_cast(1, 1.0, 0.0, multipliers, radius, start, light_map, map, visible);
    }
}

fn shadow_cast(row: usize, mut begin: f32, end: f32, mults: (isize, isize, isize, isize),
               radius: usize, start: &Point, light_map: &mut Vec<bool>, map: &Map,
               visible: &dyn Fn(&Tile) -> bool) {
    if begin < end {
        return
    }

    let radius_2: isize = radius as isize * radius as isize;
    let mut new_begin = 0.;
    let mut blocked = false;
    for y in row..radius {
        let mut dx = -1 * y as isize - 1;
        let dy = -1 * y as isize;
        while dx <= 0 {
            dx += 1;
            let current_x = start.x as isize + dx * mults.0 + dy * mults.1;
            let current_y = start.y as isize + dx * mults.2 + dy * mults.3;

            if current_x < 0 || current_y < 0 { // Make sure we are still on the map.
                continue
            }

            let current = Point::new(current_x as usize, current_y as usize);

            // Slope at right edge of current square.
            let right_slope = (dx as f32 + 0.5) / (dy as f32 - 0.5);
            if begin < right_slope {
                continue
            }

            // FIXME: We need to know valid to change light map but it feels like extra work is happening.
            if map.is_valid_loc(&current).is_none() {
                continue
            }

            let left_slope = (dx as f32 - 0.5) / (dy as f32 + 0.5);
            if end > left_slope {
                break
            }

            if dx * dx + dy * dy < radius_2 {
                let index = map.width * current.y + current.x;
                let _ = std::mem::replace(&mut light_map[index], true);
            }

            if blocked {
                if !(visible)(&map.at(&current).unwrap()) {
                    // Already blocked for the last 'column'.  More of the same continue on until
                    // we find an opon spot.  Keep track of slope to use it when we unblock (nothing
                    // to the left can be seen from this point on next rows).
                    new_begin = right_slope;
                } else {
                    // We see an open position.  Unblock.  Use previous saved scope as safe slope
                    // we know we cannot see anything to the left of.
                    blocked = false;
                    begin = new_begin;
                }
            } else {
                if !(visible)(&map.at(&current).unwrap()) && y < radius {
                    // Ran into our first blocked item.  Scan next row but only up to new slope since
                    // we know we can see nothing more to the right of it.
                    blocked = true;
                    shadow_cast(y + 1, begin, left_slope, mults, radius, start, light_map, map, visible);
                    new_begin = right_slope;
                }
            }
        }
        if blocked {
            break
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Map, Point, Tile};
    use crate::field_of_view::calculate_field_of_view;

    const FOV_MAP: &str = ".................\n\
                           .......###.......\n\
                           .......###.......\n\
                           .......###..#..##\n\
                           .......##........\n\
                           .................\n\
                           .................\n";

    const FOV_ANSWER_7_6: &str = "......@@@@@@....@\n\
                                  .......@@@@....@@\n\
                                  .......@@@....@.@\n\
                                  .......@@........\n\
                                  .................\n\
                                  .................\n\
                                  .................\n";

    #[test]
    fn test_fov() {
        let start = Point::new(7, 6);
        let mut map = Map::generate_ascii_map(FOV_MAP).unwrap();
        let mut light_map = vec![false; map.width * map.height];
        let visible = |tile: &Tile| tile.id == '.';
        calculate_field_of_view(&mut map, &start, 20, &mut light_map, &visible);

        let ascii: Vec<char> = light_map.iter().map(|v| if *v { '.'} else { '@' }).collect();
        let result: String = ascii.chunks(map.width).map(|chunk| chunk.iter().collect::<String>() + "\n" ).collect();

        println!("{}", result);
        assert_eq!(result, FOV_ANSWER_7_6)
    }
}