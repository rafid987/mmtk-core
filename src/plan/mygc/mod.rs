//! Plan: mygc (allocation-only)

pub(super) mod global;
pub(super) mod mutator;

pub use self::global::MyGC;
pub use self::global::MyGC_CONSTRAINTS;
