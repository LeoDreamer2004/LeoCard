mod state;
mod systems;
mod view;

pub use state::{AnchoredBubble, BubbleContent, BubbleStyle};
pub use systems::animate_bubbles;
pub use view::Bubble;

#[cfg(test)]
mod tests;
