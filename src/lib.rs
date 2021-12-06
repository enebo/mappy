pub mod rectangle;
pub mod builders;
mod field_of_view;
mod overlay;
mod map;

pub use map::Map;
pub use overlay::Overlay;
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


// FIXME: dynamic maps vs pre-compiled sizes and using a primitive array.
// This also makes me wonder how much you could compile a map so more actions with a map are precomputed.