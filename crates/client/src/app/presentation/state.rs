//! 可复用的表现层观察状态。

#[derive(Clone, Debug)]
pub(crate) struct Observed<K, S> {
    pub key: Option<K>,
    pub state: S,
}

impl<K, S: Default> Default for Observed<K, S> {
    fn default() -> Self {
        Self {
            key: None,
            state: S::default(),
        }
    }
}

impl<K: PartialEq, S: Default> Observed<K, S> {
    pub(crate) fn observe(&mut self, key: K) -> bool {
        if self.key.as_ref() == Some(&key) {
            return false;
        }
        self.key = Some(key);
        self.state = S::default();
        true
    }

    pub(crate) fn clear(&mut self) {
        self.key = None;
        self.state = S::default();
    }
}
