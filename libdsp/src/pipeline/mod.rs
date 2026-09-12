// pub mod arena;
// pub mod arena_inlining;
// pub mod nesting;
pub mod nesting_inlining;

// todo: move to final pipeline's Crossfader node file

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossFaderSide {
    A,
    B,
    None,
}
