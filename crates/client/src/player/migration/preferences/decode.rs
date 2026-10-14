use super::global::{AllVoices, Base, Gender, MahjongVoices};
use super::schema::{Archive, Rules};
use super::v032::MahjongRules;
use crate::player::preferences::{GlobalPreferences, MahjongPreferences, SavedPreferences};
use serde::de::DeserializeOwned;

pub(in crate::player) fn migrate_preferences(bytes: &[u8]) -> Option<SavedPreferences> {
    decode::<AllVoices, MahjongPreferences>(bytes)
        .or_else(|| decode::<MahjongVoices<Gender<Base>>, MahjongPreferences>(bytes))
        .or_else(|| decode::<Gender<Base>, MahjongPreferences>(bytes))
        .or_else(|| decode::<Base, MahjongPreferences>(bytes))
        .or_else(|| decode::<Base, Rules<MahjongRules>>(bytes))
}

fn decode<G, M>(bytes: &[u8]) -> Option<SavedPreferences>
where
    G: DeserializeOwned + Into<GlobalPreferences>,
    M: DeserializeOwned + Into<MahjongPreferences>,
{
    let (archive, remaining) = postcard::take_from_bytes::<Archive<G, M>>(bytes).ok()?;
    remaining.is_empty().then(|| archive.into())
}
