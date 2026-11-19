mod observe;
mod own;
mod remote;

pub(super) use observe::observe_discard_animations;
pub(crate) use own::MahjongOwnDiscardAnimation;
pub(super) use own::{
    DISCARD_DURATION, MahjongDiscardHandShift, MahjongDiscardRiverTile, animate_own_discard,
    render_own_discard_flight,
};
pub(crate) use remote::MahjongRemoteDiscardAnimation;
pub(super) use remote::{
    MahjongRemoteHandShift, animate_remote_discard, capture_remote_discard_source,
    render_remote_discard_river_tile,
};
