pub(crate) const MAHJONG_MAJOR_SHOWCASE_DURATION: f32 =
    MajorShowcaseSequence::EXIT + MajorShowcaseSequence::FADE;

pub(in crate::app::games::mahjong::presentation::win_view) struct MajorShowcaseSequence;

impl MajorShowcaseSequence {
    pub const TILE_START: f32 = 0.30;
    pub const WINNING_TILE_START: f32 = 0.54;
    pub const TILE_FLIGHT: f32 = 0.40;
    pub const TILE_SETTLE: f32 = 0.18;
    pub const FIRST_GLYPH: f32 =
        Self::WINNING_TILE_START + Self::TILE_FLIGHT + Self::TILE_SETTLE + 0.14;
    pub const GLYPH_INTERVAL: f32 = 0.34;
    pub const GLYPH_FALL: f32 = 0.16;
    pub const GLYPH_SETTLE: f32 = 0.18;
    // The longest current title has five glyphs. Finish its rebound before the polishing light starts.
    pub const HOLD_START: f32 = Self::FIRST_GLYPH
        + 4.0 * Self::GLYPH_INTERVAL
        + Self::GLYPH_FALL
        + Self::GLYPH_SETTLE
        + 0.04;
    pub const HOLD_DURATION: f32 = 3.2;
    pub const EXIT: f32 = Self::HOLD_START + Self::HOLD_DURATION;
    pub const FADE: f32 = 0.8;
    pub const POLISH_DURATION: f32 = Self::HOLD_DURATION - 0.3;
}
