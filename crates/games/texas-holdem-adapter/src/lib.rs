//! 德州扑克纯规则核心与 LeoCard 公共房间协议之间的权威适配层。
//!
//! 本 crate 不处理 TCP、连接生命周期或 Bevy。调用方按座位顺序提供玩家资料和
//! 洗好的牌堆，适配器负责平台玩家 ID、核心索引、私有底牌快照及公开摊牌之间的
//! 映射。

use std::collections::HashSet;
use std::fmt;

use leocard_protocol::{
    AvatarId, MatchId, PlayerId, ProfileId, SeatId, TexasHoldemBlindView, TexasHoldemEvent,
    TexasHoldemPhaseView, TexasHoldemPlayerState, TexasHoldemPotAward, TexasHoldemRevealedHand,
    TexasHoldemSnapshot, TexasHoldemViolation,
};
use leocard_texas_holdem::{
    Action, ActionOutcome, Card, GameError, GameState, Phase, PlayerId as CorePlayerId, RuleError,
    RuleSet, evaluate_player_hand,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TablePlayer {
    pub id: PlayerId,
    pub profile_id: ProfileId,
    pub name: String,
    pub avatar: Option<AvatarId>,
    pub seat: SeatId,
    pub connected: bool,
    pub reference_points: i32,
    pub completed_games: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdapterError {
    InvalidRules(RuleError),
    PlayerCount(usize),
    DuplicatePlayer(PlayerId),
    DuplicateSeat(SeatId),
    HostNotAtTable(PlayerId),
    RecipientNotAtTable(PlayerId),
    Game(GameError),
    Violation(TexasHoldemViolation),
}

impl fmt::Display for AdapterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::PlayerCount(actual) => write!(f, "德州扑克需要 3..=6 名玩家，实际为 {actual}"),
            Self::DuplicatePlayer(player) => write!(f, "玩家 ID {:?} 重复", player),
            Self::DuplicateSeat(seat) => write!(f, "座位 {:?} 重复", seat),
            Self::HostNotAtTable(player) => write!(f, "房主 {:?} 不在牌桌中", player),
            Self::RecipientNotAtTable(player) => write!(f, "快照接收者 {:?} 不在牌桌中", player),
            Self::Game(error) => error.fmt(f),
            Self::Violation(violation) => write!(f, "德州扑克动作被拒绝：{violation:?}"),
        }
    }
}

impl std::error::Error for AdapterError {}

impl From<RuleError> for AdapterError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

#[derive(Clone, Debug)]
pub struct TexasHoldemAdapter {
    match_id: MatchId,
    host_port: u16,
    host: PlayerId,
    players: Vec<TablePlayer>,
    game: GameState,
    hand_starting_stacks: Vec<u32>,
}

impl TexasHoldemAdapter {
    /// `players` 会按座位编号排序，排序后的顺序就是核心中的顺时针顺序。
    /// `deck[0]` 是第一张发出的牌。
    pub fn new(
        match_id: MatchId,
        host_port: u16,
        host: PlayerId,
        mut players: Vec<TablePlayer>,
        rules: RuleSet,
        first_dealer: PlayerId,
        deck: Vec<Card>,
    ) -> Result<Self, AdapterError> {
        if !(RuleSet::MIN_PLAYERS as usize..=RuleSet::MAX_PLAYERS as usize).contains(&players.len())
        {
            return Err(AdapterError::PlayerCount(players.len()));
        }
        validate_unique_players(&players)?;
        players.sort_by_key(|player| player.seat.0);
        if !players.iter().any(|player| player.id == host) {
            return Err(AdapterError::HostNotAtTable(host));
        }
        let dealer = players
            .iter()
            .position(|player| player.id == first_dealer)
            .map(CorePlayerId)
            .ok_or(AdapterError::RecipientNotAtTable(first_dealer))?;
        let effective_rules = RuleSet {
            player_count: players.len() as u8,
            ..rules
        }
        .validate()?;
        let game =
            GameState::new_with_deck(effective_rules, dealer, deck).map_err(AdapterError::Game)?;
        let hand_starting_stacks = game.players().iter().map(|player| player.stack()).collect();
        Ok(Self {
            match_id,
            host_port,
            host,
            players,
            game,
            hand_starting_stacks,
        })
    }

    pub const fn rules(&self) -> &RuleSet {
        self.game.rules()
    }

    pub const fn match_id(&self) -> MatchId {
        self.match_id
    }

    pub fn players(&self) -> &[TablePlayer] {
        &self.players
    }

    pub const fn game(&self) -> &GameState {
        &self.game
    }

