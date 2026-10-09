use crate::app::presentation::CustomButtonMotion;
use bevy::prelude::*;
use leocard_protocol::{PlayerGameProfiles, PlayerId};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ProfileGameTab {
    #[default]
    QiGui523,
    TexasHoldem,
    Shengji,
    Uno,
    Mahjong,
}

impl ProfileGameTab {
    pub(crate) const ALL: [(Self, &'static str); 5] = [
        (Self::QiGui523, "七鬼五二三"),
        (Self::TexasHoldem, "德州扑克"),
        (Self::Shengji, "升级"),
        (Self::Uno, "UNO"),
        (Self::Mahjong, "麻将"),
    ];
}

#[derive(Clone)]
pub(crate) struct PlayerProfilePage {
    pub id: PlayerId,
    pub name: String,
    pub avatar: Option<Handle<Image>>,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

#[derive(Component)]
pub(crate) struct SelectedProfileGameTab;

#[derive(Component)]
pub(crate) struct ProfileGameContent;

#[derive(Component)]
#[require(CustomButtonMotion)]
pub(crate) struct ProfileGameTabButton;

#[derive(Component)]
pub(crate) struct ProfileGameColumn;

#[derive(Component)]
pub(crate) struct ProfileStat;

#[derive(Default)]
pub(crate) struct ProfileUiState {
    pub open: bool,
    pub player: Option<PlayerProfilePage>,
    pub game_tab: ProfileGameTab,
}
