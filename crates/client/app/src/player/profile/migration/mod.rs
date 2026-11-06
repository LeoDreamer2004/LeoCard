mod archive;
mod blackjack;
mod doudizhu;
mod graph;
mod guandan;
pub(in crate::player::profile) mod legacy;
mod mc_guess;
mod profile;
mod texas;
mod uno;
mod v8;

pub(super) use profile::decode_player_profile;
