
#[derive(PartialEq)]
pub struct Spot<T: PartialEq, I: Default + PartialEq> {
    pub solid: T,
    pub items: Option<Vec<(I, usize)>>,
}

impl <T: PartialEq, I: Default + PartialEq> Spot<T, I> {
    pub fn new(solid: T, items: Option<Vec<(I, usize)>>) -> Self {
        Self {
            solid,
            items,
        }
    }

    pub fn add_item(&mut self, item: (I, usize)) {
        if let Some(items) = &mut self.items {
            if let Some(index) = items.iter().position(|(r, _)| r == &item.0) {
                items[index].1 = items[index].1 + item.1;
            } else {
                items.push(item);
            }
        } else {
            self.items = Some(vec![item]);
        }
    }

    pub fn remove_item(&mut self, item: (I, usize)) -> usize {
        if let Some(items) = &mut self.items {
            if let Some(index) = items.iter().position(|(r, _)| r == &item.0) {
                let size = items[index].1;
                if size > item.1 {
                    items[index].1 = size - item.1;
                    return item.1
                } else {
                    let len = items.len();
                    if len == 1 {
                        self.items = None;
                    } else {
                        items.remove(index);
                    }
                    return size
                }
            }
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use crate::Spot;

    #[test]
    fn test_add_item() {
        let mut spot: Spot<i32, i32> = Spot::new(1, None);

        assert_eq!(spot.items, None);

        spot.add_item((2, 5));
        assert_eq!(spot.items, Some(vec![(2, 5)]));

        spot.add_item((2, 1));
        assert_eq!(spot.items, Some(vec![(2, 6)]));

        spot.add_item((3, 10));
        assert_eq!(spot.items, Some(vec![(2, 6), (3, 10)]));

        spot.add_item((3, 20));
        assert_eq!(spot.items, Some(vec![(2, 6), (3, 30)]));
    }

    #[test]
    fn test_remove_item() {
        let mut spot: Spot<i32, i32> = Spot::new(1, Some(vec![(1, 10), (2, 5)]));

        assert_eq!(spot.items, Some(vec![(1, 10), (2, 5)]));
        assert_eq!(spot.remove_item((4, 1)), 0);
        assert_eq!(spot.remove_item((2, 2)), 2);
        assert_eq!(spot.items, Some(vec![(1, 10), (2, 3)]));
        assert_eq!(spot.remove_item((2, 4)), 3);
        assert_eq!(spot.items, Some(vec![(1, 10)]));
        assert_eq!(spot.remove_item((1, 10)), 10);
        assert_eq!(spot.items, None);
    }
}