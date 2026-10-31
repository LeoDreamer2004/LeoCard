use crate::app::presentation::READY;
use bevy::prelude::*;
use leocard_protocol::{TexasPhaseView, TexasPlayerView};
use leocard_texas_holdem::{TexasHandCategory, TexasStreet};

pub(super) fn texas_player_border_color(player: &TexasPlayerView, current: bool) -> Color {
    if player.folded {
        Color::srgb(0.48, 0.52, 0.50)
    } else if current {
        Color::NONE
    } else {
        READY
    }
}

pub(super) fn street_label(phase: &TexasPhaseView) -> &'static str {
    match phase {
        TexasPhaseView::Betting {
            street: TexasStreet::PreFlop,
        } => "翻牌前",
        TexasPhaseView::Betting {
            street: TexasStreet::Flop,
        } => "翻牌圈",
        TexasPhaseView::Betting {
            street: TexasStreet::Turn,
        } => "转牌圈",
        TexasPhaseView::Betting {
            street: TexasStreet::River,
        } => "河牌圈",
        TexasPhaseView::HandComplete { .. } => "结算",
    }
}

pub(super) fn texas_category_label(category: TexasHandCategory) -> &'static str {
    match category {
        TexasHandCategory::HighCard => "高牌",
        TexasHandCategory::OnePair => "一对",
        TexasHandCategory::TwoPair => "两对",
        TexasHandCategory::ThreeOfAKind => "三条",
        TexasHandCategory::Straight => "顺子",
        TexasHandCategory::Flush => "同花",
        TexasHandCategory::FullHouse => "葫芦",
        TexasHandCategory::FourOfAKind => "四条",
        TexasHandCategory::StraightFlush => "同花顺",
        TexasHandCategory::RoyalFlush => "皇家同花顺",
    }
}
