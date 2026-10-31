//! Route screens to game-owned composition resources.

use super::blackjack::BlackjackScreen;
use super::doudizhu::DoudizhuScreen;
use super::mahjong::MahjongScreen;
use super::mc_guess::McGuessScreen;
use super::qigui523::QiGui523Screen;
use super::shengji::ShengjiScreen;
use super::texas::TexasScreen;
use super::uno::UnoScreen;
use super::{GameLobbyContext, GameScreen, GameTableContext};
use crate::app::runtime::ClientResource;
use crate::app::shell::UiState;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use leocard_client::ClientPhaseRef;
use leocard_protocol::{GameKind, GameSnapshot, LobbySnapshot};
use leocard_ui::UiComposer;

#[derive(SystemParam)]
pub(crate) struct GameScreens<'w> {
    blackjack: BlackjackScreen<'w>,
    doudizhu: DoudizhuScreen<'w>,
    mc_guess: McGuessScreen<'w>,
    qigui523: QiGui523Screen<'w>,
    texas: TexasScreen<'w>,
    shengji: ShengjiScreen<'w>,
    uno: UnoScreen<'w>,
    mahjong: MahjongScreen<'w>,
}

impl GameScreens<'_> {
    pub(crate) fn requires_action(&self, game: &GameSnapshot) -> bool {
        match game {
            GameSnapshot::Blackjack(game) => self.blackjack.requires_action(game),
            GameSnapshot::Doudizhu(game) => self.doudizhu.requires_action(game),
            GameSnapshot::McGuess(game) => self.mc_guess.requires_action(game),
            GameSnapshot::QiGui523(game) => self.qigui523.requires_action(game),
            GameSnapshot::Texas(game) => self.texas.requires_action(game),
            GameSnapshot::Shengji(game) => self.shengji.requires_action(game),
            GameSnapshot::Uno(game) => self.uno.requires_action(game),
            GameSnapshot::Mahjong(game) => self.mahjong.requires_action(game),
        }
    }

    pub(crate) fn reconcile_screen_state(
        &mut self,
        client: Option<&ClientResource>,
        shell: &mut UiState,
    ) {
        self.blackjack.reconcile(client, &mut shell.social);
        self.doudizhu.reconcile(client, &mut shell.social);
        self.mc_guess.reconcile(client, &mut shell.social);
        self.qigui523.reconcile(client, &mut shell.social);
        self.texas.reconcile(client, &mut shell.social);
        self.shengji.reconcile(client, &mut shell.social);
        self.uno.reconcile(client, &mut shell.social);
        self.mahjong.reconcile(client, &mut shell.social);
        if !client
            .is_some_and(|client| matches!(client.0.model().phase(), ClientPhaseRef::Playing(_)))
            || shell.social.interaction_menu_open.is_some_and(|target| {
                client.is_none_or(|client| !client.0.model().has_player(target))
            })
        {
            shell.social.interaction_menu_open = None;
        }
    }

    pub(crate) fn render_lobby(
        &self,
        commands: &mut UiComposer,
        root: Entity,
        lobby: &LobbySnapshot,
        context: GameLobbyContext<'_>,
    ) {
        match lobby.game {
            GameKind::Doudizhu => self.doudizhu.render_lobby(commands, root, lobby, context),
            GameKind::Blackjack => self.blackjack.render_lobby(commands, root, lobby, context),
            GameKind::McGuess => self.mc_guess.render_lobby(commands, root, lobby, context),
            GameKind::QiGui523 => self.qigui523.render_lobby(commands, root, lobby, context),
            GameKind::Texas => self.texas.render_lobby(commands, root, lobby, context),
            GameKind::Shengji => self.shengji.render_lobby(commands, root, lobby, context),
            GameKind::Uno => self.uno.render_lobby(commands, root, lobby, context),
            GameKind::Mahjong => self.mahjong.render_lobby(commands, root, lobby, context),
        }
    }

    pub(crate) fn render_table(
        &mut self,
        commands: &mut UiComposer,
        root: Entity,
        game: &GameSnapshot,
        context: GameTableContext<'_>,
    ) {
        match game {
            GameSnapshot::Doudizhu(game) => {
                self.doudizhu.render_table(commands, root, game, context)
            }
            GameSnapshot::Blackjack(game) => {
                self.blackjack.render_table(commands, root, game, context)
            }
            GameSnapshot::McGuess(game) => {
                self.mc_guess.render_table(commands, root, game, context)
            }
            GameSnapshot::QiGui523(game) => {
                self.qigui523.render_table(commands, root, game, context)
            }
            GameSnapshot::Texas(game) => self.texas.render_table(commands, root, game, context),
            GameSnapshot::Shengji(game) => self.shengji.render_table(commands, root, game, context),
            GameSnapshot::Uno(game) => self.uno.render_table(commands, root, game, context),
            GameSnapshot::Mahjong(game) => self.mahjong.render_table(commands, root, game, context),
        }
    }
}
