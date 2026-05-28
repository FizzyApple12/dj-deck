use tokio::sync::mpsc::UnboundedSender;

use crate::types::deck::DeckState;

pub enum AudioSystemEvent {}

pub type DeckUpdate =
    Box<dyn FnOnce(&mut DeckState, &mut UnboundedSender<AudioSystemEvent>) + Send>;
