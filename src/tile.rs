
#[derive(Clone, Copy, Debug)]
pub struct Tile {
    pub id: char,
    pub(crate) weight: usize,
}

impl Tile {
    pub fn new(id: char, weight: usize) -> Tile {
        Tile {
            id,
            weight,
        }
    }
}