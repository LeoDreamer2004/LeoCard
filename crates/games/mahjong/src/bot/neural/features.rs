use super::super::{MahjongBotAction, MahjongBotObservation};
use crate::{
    MahjongClaim, MahjongDiscard, MahjongMeldKind, MahjongPublicPlayerState, MahjongTile,
    MahjongWind,
};
use leocard_game_common::PlayerId;

#[cfg(test)]
pub(super) fn encode(input: &MahjongBotObservation) -> Vec<f32> {
    encode_view(
        input.player,
        &input.hand,
        &input.discards,
        &input.public,
        [input.context.seat_wind, input.context.prevalent_wind],
    )
}

pub(super) fn encode_view(
    observer: PlayerId,
    tiles: &[MahjongTile],
    discards: &[MahjongDiscard],
    public: &[MahjongPublicPlayerState],
    winds: [MahjongWind; 2],
) -> Vec<f32> {
    let mut features = vec![0.0; 38 * 36];
    features[27 + winds[0].index()] = 1.0;
    features[36 + 27 + winds[1].index()] = 1.0;
    let mut hand = [0u8; 34];
    for index in tiles.iter().filter_map(|tile| tile.kind().index34()) {
        hand[index] += 1;
    }
    thermometer(&mut features, 2, &hand);
    for player in public {
        let relative = relative(player.id, observer);
        let mut discarded = [0u8; 34];
        for index in discards
            .iter()
            .filter(|discard| discard.player == player.id)
            .filter_map(|discard| discard.tile.kind().index34())
        {
            discarded[index] += 1;
        }
        thermometer(&mut features, 6 + 4 * relative, &discarded);
        let mut packs = [0u8; 34];
        for meld in &player.melds {
            let Some(index) = meld.tile.and_then(|tile| tile.index34()) else {
                continue;
            };
            match meld.kind {
                MahjongMeldKind::Chow => {
                    for count in &mut packs[index..index + 3] {
                        *count += 1;
                    }
                }
                MahjongMeldKind::Pung => packs[index] += 3,
                MahjongMeldKind::Kong(_) => packs[index] += 4,
            }
        }
        thermometer(&mut features, 22 + 4 * relative, &packs);
    }
    features
}

fn thermometer(features: &mut [f32], start: usize, counts: &[u8; 34]) {
    for (tile, count) in counts.iter().enumerate() {
        for layer in 0..usize::from((*count).min(4)) {
            features[(start + layer) * 36 + tile] = 1.0;
        }
    }
}

fn relative(player: PlayerId, observer: PlayerId) -> usize {
    (usize::from(player.0) + 4 - usize::from(observer.0)) % 4
}

pub(super) fn action_id(action: MahjongBotAction, input: &MahjongBotObservation) -> Option<usize> {
    match action {
        MahjongBotAction::Discard(tile) | MahjongBotAction::DeclareReady(tile) => {
            tile.kind().index34().map(|i| 2 + i)
        }
        MahjongBotAction::SelfDraw | MahjongBotAction::Respond(MahjongClaim::Win) => Some(1),
        MahjongBotAction::Respond(MahjongClaim::Pass) => Some(0),
        MahjongBotAction::Respond(MahjongClaim::Pung) => input.claim?.1.index34().map(|i| 99 + i),
        MahjongBotAction::Respond(MahjongClaim::Kong) => input.claim?.1.index34().map(|i| 133 + i),
        MahjongBotAction::ConcealedKong(tile) => tile.index34().map(|i| 167 + i),
        MahjongBotAction::AddedKong(tile) => tile.kind().index34().map(|i| 201 + i),
        MahjongBotAction::Respond(MahjongClaim::Chow { start }) => {
            let tile = input.claim?.1.index34()?;
            let rank = tile % 9 + 1;
            let position = rank.checked_sub(usize::from(start))?;
            (tile < 27 && (1..=7).contains(&start) && position < 3)
                .then_some(36 + tile / 9 * 21 + usize::from(start - 1) * 3 + position)
        }
    }
}
