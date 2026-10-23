use super::scores::{WaitScoreCalculator, WaitScores};
use leocard_mahjong::{
    MahjongKongKind, MahjongMeldKind, MahjongPlayerId, MahjongTileKind, Meld, waiting_tile_kinds,
};
use leocard_protocol::{MahjongPhaseView, MahjongPublicMeldView, MahjongSnapshot};
use std::collections::HashMap;

pub(in super::super) fn mahjong_discard_waits(
    game: &MahjongSnapshot,
    show_fans: bool,
) -> HashMap<MahjongTileKind, Vec<MahjongWait>> {
    let Some(calculator) = WaitCalculator::new(game, show_fans) else {
        return HashMap::new();
    };
    let mut waits = HashMap::new();
    for (discard_index, discard) in game.your_hand.iter().enumerate() {
        let discard_kind = discard.kind();
        waits.entry(discard_kind).or_insert_with(|| {
            let concealed = game
                .your_hand
                .iter()
                .enumerate()
                .filter_map(|(index, tile)| (index != discard_index).then_some(tile.kind()))
                .collect::<Vec<_>>();
            calculator.for_hand(&concealed, Some(discard_kind))
        });
    }
    waits
}

pub(super) fn mahjong_current_waits(game: &MahjongSnapshot, show_fans: bool) -> Vec<MahjongWait> {
    if game.your_drawn_tile.is_some()
        || !matches!(
            game.phase,
            MahjongPhaseView::Playing | MahjongPhaseView::WaitingForClaims
        )
        || game.your_hand.len() % 3 != 1
    {
        return Vec::new();
    }
    let Some(calculator) = WaitCalculator::new(game, show_fans) else {
        return Vec::new();
    };
    let concealed = game
        .your_hand
        .iter()
        .map(|tile| tile.kind())
        .collect::<Vec<_>>();
    calculator.for_hand(&concealed, None)
}

pub(in super::super) struct MahjongWait {
    pub kind: MahjongTileKind,
    pub remaining: u8,
    pub(super) scores: Option<WaitScores>,
}

struct WaitCalculator {
    melds: Vec<Meld>,
    visible: [u8; 34],
    scores: Option<WaitScoreCalculator>,
}

impl WaitCalculator {
    fn new(game: &MahjongSnapshot, show_fans: bool) -> Option<Self> {
        let own = game.players.iter().find(|player| player.id == game.you)?;
        let melds = own
            .melds
            .iter()
            .map(core_meld)
            .collect::<Option<Vec<_>>>()?;
        Some(Self {
            melds,
            visible: visible_kind_counts(game),
            scores: show_fans.then(|| WaitScoreCalculator::new(game)),
        })
    }

    fn for_hand(
        &self,
        concealed: &[MahjongTileKind],
        discarded: Option<MahjongTileKind>,
    ) -> Vec<MahjongWait> {
        waiting_tile_kinds(concealed, &self.melds)
            .into_iter()
            .map(|kind| {
                let index = kind.index34().expect("听牌候选不含花牌");
                MahjongWait {
                    kind,
                    remaining: 4_u8.saturating_sub(self.visible[index]),
                    scores: self
                        .scores
                        .as_ref()
                        .map(|scores| scores.calculate(concealed, &self.melds, kind, discarded)),
                }
            })
            .collect()
    }
}

pub(super) fn core_meld(meld: &MahjongPublicMeldView) -> Option<Meld> {
    let tile = meld.tile?;
    let source = MahjongPlayerId(meld.claimed_from.map_or(0, |player| player.0 as usize));
    Some(match meld.kind {
        MahjongMeldKind::Chow => {
            let MahjongTileKind::Suited { suit, rank } = tile else {
                return None;
            };
            Meld::chow(suit, rank, source)
        }
        MahjongMeldKind::Pung => Meld::pung(tile, source),
        MahjongMeldKind::Kong(MahjongKongKind::Melded) => Meld::melded_kong(tile, source),
        MahjongMeldKind::Kong(MahjongKongKind::Concealed) => Meld::concealed_kong(tile),
    })
}

fn visible_kind_counts(game: &MahjongSnapshot) -> [u8; 34] {
    let mut counts = [0_u8; 34];
    let mut add = |kind: MahjongTileKind| {
        if let Some(index) = kind.index34() {
            counts[index] = counts[index].saturating_add(1);
        }
    };
    for tile in &game.your_hand {
        add(tile.kind());
    }
    for discard in game
        .discards
        .iter()
        .filter(|discard| discard.claimed_by.is_none())
    {
        add(discard.tile.kind());
    }
    for player in &game.players {
        if player.id != game.you
            && let Some(revealed) = &player.revealed_hand
        {
            for tile in revealed {
                add(tile.kind());
            }
        }
        for meld in &player.melds {
            if player.id != game.you
                && matches!(meld.kind, MahjongMeldKind::Kong(MahjongKongKind::Concealed))
            {
                continue;
            }
            if let Some(core) = core_meld(meld) {
                for kind in core.tile_kinds() {
                    add(kind);
                }
            }
        }
    }
    counts
}
