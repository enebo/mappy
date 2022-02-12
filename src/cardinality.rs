
#[derive(PartialEq)]
pub enum Cardinality<I> {
    Zero,
    One(I),
    Many(Vec<I>),
}
