use super::{
    BIDDING_GRACE, BOTTOM_COPY_DECISION_TIMEOUT, ShengjiSession, pre_kitty_collecting_score,
};
use leocard_game_common::PlayerId;
use leocard_protocol::{
    ShengjiDeclarationView, ShengjiPhaseView, ShengjiPlayerView, ShengjiPublicPlay,
    ShengjiSnapshot, ShengjiThrowFailureView, ShengjiTrickView,
};
use leocard_shengji::{
    ShengjiBottomCopyState, ShengjiCard, ShengjiFiveTrumpCrossingStage, ShengjiGameState,
    ShengjiPhase,
};

impl ShengjiSession {
    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> ShengjiSnapshot {
        let game = self.game.as_ref().expect("game snapshot requires a game");

        let your_hand = game
            .players()
            .get(usize::from(recipient.0))
            .map(|player| player.hand.clone())
            .unwrap_or_default();
        let visible_trick = game.current_trick().or_else(|| {
            self.presentation
                .trick
                .as_ref()
                .map(|(trick, _)| trick.clone())
        });
        let trick = visible_trick.map(|trick| ShengjiTrickView {
            leader: trick.leader,
            current_player: game.current_player().unwrap_or(trick.leader),
            winning_player: trick.winner,
            plays: trick
                .plays
                .into_iter()
                .map(|(player, play)| ShengjiPublicPlay {
                    player,
                    play,
                    throw_penalty: self.presentation.throw_penalties[usize::from(player.0)],
                })
                .collect(),
            table_points: trick.points,
        });
        let missing = super::missing_suits::missing_suits(game, trick.as_ref());
        let mut played_cards: Vec<_> = game
            .history()
            .iter()
            .flat_map(|trick| {
                trick
                    .plays
                    .iter()
                    .flat_map(|(_, play)| play.cards.iter().copied())
            })
            .chain(trick.as_ref().into_iter().flat_map(|trick| {
                trick
                    .plays
                    .iter()
                    .flat_map(|play| play.play.cards.iter().copied())
            }))
            .collect();
        played_cards.sort_by(ShengjiCard::identity_cmp);
        played_cards.dedup();
        ShengjiSnapshot {
            match_id: self.match_id.expect("started game has a match id"),
            hand_number: self.hand_number,
            host_port: self.room.host_port,
            you: recipient,
            host: self.room.host_player_id().expect("started room has a host"),
            rules: self.rules,
            players: self
                .room
                .players
                .iter()
                .filter(|player| {
                    !player.left || game.players().get(usize::from(player.id.0)).is_some()
                })
                .map(|player| {
                    let public = player.public_metadata();
                    ShengjiPlayerView {
                        seat: public.seat.expect("started player has a seat"),
                        missing_suits: missing.get(&public.player.id).cloned().unwrap_or_default(),
                        hand_len: game
                            .players()
                            .get(usize::from(public.player.id.0))
                            .map_or(0, |state| state.hand.len() as u8),
                        ready: public.ready,
                        connected: public.connected,
                        auto_play: public.auto_play,
                        player: public.player,
                    }
                })
                .collect(),
            your_hand,
            your_exposed_cards: game.bidding().exposed_cards(recipient),
            levels: game.teams().levels,
            bidding_level: game.bidding().level(),
            dealer: game.dealer(),
            trump: game.trump(),
            declaration: self.declaration_view(),
            bottom_copy_decisions: self.presentation.bottom_copy_decisions.clone(),
            current_player: (self.presentation.trick.is_none()
                && self.presentation.throw_failure.is_none()
                && self.flow.bottom_flip_remaining.is_none())
            .then(|| game.current_player())
            .flatten(),
            trick,
            played_cards,
            throw_failure: self.presentation.throw_failure.as_ref().map(|failure| {
                ShengjiThrowFailureView {
                    player: failure.player,
                    attempted: failure.attempted.clone(),
                    forced: failure.forced.clone(),
                    penalty_points: failure.penalty_points,
                    stage: failure.stage,
                }
            }),
            collecting_score: if self.presentation.trick.is_some() {
                match game.phase() {
                    ShengjiPhase::Finished(result) => pre_kitty_collecting_score(result),
                    _ => game.collecting_score(),
                }
            } else {
                game.collecting_score()
            },
            buried_count: game.buried().len() as u8,
            your_buried: if game.bottom_burier() == Some(recipient) {
                game.buried().to_vec()
            } else {
                Vec::new()
            },
            phase: self.phase_view(game, recipient),
        }
    }

