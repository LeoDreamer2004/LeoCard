mod decode;
mod global;
mod schema;
#[cfg(test)]
mod tests;
mod v032;
mod v091;

pub(in crate::player) use decode::migrate_preferences;
