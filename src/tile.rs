
#[derive(Clone, Copy, Debug)]
pub struct Tile<T> {
    pub id: T,
    pub(crate) weight: usize,
}

impl<T> Tile<T> {
    pub fn new(id: T, weight: usize) -> Self {
        Tile {
            id,
            weight,
        }
    }
}