    pub fn current_player(&self) -> Option<PlayerId> {
        self.game
            .current_player()
            .map(|player| self.protocol_player(player))
    }

    pub fn table_winner(&self) -> Option<PlayerId> {
        self.game
            .table_winner()
            .map(|player| self.protocol_player(player))
    }

    pub fn set_connected(&mut self, player: PlayerId, connected: bool) -> Result<(), AdapterError> {
        let participant = self
            .players
            .iter_mut()
            .find(|participant| participant.id == player)
            .ok_or(AdapterError::RecipientNotAtTable(player))?;
        participant.connected = connected;
        Ok(())
    }

    pub fn act(
        &mut self,
        player: PlayerId,
        action: Action,
    ) -> Result<Vec<TexasHoldemEvent>, AdapterError> {
        let core_player = self.core_player(player)?;
        let community_before = self.game.community().len();
        let committed_before = self.game.players()[core_player.0].committed_total();
        let outcome = self
            .game
            .act(core_player, action)
            .map_err(map_action_error)?;
        let amount = self.game.players()[core_player.0]
            .committed_total()
            .saturating_sub(committed_before);
        let mut events = vec![TexasHoldemEvent::ActionApplied {
            player,
            action,
            amount,
        }];
        match outcome {
            ActionOutcome::BlindPosted { .. } => {}
            ActionOutcome::Acted { .. } => {}
            ActionOutcome::StreetAdvanced { street, .. } => {
                events.push(TexasHoldemEvent::StreetAdvanced {
                    street,
                    dealt: self.game.community()[community_before..].to_vec(),
                });
            }
            ActionOutcome::HandComplete(result) => {
                events.push(TexasHoldemEvent::HandFinished {
                    showdown: result.showdown,
                });
            }
        }
        Ok(events)
    }

    /// 保留当前筹码并顺时针移动庄家按钮，开始下一手牌。
    pub fn start_next_hand(&mut self, deck: Vec<Card>) -> Result<(), AdapterError> {
        let starting_stacks = self
            .game
            .players()
            .iter()
            .map(|player| player.stack())
            .collect();
        self.game
            .start_next_hand(deck)
            .map_err(AdapterError::Game)?;
        self.hand_starting_stacks = starting_stacks;
        Ok(())
    }

    pub fn snapshot(&self, recipient: PlayerId) -> Result<TexasHoldemSnapshot, AdapterError> {
        let recipient_index = self.player_index(recipient)?;
        let recipient_core = CorePlayerId(recipient_index);
        let revealed_hands = self.revealed_hands();
        let players = self
            .players
            .iter()
            .zip(self.game.players())
            .enumerate()
            .map(|(index, (participant, state))| TexasHoldemPlayerState {
                id: participant.id,
                profile_id: participant.profile_id,
                name: participant.name.clone(),
                avatar: participant.avatar,
                seat: participant.seat,
                stack: state.stack(),
                hand_start_stack: self.hand_starting_stacks[index],
                committed_street: state.committed_street(),
                committed_total: state.committed_total(),
                folded: state.folded(),
                all_in: state.all_in(),
                connected: participant.connected,
                auto_play: false,
                ready: false,
                reference_points: participant.reference_points,
                completed_games: participant.completed_games,
            })
            .collect();
        let phase = match self.game.phase() {
            Phase::Betting(street) => TexasHoldemPhaseView::Betting { street: *street },
            Phase::Complete(result) => TexasHoldemPhaseView::HandComplete {
                showdown: result.showdown,
                awards: result
                    .awards
                    .iter()
                    .map(|award| TexasHoldemPotAward {
                        amount: award.amount,
                        winners: award
                            .winners
                            .iter()
                            .map(|winner| self.protocol_player(*winner))
                            .collect(),
                        winning_category: award.winning_hand.map(|hand| hand.category()),
                    })
                    .collect(),
                table_winner: self
                    .game
                    .table_winner()
                    .map(|winner| self.protocol_player(winner)),
                tournament_complete: self.game.players().iter().any(|player| player.stack() == 0),
                reference_changes: Vec::new(),
            },
        };
        Ok(TexasHoldemSnapshot {
            match_id: self.match_id,
            hand_number: self.game.hand_number(),
            host_port: self.host_port,
            you: recipient,
            host: self.host,
            players,
            your_hole_cards: self.game.players()[recipient_index].hole_cards().to_vec(),
            revealed_hands,
            community: self.game.community().to_vec(),
            draw_pile_len: self.game.draw_pile_len() as u16,
            dealer: self.protocol_player(self.game.dealer()),
            small_blind: self.protocol_player(self.game.small_blind()),
            big_blind: self.protocol_player(self.game.big_blind()),
            current_player: self
                .game
                .current_player()
                .map(|player| self.protocol_player(player)),
            blind_to_post: self.game.blind_to_post().map(|(player, kind, amount)| {
                TexasHoldemBlindView {
                    player: self.protocol_player(player),
                    kind,
                    amount,
                }
            }),
            current_bet: self.game.current_bet(),
            minimum_raise_to: self.game.minimum_raise_to(),
            amount_to_call: self
                .game
                .amount_to_call(recipient_core)
                .expect("recipient maps to a core player"),
            raise_allowed: self
                .game
                .raise_allowed(recipient_core)
                .expect("recipient maps to a core player"),
            pot: self.game.pot(),
            phase,
        })
    }

