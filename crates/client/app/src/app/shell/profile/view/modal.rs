//! 玩家档案弹窗、游戏标签和互动统计视图。

use super::super::super::{
    CozyModalKind, CozyModalPanel, NavigationUiAction, ShellOverlayLayer, UiAction,
    add_centered_modal, add_cozy_close_button, cozy_panel_transform,
};
use super::super::{ProfileGameTab, ProfileMotion, ProfileUiState};
use super::archive::ProfileArchive;
use super::identity::ProfileIdentity;
use crate::app::presentation::{PAGE_FRAME_TITLE, add_title, page_frame_skin};
use crate::app::runtime::{AvatarImages, ClientResource, ConnectionDraft, UiAssets};
use bevy::prelude::*;
use leocard_client::LocalPlayerProfile;
use leocard_protocol::{PlayerGameProfiles, PlayerGender};
use leocard_ui::{PageFrame, UiComposer, spawn_node};

pub(crate) struct ProfileModal<'a> {
    player_name: &'a str,
    gender: PlayerGender,
    avatar: Option<&'a Handle<Image>>,
    reference_points: i32,
    completed_games: u32,
    game_profiles: &'a PlayerGameProfiles,
    selected_game: ProfileGameTab,
    assets: &'a UiAssets,
}

impl<'a> ProfileModal<'a> {
    pub(crate) fn for_selection(
        selection: &'a ProfileUiState,
        client: Option<&'a ClientResource>,
        connection: &'a ConnectionDraft,
        avatars: &'a AvatarImages,
        profile: &'a LocalPlayerProfile,
        assets: &'a UiAssets,
    ) -> Self {
        let local_profile = selection.player.as_ref().is_none_or(|player| {
            client.is_some_and(|client| client.0.model().you() == Some(player.id))
        });
        let (name, gender, avatar, reference_points, completed_games, game_profiles) =
            if let Some(player) = selection.player.as_ref() {
                (
                    player.name.as_str(),
                    player.game_profiles.gender,
                    if local_profile {
                        avatars.local.as_ref()
                    } else {
                        client
                            .and_then(|client| client.0.model().player_avatar(player.id))
                            .and_then(|id| avatars.remote.get(&id))
                            .or(player.avatar.as_ref())
                    },
                    player.reference_points,
                    player.completed_games,
                    client
                        .and_then(|client| {
                            if client.0.model().you() == Some(player.id) {
                                Some(profile.game_profiles())
                            } else {
                                client.0.model().player_game_profiles(player.id)
                            }
                        })
                        .unwrap_or(&player.game_profiles),
                )
            } else {
                (
                    connection.player_name.as_str(),
                    connection.gender,
                    avatars.local.as_ref(),
                    profile.reference_points(),
                    profile.completed_games(),
                    profile.game_profiles(),
                )
            };
        Self {
            player_name: name,
            gender,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            selected_game: selection.game_tab,
            assets,
        }
    }

    pub(crate) fn render(self, commands: &mut UiComposer, root: Entity, motion: &ProfileMotion) {
        let Self {
            player_name,
            gender,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            selected_game,
            assets,
        } = self;
        let progress = motion.progress;
        let close = UiAction::Navigation(NavigationUiAction::ToggleProfile);
        let overlay = add_centered_modal(
            commands,
            root,
            ShellOverlayLayer::Dialog,
            CozyModalKind::Profile,
            progress,
            motion.target_open.then(|| close.clone()),
        );

        let mut frame = PageFrame::new(page_frame_skin(assets));
        frame.node.width = px(820);
        frame.node.max_width = percent(90);
        frame.node.max_height = percent(92);
        frame.heading.height = px(48);
        frame.content.row_gap = px(10);
        let frame = frame.mount(commands, overlay);
        commands.entity(frame.panel).insert((
            CozyModalPanel(CozyModalKind::Profile),
            cozy_panel_transform(progress),
        ));
        let heading = frame.heading;
        add_title(
            commands,
            heading,
            "个人资料",
            28.0,
            PAGE_FRAME_TITLE,
            assets,
        );
        add_cozy_close_button(commands, heading, close, assets);
        let content = frame.content;
        ProfileIdentity::new(
            player_name,
            gender,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            assets,
        )
        .render(commands, content);
        spawn_node(
            commands,
            content,
            Node {
                width: percent(100),
                height: px(1),
                ..default()
            },
            Some(Color::srgba(0.70, 0.68, 0.78, 0.36)),
        );
        ProfileArchive::new(game_profiles, selected_game, assets).render(commands, content);
    }
}
