// pub mod arena;
// pub mod arena_inlining;
// pub mod nesting;
pub mod nesting_inlining;

// todo: move to final pipeline's Crossfader node file

use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Archive, Deserialize, Serialize, PartialEq)]
pub enum CrossFaderSide {
    A,
    B,
    None,
}
