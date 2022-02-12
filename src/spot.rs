use std::mem::take;
use crate::cardinality::Cardinality;
use crate::Cardinality::{Many, One, Zero};

#[derive(PartialEq)]
pub struct Spot<T: PartialEq, I: Default + PartialEq> {
    pub solid: T,
    pub items: Cardinality<I>,
}

impl <T: PartialEq, I: Default + PartialEq> Spot<T, I> {
    pub fn new(solid: T, items: Cardinality<I>) -> Self {
        Self {
            solid,
            items
        }
    }

    pub fn add_item(&mut self, item: I) {
        match &mut self.items {
            Zero => self.items = One(item),
            One(i) => self.items = Many(vec![take(i), item]),
            Many(items) => items.push(item),
        }
    }

    pub fn remove_item(&mut self, item: I) -> bool {
        match &mut self.items {
            Zero => false,
            One(i) if i == &item => {
                self.items = Zero;
                true
            },
            One(_) => false,
            Many(items) if items.len() == 2 => {
                let index = items.iter().position(|r| r != &item).unwrap();
                self.items = One(take(&mut items[index]));
                true
            },
            Many(items) => {
                if let Some(index) = items.iter().position(|r| r == &item) {
                    items.remove(index);
                    true
                } else {
                    false
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Cardinality::{Many, One, Zero};
    use crate::Spot;

    #[test]
    fn test_add_item() {
        let mut spot: Spot<i32, i32> = Spot::new(1, Zero);

        assert_eq!(spot.items, Zero);

        spot.add_item(2);

        assert_eq!(spot.items, One(2));

        spot.add_item(3);

        assert_eq!(spot.items, Many(vec![2, 3]));
    }

    #[test]
    fn test_remove_item() {
        let mut spot: Spot<i32, i32> = Spot::new(1, Many(vec![1, 2, 3]));

        assert_eq!(spot.items, Many(vec![1, 2, 3]));
        assert!(!spot.remove_item(4));
        assert!(spot.remove_item(2));
        assert_eq!(spot.items, Many(vec![1, 3]));
        assert!(spot.remove_item(1));
        assert_eq!(spot.items, One(3));
        assert!(spot.remove_item(3));
        assert_eq!(spot.items, Zero);
    }
}