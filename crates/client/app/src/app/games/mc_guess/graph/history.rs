use crate::app::presentation::ease_out_cubic;
use bevy::prelude::Vec2;
use leocard_mc_guess::{McGuessGraphNodeView, McGuessRoundView};

pub(super) struct HistoryContent {
    pub revision: u64,
    pub node: McGuessGraphNodeView,
    pub round: McGuessRoundView,
    pub scroll_y: f32,
}

#[derive(Clone, Copy, PartialEq)]
pub(super) struct PanelPose {
    pub opacity: f32,
    pub offset: Vec2,
}

struct Transition {
    elapsed: f32,
    source: PanelPose,
    ready: bool,
}

/// The old table leaves before the next question enters.
#[derive(Default)]
pub(in super::super) struct GraphHistory {
    pub(super) content: Option<HistoryContent>,
    pub(super) previous: Option<HistoryContent>,
    transition: Option<Transition>,
    revision: u64,
}

impl GraphHistory {
    const EXIT_DURATION: f32 = 0.4;
    const ENTRY_DURATION: f32 = 0.4;

    pub(super) fn pose(&self, outgoing: bool) -> PanelPose {
        let Some(transition) = &self.transition else {
            return PanelPose {
                opacity: if outgoing { 0.0 } else { 1.0 },
                offset: if outgoing { Vec2::X } else { Vec2::ZERO },
            };
        };
        if outgoing {
            let t = (transition.elapsed / Self::EXIT_DURATION).clamp(0.0, 1.0);
            let progress = t * t * t;
            PanelPose {
                opacity: transition.source.opacity * (1.0 - progress),
                offset: transition.source.offset.lerp(Vec2::X, progress),
            }
        } else {
            let t =
                ((transition.elapsed - Self::EXIT_DURATION) / Self::ENTRY_DURATION).clamp(0.0, 1.0);
            let progress = ease_out_cubic(t);
            PanelPose {
                opacity: progress,
                offset: -Vec2::Y * (1.0 - progress),
            }
        }
    }

    /// Returns whether table contents or the outgoing subtree need recomposition.
    pub fn sync(
        &mut self,
        node: &McGuessGraphNodeView,
        round: &McGuessRoundView,
        scroll_y: f32,
    ) -> bool {
        let Some(content) = &self.content else {
            self.replace(node, round, scroll_y);
            return true;
        };
        let mut changed = false;
        if content.node.number != node.number {
            let source = self.pose(false);
            self.previous = self.content.take();
            self.replace(node, round, scroll_y);
            self.transition = Some(Transition {
                elapsed: 0.0,
                source,
                ready: false,
            });
            changed = true;
        }
        if let Some(content) = &mut self.content {
            if content.node != *node || content.round != *round {
                content.node.clone_from(node);
                content.round.clone_from(round);
                self.revision += 1;
                content.revision = self.revision;
                changed = true;
            }
            content.scroll_y = scroll_y;
        }
        changed
    }

    /// Start the clock only once the new table has completed a layout frame.
    pub fn advance(&mut self, delta: f32, ready: bool) -> bool {
        let Some(transition) = &mut self.transition else {
            return false;
        };
        if !transition.ready {
            transition.ready = ready;
            return false;
        }
        // A slow composition frame must not consume the entire visible animation.
        transition.elapsed += delta.min(1.0 / 30.0);
        if transition.elapsed < Self::EXIT_DURATION + Self::ENTRY_DURATION {
            return false;
        }
        self.previous = None;
        self.transition = None;
        true
    }

    fn replace(&mut self, node: &McGuessGraphNodeView, round: &McGuessRoundView, scroll_y: f32) {
        self.revision += 1;
        self.content = Some(HistoryContent {
            revision: self.revision,
            node: node.clone(),
            round: round.clone(),
            scroll_y,
        });
    }
}
