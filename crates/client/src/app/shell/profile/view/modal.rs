//! 玩家档案弹窗、游戏标签和互动统计视图。

use super::super::super::{
    CozyModalBackdrop, CozyModalKind, CozyModalPanel, NavigationUiAction, UiAction,
    add_cozy_close_button, add_cozy_panel, cozy_backdrop_color, cozy_panel_transform,
};
use super::super::{ProfileGameTab, ProfileUiState};
use crate::app::presentation::{TEXT, add_text, spawn_node};
use crate::app::runtime::{AvatarImages, ClientResource, ConnectionDraft, UiAssets};
use bevy::picking::Pickable;
use bevy::prelude::*;
use leocard_client::LocalPlayerProfile;
use leocard_protocol::{PlayerGameProfiles, PlayerGender};

use super::archive::ProfileArchive;
use super::identity::ProfileIdentity;

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
        let (name, gender, avatar, reference_points, completed_games, game_profiles) =
            if let Some(player) = selection.player.as_ref() {
                (
                    player.name.as_str(),
                    player.game_profiles.gender,
                    player.avatar.as_ref(),
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
        Self::new(
            name,
            gender,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            selection.game_tab,
            assets,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "profile identity and statistics are passed explicitly"
    )]
    pub(crate) fn new(
        player_name: &'a str,
        gender: PlayerGender,
        avatar: Option<&'a Handle<Image>>,
        reference_points: i32,
        completed_games: u32,
        game_profiles: &'a PlayerGameProfiles,
        selected_game: ProfileGameTab,
        assets: &'a UiAssets,
    ) -> Self {
        Self {
            player_name,
            gender,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            selected_game,
            assets,
        }
    }

    pub(crate) fn render(self, commands: &mut Commands, root: Entity, progress: f32) {
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
        let overlay = spawn_node(
            commands,
            root,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(0),
                top: px(0),
                bottom: px(0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            Some(cozy_backdrop_color(progress)),
        );
        commands.entity(overlay).insert((
            GlobalZIndex(2000),
            Pickable::default(),
            CozyModalBackdrop(CozyModalKind::Profile),
        ));

        let modal = add_cozy_panel(
            commands,
            overlay,
            Node {
                width: px(820),
                max_width: percent(90),
                min_height: px(520),
                padding: UiRect::all(px(24)),
                flex_direction: FlexDirection::Column,
                row_gap: px(12),
                ..default()
            },
            assets,
        );
        commands.entity(modal).insert((
            CozyModalPanel(CozyModalKind::Profile),
            cozy_panel_transform(progress),
        ));
        let heading = spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            None,
        );
        add_text(commands, heading, "个人资料", 28.0, TEXT, assets);
        add_cozy_close_button(
            commands,
            heading,
            UiAction::Navigation(NavigationUiAction::ToggleProfile),
            assets,
        );
        spawn_node(
            commands,
            modal,
            Node {
                width: px(96),
                height: px(2),
                ..default()
            },
            Some(Color::srgb(0.64, 0.59, 0.93)),
        );
        ProfileIdentity::new(
            player_name,
            gender,
            avatar,
            reference_points,
            completed_games,
            game_profiles,
            assets,
        )
        .render(commands, modal);
        spawn_node(
            commands,
            modal,
            Node {
                width: percent(100),
                height: px(1),
                ..default()
            },
            Some(Color::srgba(0.70, 0.68, 0.78, 0.36)),
        );
        ProfileArchive::new(game_profiles, selected_game, assets).render(commands, modal);
    }
}
