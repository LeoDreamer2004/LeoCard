use super::global::{AllVoices, Base, Gender, MahjongVoices};
use super::schema::{Archive, Rules};
use super::v032::MahjongRules;
use crate::player::preferences::{
    GlobalPreferences, MahjongPreferences, SavedPreferences, TexasHoldemPreferences,
};
use serde::de::DeserializeOwned;

pub(in crate::player) fn migrate_preferences(bytes: &[u8]) -> Option<SavedPreferences> {
    if let Some(data) = bytes.strip_prefix(b"LeoCard/prefs/v1\0") {
        return decode::<Archive<GlobalPreferences, MahjongPreferences, TexasHoldemPreferences>>(
            data,
        );
    }
    decode::<Archive<AllVoices>>(bytes)
        .or_else(|| decode::<Archive<MahjongVoices<Gender<Base>>>>(bytes))
        .or_else(|| decode::<Archive<Gender<Base>>>(bytes))
        .or_else(|| decode::<Archive<Base>>(bytes))
        .or_else(|| decode::<Archive<Base, Rules<MahjongRules>>>(bytes))
}

fn decode<A>(bytes: &[u8]) -> Option<SavedPreferences>
where
    A: DeserializeOwned + Into<SavedPreferences>,
{
    let (archive, remaining) = postcard::take_from_bytes::<A>(bytes).ok()?;
    remaining.is_empty().then(|| archive.into())
}
