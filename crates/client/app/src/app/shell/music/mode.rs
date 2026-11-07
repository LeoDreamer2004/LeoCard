use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Default, Deserialize, PartialEq, Serialize)]
pub(super) enum PlayMode {
    #[default]
    Sequence,
    Shuffle,
}

impl PlayMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sequence => "顺序播放",
            Self::Shuffle => "随机播放",
        }
    }

    pub fn cycle(self) -> Self {
        match self {
            Self::Sequence => Self::Shuffle,
            Self::Shuffle => Self::Sequence,
        }
    }

    pub fn next(self, index: usize, length: usize, manual: bool) -> Option<usize> {
        if length == 0 || index >= length {
            return None;
        }
        match self {
            Self::Shuffle => Some(if length == 1 {
                index
            } else {
                (index + 1 + fastrand::usize(0..length - 1)) % length
            }),
            Self::Sequence if manual => Some((index + 1) % length),
            Self::Sequence => (index + 1 < length).then_some(index + 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PlayMode;

    #[test]
    fn queue_finishes_and_manual_next_wraps() {
        assert_eq!(PlayMode::Sequence.next(2, 3, false), None);
        assert_eq!(PlayMode::Sequence.next(2, 3, true), Some(0));
        assert_eq!(PlayMode::Sequence.next(0, 0, true), None);
    }

    #[test]
    fn mode_switches_between_sequence_and_shuffle_without_repeating_current_song() {
        assert!(PlayMode::Sequence.cycle() == PlayMode::Shuffle);
        assert!(PlayMode::Shuffle.cycle() == PlayMode::Sequence);
        assert_eq!(PlayMode::Shuffle.next(0, 1, false), Some(0));
        for _ in 0..20 {
            assert_ne!(PlayMode::Shuffle.next(1, 3, false), Some(1));
        }
    }
}
