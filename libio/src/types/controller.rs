use libdj::types::deck::DeckState;
use rkyv::{Archive, Deserialize, Serialize};

#[derive(Debug, Clone, Archive, Deserialize, Serialize)]
pub enum ControllerMessage {
    UpdateDeckState(DeckState),
    UpdateCurrentSamples([f32; 4]),
}
