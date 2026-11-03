use leocard_game_common::SearchBudget;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const SEARCH_TIME: Duration = Duration::from_millis(2400);
const RESPONSE_LIMIT: Duration = Duration::from_millis(3500);

#[derive(Clone, Debug)]
struct Answer<A> {
    best: A,
    complete: bool,
}

#[derive(Clone, Debug)]
struct Pending<O, A> {
    observation: O,
    answer: Arc<Mutex<Answer<A>>>,
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
        self.poll_with_progress(
            observation,
            |_| fallback,
            |input, budget, _| compute(input, budget),
        )
    }

    pub fn poll_with_progress(
        &mut self,
        observation: O,
        initial: impl FnOnce(&O) -> A,
        compute: impl FnOnce(&O, &SearchBudget, &mut dyn FnMut(A)) -> A + Send + 'static,
    ) -> Option<A> {
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.observation != observation)
        {
            self.clear();
        }
        if let Some(pending) = &self.pending {
            let answer = pending.answer.lock().expect("bot answer lock");
            if answer.complete || pending.started.elapsed() >= RESPONSE_LIMIT {
                let action = answer.best.clone();
                drop(answer);
                self.clear();
                return Some(action);
            }
            return None;
        }
        let answer = Arc::new(Mutex::new(Answer {
            best: initial(&observation),
            complete: false,
        }));
        let budget = SearchBudget::new(SEARCH_TIME);
        let result = answer.clone();
        let input = observation.clone();
        let search_budget = budget.clone();
        thread::spawn(move || {
            let action = compute(&input, &search_budget, &mut |action| {
                result.lock().expect("bot answer lock").best = action;
            });
            *result.lock().expect("bot answer lock") = Answer {
                best: action,
                complete: true,
            };
        });
        self.pending = Some(Pending {
            observation,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn deadline_uses_published_plan_and_cancels_search_instead_of_initial_fallback() {
        let mut decision = BotDecision::default();
        let (published, received) = mpsc::channel();
        let (release, blocked) = mpsc::channel();
        let (cancelled, stopped) = mpsc::channel();
        assert_eq!(
            decision.poll_with_progress(
                0,
                |_| 1,
                move |_, budget, report| {
                    report(7);
                    published.send(()).unwrap();
                    blocked.recv().unwrap();
                    cancelled.send(budget.exhausted()).unwrap();
                    9
                }
            ),
            None
        );
        received.recv_timeout(Duration::from_secs(2)).unwrap();
        decision.pending.as_mut().unwrap().started = Instant::now() - RESPONSE_LIMIT;
        assert_eq!(
            decision.poll_with_progress(0, |_| 1, |_, _, _| unreachable!()),
            Some(7)
        );
        assert!(decision.pending.is_none());
        release.send(()).unwrap();
        assert!(stopped.recv_timeout(Duration::from_secs(2)).unwrap());
    }

    #[test]
    fn obsolete_worker_cannot_overwrite_a_new_observations_plan() {
        let mut decision = BotDecision::default();
        let (published, received) = mpsc::channel();
        let (release, blocked) = mpsc::channel();
        let (new_published, new_received) = mpsc::channel();
        let (new_release, new_blocked) = mpsc::channel();
        decision.poll_with_progress(
            0,
            |_| 1,
            move |_, _, report| {
                report(7);
                published.send(()).unwrap();
                blocked.recv().unwrap();
                report(99);
                99
            },
        );
        received.recv_timeout(Duration::from_secs(2)).unwrap();
        let old_budget = decision.pending.as_ref().unwrap().budget.clone();
        decision.poll_with_progress(
            1,
            |_| 2,
            move |_, _, report| {
                report(8);
                new_published.send(()).unwrap();
                new_blocked.recv().unwrap();
                8
            },
        );
        assert!(old_budget.exhausted());
        release.send(()).unwrap();
        new_received.recv_timeout(Duration::from_secs(2)).unwrap();
        decision.pending.as_mut().unwrap().started = Instant::now() - RESPONSE_LIMIT;
        assert_eq!(
            decision.poll_with_progress(1, |_| 2, |_, _, _| unreachable!()),
            Some(8)
        );
        new_release.send(()).unwrap();
    }
}
