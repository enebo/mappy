use nalgebra::{Point2};

pub type Point = Point2<usize>;

pub mod rectangle;
pub mod builders;
mod field_of_view;
mod overlay;
mod map;
mod tile;

pub use map::Map;
pub use overlay::Overlay;
pub use tile::Tile;

#[derive(Debug)]
pub struct MyError {}


// FIXME: dynamic maps vs pre-compiled sizes and using a primitive array.
// This also makes me wonder how much you could compile a map so more actions with a map are precomputed.