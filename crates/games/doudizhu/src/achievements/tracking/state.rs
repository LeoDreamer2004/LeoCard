use crate::{GameState, Phase, Play, PlayKind};

#[derive(Clone, Debug, Default)]
pub(super) struct HandProgress {
    pub play_count: u32,
    pub consecutive_bombs: u32,
    pub trick_bombs: u32,
    pub unavailable_exchange: bool,
    pub all_bomb_hand: bool,
}

#[derive(Clone, Debug)]
pub(super) struct PlayerProgress {
    pub hand: HandProgress,
    pub perfect_spring: bool,
}

#[derive(Clone, Debug, Default)]
pub struct DoudizhuAchievementTracker {
    pub(super) players: Vec<PlayerProgress>,
    pub(super) flower_senders: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct DoudizhuActionContext {
    pub(super) phase: Phase,
    pub(super) previous: Option<(u8, Play)>,
}

impl DoudizhuActionContext {
    pub fn capture(game: &GameState) -> Self {
        Self {
            phase: game.phase,
            previous: game.table.clone(),
        }
    }
}

impl DoudizhuAchievementTracker {
    pub fn new(player_count: u8) -> Self {
        Self {
            players: vec![
                PlayerProgress {
                    hand: HandProgress::default(),
                    perfect_spring: true
                };
                usize::from(player_count)
            ],
            flower_senders: Vec::new(),
        }
    }

    pub fn begin_hand(&mut self) {
        for player in &mut self.players {
            player.hand = HandProgress::default();
        }
        self.flower_senders.clear();
    }

    pub fn observe_flower(&mut self, source: u8, target: u8, game: &GameState) {
        if game.phase == Phase::Playing
            && game.current == target
            && source != target
            && game
                .landlord
                .is_some_and(|landlord| source != landlord && target != landlord)
            && usize::from(source) < self.players.len()
            && !self.flower_senders.contains(&source)
        {
            self.flower_senders.push(source);
        }
    }
}

pub(super) fn is_airplane(kind: PlayKind) -> bool {
    matches!(
        kind,
        PlayKind::Airplane | PlayKind::AirplaneSingles | PlayKind::AirplanePairs
    )
}

pub(super) fn is_bomb(kind: PlayKind) -> bool {
    matches!(kind, PlayKind::Bomb | PlayKind::Rocket)
}
