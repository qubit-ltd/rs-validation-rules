//! Collection and range validation rules.

mod item_count;
mod range;

pub use item_count::ItemCount;
pub use item_count::ItemCountError;
pub use range::Range;
pub use range::RangeError;
