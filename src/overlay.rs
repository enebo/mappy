use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::BitOrAssign;
use ndarray::{Array, Axis, Ix2};

pub struct Overlay<T: Sized + Clone + BitOrAssign> {
    data: Array<T, Ix2>,
    default: T
}

impl<T: Sized + Clone + std::ops::BitOrAssign> Overlay<T> {
    pub fn new(width: usize, height: usize, default: T) -> Self {
        Overlay {
            data: Array::<T, Ix2>::from_elem((width, height), default.clone()),
            default,
        }
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item=((usize, usize), &'a T)> + 'a {
        OverlayIterator::new(self)
    }

    pub fn or(&mut self, other: &Self) {
        for (n, o) in self.data.iter_mut().zip(other.data.iter()) {
            *n |= o.clone();
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

struct OverlayIterator<'a, T>  where T: Sized + Clone + BitOrAssign {
    overlay: &'a Overlay<T>,
    index: usize,
    width: usize,
}

impl<'a, T> OverlayIterator<'a, T> where T: Sized + Clone + BitOrAssign {
    fn new(overlay: &'a Overlay<T>) -> Self {
        let width = overlay.data.len_of(Axis(0));

        Self {
            width,
            overlay,
            index: 0,
        }
    }
}

impl<'a, T> Iterator for OverlayIterator<'a, T>  where T: Sized + Clone + BitOrAssign {
    type Item = ((usize, usize), &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.overlay.data.len() {
            return None;
        }

        let loc: (usize, usize) = (self.index % self.width, self.index / self.width);
        let element = self.overlay.get(loc).unwrap();
        self.index += 1;

        Some((loc, element))
    }
}

impl<T: Sized + Clone + BitOrAssign + PartialEq> Display for Overlay<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for line in self.data.axis_iter(Axis(1)) {
            let line: String = line.iter().map(|t| if t == &self.default { '#' } else { '.' }).collect();
            let _ = writeln!(f, "{}", line);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Overlay;

    #[test]
    fn test_or() {
        let mut o1 = Overlay::new(3, 4, false);
        let mut o2 = Overlay::new(3, 4, false);

        o1.set((0, 0), true);
        o2.set((1, 1), true);

        o1.or(&o2);

        assert!(o1.get((0, 0)).unwrap());
        assert!(o1.get((1, 1)).unwrap());
        assert!(!o1.get((2, 1)).unwrap());
    }

    #[test]
    fn test_iter() {
        let mut o1 = Overlay::new(3, 4, false);
        o1.set((0, 0), true);
        o1.set((2, 0), true);
        o1.set((0, 1), true);

        let mut iter = o1.iter();

        assert_eq!(iter.next(), Some(((0, 0), (&true))));
        assert_eq!(iter.next(), Some(((1, 0), (&false))));
        assert_eq!(iter.next(), Some(((2, 0), (&true))));
        assert_eq!(iter.next(), Some(((0, 1), (&true))));
        assert_eq!(iter.next(), Some(((1, 1), (&false))));
    }
}