use crate::cardinality::Cardinality;

#[derive(PartialEq)]
pub struct Spot<T: PartialEq, I: PartialEq> {
    pub solid: T,
    pub items: Cardinality<I>,
}

impl <T: PartialEq, I: PartialEq> Spot<T, I> {
    pub fn new(solid: T, items: Cardinality<I>) -> Self {
        Self {
            solid,
            items
        }
    }
}