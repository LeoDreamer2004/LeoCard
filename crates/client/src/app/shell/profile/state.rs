use super::*;
pub use leocard_client::{LocalPlayerProfile, PlayerRatingProfile};
use leocard_protocol::PlayerGameProfiles;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProfileGameTab {
    #[default]
    QiGui523,
    TexasHoldem,
    Shengji,
    Uno,
}

impl ProfileGameTab {
    pub const ALL: [(Self, &'static str); 4] = [
        (Self::QiGui523, "七鬼五二三"),
        (Self::TexasHoldem, "德州扑克"),
        (Self::Shengji, "升级"),
        (Self::Uno, "UNO"),
    ];
}

#[derive(Clone)]
pub struct PlayerProfilePage {
    pub name: String,
    pub avatar: Option<Handle<Image>>,
    pub reference_points: i32,
    pub completed_games: u32,
    pub game_profiles: PlayerGameProfiles,
}

#[derive(Component)]
pub struct SelectedProfileGameTab;

#[derive(Component)]
pub struct ProfileGameContent;

#[derive(Component)]
pub struct ProfileGameTabButton;

#[derive(Component)]
pub struct ProfileGameColumn;

#[derive(Component)]
pub struct ProfileStat;