    fn revealed_hands(&self) -> Vec<TexasHoldemRevealedHand> {
        let Phase::Complete(result) = self.game.phase() else {
            return Vec::new();
        };
        // 无争议收池时任何玩家都不亮底牌，包括最后留下的赢家。
        if !result.showdown {
            return Vec::new();
        }
        self.game
            .players()
            .iter()
            .filter(|player| {
                !player.folded() && player.hole_cards().len() == self.game.rules().hole_card_count()
            })
            .map(|player| {
                let cards = player.hole_cards().to_vec();
                TexasHoldemRevealedHand {
                    player: self.protocol_player(player.id()),
                    cards,
                    best: (self.game.community().len() >= 3)
                        .then(|| {
                            evaluate_player_hand(
                                player.hole_cards(),
                                self.game.community(),
                                self.game.rules(),
                            )
                            .ok()
                        })
                        .flatten(),
                }
            })
            .collect()
    }

    fn player_index(&self, player: PlayerId) -> Result<usize, AdapterError> {
        self.players
            .iter()
            .position(|participant| participant.id == player)
            .ok_or(AdapterError::RecipientNotAtTable(player))
    }

    fn core_player(&self, player: PlayerId) -> Result<CorePlayerId, AdapterError> {
        self.player_index(player).map(CorePlayerId)
    }

    fn protocol_player(&self, player: CorePlayerId) -> PlayerId {
        self.players[player.0].id
    }
}

fn validate_unique_players(players: &[TablePlayer]) -> Result<(), AdapterError> {
    let mut ids = HashSet::with_capacity(players.len());
    let mut seats = HashSet::with_capacity(players.len());
    for player in players {
        if !ids.insert(player.id) {
            return Err(AdapterError::DuplicatePlayer(player.id));
        }
        if !seats.insert(player.seat) {
            return Err(AdapterError::DuplicateSeat(player.seat));
        }
    }
    Ok(())
}

fn map_action_error(error: GameError) -> AdapterError {
    let violation = match error {
        GameError::InvalidPlayer(_) => TexasHoldemViolation::InvalidPlayer,
        GameError::NotPlayersTurn { .. } => TexasHoldemViolation::NotPlayersTurn,
        GameError::HandAlreadyComplete => TexasHoldemViolation::HandAlreadyComplete,
        GameError::PlayerCannotAct(_) => TexasHoldemViolation::PlayerCannotAct,
        GameError::MustPostBlind => TexasHoldemViolation::MustPostBlind,
        GameError::NoBlindToPost => TexasHoldemViolation::NoBlindToPost,
        GameError::CannotCheckWhileFacingBet { amount_to_call } => {
            TexasHoldemViolation::CannotCheckWhileFacingBet { amount_to_call }
        }
        GameError::NothingToCall => TexasHoldemViolation::NothingToCall,
        GameError::RaiseMustExceedCurrentBet {
            current_bet,
            target,
        } => TexasHoldemViolation::RaiseMustExceedCurrentBet {
            current_bet,
            target,
        },
        GameError::RaiseBelowMinimum {
            minimum_target,
            target,
        } => TexasHoldemViolation::RaiseBelowMinimum {
            minimum_target,
            target,
        },
        GameError::RaiseExceedsStack {
            maximum_target,
            target,
        } => TexasHoldemViolation::RaiseExceedsStack {
            maximum_target,
            target,
        },
        GameError::RaiseNotReopened => TexasHoldemViolation::RaiseNotReopened,
        other => return AdapterError::Game(other),
    };
    AdapterError::Violation(violation)
}

#[cfg(test)]
mod tests {
    use super::*;
    use leocard_protocol::{
        GameSnapshot, PROTOCOL_VERSION, Revision, RoomId, ServerEvent, ServerMessage,
        TexasHoldemEvent, decode_frame, encode_frame,
    };
    use leocard_texas_holdem::build_deck;

