use leocard_game_common::SearchBudget;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SEARCH_TIME: Duration = Duration::from_millis(2400);
const RESPONSE_LIMIT: Duration = Duration::from_millis(3500);

#[derive(Clone, Debug)]
struct Pending<O, A> {
    observation: O,
    fallback: A,
    answer: Arc<Mutex<Option<A>>>,
    budget: SearchBudget,
    started: Instant,
}

#[derive(Clone, Debug)]
pub(crate) struct BotDecision<O, A> {
    pending: Option<Pending<O, A>>,
}

impl<O, A> Default for BotDecision<O, A> {
    fn default() -> Self {
        Self { pending: None }
    }
}

impl<O: Clone + PartialEq + Send + 'static, A: Clone + Send + 'static> BotDecision<O, A> {
    pub fn poll(
        &mut self,
        observation: O,
        fallback: A,
        compute: impl FnOnce(&O, &SearchBudget) -> A + Send + 'static,
    ) -> Option<A> {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.observation != observation)
        {
            self.clear();
        }
        if let Some(pending) = &self.pending {
            let result = pending.answer.lock().expect("bot answer lock").take();
            if result.is_some() {
                self.clear();
                return result;
            }
            if pending.started.elapsed() >= RESPONSE_LIMIT {
                let action = pending.fallback.clone();
                self.clear();
                return Some(action);
            }
            return None;
        }
        let answer = Arc::new(Mutex::new(None));
        let budget = SearchBudget::new(SEARCH_TIME);
        let result = answer.clone();
        let input = observation.clone();
        let search_budget = budget.clone();
        thread::spawn(move || {
            let action = compute(&input, &search_budget);
            *result.lock().expect("bot answer lock") = Some(action);
        });
        self.pending = Some(Pending {
            observation,
            fallback,
            answer,
            budget,
            started: Instant::now(),
        });
        None
    }

    pub fn clear(&mut self) {
        if let Some(pending) = self.pending.take() {
            pending.budget.cancel();
        }
    }
}
