use super::*;

pub(super) fn texas_player_status(player: &TexasHoldemPlayerState) -> String {
    if !player.connected {
        "已离线".to_owned()
    } else if player.folded {
        "已弃牌".to_owned()
    } else if player.all_in {
        format!("全下 · 本轮 {}", player.committed_street)
    } else if player.committed_street > 0 {
        format!("本轮投入 {}", player.committed_street)
    } else {
        "等待行动".to_owned()
    }
}

pub(super) fn texas_player_border_color(player: &TexasHoldemPlayerState, current: bool) -> Color {
    if player.folded {
        Color::srgb(0.48, 0.52, 0.50)
    } else if current {
        Color::NONE
    } else {
        READY
    }
}

pub(super) fn street_label(phase: &TexasHoldemPhaseView) -> &'static str {
    match phase {
        TexasHoldemPhaseView::Betting {
            street: TexasHoldemStreet::PreFlop,
        } => "翻牌前",
        TexasHoldemPhaseView::Betting {
            street: TexasHoldemStreet::Flop,
        } => "翻牌圈",
        TexasHoldemPhaseView::Betting {
            street: TexasHoldemStreet::Turn,
        } => "转牌圈",
        TexasHoldemPhaseView::Betting {
            street: TexasHoldemStreet::River,
        } => "河牌圈",
        TexasHoldemPhaseView::HandComplete { .. } => "本手结算",
    }
}

pub(super) fn texas_category_label(category: TexasHoldemHandCategory) -> &'static str {
    match category {
        TexasHoldemHandCategory::HighCard => "高牌",
        TexasHoldemHandCategory::OnePair => "一对",
        TexasHoldemHandCategory::TwoPair => "两对",
        TexasHoldemHandCategory::ThreeOfAKind => "三条",
        TexasHoldemHandCategory::Straight => "顺子",
        TexasHoldemHandCategory::Flush => "同花",
        TexasHoldemHandCategory::FullHouse => "葫芦",
        TexasHoldemHandCategory::FourOfAKind => "四条",
        TexasHoldemHandCategory::StraightFlush => "同花顺",
        TexasHoldemHandCategory::RoyalFlush => "皇家同花顺",
    }
}
