use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::prelude::*;
use leocard_client::{ItemId, PlayerEconomy};
use leocard_mahjong::{
    MahjongPlayerId, MahjongScoreResult, MahjongTileKind, Meld, ScoreInput, WinContext, WinSource,
    score_hand,
};
use leocard_protocol::MahjongSnapshot;

#[derive(Resource, Default)]
pub(crate) struct MahjongFanCalculator {
    pub active: bool,
}

pub(in super::super) fn sync_fan_calculator(
    economy: Res<PlayerEconomy>,
    mut calculator: ResMut<MahjongFanCalculator>,
    client: Option<Res<ClientResource>>,
    mut ui: ResMut<UiState>,
) {
    let active = economy.active(ItemId::MahjongFanCalculator);
    if calculator.active != active {
        calculator.active = active;
        if client.is_some_and(|client| client.0.model().mahjong_game().is_some()) {
            ui.dirty = true;
        }
    }
}

pub(super) struct WaitScores {
    pub self_draw: MahjongScoreResult,
    pub discard: MahjongScoreResult,
}

pub(super) struct WaitScoreCalculator {
    context: WinContext,
    exposed: [u8; 34],
}

impl WaitScoreCalculator {
    pub fn new(game: &MahjongSnapshot) -> Self {
        let own = game
            .players
            .iter()
            .find(|player| player.id == game.you)
            .expect("麻将快照必须包含自己");
        let mut exposed = [0; 34];
        for discard in game
            .discards
            .iter()
            .filter(|discard| discard.claimed_by.is_none())
        {
            if let Some(index) = discard.tile.kind().index34() {
                exposed[index] += 1;
            }
        }
        for meld in game.players.iter().flat_map(|player| &player.melds) {
            if let Some(core) = super::waits::core_meld(meld).filter(|meld| meld.is_open()) {
                for kind in core.tile_kinds() {
                    exposed[kind.index34().expect("副露不包含花牌")] += 1;
                }
            }
        }
        Self {
            context: WinContext {
                source: WinSource::SelfDraw,
                seat_wind: own.seat_wind,
                prevalent_wind: game.prevalent_wind,
                // 假设取得下一张牌，不预判未来补杠、抢杠或补花。
                last_wall_tile: game.wall_len <= 1,
                last_of_kind: false,
                flower_count: own.flowers.len() as u8,
            },
            exposed,
        }
    }

    pub fn calculate(
        &self,
        concealed: &[MahjongTileKind],
        melds: &[Meld],
        winning_tile: MahjongTileKind,
        discarded: Option<MahjongTileKind>,
    ) -> WaitScores {
        let index = winning_tile.index34().expect("听牌候选不包含花牌");
        let mut context = self.context;
        // 悬停提示模拟打出该牌后听牌；自己的弃牌同样属于公开牌。
        context.last_of_kind = self.exposed[index] + u8::from(discarded == Some(winning_tile)) >= 3;
        let mut input = ScoreInput {
            concealed: concealed.iter().copied().chain([winning_tile]).collect(),
            melds: melds.to_vec(),
            winning_tile,
            context,
        };
        let self_draw = score_hand(&input).expect("结构听牌必须能够算番");
        input.context.source = WinSource::Discard(MahjongPlayerId(0));
        let discard = score_hand(&input).expect("结构听牌必须能够算番");
        WaitScores { self_draw, discard }
    }
}