    const HOST: PlayerId = PlayerId(20);
    const LEFT: PlayerId = PlayerId(30);
    const RIGHT: PlayerId = PlayerId(10);

    fn player(id: PlayerId, seat: u8) -> TablePlayer {
        TablePlayer {
            id,
            profile_id: ProfileId([id.0; 32]),
            name: format!("玩家{}", id.0),
            avatar: None,
            seat: SeatId(seat),
            connected: true,
            reference_points: 0,
            completed_games: 0,
        }
    }

    fn raw_adapter() -> TexasHoldemAdapter {
        TexasHoldemAdapter::new(
            MatchId([7; 16]),
            52300,
            HOST,
            vec![player(RIGHT, 2), player(HOST, 0), player(LEFT, 1)],
            RuleSet::default(),
            HOST,
            build_deck(false),
        )
        .unwrap()
    }

    fn adapter() -> TexasHoldemAdapter {
        let mut game = raw_adapter();
        game.act(LEFT, Action::PostBlind).unwrap();
        game.act(RIGHT, Action::PostBlind).unwrap();
        game
    }

    #[test]
    fn blind_posting_is_exposed_before_normal_preflop_actions() {
        let mut game = raw_adapter();
        let first = game.snapshot(HOST).unwrap().blind_to_post.unwrap();
        assert_eq!(first.player, LEFT);
        assert_eq!(first.kind, leocard_texas_holdem::BlindKind::Small);
        assert_eq!(first.amount, 1);
        assert!(matches!(
            game.act(LEFT, Action::Call),
            Err(AdapterError::Violation(TexasHoldemViolation::MustPostBlind))
        ));
        game.act(LEFT, Action::PostBlind).unwrap();
        let second = game.snapshot(HOST).unwrap().blind_to_post.unwrap();
        assert_eq!(second.player, RIGHT);
        assert_eq!(second.kind, leocard_texas_holdem::BlindKind::Big);
        game.act(RIGHT, Action::PostBlind).unwrap();
        assert!(game.snapshot(HOST).unwrap().blind_to_post.is_none());
        assert_eq!(game.current_player(), Some(HOST));
    }

    #[test]
    fn seat_order_maps_platform_ids_to_core_positions() {
        let game = adapter();
        let snapshot = game.snapshot(HOST).unwrap();
        assert_eq!(
            snapshot.players.iter().map(|p| p.id).collect::<Vec<_>>(),
            vec![HOST, LEFT, RIGHT]
        );
        assert_eq!(snapshot.dealer, HOST);
        assert_eq!(snapshot.small_blind, LEFT);
        assert_eq!(snapshot.big_blind, RIGHT);
        assert_eq!(snapshot.current_player, Some(HOST));
    }

    #[test]
    fn betting_snapshots_only_contain_the_recipient_hole_cards() {
        let game = adapter();
        let host = game.snapshot(HOST).unwrap();
        let left = game.snapshot(LEFT).unwrap();
        assert_eq!(host.your_hole_cards.len(), 2);
        assert_eq!(left.your_hole_cards.len(), 2);
        assert_ne!(host.your_hole_cards, left.your_hole_cards);
        assert!(host.revealed_hands.is_empty());
        assert!(left.revealed_hands.is_empty());
        assert_eq!(host.players, left.players);
    }

