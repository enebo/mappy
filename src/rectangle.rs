use rand::{thread_rng, Rng};
use crate::rectangle::RectangleIteratorType::{BODY, BORDER};

/// Rectangle with a single width border.
pub struct Rectangle {
    pub ulc: (usize, usize),
    pub lrc: (usize, usize),
}

impl Rectangle {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Result<Self, String> {
        // We need a minimum of a wall on each side of the room but the room may have no
        // open spaces.
        if width < 2 || height < 2 {
            return Err("Rectangle dimensions too small".to_string())
        }

        Ok(Rectangle {
            ulc: (x, y),
            lrc: (x + width, y + height)
        })
    }

    pub fn center(&self) -> (usize, usize) {
        ((self.ulc.0 + self.lrc.0) / 2, (self.ulc.1 + self.lrc.1) / 2)
    }

    pub fn intersect(&self, other: &Rectangle) -> bool {
        self.ulc.0 <= other.lrc.0 && self.lrc.0 >= other.ulc.0
            && self.ulc.1 <= other.lrc.1 && self.lrc.1 >= other.ulc.1
    }

    pub fn iter(&self) -> impl Iterator<Item=((usize, usize), RectangleIteratorType)> {
        RectangleIterator::new(self)
    }

    /// Give us a random valid x coordinate in/on this rectangle.
    /// If include_wall is true it will include the edges as a valid x
    /// value.  If not it will only be the body of the rectangle.
    pub fn random_x(&self, include_wall: bool) -> usize {
        let mut rng = thread_rng();

        let (start, end) = if include_wall {
            (0, self.lrc.0 - self.ulc.0)
        } else {
            (1, self.lrc.0 - self.ulc.0 - 1)
        };

        self.ulc.0 + rng.gen_range(start, end)
    }

    /// Give us a random valid x coordinate in/on this rectangle.
    /// If include_wall is true it will include the edges as a valid y
    /// value.  If not it will only be the body of the rectangle.
    pub fn random_y(&self, include_wall: bool) -> usize {
        let mut rng = thread_rng();

        let (start, end) = if include_wall {
            (0, self.lrc.1 - self.ulc.1)
        } else {
            (1, self.lrc.1 - self.ulc.1 - 1)
        };

        self.ulc.1 + rng.gen_range(start, end)
    }
}

#[derive(Debug, PartialEq)]
pub enum RectangleIteratorType {
    BORDER, BODY
}
struct RectangleIterator {
    x_index: usize,
    y_index: usize,
    x_beg: usize,
    y_beg: usize,
    x_end: usize,
    y_end: usize,
}

impl RectangleIterator {
    fn new(rect: &Rectangle) -> Self {
        Self {
            x_index: rect.ulc.0,
            y_index: rect.ulc.1,
            x_beg: rect.ulc.0,
            y_beg: rect.ulc.1,
            x_end: rect.lrc.0,
            y_end: rect.lrc.1,
        }
    }
}

impl Iterator for RectangleIterator {
    type Item = ((usize, usize), RectangleIteratorType);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y_index >= self.y_end && self.x_index > self.x_end {
            return None;
        }

        let x = if self.x_index > self.x_end {
            self.y_index += 1;
            self.x_index = self.x_beg + 1;
            self.x_beg
        } else {
            self.x_index += 1;
            self.x_index - 1
        };

        let point_type = if x == self.x_beg || x == self.x_end
            || self.y_index == self.y_beg || self.y_index == self.y_end {
            BORDER
        } else {
            BODY
        };


        Some(((x, self.y_index), point_type))
    }
}

#[cfg(test)]
mod tests {
    use crate::rectangle::Rectangle;
    use crate::rectangle::RectangleIteratorType::BORDER;

    #[test]
    fn test_new() {
        let rect = Rectangle::new(0, 0, 2, 2);
        assert!(rect.is_ok());

        let rect = Rectangle::new(0, 0, 1, 1);
        assert!(rect.is_err());
    }

    #[test]
    fn test_center() {
        let rect = Rectangle::new(0, 0, 4, 4).unwrap();

        assert_eq!(rect.center(), (2, 2));

        // We round down since this is integer result.
        let rect = Rectangle::new(0, 0, 5, 5).unwrap();

        assert_eq!(rect.center(), (2, 2));
    }

    #[test]
    fn test_intersect() {
        let rect1 = Rectangle::new(0, 0, 4, 4).unwrap();
        let rect2 = Rectangle::new(1, 1, 4, 4).unwrap();

        assert!(rect1.intersect(&rect2));
        assert!(rect2.intersect(&rect1));
        assert!(rect1.intersect(&rect1));

        let rect3 = Rectangle::new(1, 1, 2, 2).unwrap();
        assert!(rect1.intersect(&rect3));
    }

    #[test]
    fn test_rect_iterator() {
        let rect = Rectangle::new(1, 1, 2, 2).unwrap();
        let string: String = rect
            .iter()
            .map(|(point, point_type)|
                format!("({},{},{})",
                        point.0,
                        point.1,
                        if point_type == BORDER { '#' } else { '.' }))
            .collect();

        assert_eq!(string, "(1,1,#)(2,1,#)(3,1,#)(1,2,#)(2,2,.)(3,2,#)(1,3,#)(2,3,#)(3,3,#)");
    }

    #[test]
    fn test_rand_x() {
        //   012345
        // 0 .......
        // 1 .####..
        // 2 .#..#..
        // 3 .####..
        let rect = Rectangle::new(1, 1, 4, 3).unwrap();

        let mut x = rect.random_x(true);
        assert!(x >= 1 && x < 5);
        x = rect.random_x(false);
        assert!(x > 1 && x < 4);
    }

    #[test]
    fn test_rand_y() {
        //   012345
        // 0 .......
        // 1 .####..
        // 2 .#..#..
        // 3 .####..
        let rect = Rectangle::new(1, 1, 4, 3).unwrap();

        let mut y = rect.random_y(true);
        assert!(y >= 1 && y < 5);
        y = rect.random_y(false);
        assert!(y > 1 && y < 4);
    }
}