use std::fmt;
use std::fmt::{Display, Formatter};
use ndarray::{Array, Axis, Ix2};

pub struct Overlay<T: Sized + Clone> {
    data: Array<T, Ix2>,
    default: T
}

impl<T: Sized + Clone> Overlay<T> {
    pub fn new(width: usize, height: usize, default: T) -> Self {
        Overlay {
            data: Array::<T, Ix2>::from_elem((width, height), default.clone()),
            default,
        }
    }

    pub fn reset(&mut self) {
        for elem in self.data.iter_mut() {
            *elem = self.default.clone();
        }
    }

    #[inline]
    pub fn get(&self, loc: (usize, usize)) -> Option<&T> {
        self.data.get(loc)
    }

    #[inline]
    pub fn set(&mut self, loc: (usize, usize), value: T) -> bool {
        let spot = self.data.get_mut(loc);
        let found = spot.is_some();

        if found {
            *spot.unwrap() = value;
        }

        found
    }
}

impl<T: Sized + Clone + PartialEq> Display for Overlay<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for line in self.data.axis_iter(Axis(1)) {
            let line: String = line.iter().map(|t| if t == &self.default { '#' } else { '.' }).collect();
            let _ = writeln!(f, "{}", line);
        }
        Ok(())
    }
}