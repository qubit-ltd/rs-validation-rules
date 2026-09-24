//! Dynamic registration adapters for typed rules.

mod collection;
#[cfg(feature = "regex")]
mod regex;
mod registrations;
mod text;

pub use registrations::registrations;
