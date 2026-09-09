use super::{
    ActionOutcome, GameError, GameState, PendingSwap, PendingSwapState, Phase, PlayerState,
    TurnState, UnoCard, UnoColor, UnoDirection, UnoFace, UnoFlipSide, UnoPlayerId, UnoRuleSet,
    VecDeque, validate_deck,
};

impl GameState {
    /// `deck[0]` 是第一张发出的牌。
    pub fn new_with_deck(
        rules: UnoRuleSet,
        player_count: u8,
        deck: Vec<UnoCard>,
    ) -> Result<Self, GameError> {
        let rules = rules.validate()?;
        let player_count = UnoRuleSet::validate_player_count(player_count)?;
        validate_deck(&deck, rules)?;
        let mut draw_pile = VecDeque::from(deck);
        let mut players = (0..player_count)
            .map(|index| PlayerState {
                id: UnoPlayerId(index),
                hand: Vec::with_capacity(usize::from(UnoRuleSet::HAND_SIZE)),
                eliminated: false,
            })
            .collect::<Vec<_>>();
        for _ in 0..UnoRuleSet::HAND_SIZE {
            for player in &mut players {
                player.hand.push(
                    draw_pile
                        .pop_front()
                        .expect("a valid UNO deck is large enough"),
                );
            }
        }
        for player in &mut players {
            player.hand.sort_by(UnoCard::display_cmp);
        }

        // 经典模式轮换万能摸四，FLIP 轮换万能摸二；No Mercy 不执行起始功能牌。
        let top_card = loop {
            let card = draw_pile
                .pop_front()
                .expect("a valid UNO deck has a starting card");
            if (rules.is_classic() && card.face() == UnoFace::WildDrawFour)
                || (rules.is_flip() && card.face() == UnoFace::WildDrawTwo)
                || (rules.is_no_mercy() && !matches!(card.face(), UnoFace::Number(_)))
            {
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
            current_player: UnoPlayerId(0),
            direction: UnoDirection::Clockwise,
            current_color: top_card.color(),
            pending_draw: 0,
            pending_kind: None,
            pending_draw_source: None,
            pending_draw_colors: Vec::new(),
            challenge: None,
            drawn_card: None,
            pending_skip: 0,
            pending_skip_everyone: false,
            pending_skip_source: None,
            skip_turns: vec![0; player_count],
            uno_exposed: vec![false; player_count],
            uno_declared: vec![false; player_count],
            jump_in_open: false,
            pending_swap: None,
            pending_finisher: None,
            set_aside_cards: Vec::new(),
            elimination_order: Vec::new(),
            flip_side: rules.is_flip().then_some(UnoFlipSide::Light),
            phase: Phase::Playing,
        };
        if rules.is_classic() || rules.is_flip() {
            state.apply_starting_card()?;
        }
        Ok(state)
    }

    pub const fn rules(&self) -> &UnoRuleSet {
        &self.rules
    }

    pub fn players(&self) -> &[PlayerState] {
        &self.players
    }

    pub fn player(&self, player: UnoPlayerId) -> Option<&PlayerState> {
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
            pending_draw_source: self.pending_draw_source,
            challenge_offender: self.challenge.map(|challenge| challenge.offender),
            drawn_card: self.drawn_card,
            pending_skip: self.pending_skip,
            skipped_turns_remaining: self.skip_turns[self.current_player.0],
            pending_swap: self.pending_swap.map(PendingSwapState::public),
        })
    }

    pub fn pending_swap(&self) -> Option<PendingSwap> {
        self.pending_swap.map(PendingSwapState::public)
    }

    pub fn discard_pile(&self) -> &[UnoCard] {
        &self.discard_pile
    }

    pub fn draw_pile_len(&self) -> usize {
        self.draw_pile.len()
    }

    pub fn top_card(&self) -> UnoCard {
        *self.discard_pile.last().expect("a game has a discard")
    }

    pub const fn current_color(&self) -> Option<UnoColor> {
        self.current_color
    }

    pub const fn direction(&self) -> UnoDirection {
        self.direction
    }

    pub const fn flip_side(&self) -> Option<UnoFlipSide> {
        self.flip_side
    }

    pub fn draw_pile(&self) -> impl Iterator<Item = UnoCard> + '_ {
        self.draw_pile.iter().copied()
    }

    pub fn skipped_turns(&self, player: UnoPlayerId) -> Option<u16> {
        self.skip_turns.get(player.0).copied()
    }

    pub fn uno_exposed_players(&self) -> impl Iterator<Item = UnoPlayerId> + '_ {
        self.uno_exposed
            .iter()
            .enumerate()
            .filter_map(|(index, exposed)| exposed.then_some(UnoPlayerId(index)))
    }

    pub fn uno_declared_players(&self) -> impl Iterator<Item = UnoPlayerId> + '_ {
        self.uno_declared
            .iter()
            .enumerate()
            .filter_map(|(index, declared)| declared.then_some(UnoPlayerId(index)))
    }

    /// 返回该玩家当前唯一可抢出的物理牌。经典 108 张牌中每种彩色牌面至多
    /// 两张，桌面已经有一张，因此至多只会找到一张候选牌。
    pub fn jump_in_card(&self, player: UnoPlayerId) -> Option<UnoCard> {
        if !matches!(self.phase, Phase::Playing)
            || !self.jump_in_enabled()
            || !self.jump_in_open
            || self.pending_swap.is_some()
            || player == self.current_player
            || self.skip_turns.get(player.0).copied().unwrap_or(0) > 0
        {
            return None;
        }
        let top = self.top_card();
        top.color()?;
        if top.face().is_extension()
            || (!self.action_stacking_enabled() && !matches!(top.face(), UnoFace::Number(_)))
        {
            return None;
        }
        self.players
            .get(player.0)?
            .hand
            .iter()
            .copied()
            .find(|card| *card != top && card.color() == top.color() && card.face() == top.face())
    }

    pub fn choose_initial_color(
        &mut self,
        player: UnoPlayerId,
        color: UnoColor,
    ) -> Result<ActionOutcome, GameError> {
        self.ensure_playing()?;
        self.ensure_player(player)?;
        if !self.color_allowed(color) {
            return Err(GameError::UnexpectedColor);
        }
        if let Some(PendingSwapState::ChooseColor { player: expected }) = self.pending_swap {
            if player != expected {
                return Err(GameError::NotPlayersTurn {
                    expected,
                    actual: player,
                });
            }
            self.current_color = Some(color);
            self.pending_swap = None;
            self.current_player = self.next_player(player);
            self.finish_pending_game();
            return Ok(ActionOutcome::ColorChosen { player, color });
        }
        if let Some(PendingSwapState::ColorRoulette { player: expected }) = self.pending_swap {
            if player != expected {
                return Err(GameError::NotPlayersTurn {
                    expected,
                    actual: player,
                });
            }
            let cards = self.draw_until_color(player, color)?;
            self.current_color = Some(color);
            self.pending_swap = None;
            self.check_mercy_elimination(player);
            let next_player = self.next_player(player);
            self.current_player = next_player;
            self.finish_pending_game();
            return Ok(ActionOutcome::ColorRouletteResolved {
                player,
                color,
                cards,
                next_player,
            });
        }
        if self.pending_swap.is_some() {
            return Err(GameError::MustResolveSwapEffect);
        }
        if self.current_color.is_some()
            || !self
                .discard_pile
                .last()
                .is_some_and(|card| card.face().is_wild())
        {
            return Err(GameError::InitialColorAlreadyChosen);
        }
        self.ensure_turn(player)?;
        self.current_color = Some(color);
        Ok(ActionOutcome::ColorChosen { player, color })
    }
}
