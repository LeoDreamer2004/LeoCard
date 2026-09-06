use super::*;

impl QiGui523Session {
    pub(super) fn initialize_turn_timer(&mut self) {
        if self.rules.time_control.is_unlimited() {
            self.turn_timer = None;
            return;
        }
        let Some(game) = self.game.as_ref() else {
            self.turn_timer = None;
            return;
        };
        let Some(current) = game
            .trick()
            .map(|trick| from_core_player(trick.current_player()))
        else {
            self.turn_timer = None;
            return;
        };
        let control = self.rules.time_control;
        self.turn_timer = Some(TurnTimerState {
            player: current,
            base_remaining: Duration::from_secs(u64::from(control.base_seconds())),
            reserve_remaining: vec![
                Duration::from_secs(u64::from(control.reserve_seconds()));
                game.players().len()
            ],
        });
    }

    pub(super) fn reset_timer_for_current_turn(&mut self) {
        self.reset_auto_play_delay_for_current_turn();
        if self.rules.time_control.is_unlimited() {
            self.turn_timer = None;
            return;
        }
        let Some(current) = self.game.as_ref().and_then(|game| {
            matches!(game.phase(), Phase::Playing)
                .then(|| {
                    game.trick()
                        .map(|trick| from_core_player(trick.current_player()))
                })
                .flatten()
        }) else {
            self.turn_timer = None;
            return;
        };
        let base = Duration::from_secs(u64::from(self.rules.time_control.base_seconds()));
        match self.turn_timer.as_mut() {
            Some(timer) => {
                timer.player = current;
                timer.base_remaining = base;
            }
            None => self.initialize_turn_timer(),
        }
    }

    pub(super) fn turn_timer_view(&self) -> Option<TurnTimerView> {
        let timer = self.turn_timer.as_ref()?;
        Some(TurnTimerView {
            player: timer.player,
            base_seconds: duration_ceil_seconds(timer.base_remaining),
            reserve_seconds: duration_ceil_seconds(
                timer.reserve_remaining[usize::from(timer.player.0)],
            ),
        })
    }

    pub(super) fn current_player(&self) -> Option<PlayerId> {
        self.game.as_ref().and_then(|game| {
            matches!(game.phase(), Phase::Playing)
                .then(|| {
                    game.trick()
                        .map(|trick| from_core_player(trick.current_player()))
                })
                .flatten()
        })
    }

    pub(super) fn current_player_is_disconnected(&self) -> bool {
        let Some(current) = self.current_player() else {
            return false;
        };
        self.players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| !player.connected && !player.is_bot)
    }

    pub(super) fn current_auto_play_player(&self) -> Option<PlayerId> {
        let current = self.current_player()?;
        self.players
            .iter()
            .find(|player| player.id == current)
            .is_some_and(|player| (player.connected || player.is_bot) && player.auto_play)
            .then_some(current)
    }

    pub(super) fn reset_auto_play_delay_for_current_turn(&mut self) {
        self.auto_play_delay = self
            .current_auto_play_player()
            .map(|player| AutoPlayDelayState {
                player,
                remaining: AUTO_PLAY_DELAY,
            });
    }

    pub(super) fn play_automatic_action(&mut self) -> Option<(PlayerId, PublicPlay)> {
        let (player, cards) = {
            let game = self.game.as_ref().expect("a running timer has a game");
            let trick = game.trick().expect("a playing game has a trick");
            let player = trick.current_player();
            let cards = if let Some(current_play) = trick.winning_play() {
                let played_cards = trick
                    .records()
                    .iter()
                    .flat_map(|record| match record {
                        PlayRecord::Played { play, .. } => play.cards(),
                        PlayRecord::Passed { .. } => &[],
                    })
                    .copied()
                    .collect::<Vec<_>>();
                QiGui523Bot::new()
                    .choose(QiGui523BotRequest {
                        hand: game.player(player).expect("current player exists").hand(),
                        current_play,
                        played_cards: &played_cards,
                        rules: game.rules(),
                    })
                    .map(|play| play.cards().to_vec())
            } else {
                game.player(player)
                    .expect("current player exists")
                    .hand()
                    .iter()
                    .copied()
                    .min_by_key(|card| {
                        (card.rank().strength(), card.suit().strength(), card.deck())
                    })
                    .map(|card| vec![card])
            };
            (player, cards)
        };

        let game = self.game.as_mut().expect("a running timer has a game");
        if let Some(cards) = cards {
            let play = classify(&cards, game.rules())
                .expect("the timeout strategy only returns classifiable cards");
            let effect = PublicPlay {
                kind: play.kind().clone(),
                cards: cards.clone(),
            };
            game.play_cards(player, &cards)
                .expect("the timeout strategy only returns legal cards");
            Some((from_core_player(player), effect))
        } else {
            game.pass(player)
                .expect("a player without a legal response may pass");
            None
        }
    }
}