    #[test]
    fn omaha_snapshots_deal_and_reveal_four_cards_with_an_evaluated_hand() {
        let mut game = TexasHoldemAdapter::new(
            MatchId([8; 16]),
            52300,
            HOST,
            vec![player(RIGHT, 2), player(HOST, 0), player(LEFT, 1)],
            RuleSet {
                omaha: true,
                ..RuleSet::default()
            },
            HOST,
            build_deck(false),
        )
        .unwrap();
        assert_eq!(game.snapshot(HOST).unwrap().your_hole_cards.len(), 4);

        game.act(LEFT, Action::PostBlind).unwrap();
        game.act(RIGHT, Action::PostBlind).unwrap();
        game.act(HOST, Action::AllIn).unwrap();
        game.act(LEFT, Action::AllIn).unwrap();
        game.act(RIGHT, Action::Call).unwrap();

        let snapshot = game.snapshot(HOST).unwrap();
        assert_eq!(snapshot.community.len(), 5);
        assert_eq!(snapshot.revealed_hands.len(), 3);
        assert!(
            snapshot
                .revealed_hands
                .iter()
                .all(|hand| hand.cards.len() == 4 && hand.best.is_some())
        );

        let message = ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(523),
            revision: Revision(9),
            in_reply_to: None,
            event: ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot)),
        };
        let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
        assert_eq!(decoded, message);
    }

    #[test]
    fn adapter_emits_action_and_street_events() {
        let mut game = adapter();
        game.act(HOST, Action::Call).unwrap();
        game.act(LEFT, Action::Call).unwrap();
        let events = game.act(RIGHT, Action::Check).unwrap();
        assert_eq!(events.len(), 2);
        assert!(matches!(
            events[0],
            TexasHoldemEvent::ActionApplied {
                player: RIGHT,
                action: Action::Check,
                amount: 0,
            }
        ));
        assert!(matches!(
            &events[1],
            TexasHoldemEvent::StreetAdvanced { street, dealt }
                if *street == leocard_texas_holdem::Street::Flop && dealt.len() == 3
        ));
        assert_eq!(game.snapshot(HOST).unwrap().community.len(), 3);
    }

    #[test]
    fn rejected_action_returns_a_wire_violation_without_mutating_state() {
        let mut game = adapter();
        let before = game.game().clone();
        assert_eq!(
            game.act(LEFT, Action::Call),
            Err(AdapterError::Violation(
                TexasHoldemViolation::NotPlayersTurn
            ))
        );
        assert_eq!(game.game(), &before);
    }

    #[test]
    fn showdown_reveals_all_hands_and_exposes_pot_awards() {
        let mut game = adapter();
        game.act(HOST, Action::AllIn).unwrap();
        game.act(LEFT, Action::AllIn).unwrap();
        let events = game.act(RIGHT, Action::Call).unwrap();
        assert!(matches!(
            events.last(),
            Some(TexasHoldemEvent::HandFinished { showdown: true })
        ));
        let snapshot = game.snapshot(HOST).unwrap();
        assert_eq!(snapshot.community.len(), 5);
        assert_eq!(snapshot.revealed_hands.len(), 3);
        let TexasHoldemPhaseView::HandComplete {
            showdown, awards, ..
        } = snapshot.phase
        else {
            panic!("the hand should be complete");
        };
        assert!(showdown);
        assert_eq!(awards.iter().map(|award| award.amount).sum::<u32>(), 60);
    }

    #[test]
    fn showdown_keeps_previously_folded_hands_private() {
        let mut game = adapter();
        game.act(HOST, Action::Fold).unwrap();
        game.act(LEFT, Action::AllIn).unwrap();
        game.act(RIGHT, Action::Call).unwrap();
        let snapshot = game.snapshot(HOST).unwrap();
        assert_eq!(snapshot.revealed_hands.len(), 2);
        assert!(
            snapshot
                .revealed_hands
                .iter()
                .all(|hand| hand.player != HOST)
        );
    }

    #[test]
    fn uncontested_win_does_not_reveal_any_hole_cards() {
        let mut game = adapter();
        game.act(HOST, Action::Fold).unwrap();
        let events = game.act(LEFT, Action::Fold).unwrap();
        assert!(matches!(
            events.last(),
            Some(TexasHoldemEvent::HandFinished { showdown: false })
        ));
        let snapshot = game.snapshot(RIGHT).unwrap();
        assert!(snapshot.revealed_hands.is_empty());
    }

    #[test]
    fn next_hand_keeps_stacks_and_rotates_the_dealer() {
        let mut game = adapter();
        game.act(HOST, Action::Fold).unwrap();
        game.act(LEFT, Action::Fold).unwrap();
        game.start_next_hand(build_deck(false)).unwrap();
        let snapshot = game.snapshot(HOST).unwrap();
        assert_eq!(snapshot.hand_number, 1);
        assert_eq!(snapshot.dealer, LEFT);
        assert!(matches!(
            snapshot.phase,
            TexasHoldemPhaseView::Betting {
                street: leocard_texas_holdem::Street::PreFlop
            }
        ));
    }

    #[test]
    fn private_snapshot_round_trips_through_the_wire_frame() {
        let snapshot = adapter().snapshot(HOST).unwrap();
        let message = ServerMessage {
            protocol_version: PROTOCOL_VERSION,
            room_id: RoomId(523),
            revision: Revision(8),
            in_reply_to: None,
            event: ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(snapshot.clone())),
        };
        let decoded: ServerMessage = decode_frame(&encode_frame(&message).unwrap()).unwrap();
        assert_eq!(decoded, message);
        let ServerEvent::GameSnapshot(GameSnapshot::TexasHoldem(decoded_snapshot)) = decoded.event
        else {
            panic!("wire frame changed the concrete game variant");
        };
        assert_eq!(decoded_snapshot.your_hole_cards, snapshot.your_hole_cards);
        assert!(decoded_snapshot.revealed_hands.is_empty());
    }
}
