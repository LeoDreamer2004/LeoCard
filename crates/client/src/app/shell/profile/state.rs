use bevy::prelude::*;
use leocard_protocol::PlayerGameProfiles;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ProfileGameTab {
    #[default]
    QiGui523,
    TexasHoldem,
    Shengji,
    Uno,
}

impl ProfileGameTab {
    pub(crate) const ALL: [(Self, &'static str); 4] = [
        (Self::QiGui523, "七鬼五二三"),
        (Self::TexasHoldem, "德州扑克"),
        (Self::Shengji, "升级"),
        (Self::Uno, "UNO"),
    ];
}

#[derive(Clone)]
pub(crate) struct PlayerProfilePage {
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
pub(crate) struct ProfileGameTabButton;

#[derive(Component)]
pub(crate) struct ProfileGameColumn;

#[derive(Component)]
pub(crate) struct ProfileStat;
