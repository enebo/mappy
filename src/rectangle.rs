use crate::Point;
use rand::{thread_rng, Rng};
use crate::rectangle::RectangleIteratorType::{BODY, BORDER};

/// Rectangle with a single width border.
pub struct Rectangle {
    pub ulc: Point,
    pub lrc: Point,
}

impl Rectangle {
    pub fn new(x: usize, y: usize, width: usize, height: usize) -> Result<Self, String> {
        // We need a minimum of a wall on each side of the room but the room may have no
        // open spaces.
        if width < 2 || height < 2 {
            return Err("Rectangle dimensions too small".to_string())
        }

        Ok(Rectangle {
            ulc: Point::new(x, y),
            lrc: Point::new(x + width, y + height)
        })
    }

    pub fn center(&self) -> Point {
        Point::new((self.ulc.x + self.lrc.x) / 2, (self.ulc.y + self.lrc.y) / 2)
    }

    pub fn intersect(&self, other: &Rectangle) -> bool {
        self.ulc.x <= other.lrc.x && self.lrc.x >= other.ulc.x
            && self.ulc.y <= other.lrc.y && self.lrc.y >= other.ulc.y
    }

    pub fn iter(&self) -> impl Iterator<Item=(Point, RectangleIteratorType)> {
        RectangleIterator::new(self)
    }

    pub fn random_x(&self) -> usize {
        let mut rng = thread_rng();

        self.ulc.x + rng.gen_range(0, self.lrc.x - self.ulc.x)
    }

    pub fn random_y(&self) -> usize {
        let mut rng = thread_rng();

        self.ulc.y + rng.gen_range(0, self.lrc.y - self.ulc.y)
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
            x_index: rect.ulc.x,
            y_index: rect.ulc.y,
            x_beg: rect.ulc.x,
            y_beg: rect.ulc.y,
            x_end: rect.lrc.x,
            y_end: rect.lrc.y,
        }
    }
}

impl Iterator for RectangleIterator {
    type Item = (Point, RectangleIteratorType);

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


        Some((Point::new(x, self.y_index), point_type))
    }
}

#[cfg(test)]
mod tests {
    use crate::Point;
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

        assert_eq!(rect.center(), Point::new(2, 2));

        // We round down since this is integer result.
        let rect = Rectangle::new(0, 0, 5, 5).unwrap();

        assert_eq!(rect.center(), Point::new(2, 2));
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
                        point.x,
                        point.y,
                        if point_type == BORDER { '#' } else { '.' }))
            .collect();

        assert_eq!(string, "(1,1,#)(2,1,#)(3,1,#)(1,2,#)(2,2,.)(3,2,#)(1,3,#)(2,3,#)(3,3,#)");
    }

    #[test]
    fn test_rand_x() {
        let rect = Rectangle::new(1, 1, 3, 3).unwrap();

        let x = rect.random_x();

        assert!(x >= 1 && x < 4);
    }

    #[test]
    fn test_rand_y() {
        let rect = Rectangle::new(1, 1, 3, 3).unwrap();

        let y = rect.random_y();

        assert!(y >= 1 && y < 4);
    }
}