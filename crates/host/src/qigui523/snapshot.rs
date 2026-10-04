use super::{QiGui523Session, from_core_player};
use leocard_protocol::{
    GamePhaseView, PlayerId, PlayerPublicState, PlayerScore, PublicPlay, PublicPlayRecord,
    QiGui523Snapshot, RevealedHand, StartingCardView, TrickView,
};
use leocard_qigui523::{Phase, PlayRecord};

impl QiGui523Session {
    pub(super) fn game_snapshot(&self, recipient: PlayerId) -> QiGui523Snapshot {
        let game = self.game.as_ref().expect("game snapshot requires a game");
        let recipient_index = usize::from(recipient.0);
        let players = game
            .players()
            .iter()
            .zip(&self.players)
            .map(|(state, participant)| {
                let public = participant.public_metadata();
                PlayerPublicState {
                    id: public.id,
                    profile_id: public.profile_id,
                    name: public.name,
                    avatar: public.avatar,
                    seat: public
                        .seat
                        .expect("all game participants have selected seats"),
                    hand_len: state.hand().len() as u16,
                    score: state.score(),
                    ready: public.ready,
                    connected: public.connected,
                    auto_play: public.auto_play,
                    reference_points: public.reference_points,
                    completed_games: public.completed_games,
                    game_profiles: public.game_profiles,
                }
            })
            .collect();
        let starting = game.starting_card();

        QiGui523Snapshot {
            match_id: self.match_id.expect("a running game has a match id"),
            host_port: self.host_port,
            you: recipient,
            host: self
                .host_connection
                .and_then(|connection| {
                    self.players
                        .iter()
                        .find(|player| player.connection == connection)
                        .map(|player| player.id)
                })
                .expect("a running game has a room host"),
            players,
            your_hand: game.players()[recipient_index].hand().to_vec(),
            draw_pile_len: game.draw_pile_len() as u16,
            starting_card: StartingCardView {
                player: from_core_player(starting.player),
                card: starting.card,
            },
            trick: game.trick().map(|trick| TrickView {
                leader: from_core_player(trick.leader()),
                current_player: from_core_player(trick.current_player()),
                winning_player: trick.winning_player().map(from_core_player),
                winning_play: trick.winning_play().map(|play| PublicPlay {
                    kind: play.kind().clone(),
                    cards: play.cards().to_vec(),
                }),
                records: trick
                    .records()
                    .iter()
                    .map(|record| match record {
                        PlayRecord::Played { player, play } => PublicPlayRecord::Played {
                            player: from_core_player(*player),
                            play: PublicPlay {
                                kind: play.kind().clone(),
                                cards: play.cards().to_vec(),
                            },
                        },
                        PlayRecord::Passed { player } => PublicPlayRecord::Passed {
                            player: from_core_player(*player),
                        },
                    })
                    .collect(),
                table_points: trick.table_points(),
            }),
            turn_timer: self.turn_timer_view(),
            phase: match game.phase() {
                Phase::Playing => GamePhaseView::Playing,
                Phase::Finished(result) => GamePhaseView::Finished {
                    match_id: self.match_id.expect("a running game has a match id"),
                    finisher: from_core_player(result.finisher),
                    scores: result
                        .scores
                        .iter()
                        .enumerate()
                        .map(|(index, score)| PlayerScore {
                            player: PlayerId(index as u8),
                            score: *score,
                        })
                        .collect(),
                    remaining_hands: game
                        .players()
                        .iter()
                        .enumerate()
                        .map(|(index, player)| RevealedHand {
                            player: PlayerId(index as u8),
                            cards: player.hand().to_vec(),
                        })
                        .collect(),
                    reference_changes: self
                        .finished_reference_changes
                        .clone()
                        .expect("finished points are applied before broadcasting a snapshot"),
                    captured_hand_points: result.captured_hand_points,
                },
            },
        }
    }
}
