use crate::types::deck::DeckState;

pub type DeckUpdate = Box<dyn FnOnce(&mut DeckState) + Send>;
