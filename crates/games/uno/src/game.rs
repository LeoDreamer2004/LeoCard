use std::collections::{HashSet, VecDeque};
use std::fmt;

use crate::rating::placements;
use crate::{Card, Color, Face, RuleError, RuleSet, build_deck, reference_point_deltas};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerId(pub usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerState {
    id: PlayerId,
    hand: Vec<Card>,
}

impl PlayerState {
    pub const fn id(&self) -> PlayerId {
        self.id
    }

    pub fn hand(&self) -> &[Card] {
        &self.hand
    }

    pub fn hand_score(&self) -> u16 {
        self.hand.iter().map(|card| card.score()).sum()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Direction {
    Clockwise,
    CounterClockwise,
}

impl Direction {
    const fn reversed(self) -> Self {
        match self {
            Self::Clockwise => Self::CounterClockwise,
            Self::CounterClockwise => Self::Clockwise,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PendingDrawKind {
    DrawTwo,
    WildDrawFour,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TurnState {
    pub current_player: PlayerId,
    pub direction: Direction,
    pub current_color: Option<Color>,
    pub top_card: Card,
    pub pending_draw: u16,
    pub pending_kind: Option<PendingDrawKind>,
    pub challenge_offender: Option<PlayerId>,
    pub drawn_card: Option<Card>,
    pub pending_skip: u16,
    pub skipped_turns_remaining: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameResult {
    pub winner: PlayerId,
    pub hand_scores: Vec<u16>,
    pub placements: Vec<u8>,
    pub reference_deltas: Vec<i16>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Phase {
    Playing,
    Finished(GameResult),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChallengeResult {
    Successful,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ActionOutcome {
    ColorChosen {
        player: PlayerId,
        color: Color,
    },
    Played {
        player: PlayerId,
        card: Card,
        next_player: PlayerId,
    },
    DrewCard {
        player: PlayerId,
        card: Card,
        playable: bool,
        next_player: PlayerId,
    },
    PassedAfterDraw {
        player: PlayerId,
        next_player: PlayerId,
    },
    PenaltyDrawn {
        player: PlayerId,
        cards: Vec<Card>,
        next_player: PlayerId,
    },
    ChallengeResolved {
        challenger: PlayerId,
        offender: PlayerId,
        result: ChallengeResult,
        penalized: PlayerId,
        cards: Vec<Card>,
        next_player: PlayerId,
    },
    UnoCalled {
        player: PlayerId,
    },
    UnoReported {
        reporter: PlayerId,
        target: PlayerId,
        cards: Vec<Card>,
    },
    SkipResolved {
        player: PlayerId,
        cards: Vec<Card>,
        remaining: u16,
        next_player: PlayerId,
    },
    GameFinished(GameResult),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GameError {
    InvalidRules(RuleError),
    InvalidDeckSize {
        expected: usize,
        actual: usize,
    },
    InvalidDeckContents,
    InvalidPlayer(PlayerId),
    NotPlayersTurn {
        expected: PlayerId,
        actual: PlayerId,
    },
    GameAlreadyFinished,
    InitialColorChoiceRequired,
    InitialColorAlreadyChosen,
    CardNotInHand(Card),
    CardDoesNotMatch,
    ColorRequired,
    UnexpectedColor,
    MustPlayDrawnCard(Card),
    MustResolveDrawPenalty,
    NoDrawPenalty,
    CannotStack(Card),
    CannotChallenge,
    MustDrawBeforePassing,
    MustResolveSkip,
    NoSkipToResolve,
    UnoCalloutDisabled,
    CannotCallUno(PlayerId),
    MustPlayAfterUno,
    CannotReportSelf,
    PlayerNotReportable(PlayerId),
    CannotPlayTogether,
    CannotJumpIn,
    DrawPileExhausted,
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRules(error) => error.fmt(f),
            Self::InvalidDeckSize { expected, actual } => {
                write!(f, "牌堆张数错误：应为 {expected}，实际为 {actual}")
            }
            Self::InvalidDeckContents => f.write_str("牌堆有缺牌、重复牌或非法牌"),
            Self::InvalidPlayer(player) => write!(f, "玩家 {:?} 不存在", player),
            Self::NotPlayersTurn { expected, actual } => {
                write!(f, "尚未轮到 {:?}；当前应由 {:?} 操作", actual, expected)
            }
            Self::GameAlreadyFinished => f.write_str("本局已经结束"),
            Self::InitialColorChoiceRequired => f.write_str("先手必须先为万能牌选择颜色"),
            Self::InitialColorAlreadyChosen => f.write_str("当前不需要选择初始颜色"),
            Self::CardNotInHand(card) => write!(f, "玩家手中没有这张牌：{card}"),
            Self::CardDoesNotMatch => f.write_str("这张牌与当前颜色、数字或符号不匹配"),
            Self::ColorRequired => f.write_str("万能牌必须指定后续颜色"),
            Self::UnexpectedColor => f.write_str("非万能牌不能指定后续颜色"),
            Self::MustPlayDrawnCard(card) => write!(f, "摸牌后只能打出刚摸到的牌：{card}"),
            Self::MustResolveDrawPenalty => f.write_str("必须叠加罚牌、质疑或接受累计罚牌"),
            Self::NoDrawPenalty => f.write_str("当前没有待结算的罚牌"),
            Self::CannotStack(card) => write!(f, "当前不能叠加这张牌：{card}"),
            Self::CannotChallenge => f.write_str("当前没有可质疑的万能摸四"),
            Self::MustDrawBeforePassing => f.write_str("只有摸到可出的牌后才能选择结束回合"),
            Self::MustResolveSkip => f.write_str("必须叠加禁手牌或接受禁手"),
            Self::NoSkipToResolve => f.write_str("当前没有待结算的禁手"),
            Self::UnoCalloutDisabled => f.write_str("本房间未启用 UNO 宣告和检举"),
            Self::CannotCallUno(player) => write!(f, "玩家 {:?} 当前不能宣告 UNO", player),
            Self::MustPlayAfterUno => f.write_str("宣告 UNO 后必须打出倒数第二张牌"),
            Self::CannotReportSelf => f.write_str("不能检举自己"),
            Self::PlayerNotReportable(player) => write!(f, "玩家 {:?} 当前不可被检举", player),
            Self::CannotPlayTogether => f.write_str("这些牌当前不能一次打出"),
            Self::CannotJumpIn => f.write_str("当前不能抢出这张牌"),
            Self::DrawPileExhausted => f.write_str("没有可继续摸取的牌"),
        }
    }
}

impl std::error::Error for GameError {}

impl From<RuleError> for GameError {
    fn from(value: RuleError) -> Self {
        Self::InvalidRules(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ChallengeState {
    offender: PlayerId,
    was_legal: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GameState {
    rules: RuleSet,
    players: Vec<PlayerState>,
    draw_pile: VecDeque<Card>,
    discard_pile: Vec<Card>,
    current_player: PlayerId,
    direction: Direction,
    current_color: Option<Color>,
    pending_draw: u16,
    pending_kind: Option<PendingDrawKind>,
    challenge: Option<ChallengeState>,
    drawn_card: Option<Card>,
    pending_skip: u16,
    skip_turns: Vec<u16>,
    uno_exposed: Vec<bool>,
    uno_declared: Vec<bool>,
    jump_in_open: bool,
    phase: Phase,
}

impl GameState {
    /// `deck[0]` 是第一张发出的牌。
    pub fn new_with_deck(
        rules: RuleSet,
        player_count: u8,
        deck: Vec<Card>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        let player_count = RuleSet::validate_player_count(player_count)?;
        validate_deck(&deck)?;
        let mut draw_pile = VecDeque::from(deck);
        let mut players = (0..player_count)
            .map(|index| PlayerState {
                id: PlayerId(index),
                hand: Vec::with_capacity(usize::from(RuleSet::HAND_SIZE)),
            })
            .collect::<Vec<_>>();
        for _ in 0..RuleSet::HAND_SIZE {
            for player in &mut players {
                player.hand.push(
                    draw_pile
                        .pop_front()
                        .expect("a valid UNO deck is large enough"),
                );
            }
        }
        for player in &mut players {
            player.hand.sort_by(Card::display_cmp);
        }

        // 万能摸四不能作为起始牌；把它放回牌堆末尾并继续翻牌。
        let top_card = loop {
            let card = draw_pile
                .pop_front()
                .expect("a valid UNO deck has a starting card");
            if card.face() == Face::WildDrawFour {
                draw_pile.push_back(card);
            } else {
                break card;
            }
        };
        let mut state = Self {
            rules,
            players,
            draw_pile,
            discard_pile: vec![top_card],
            current_player: PlayerId(0),
            direction: Direction::Clockwise,
            current_color: top_card.color(),
            pending_draw: 0,
            pending_kind: None,
            challenge: None,
            drawn_card: None,
            pending_skip: 0,
            skip_turns: vec![0; player_count],
            uno_exposed: vec![false; player_count],
            uno_declared: vec![false; player_count],
            jump_in_open: false,
            phase: Phase::Playing,
        };
        state.apply_starting_card()?;
        Ok(state)
    }

    pub const fn rules(&self) -> &RuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    pub fn player(&self, player: PlayerId) -> Option<&PlayerState> {
        self.players.get(player.0)
    }

    pub const fn phase(&self) -> &Phase {
        &self.phase
    }

    pub fn turn(&self) -> Option<TurnState> {
        matches!(self.phase, Phase::Playing).then(|| TurnState {
            current_player: self.current_player,
            direction: self.direction,
            current_color: self.current_color,
            top_card: *self.discard_pile.last().expect("a game has a discard"),
            pending_draw: self.pending_draw,
            pending_kind: self.pending_kind,
            challenge_offender: self.challenge.map(|challenge| challenge.offender),
            drawn_card: self.drawn_card,
            pending_skip: self.pending_skip,
            skipped_turns_remaining: self.skip_turns[self.current_player.0],
        })
    }

    pub fn discard_pile(&self) -> &[Card] {
        &self.discard_pile
    }

    pub fn draw_pile_len(&self) -> usize {
        self.draw_pile.len()
    }

    pub fn top_card(&self) -> Card {
        *self.discard_pile.last().expect("a game has a discard")
    }

    pub const fn current_color(&self) -> Option<Color> {
        self.current_color
    }

    pub const fn direction(&self) -> Direction {
        self.direction
    }

    pub fn skipped_turns(&self, player: PlayerId) -> Option<u16> {
        self.skip_turns.get(player.0).copied()
    }

    pub fn uno_exposed_players(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.uno_exposed
            .iter()
            .enumerate()
            .filter_map(|(index, exposed)| exposed.then_some(PlayerId(index)))
    }

    pub fn uno_declared_players(&self) -> impl Iterator<Item = PlayerId> + '_ {
        self.uno_declared
            .iter()
            .enumerate()
            .filter_map(|(index, declared)| declared.then_some(PlayerId(index)))
    }

    /// 返回该玩家当前唯一可抢出的物理牌。经典 108 张牌中每种彩色牌面至多
    /// 两张，桌面已经有一张，因此至多只会找到一张候选牌。
    pub fn jump_in_card(&self, player: PlayerId) -> Option<Card> {
        if !matches!(self.phase, Phase::Playing)
            || !self.rules.jump_in
            || !self.jump_in_open
            || player == self.current_player
            || self.skip_turns.get(player.0).copied().unwrap_or(0) > 0
        {
            return None;
        }
        let top = self.top_card();
        top.color()?;
        self.players
            .get(player.0)?
            .hand
            .iter()
            .copied()
            .find(|card| *card != top && card.color() == top.color() && card.face() == top.face())
    }

    pub fn choose_initial_color(
        &mut self,
        player: PlayerId,
        color: Color,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if self.current_color.is_some()
            || self.discard_pile.last().map(|card| card.face()) != Some(Face::Wild)
        {
            return Err(GameError::InitialColorAlreadyChosen);
        }
        self.ensure_turn(player)?;
        self.current_color = Some(color);
        Ok(ActionOutcome::ColorChosen { player, color })
    }

    pub fn can_play(&self, player: PlayerId, card: Card) -> bool {
        self.ensure_turn(player).is_ok()
            && self.current_color.is_some()
            && self.players[player.0].hand.contains(&card)
            && self.card_allowed_in_current_state(card)
            && self.drawn_card.is_none_or(|drawn_card| drawn_card == card)
    }

    pub fn play_card(
        &mut self,
        player: PlayerId,
        card: Card,
        chosen_color: Option<Color>,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        if self.current_color.is_none() {
            return Err(GameError::InitialColorChoiceRequired);
        }
        if !self.players[player.0].hand.contains(&card) {
            return Err(GameError::CardNotInHand(card));
        }
        match (card.face().is_wild(), chosen_color) {
            (true, None) => return Err(GameError::ColorRequired),
            (false, Some(_)) => return Err(GameError::UnexpectedColor),
            _ => {}
        }
        if let Some(drawn_card) = self.drawn_card
            && card != drawn_card
        {
            return Err(GameError::MustPlayDrawnCard(drawn_card));
        }
        if self.skip_turns[player.0] > 0 {
            return Err(GameError::MustResolveSkip);
        }
        if self.pending_skip > 0 {
            if !self.rules.stack_skip || card.face() != Face::Skip {
                return Err(GameError::MustResolveSkip);
            }
        } else if self.pending_draw > 0 {
            if !self.stack_allowed(card) {
                return Err(GameError::CannotStack(card));
            }
        } else if !self.matches_top(card) {
            return Err(GameError::CardDoesNotMatch);
        }

        let previous_color = self.current_color.expect("checked above");
        let draw_four_was_legal = card.face() != Face::WildDrawFour
            || !self.players[player.0]
                .hand
                .iter()
                .any(|other| *other != card && other.color() == Some(previous_color));
        let was_exposed = self.uno_exposed[player.0];
        let declared_uno = self.uno_declared[player.0];
        self.uno_exposed[player.0] = false;
        self.uno_declared[player.0] = false;
        self.drawn_card = None;
        self.jump_in_open = false;
        remove_card(&mut self.players[player.0].hand, card);
        self.discard_pile.push(card);
        self.current_color = chosen_color.or(card.color());

        // 按约定，最后一张一落桌就结束；不再执行功能牌、叠加或质疑。
        if self.players[player.0].hand.is_empty() {
            let result = self.finish(player);
            return Ok(ActionOutcome::GameFinished(result));
        }

        if self.rules.uno_callout && self.players[player.0].hand.len() == 1 && !declared_uno {
            self.uno_exposed[player.0] = true;
        } else if was_exposed {
            self.uno_exposed[player.0] = false;
        }

        let next_player = match card.face() {
            Face::DrawTwo => {
                self.pending_draw += 2;
                self.pending_kind = Some(PendingDrawKind::DrawTwo);
                self.challenge = None;
                self.next_player(player)
            }
            Face::WildDrawFour => {
                self.pending_draw += 4;
                self.pending_kind = Some(PendingDrawKind::WildDrawFour);
                if self.challenge.is_none() {
                    self.challenge = Some(ChallengeState {
                        offender: player,
                        was_legal: draw_four_was_legal,
                    });
                }
                self.next_player(player)
            }
            Face::Reverse if self.players.len() == 2 => player,
            Face::Reverse => {
                self.direction = self.direction.reversed();
                self.next_player(player)
            }
            Face::Skip => {
                self.pending_skip = if self.rules.stack_skip {
                    self.pending_skip.saturating_add(1)
                } else {
                    let target = self.next_player(player);
                    self.skip_turns[target.0] = self.skip_turns[target.0].max(1);
                    0
                };
                self.next_player(player)
            }
            Face::Number(_) | Face::Wild => self.next_player(player),
        };
        self.current_player = next_player;
        self.jump_in_open = self.rules.jump_in && card.color().is_some();
        Ok(ActionOutcome::Played {
            player,
            card,
            next_player,
        })
    }

    /// 一次提交一张普通牌，或在抢出规则下提交两张完全相同的彩色牌。
    /// 双牌通过克隆状态原子结算，第二张失败时不会留下只打出第一张的半成品。
    pub fn play_cards(
        &mut self,
        player: PlayerId,
        cards: &[Card],
        chosen_color: Option<Color>,
    ) -> Result<ActionOutcome, GameError> {
        let [first, second] = cards else {
            return match cards {
                [card] => self.play_card(player, *card, chosen_color),
                _ => Err(GameError::CannotPlayTogether),
            };
        };
        if !self.rules.jump_in
            || chosen_color.is_some()
            || first == second
            || first.color().is_none()
            || first.color() != second.color()
            || first.face() != second.face()
            || self.uno_declared.get(player.0).copied().unwrap_or(false)
            || !self
                .players
                .get(player.0)
                .is_some_and(|state| state.hand.contains(first) && state.hand.contains(second))
        {
            return Err(GameError::CannotPlayTogether);
        }

        let mut staged = self.clone();
        if matches!(
            staged.play_card(player, *first, None)?,
            ActionOutcome::GameFinished(_)
        ) {
            return Err(GameError::CannotPlayTogether);
        }
        let outcome = if staged.current_player == player {
            staged.play_card(player, *second, None)?
        } else {
            staged.jump_in(player, *second)?
        };
        *self = staged;
        Ok(outcome)
    }

    /// 非下家在窗口关闭前抢出与桌面牌颜色、牌面完全一致的另一张物理牌。
    pub fn jump_in(&mut self, player: PlayerId, card: Card) -> Result<ActionOutcome, GameError> {
        if self.jump_in_card(player) != Some(card) {
            return Err(GameError::CannotJumpIn);
        }
        self.current_player = player;
        self.play_card(player, card, None)
    }

    pub fn draw_card(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_skip_resolved(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.current_color.is_none() {
            return Err(GameError::InitialColorChoiceRequired);
        }
        if self.pending_draw > 0 {
            return Err(GameError::MustResolveDrawPenalty);
        }
        if self.drawn_card.is_some() {
            return Err(GameError::MustDrawBeforePassing);
        }
        let card = self.draw_cards_for(player, 1)?.remove(0);
        self.jump_in_open = false;
        let playable = self.matches_top(card);
        let next_player = if playable {
            self.drawn_card = Some(card);
            player
        } else {
            self.next_player(player)
        };
        self.current_player = next_player;
        Ok(ActionOutcome::DrewCard {
            player,
            card,
            playable,
            next_player,
        })
    }

    pub fn pass_after_draw(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_skip_resolved(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.drawn_card.is_none() {
            return Err(GameError::MustDrawBeforePassing);
        }
        self.drawn_card = None;
        self.jump_in_open = false;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        Ok(ActionOutcome::PassedAfterDraw {
            player,
            next_player,
        })
    }

    pub fn accept_draw_penalty(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.pending_draw == 0 {
            return Err(GameError::NoDrawPenalty);
        }
        let must_resolve_skip = self.pending_skip > 0 || self.skip_turns[player.0] > 0;
        let count = self.pending_draw;
        let cards = self.draw_cards_for(player, count)?;
        self.jump_in_open = false;
        self.clear_pending_draw();
        // 罚牌始终属于功能牌出牌者的直接下家。若该玩家同时仍被禁手，先由其
        // 收下罚牌并留在当前回合，随后再单独消耗禁手，不能把罚牌传给下下家。
        let next_player = if must_resolve_skip {
            player
        } else {
            self.next_player(player)
        };
        self.current_player = next_player;
        Ok(ActionOutcome::PenaltyDrawn {
            player,
            cards,
            next_player,
        })
    }

    pub fn challenge_draw_four(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_uno_followup_resolved(player)?;
        let challenge = self.challenge.ok_or(GameError::CannotChallenge)?;
        let must_resolve_skip = self.pending_skip > 0 || self.skip_turns[player.0] > 0;
        let (result, penalized, count, next_player) = if challenge.was_legal {
            (
                ChallengeResult::Failed,
                player,
                self.pending_draw + 2,
                if must_resolve_skip {
                    player
                } else {
                    self.next_player(player)
                },
            )
        } else {
            (
                ChallengeResult::Successful,
                challenge.offender,
                self.pending_draw,
                player,
            )
        };
        let cards = self.draw_cards_for(penalized, count)?;
        self.jump_in_open = false;
        self.clear_pending_draw();
        self.current_player = next_player;
        Ok(ActionOutcome::ChallengeResolved {
            challenger: player,
            offender: challenge.offender,
            result,
            penalized,
            cards,
            next_player,
        })
    }

    pub fn call_uno(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if !self.rules.uno_callout {
            return Err(GameError::UnoCalloutDisabled);
        }
        if self.uno_declared[player.0] {
            return Err(GameError::CannotCallUno(player));
        }

        // 正常宣告发生在自己的回合、手里还有两张牌且可以立刻打出一张时。
        let can_declare_before_play = player == self.current_player
            && self.players[player.0].hand.len() == 2
            && self.pending_draw == 0
            && self.pending_skip == 0
            && self.skip_turns[player.0] == 0
            && self.players[player.0]
                .hand
                .iter()
                .copied()
                .any(|card| self.can_play(player, card));
        // 若倒数第二张牌已经落桌，漏喊窗口仍持续到被检举或该玩家下次出牌；
        // 这段时间允许本人补喊，即使当前已经轮到其他玩家。
        let can_recover_after_play =
            self.players[player.0].hand.len() == 1 && self.uno_exposed[player.0];
        if !can_declare_before_play && !can_recover_after_play {
            return Err(GameError::CannotCallUno(player));
        }

        if can_recover_after_play {
            self.uno_exposed[player.0] = false;
        } else {
            self.uno_declared[player.0] = true;
        }
        if player == self.current_player {
            self.jump_in_open = false;
        }
        Ok(ActionOutcome::UnoCalled { player })
    }

    pub fn report_uno(
        &mut self,
        reporter: PlayerId,
        target: PlayerId,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(reporter)?;
        self.ensure_player(target)?;
        if !self.rules.uno_callout {
            return Err(GameError::UnoCalloutDisabled);
        }
        if reporter == target {
            return Err(GameError::CannotReportSelf);
        }
        if !self.uno_exposed[target.0] {
            return Err(GameError::PlayerNotReportable(target));
        }
        let cards = self.draw_cards_for(target, 2)?;
        if reporter == self.current_player {
            self.jump_in_open = false;
        }
        self.uno_exposed[target.0] = false;
        Ok(ActionOutcome::UnoReported {
            reporter,
            target,
            cards,
        })
    }

    pub fn resolve_skip(&mut self, player: PlayerId) -> Result<ActionOutcome, GameError> {
        self.ensure_turn(player)?;
        self.ensure_uno_followup_resolved(player)?;
        if self.pending_draw > 0 {
            return Err(GameError::MustResolveDrawPenalty);
        }
        let pending = self.pending_skip;
        let existing = self.skip_turns[player.0];
        if pending == 0 && existing == 0 {
            return Err(GameError::NoSkipToResolve);
        }
        let total = if self.rules.stack_skip {
            existing.saturating_add(pending)
        } else {
            existing.max(pending.min(1))
        };
        let cards = if self.rules.skip_draw_penalty {
            self.draw_cards_for(player, 1)?
        } else {
            Vec::new()
        };
        self.jump_in_open = false;
        self.pending_skip = 0;
        let remaining = total.saturating_sub(1);
        self.skip_turns[player.0] = remaining;
        self.drawn_card = None;
        let next_player = self.next_player(player);
        self.current_player = next_player;
        Ok(ActionOutcome::SkipResolved {
            player,
            cards,
            remaining,
            next_player,
        })
    }

    fn apply_starting_card(&mut self) -> Result<(), GameError> {
        let card = *self.discard_pile.last().expect("starting card exists");
        match card.face() {
            Face::DrawTwo => {
                let player = self.current_player;
                self.draw_cards_for(player, 2)?;
                self.current_player = self.next_player(player);
            }
            Face::Reverse => {
                self.direction = Direction::CounterClockwise;
                self.current_player = self.next_player(self.current_player);
            }
            Face::Skip => {
                if self.rules.stack_skip {
                    self.pending_skip = 1;
                } else {
                    self.skip_turns[self.current_player.0] = 1;
                }
            }
            Face::Wild => self.current_color = None,
            Face::Number(_) => {}
            Face::WildDrawFour => unreachable!("initial wild draw four was rotated away"),
        }
        Ok(())
    }

    fn ensure_playing(&self) -> Result<(), GameError> {
        if matches!(self.phase, Phase::Finished(_)) {
            Err(GameError::GameAlreadyFinished)
        } else {
            Ok(())
        }
    }

    fn ensure_player(&self, player: PlayerId) -> Result<(), GameError> {
        if player.0 < self.players.len() {
            Ok(())
        } else {
            Err(GameError::InvalidPlayer(player))
        }
    }

    fn ensure_turn(&self, player: PlayerId) -> Result<(), GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if player == self.current_player {
            Ok(())
        } else {
            Err(GameError::NotPlayersTurn {
                expected: self.current_player,
                actual: player,
            })
        }
    }

    fn ensure_skip_resolved(&self, player: PlayerId) -> Result<(), GameError> {
        if self.pending_skip > 0 || self.skip_turns[player.0] > 0 {
            Err(GameError::MustResolveSkip)
        } else {
            Ok(())
        }
    }

    fn ensure_uno_followup_resolved(&self, player: PlayerId) -> Result<(), GameError> {
        if self.uno_declared[player.0] {
            Err(GameError::MustPlayAfterUno)
        } else {
            Ok(())
        }
    }

    fn matches_top(&self, card: Card) -> bool {
        card.face().is_wild()
            || card.color() == self.current_color
            || self
                .discard_pile
                .last()
                .is_some_and(|top| top.face() == card.face())
    }

    fn stack_allowed(&self, card: Card) -> bool {
        match (self.pending_kind, card.face()) {
            (Some(PendingDrawKind::DrawTwo), Face::DrawTwo)
            | (Some(PendingDrawKind::WildDrawFour), Face::WildDrawFour) => true,
            (Some(PendingDrawKind::DrawTwo), Face::WildDrawFour) => {
                self.rules.stack_draw_four_on_draw_two
            }
            _ => false,
        }
    }

    fn card_allowed_in_current_state(&self, card: Card) -> bool {
        if self.skip_turns[self.current_player.0] > 0 {
            false
        } else if self.pending_skip > 0 {
            self.rules.stack_skip && card.face() == Face::Skip
        } else if self.pending_draw > 0 {
            self.stack_allowed(card)
        } else {
            self.matches_top(card)
        }
    }

    fn next_player(&self, player: PlayerId) -> PlayerId {
        let count = self.players.len();
        match self.direction {
            Direction::Clockwise => PlayerId((player.0 + 1) % count),
            Direction::CounterClockwise => PlayerId((player.0 + count - 1) % count),
        }
    }

    fn draw_cards_for(&mut self, player: PlayerId, count: u16) -> Result<Vec<Card>, GameError> {
        self.ensure_player(player)?;
        let mut cards = Vec::with_capacity(usize::from(count));
        for _ in 0..count {
            self.replenish_draw_pile();
            let card = self
                .draw_pile
                .pop_front()
                .ok_or(GameError::DrawPileExhausted)?;
            self.players[player.0].hand.push(card);
            cards.push(card);
        }
        self.players[player.0].hand.sort_by(Card::display_cmp);
        Ok(cards)
    }

    fn replenish_draw_pile(&mut self) {
        if !self.draw_pile.is_empty() || self.discard_pile.len() <= 1 {
            return;
        }
        let top = self.discard_pile.pop().expect("checked above");
        let mut recycled = self.discard_pile.drain(..).collect::<Vec<_>>();
        recycled.reverse();
        self.draw_pile = recycled.into();
        self.discard_pile.push(top);
    }

    fn clear_pending_draw(&mut self) {
        self.pending_draw = 0;
        self.pending_kind = None;
        self.challenge = None;
    }

    fn finish(&mut self, winner: PlayerId) -> GameResult {
        self.clear_pending_draw();
        self.pending_skip = 0;
        self.skip_turns.fill(0);
        self.drawn_card = None;
        self.uno_exposed.fill(false);
        self.uno_declared.fill(false);
        self.jump_in_open = false;
        let hand_scores = self
            .players
            .iter()
            .map(PlayerState::hand_score)
            .collect::<Vec<_>>();
        let result = GameResult {
            winner,
            placements: placements(winner, &hand_scores),
            reference_deltas: reference_point_deltas(winner, &hand_scores)
                .expect("validated player count"),
            hand_scores,
        };
        self.phase = Phase::Finished(result.clone());
        result
    }
}

fn remove_card(hand: &mut Vec<Card>, card: Card) {
    let index = hand
        .iter()
        .position(|candidate| *candidate == card)
        .expect("caller checked that the card is in hand");
    hand.remove(index);
}

fn validate_deck(deck: &[Card]) -> Result<(), GameError> {
    let expected = build_deck();
    if deck.len() != expected.len() {
        return Err(GameError::InvalidDeckSize {
            expected: expected.len(),
            actual: deck.len(),
        });
    }
    let expected = expected.into_iter().collect::<HashSet<_>>();
    let actual = deck.iter().copied().collect::<HashSet<_>>();
    if actual.len() != deck.len() || actual != expected {
        return Err(GameError::InvalidDeckContents);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn deck_with_prefix(prefix: &[Card]) -> Vec<Card> {
        let mut deck = build_deck();
        for card in prefix.iter().rev() {
            let index = deck.iter().position(|candidate| candidate == card).unwrap();
            deck.remove(index);
            deck.insert(0, *card);
        }
        deck
    }

    fn card(color: Color, face: Face, copy: u8) -> Card {
        match face {
            Face::Number(value) => Card::number(color, value, copy),
            Face::DrawTwo | Face::Reverse | Face::Skip => Card::action(color, face, copy),
            Face::Wild | Face::WildDrawFour => Card::wild(face, copy),
        }
    }

    #[test]
    fn deals_seven_cards_and_applies_number_start() {
        let game = GameState::new_with_deck(RuleSet::default(), 6, build_deck()).unwrap();
        assert!(game.players().iter().all(|player| player.hand().len() == 7));
        assert_eq!(game.players().len(), 6);
        assert_eq!(game.draw_pile_len(), 65);
        assert_eq!(game.turn().unwrap().current_player, PlayerId(0));
    }

    #[test]
    fn two_to_six_players_are_supported_but_seven_are_rejected() {
        for player_count in RuleSet::MIN_PLAYERS..=RuleSet::MAX_PLAYERS {
            let game =
                GameState::new_with_deck(RuleSet::default(), player_count, build_deck()).unwrap();
            assert_eq!(game.players().len(), usize::from(player_count));
            assert!(game.players().iter().all(|player| player.hand().len() == 7));
        }
        assert!(matches!(
            GameState::new_with_deck(RuleSet::default(), 7, build_deck()),
            Err(GameError::InvalidRules(_))
        ));
    }

    #[test]
    fn draw_two_stacks_and_draw_four_on_two_is_configurable() {
        let p0_draw_two = card(Color::Red, Face::DrawTwo, 0);
        let p1_draw_two = card(Color::Yellow, Face::DrawTwo, 0);
        let p2_draw_four = Card::wild(Face::WildDrawFour, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        // Round-robin dealing positions are 0..=5; put the required cards explicitly.
        let mut deck = build_deck();
        for (position, required) in [
            (0, p0_draw_two),
            (1, p1_draw_two),
            (2, p2_draw_four),
            (42, start),
        ] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck.clone()).unwrap();
        game.play_card(PlayerId(0), p0_draw_two, None).unwrap();
        game.play_card(PlayerId(1), p1_draw_two, None).unwrap();
        assert_eq!(game.turn().unwrap().pending_draw, 4);
        assert_eq!(
            game.play_card(PlayerId(2), p2_draw_four, Some(Color::Blue)),
            Err(GameError::CannotStack(p2_draw_four))
        );

        let mut game = GameState::new_with_deck(
            RuleSet {
                stack_draw_four_on_draw_two: true,
                ..RuleSet::default()
            },
            6,
            deck,
        )
        .unwrap();
        game.play_card(PlayerId(0), p0_draw_two, None).unwrap();
        game.play_card(PlayerId(1), p1_draw_two, None).unwrap();
        game.play_card(PlayerId(2), p2_draw_four, Some(Color::Blue))
            .unwrap();
        assert_eq!(game.turn().unwrap().pending_draw, 8);
    }

    #[test]
    fn successful_challenge_returns_whole_stack_to_first_offender() {
        let draw_four = Card::wild(Face::WildDrawFour, 0);
        let matching = card(Color::Red, Face::Number(7), 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, draw_four), (6, matching), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.play_card(PlayerId(0), draw_four, Some(Color::Blue))
            .unwrap();
        let before = game.player(PlayerId(0)).unwrap().hand().len();
        let outcome = game.challenge_draw_four(PlayerId(1)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::ChallengeResolved {
                result: ChallengeResult::Successful,
                penalized: PlayerId(0),
                ref cards,
                next_player: PlayerId(1),
                ..
            } if cards.len() == 4
        ));
        assert_eq!(game.player(PlayerId(0)).unwrap().hand().len(), before + 4);
    }

    #[test]
    fn failed_challenge_adds_two_to_the_whole_stack() {
        let draw_four = Card::wild(Face::WildDrawFour, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, draw_four), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        // Ensure player zero has no red card besides the wild.
        for position in [6, 12, 18, 24, 30, 36] {
            if deck[position].color() == Some(Color::Red) {
                let replacement = (43..deck.len())
                    .find(|index| deck[*index].color() != Some(Color::Red))
                    .unwrap();
                deck.swap(position, replacement);
            }
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.play_card(PlayerId(0), draw_four, Some(Color::Blue))
            .unwrap();
        let before = game.player(PlayerId(1)).unwrap().hand().len();
        let outcome = game.challenge_draw_four(PlayerId(1)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::ChallengeResolved {
                result: ChallengeResult::Failed,
                penalized: PlayerId(1),
                ref cards,
                next_player: PlayerId(2),
                ..
            } if cards.len() == 6
        ));
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), before + 6);
    }

    #[test]
    fn uno_may_be_called_before_or_immediately_after_the_penultimate_card() {
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, build_deck()).unwrap();
        game.players[0].hand.truncate(2);
        let playable = game.players[0]
            .hand
            .iter()
            .copied()
            .find(|card| game.matches_top(*card))
            .unwrap_or_else(|| {
                let card = *game.discard_pile.last().unwrap();
                game.players[0].hand[0] = card;
                card
            });
        game.call_uno(PlayerId(0)).unwrap();
        assert_eq!(
            game.uno_declared_players().collect::<Vec<_>>(),
            vec![PlayerId(0)]
        );
        assert_eq!(
            game.draw_card(PlayerId(0)),
            Err(GameError::MustPlayAfterUno)
        );
        game.play_card(PlayerId(0), playable, None).unwrap();
        assert!(game.uno_exposed_players().next().is_none());

        game.current_player = PlayerId(1);
        game.players[1].hand.truncate(2);
        let playable = game.players[1]
            .hand
            .iter()
            .copied()
            .find(|card| game.matches_top(*card))
            .unwrap_or_else(|| {
                let card = *game.discard_pile.last().unwrap();
                game.players[1].hand[0] = card;
                card
            });
        game.play_card(PlayerId(1), playable, None).unwrap();
        assert_eq!(
            game.uno_exposed_players().collect::<Vec<_>>(),
            vec![PlayerId(1)]
        );
        assert!(matches!(
            game.call_uno(PlayerId(1)),
            Ok(ActionOutcome::UnoCalled {
                player: PlayerId(1)
            })
        ));
        assert!(game.uno_exposed_players().next().is_none());
        assert!(game.uno_declared_players().next().is_none());

        // 补喊后不能被重复检举，也不会留下“必须立刻出牌”的预喊状态。
        let before = game.player(PlayerId(1)).unwrap().hand().len();
        assert_eq!(
            game.report_uno(PlayerId(2), PlayerId(1)),
            Err(GameError::PlayerNotReportable(PlayerId(1)))
        );
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), before);
    }

    #[test]
    fn last_draw_card_finishes_without_challenge_or_penalty_resolution() {
        let wild_draw_four = Card::wild(Face::WildDrawFour, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, wild_draw_four), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.players[0].hand = vec![wild_draw_four];
        let outcome = game
            .play_card(PlayerId(0), wild_draw_four, Some(Color::Blue))
            .unwrap();
        assert!(matches!(outcome, ActionOutcome::GameFinished(_)));
        assert!(game.turn().is_none());
        assert_eq!(
            game.challenge_draw_four(PlayerId(1)),
            Err(GameError::GameAlreadyFinished)
        );
    }

    #[test]
    fn playable_drawn_card_may_be_played_or_passed_but_no_other_card_can() {
        let drawn = card(Color::Red, Face::Number(9), 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = deck_with_prefix(&[]);
        for (position, required) in [(42, start), (43, drawn)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        let other = game.players[0].hand[0];
        let outcome = game.draw_card(PlayerId(0)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::DrewCard { playable: true, .. }
        ));
        assert_eq!(
            game.play_card(
                PlayerId(0),
                other,
                other.face().is_wild().then_some(Color::Blue)
            ),
            Err(GameError::MustPlayDrawnCard(drawn))
        );
        game.pass_after_draw(PlayerId(0)).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
    }

    #[test]
    fn uno_may_be_called_after_drawing_a_playable_card_from_one_to_two() {
        let drawn = card(Color::Red, Face::Number(9), 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = deck_with_prefix(&[]);
        for (position, required) in [(42, start), (43, drawn)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.players[0].hand.truncate(1);

        assert!(matches!(
            game.draw_card(PlayerId(0)),
            Ok(ActionOutcome::DrewCard {
                card,
                playable: true,
                ..
            }) if card == drawn
        ));
        assert!(matches!(
            game.call_uno(PlayerId(0)),
            Ok(ActionOutcome::UnoCalled {
                player: PlayerId(0)
            })
        ));
        game.play_card(PlayerId(0), drawn, None).unwrap();
        assert_eq!(game.player(PlayerId(0)).unwrap().hand().len(), 1);
        assert!(game.uno_exposed_players().next().is_none());
        assert!(game.uno_declared_players().next().is_none());
    }

    #[test]
    fn stacked_draw_fours_keep_the_first_player_responsible() {
        let first = Card::wild(Face::WildDrawFour, 0);
        let second = Card::wild(Face::WildDrawFour, 1);
        let matching = card(Color::Red, Face::Number(7), 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, first), (1, second), (6, matching), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.play_card(PlayerId(0), first, Some(Color::Blue))
            .unwrap();
        game.play_card(PlayerId(1), second, Some(Color::Green))
            .unwrap();
        let outcome = game.challenge_draw_four(PlayerId(2)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::ChallengeResolved {
                offender: PlayerId(0),
                result: ChallengeResult::Successful,
                penalized: PlayerId(0),
                ref cards,
                ..
            } if cards.len() == 8
        ));
    }

    #[test]
    fn skip_stacking_transfers_multiple_blocked_turns() {
        let first = card(Color::Red, Face::Skip, 0);
        let second = card(Color::Yellow, Face::Skip, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, first), (1, second), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(
            RuleSet {
                stack_skip: true,
                skip_draw_penalty: true,
                ..RuleSet::default()
            },
            6,
            deck,
        )
        .unwrap();
        game.play_card(PlayerId(0), first, None).unwrap();
        assert_eq!(game.turn().unwrap().pending_skip, 1);
        game.play_card(PlayerId(1), second, None).unwrap();
        assert_eq!(game.turn().unwrap().pending_skip, 2);
        let before = game.player(PlayerId(2)).unwrap().hand().len();
        let outcome = game.resolve_skip(PlayerId(2)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::SkipResolved {
                player: PlayerId(2),
                remaining: 1,
                ref cards,
                next_player: PlayerId(3),
            } if cards.len() == 1
        ));
        assert_eq!(game.skipped_turns(PlayerId(2)), Some(1));
        assert_eq!(game.player(PlayerId(2)).unwrap().hand().len(), before + 1);

        game.current_player = PlayerId(2);
        game.resolve_skip(PlayerId(2)).unwrap();
        assert_eq!(game.skipped_turns(PlayerId(2)), Some(0));
        assert_eq!(game.player(PlayerId(2)).unwrap().hand().len(), before + 2);
    }

    #[test]
    fn ordinary_skip_blocks_exactly_one_turn() {
        let skip = card(Color::Red, Face::Skip, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, skip), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.play_card(PlayerId(0), skip, None).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(1));
        assert_eq!(game.draw_card(PlayerId(1)), Err(GameError::MustResolveSkip));
        game.resolve_skip(PlayerId(1)).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(2));
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(0));
    }

    #[test]
    fn draw_two_is_paid_by_the_skipped_direct_next_player_before_skip_resolves() {
        let draw_two = card(Color::Red, Face::DrawTwo, 0);
        let start = card(Color::Red, Face::Number(5), 0);
        let mut deck = build_deck();
        for (position, required) in [(0, draw_two), (42, start)] {
            let current = deck
                .iter()
                .position(|candidate| *candidate == required)
                .unwrap();
            deck.swap(position, current);
        }
        let mut game = GameState::new_with_deck(RuleSet::default(), 6, deck).unwrap();
        game.skip_turns[1] = 1;
        let before = game.player(PlayerId(1)).unwrap().hand().len();

        game.play_card(PlayerId(0), draw_two, None).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
        assert_eq!(
            game.resolve_skip(PlayerId(1)),
            Err(GameError::MustResolveDrawPenalty)
        );

        let outcome = game.accept_draw_penalty(PlayerId(1)).unwrap();
        assert!(matches!(
            outcome,
            ActionOutcome::PenaltyDrawn {
                player: PlayerId(1),
                ref cards,
                next_player: PlayerId(1),
            } if cards.len() == 2
        ));
        assert_eq!(game.player(PlayerId(1)).unwrap().hand().len(), before + 2);
        assert_eq!(game.turn().unwrap().current_player, PlayerId(1));
        assert_eq!(game.turn().unwrap().pending_draw, 0);
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(1));

        game.resolve_skip(PlayerId(1)).unwrap();
        assert_eq!(game.turn().unwrap().current_player, PlayerId(2));
        assert_eq!(game.skipped_turns(PlayerId(1)), Some(0));
    }

    fn jump_in_game() -> GameState {
        let mut game = GameState::new_with_deck(
            RuleSet {
                stack_skip: true,
                jump_in: true,
                ..RuleSet::default()
            },
            3,
            build_deck(),
        )
        .unwrap();
        game.direction = Direction::Clockwise;
        game
    }

    #[test]
    fn non_next_player_can_jump_in_and_then_recover_uno_at_one_card() {
        let top = card(Color::Red, Face::Number(7), 0);
        let matching = card(Color::Red, Face::Number(7), 1);
        let filler = card(Color::Blue, Face::Number(3), 0);
        let mut game = jump_in_game();
        game.discard_pile = vec![top];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(1);
        game.jump_in_open = true;
        game.players[2].hand = vec![matching, filler];

        assert_eq!(game.jump_in_card(PlayerId(1)), None);
        assert_eq!(game.jump_in_card(PlayerId(2)), Some(matching));
        assert!(matches!(
            game.jump_in(PlayerId(2), matching),
            Ok(ActionOutcome::Played {
                player: PlayerId(2),
                next_player: PlayerId(0),
                ..
            })
        ));
        assert_eq!(game.player(PlayerId(2)).unwrap().hand(), &[filler]);
        assert_eq!(
            game.uno_exposed_players().collect::<Vec<_>>(),
            vec![PlayerId(2)]
        );
        assert!(matches!(
            game.call_uno(PlayerId(2)),
            Ok(ActionOutcome::UnoCalled {
                player: PlayerId(2)
            })
        ));
    }

    #[test]
    fn current_players_first_successful_action_closes_the_jump_window() {
        let top = card(Color::Red, Face::Number(7), 0);
        let matching = card(Color::Red, Face::Number(7), 1);
        let mut game = jump_in_game();
        game.discard_pile = vec![top];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(1);
        game.jump_in_open = true;
        game.players[2].hand = vec![matching, card(Color::Blue, Face::Number(3), 0)];

        assert_eq!(game.jump_in_card(PlayerId(2)), Some(matching));
        game.draw_card(PlayerId(1)).unwrap();
        assert_eq!(game.jump_in_card(PlayerId(2)), None);
        assert_eq!(
            game.jump_in(PlayerId(2), matching),
            Err(GameError::CannotJumpIn)
        );
    }

    #[test]
    fn jumped_draw_two_and_skip_extend_the_existing_stack() {
        let filler = card(Color::Blue, Face::Number(3), 0);

        let draw_top = card(Color::Red, Face::DrawTwo, 0);
        let draw_match = card(Color::Red, Face::DrawTwo, 1);
        let mut draw_game = jump_in_game();
        draw_game.discard_pile = vec![draw_top];
        draw_game.current_color = Some(Color::Red);
        draw_game.current_player = PlayerId(1);
        draw_game.pending_draw = 2;
        draw_game.pending_kind = Some(PendingDrawKind::DrawTwo);
        draw_game.jump_in_open = true;
        draw_game.players[2].hand = vec![draw_match, filler];
        draw_game.jump_in(PlayerId(2), draw_match).unwrap();
        assert_eq!(draw_game.turn().unwrap().pending_draw, 4);
        assert_eq!(draw_game.turn().unwrap().current_player, PlayerId(0));

        let skip_top = card(Color::Yellow, Face::Skip, 0);
        let skip_match = card(Color::Yellow, Face::Skip, 1);
        let mut skip_game = jump_in_game();
        skip_game.discard_pile = vec![skip_top];
        skip_game.current_color = Some(Color::Yellow);
        skip_game.current_player = PlayerId(1);
        skip_game.pending_skip = 1;
        skip_game.jump_in_open = true;
        skip_game.players[2].hand = vec![skip_match, filler];
        skip_game.jump_in(PlayerId(2), skip_match).unwrap();
        assert_eq!(skip_game.turn().unwrap().pending_skip, 2);
        assert_eq!(skip_game.turn().unwrap().current_player, PlayerId(0));
    }

    #[test]
    fn identical_pair_is_atomic_and_two_reverses_restore_direction() {
        let first = card(Color::Red, Face::Reverse, 0);
        let second = card(Color::Red, Face::Reverse, 1);
        let filler = card(Color::Blue, Face::Number(3), 0);
        let mut game = jump_in_game();
        game.discard_pile = vec![card(Color::Red, Face::Number(5), 0)];
        game.current_color = Some(Color::Red);
        game.current_player = PlayerId(0);
        game.players[0].hand = vec![first, second, filler];

        assert!(matches!(
            game.play_cards(PlayerId(0), &[first, second], None),
            Ok(ActionOutcome::Played {
                player: PlayerId(0),
                next_player: PlayerId(1),
                ..
            })
        ));
        assert_eq!(game.direction(), Direction::Clockwise);
        assert_eq!(game.player(PlayerId(0)).unwrap().hand(), &[filler]);
        assert_eq!(
            &game.discard_pile()[game.discard_pile().len() - 2..],
            &[first, second]
        );
    }

    #[test]
    fn identical_last_pair_finishes_together_but_declared_uno_forces_one_card() {
        let first = card(Color::Green, Face::Number(8), 0);
        let second = card(Color::Green, Face::Number(8), 1);
        let mut game = jump_in_game();
        game.discard_pile = vec![card(Color::Green, Face::Number(4), 0)];
        game.current_color = Some(Color::Green);
        game.current_player = PlayerId(0);
        game.players[0].hand = vec![first, second];
        assert!(matches!(
            game.play_cards(PlayerId(0), &[first, second], None),
            Ok(ActionOutcome::GameFinished(GameResult {
                winner: PlayerId(0),
                ..
            }))
        ));

        let mut declared = jump_in_game();
        declared.discard_pile = vec![card(Color::Green, Face::Number(4), 0)];
        declared.current_color = Some(Color::Green);
        declared.current_player = PlayerId(0);
        declared.players[0].hand = vec![first, second];
        declared.uno_declared[0] = true;
        assert_eq!(
            declared.play_cards(PlayerId(0), &[first, second], None),
            Err(GameError::CannotPlayTogether)
        );
        assert_eq!(
            declared.player(PlayerId(0)).unwrap().hand(),
            &[first, second]
        );
    }
}
