use super::rows::ProfileRow;

/// Each game's archive owns its rows, including unavailable values and statistic periods.
pub(crate) trait GameProfileRows: Sized {
    fn rows(stats: Option<&Self>) -> Vec<ProfileRow>;
}
