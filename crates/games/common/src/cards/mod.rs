mod deck;
mod selection;
#[cfg(test)]
mod tests;

pub use deck::{DeckError, validate_deck};
pub use selection::{contains_unique_cards, has_unique_cards};
