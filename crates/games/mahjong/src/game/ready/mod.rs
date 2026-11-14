mod declaration;
mod kongs;

pub(in crate::game) use kongs::ReadyKong;

#[cfg(test)]
mod tests;
