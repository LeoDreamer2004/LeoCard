pub const CAN_HOST: bool = !cfg!(feature = "web");
pub const HAS_LAN: bool = !cfg!(feature = "web");
pub const HAS_FRP: bool = cfg!(feature = "desktop");
pub const HAS_UPDATES: bool = !cfg!(feature = "web");

pub fn is_mobile() -> bool {
    #[cfg(feature = "web")]
    return super::browser::is_mobile();
    #[cfg(not(feature = "web"))]
    cfg!(feature = "android")
}

pub fn side_control_scale() -> f32 {
    if is_mobile() { 1.5 } else { 1.0 }
}
