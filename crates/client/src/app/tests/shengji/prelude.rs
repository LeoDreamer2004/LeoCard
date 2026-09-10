pub(super) use super::super::prelude::*;
pub(super) use leocard_protocol::{
    MatchId, PlayerGameProfiles, PlayerId, ProfileId, SeatId, ShengjiDeclarationView,
    ShengjiPhaseView, ShengjiPlayerState, ShengjiPublicPlay, ShengjiSnapshot, ShengjiTrickView,
};
pub(super) use leocard_shengji::{
    ShengjiBidKind, ShengjiBidTrump, ShengjiCard, ShengjiRank, ShengjiRuleSet, ShengjiSuit,
    ShengjiTrump, TrickPlay, classify_lead,
};

pub(super) fn shengji_ui_snapshot(
    hand: Vec<ShengjiCard>,
    declaration: Option<ShengjiDeclarationView>,
) -> ShengjiSnapshot {
    ShengjiSnapshot {
        match_id: MatchId([7; 16]),
        hand_number: 1,
        host_port: 52300,
        you: PlayerId(0),
        host: PlayerId(0),
        rules: ShengjiRuleSet::default(),
        players: (0..4)
            .map(|id| ShengjiPlayerState {
                id: PlayerId(id),
                profile_id: ProfileId([id; 32]),
                name: format!("玩家{id}"),
                avatar: None,
                seat: SeatId(id),
                hand_len: if id == 0 { hand.len() as u8 } else { 0 },
                ready: false,
                connected: true,
                auto_play: false,
                reference_points: 0,
                completed_games: 0,
                game_profiles: PlayerGameProfiles::default(),
            })
            .collect(),
        your_hand: hand,
        your_exposed_cards: declaration
            .as_ref()
            .filter(|declaration| declaration.player == PlayerId(0))
            .map_or_else(Vec::new, |declaration| declaration.cards.clone()),
        levels: [ShengjiRank::Ten, ShengjiRank::Ten],
        bidding_level: ShengjiRank::Ten,
        dealer: None,
        trump: None,
        declaration,
        current_player: None,
        trick: None,
        throw_failure: None,
        collecting_score: 0,
        buried_count: 0,
        your_buried: Vec::new(),
        phase: ShengjiPhaseView::Dealing {
            cards_remaining: 80,
        },
    }
}
