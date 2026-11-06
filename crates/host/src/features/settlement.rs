/// Commit the authoritative result once, independently of client presentation.
pub(crate) trait MatchSettlement {
    type Outcome: ?Sized;

    fn commit_match(&mut self, outcome: &Self::Outcome);
}

/// Finish an independently tracked hand without conflating it with the full match.
pub(crate) trait HandSettlement {
    type Outcome: ?Sized;
    type Output;

    fn commit_hand(&mut self, outcome: &Self::Outcome) -> Self::Output;
}