    fn phase_view(&self, game: &ShengjiGameState, recipient: PlayerId) -> ShengjiPhaseView {
        if self.flow.bottom_flip_remaining.is_some() {
            return self.bottom_flip_phase_view();
        }
        if self.presentation.trick.is_some() && matches!(game.phase(), ShengjiPhase::Finished(_)) {
            return ShengjiPhaseView::Playing;
        }
        match game.phase() {
            ShengjiPhase::Dealing => {
                let dealt = game
                    .players()
                    .iter()
                    .map(|player| player.hand.len())
                    .sum::<usize>();
                ShengjiPhaseView::Dealing {
                    cards_remaining: (usize::from(self.rules.player_count())
                        * self.rules.hand_size()
                        - dealt) as u8,
                }
            }
            ShengjiPhase::BiddingGrace => ShengjiPhaseView::BiddingGrace {
                milliseconds_remaining: self
                    .flow
                    .bidding_remaining
                    .unwrap_or(BIDDING_GRACE)
                    .as_millis()
                    .min(u128::from(u16::MAX)) as u16,
                power_outage: game.power_outage_used(),
                confirmed_count: self
                    .flow
                    .bid_pass_confirmed
                    .iter()
                    .take(usize::from(self.rules.player_count()))
                    .filter(|confirmed| **confirmed)
                    .count() as u8,
                you_confirmed: self
                    .flow
                    .bid_pass_confirmed
                    .get(usize::from(recipient.0))
                    .copied()
                    .unwrap_or(false),
            },
            ShengjiPhase::BottomFlipping => self.bottom_flip_phase_view(),
            ShengjiPhase::Burying => ShengjiPhaseView::Burying,
            ShengjiPhase::BottomCopying => ShengjiPhaseView::BottomCopying {
                player: game
                    .bottom_copy()
                    .and_then(ShengjiBottomCopyState::current)
                    .expect("抄底询问阶段始终有当前玩家"),
                milliseconds_remaining: self
                    .flow
                    .bottom_copy_remaining
                    .unwrap_or(BOTTOM_COPY_DECISION_TIMEOUT)
                    .as_millis()
                    .min(u128::from(u16::MAX)) as u16,
            },
            ShengjiPhase::BottomCopyBurying => ShengjiPhaseView::BottomCopyBurying {
                player: game
                    .bottom_copy()
                    .and_then(ShengjiBottomCopyState::bottom_holder)
                    .expect("抄底重埋阶段始终有持底玩家"),
            },
            ShengjiPhase::FiveTrumpCrossing => {
                let crossing = game
                    .five_trump_crossing()
                    .expect("过江阶段始终保留过江状态");
                let players = (0..self.rules.player_count()).map(PlayerId);
                let eligible = players
                    .clone()
                    .filter(|player| crossing.eligible(*player))
                    .collect();
                let decided = players
                    .clone()
                    .filter(|player| crossing.decision_made(*player))
                    .collect();
                // 决定阶段只公布谁已经完成选择，不提前泄露其是否过江。
                let crossing_players =
                    if crossing.stage() == ShengjiFiveTrumpCrossingStage::Returning {
                        players
                            .clone()
                            .filter(|player| crossing.crossing(*player))
                            .collect()
                    } else {
                        Vec::new()
                    };
                let returned = players
                    .filter(|player| crossing.return_made(*player))
                    .collect();
                ShengjiPhaseView::FiveTrumpCrossing {
                    stage: crossing.stage(),
                    eligible,
                    decided,
                    crossing: crossing_players,
                    returned,
                }
            }
            ShengjiPhase::Playing => ShengjiPhaseView::Playing,
            ShengjiPhase::Finished(result) => ShengjiPhaseView::Finished {
                result: self.hand_result_view(result),
                buried: game.buried().to_vec(),
            },
            ShengjiPhase::RedealRequired(_) => ShengjiPhaseView::Redealing,
        }
    }

    fn bottom_flip_phase_view(&self) -> ShengjiPhaseView {
        ShengjiPhaseView::BottomFlipping {
            reveal: self.flow.bottom_flip_reveal.clone(),
        }
    }

    pub(super) fn declaration_view(&self) -> Option<ShengjiDeclarationView> {
        let declaration = self.game.as_ref()?.bidding().current()?;
        Some(ShengjiDeclarationView {
            player: declaration.player,
            trump: declaration.trump,
            kind: declaration.kind,
            protected: declaration.protected,
            cards: declaration.cards.clone(),
        })
    }
}
