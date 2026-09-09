use super::{AUTOMATIC_ACTION_DELAY, ShengjiSession, card_sort_key, to_core_player};
use leocard_protocol::ShengjiEvent;
use leocard_shengji::bid_joker_for_suit;
use leocard_shengji::{
    ActionOutcome, FiveTrumpCrossingStage, Phase, ShengjiCard, ShengjiGreedyBot,
    ShengjiGreedyBotRequest, ShengjiPlayerId,
};

impl ShengjiSession {
    pub(super) fn bot_open_cards(
        &self,
        player: ShengjiPlayerId,
        dealt_card: ShengjiCard,
    ) -> Option<Vec<ShengjiCard>> {
        let participant = self.room.players.get(usize::from(player.0))?;
        let game = self.game.as_ref()?;
        if !participant.is_bot || game.bidding().current().is_some() {
            return None;
        }
        let level = game.bidding().level();
        if !game.rules().bid_with_joker {
            return (dealt_card.rank() == level).then(|| vec![dealt_card]);
        }
        let hand = &game.players().get(usize::from(player.0))?.hand;
        hand.iter().copied().find_map(|level_card| {
            let suit = (level_card.rank() == level)
                .then(|| level_card.suit())
                .flatten()?;
            let joker_rank = bid_joker_for_suit(suit);
            let joker = hand
                .iter()
                .copied()
                .find(|card| card.rank() == joker_rank && card.suit().is_none())?;
            Some(vec![level_card, joker])
        })
    }

    pub(super) fn open_power_outage_bot_declaration(&mut self) -> Option<ShengjiEvent> {
        let bot_players = self
            .room
            .players
            .iter()
            .filter(|participant| participant.is_bot && !participant.left)
            .map(|participant| to_core_player(participant.id))
            .collect::<Vec<_>>();
        for player in bot_players {
            let cards = {
                let game = self.game.as_ref()?;
                if game.bidding().current().is_some() {
                    return None;
                }
                let level = game.bidding().level();
                let hand = &game.players().get(usize::from(player.0))?.hand;
                if !game.rules().bid_with_joker {
                    hand.iter()
                        .copied()
                        .find(|card| card.rank() == level)
                        .map(|card| vec![card])
                } else {
                    hand.iter().copied().find_map(|level_card| {
                        let suit = (level_card.rank() == level)
                            .then(|| level_card.suit())
                            .flatten()?;
                        let joker_rank = bid_joker_for_suit(suit);
                        let joker = hand
                            .iter()
                            .copied()
                            .find(|card| card.rank() == joker_rank && card.suit().is_none())?;
                        Some(vec![level_card, joker])
                    })
                }
            };
            if let Some(cards) = cards
                && self.game.as_mut()?.declare(player, &cards).is_ok()
            {
                self.record_current_declaration();
                return Some(ShengjiEvent::DeclarationChanged {
                    declaration: self.declaration_view()?,
                });
            }
        }
        None
    }

    pub(super) fn automatic_player(&self) -> Option<ShengjiPlayerId> {
        if self.presentation.trick.is_some() || self.presentation.throw_failure.is_some() {
            return None;
        }
        let game = self.game.as_ref()?;
        let player = match game.phase() {
            Phase::Burying => game.dealer()?,
            Phase::BottomCopying => game.bottom_copy()?.current()?,
            Phase::BottomCopyBurying => game.bottom_copy()?.bottom_holder()?,
            Phase::FiveTrumpCrossing => game
                .five_trump_crossing()?
                .pending_players()
                .into_iter()
                .find(|player| self.is_automatic_participant(*player))?,
            Phase::Playing => game.current_player()?,
            _ => return None,
        };
        self.is_automatic_participant(player).then_some(player)
    }

    fn is_automatic_participant(&self, player: ShengjiPlayerId) -> bool {
        self.room
            .players
            .get(usize::from(player.0))
            .is_some_and(|participant| {
                participant.is_bot
                    || participant.auto_play
                    || !participant.connected
                    || participant.left
            })
    }

    pub(super) fn reset_automatic_action(&mut self) {
        self.flow.automatic_action = self
            .automatic_player()
            .map(|player| (player, AUTOMATIC_ACTION_DELAY));
    }

    pub(super) fn play_automatic_action(
        &mut self,
        player: ShengjiPlayerId,
    ) -> Option<ActionOutcome> {
        let game = self.game.as_mut()?;
        let outcome = match game.phase() {
            Phase::Burying => {
                let trump = game.trump()?;
                let mut hand = game.players()[usize::from(player.0)].hand.clone();
                hand.sort_by_key(|card| card_sort_key(*card, trump));
                game.bury(player, &hand[..game.rules().kitty_size()]).ok()?
            }
            Phase::BottomCopying => game.choose_bottom_copy(player, None).ok()?,
            Phase::BottomCopyBurying => {
                let trump = game.trump()?;
                let mut hand = game.players()[usize::from(player.0)].hand.clone();
                hand.sort_by_key(|card| card_sort_key(*card, trump));
                game.bury(player, &hand[..game.rules().kitty_size()]).ok()?
            }
            Phase::FiveTrumpCrossing => match game.five_trump_crossing()?.stage() {
                FiveTrumpCrossingStage::Deciding => {
                    game.choose_five_trump_crossing(player, None).ok()?
                }
                FiveTrumpCrossingStage::Returning => {
                    let mut hand = game.players()[usize::from(player.0)].hand.clone();
                    let trump = game.trump()?;
                    hand.sort_by_key(|card| card_sort_key(*card, trump));
                    game.return_five_trump_crossing(player, &hand[..5]).ok()?
                }
            },
            Phase::Playing => {
                let trick = game.current_trick();
                let play = ShengjiGreedyBot::choose(ShengjiGreedyBotRequest {
                    hand: &game.players()[usize::from(player.0)].hand,
                    lead: trick.as_ref().map(|trick| &trick.plays[0].1),
                    trump: game.trump()?,
                })
                .ok()?;
                game.play_cards(player, &play.cards).ok()?
            }
            _ => return None,
        };
        Some(outcome)
    }
}
