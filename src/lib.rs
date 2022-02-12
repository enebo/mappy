pub mod rectangle;
pub mod builders;
mod field_of_view;
mod overlay;
pub mod map;
pub mod spot;
pub mod cardinality;

pub use cardinality::Cardinality;
pub use map::Map;
pub use overlay::Overlay;
pub use rectangle::{Rectangle, RectangleIteratorType};
pub use spot::Spot;
pub use field_of_view::calculate_field_of_view;

#[derive(Debug)]
pub struct MyError {}

#[inline]
pub const fn math_is_hard(x: usize, d: isize) -> Option<usize> {
    let result = x as isize + d;

    if result.is_negative() {
        None
    } else {
        Some(result as usize)
    }
}

#[inline]
pub const fn add_delta(loc: &(usize, usize), delta: &(isize, isize)) -> Option<(usize, usize)> {
    if let Some(nx) = math_is_hard(loc.0, delta.0) {
        if let Some(ny) = math_is_hard(loc.1, delta.1) {
            return Some((nx, ny))
        }
    }

    None
}


// FIXME: dynamic maps vs pre-compiled sizes and using a primitive array.
// This also makes me wonder how much you could compile a map so more actions with a map are precomputed.