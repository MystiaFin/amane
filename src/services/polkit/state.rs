use std::collections::{HashSet, VecDeque};

use super::{Polkit, PolkitError, PolkitFlow, PolkitResult};

const COOKIE_HISTORY_LIMIT: usize = 32;

pub(super) struct Pending {
    pub flow: PolkitFlow,
    pub cookie: String,
    pub reply: async_channel::Sender<super::agent::Reply>,
}

pub(super) struct State {
    pub view: Polkit,
    pub active: Option<Pending>,
    pub queue: VecDeque<Pending>,
    pub generation: u64,
    next_prompt: u64,
    cancelled: HashSet<String>,
    finished_cookies: VecDeque<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            view: Polkit::new_state(),
            active: None,
            queue: VecDeque::new(),
            generation: 0,
            next_prompt: 0,
            cancelled: HashSet::new(),
            finished_cookies: VecDeque::new(),
        }
    }

    pub fn begin(&mut self, request: Pending) -> bool {
        // D-Bus handlers can enqueue cancellation before the corresponding begin.
        if self.cancelled.remove(&request.cookie) || self.finished_cookies.contains(&request.cookie)
        {
            let _ = request
                .reply
                .try_send(Err(super::agent::AgentError::Cancelled(
                    "request cancelled before authentication started".into(),
                )));
            return false;
        }
        if self.active.is_some() {
            self.queue.push_back(request);
            return false;
        }
        self.activate(request);
        true
    }

    fn activate(&mut self, request: Pending) {
        self.generation += 1;
        self.view.flow = Some(request.flow.clone());
        self.active = Some(request);
    }

    pub fn finish(&mut self, result: PolkitResult) -> bool {
        let Some(request) = self.active.take() else {
            return false;
        };
        if let Some(mut flow) = self.view.flow.take() {
            flow.prompt = None;
            self.view.completion = Some(super::PolkitCompletion {
                flow,
                result: result.clone(),
            });
        }
        let reply = match result {
            PolkitResult::Success => Ok(()),
            PolkitResult::Cancelled => Err(super::agent::AgentError::Cancelled(
                "dismissed by the user or authority".into(),
            )),
            PolkitResult::Error(error) => Err(super::agent::AgentError::Failed(error.to_string())),
        };
        self.remember_finished(request.cookie);
        let _ = request.reply.try_send(reply);
        self.generation += 1;
        if let Some(next) = self.queue.pop_front() {
            self.activate(next);
            return true;
        }
        false
    }

    pub fn prompt(&mut self, generation: u64, text: String, visible: bool) {
        if generation != self.generation {
            return;
        }
        let Some(flow) = self.view.flow.as_mut() else {
            return;
        };
        self.next_prompt += 1;
        flow.prompt = Some(super::PolkitPrompt {
            id: self.next_prompt,
            text,
            visible,
        });
    }

    pub fn take_response(&mut self, request: u64, prompt: u64) -> bool {
        let Some(flow) = self.view.flow.as_mut().filter(|flow| flow.id == request) else {
            return false;
        };
        if flow.prompt.as_ref().is_none_or(|value| value.id != prompt) {
            return false;
        }
        flow.prompt = None;
        true
    }

    pub fn restart(&mut self, request: u64, identity: Option<usize>) -> bool {
        let Some(flow) = self.view.flow.as_mut().filter(|flow| flow.id == request) else {
            return false;
        };
        if let Some(index) = identity {
            if index >= flow.identities.len() || index == flow.selected {
                return false;
            }
            flow.selected = index;
        } else if !flow.retry_allowed {
            return false;
        }
        flow.prompt = None;
        flow.supplementary = None;
        flow.retry_allowed = false;
        self.generation += 1;
        true
    }

    pub fn failed(&mut self) {
        let Some(flow) = self.view.flow.as_mut() else {
            return;
        };
        self.generation += 1;
        flow.prompt = None;
        flow.failed = true;
        flow.retry_allowed = true;
        if flow
            .supplementary
            .as_ref()
            .is_none_or(|message| !message.error)
        {
            flow.supplementary = Some(super::PolkitMessage {
                text: "Authentication failed. Try again or cancel.".into(),
                error: true,
            });
        }
    }

    // True when cancelling the active request promoted the next queued request.
    pub fn cancel(&mut self, request: u64) -> bool {
        if self
            .active
            .as_ref()
            .is_some_and(|active| active.flow.id == request)
        {
            return self.finish(PolkitResult::Cancelled);
        }
        if let Some(index) = self
            .queue
            .iter()
            .position(|pending| pending.flow.id == request)
            && let Some(pending) = self.queue.remove(index)
        {
            self.remember_finished(pending.cookie);
            let _ = pending
                .reply
                .try_send(Err(super::agent::AgentError::Cancelled(
                    "request cancelled before authentication started".into(),
                )));
        }
        false
    }

    pub fn cancel_all(&mut self, result: PolkitResult) {
        for pending in self.queue.drain(..) {
            let _ = pending
                .reply
                .try_send(Err(super::agent::AgentError::Cancelled(
                    "agent stopped".into(),
                )));
        }
        self.finish(result);
        self.cancelled.clear();
        self.finished_cookies.clear();
    }

    pub fn cancel_cookie(&mut self, cookie: &str) -> Result<bool, PolkitError> {
        let request = self
            .active
            .as_ref()
            .filter(|request| request.cookie == cookie)
            .map(|request| request.flow.id)
            .or_else(|| {
                self.queue
                    .iter()
                    .find(|request| request.cookie == cookie)
                    .map(|request| request.flow.id)
            });
        if let Some(request) = request {
            Ok(self.cancel(request))
        } else if self
            .finished_cookies
            .iter()
            .any(|finished| finished == cookie)
            || self.cancelled.contains(cookie)
        {
            Ok(false)
        } else {
            // Evicting early cancellations could start a helper for a dismissed request.
            if self.cancelled.len() == COOKIE_HISTORY_LIMIT {
                return Err(PolkitError::Disconnected(
                    "too many unmatched authentication cancellations; register again".into(),
                ));
            }
            self.cancelled.insert(cookie.into());
            Ok(false)
        }
    }

    fn remember_finished(&mut self, cookie: String) {
        if self.finished_cookies.len() == COOKIE_HISTORY_LIMIT {
            self.finished_cookies.pop_front();
        }
        self.finished_cookies.push_back(cookie);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::polkit::PolkitIdentity;

    fn pending(id: u64) -> (Pending, async_channel::Receiver<super::super::agent::Reply>) {
        let (reply, receiver) = async_channel::bounded(1);
        (
            Pending {
                flow: PolkitFlow::new(
                    id,
                    "org.example.action".into(),
                    "Authorize this action".into(),
                    "dialog-password".into(),
                    Default::default(),
                    vec![PolkitIdentity {
                        uid: 1000,
                        name: "alice".into(),
                    }],
                ),
                cookie: format!("cookie-{id}"),
                reply,
            },
            receiver,
        )
    }

    #[test]
    fn overlapping_requests_wait_without_replacing_the_current_dialog() {
        let mut state = State::new();
        let (first, first_reply) = pending(1);
        let (second, second_reply) = pending(2);
        assert!(state.begin(first));
        assert!(!state.begin(second));
        assert_eq!(state.view.flow().unwrap().id(), 1);
        assert!(first_reply.try_recv().is_err());
        assert!(second_reply.try_recv().is_err());
        assert!(state.finish(PolkitResult::Success));
        assert!(first_reply.try_recv().unwrap().is_ok());
        assert_eq!(state.view.flow().unwrap().id(), 2);
        assert!(second_reply.try_recv().is_err());
    }

    #[test]
    fn stale_or_duplicate_responses_cannot_answer_a_later_prompt() {
        let mut state = State::new();
        state.begin(pending(1).0);
        state.prompt(state.generation, "Password:".into(), false);
        let password = state.view.flow().unwrap().prompt().unwrap().id();
        assert!(!state.take_response(2, password));
        assert!(state.take_response(1, password));
        assert!(!state.take_response(1, password));
        state.prompt(state.generation, "OTP:".into(), true);
        assert!(!state.take_response(1, password));
        let otp = state.view.flow().unwrap().prompt().unwrap();
        assert_eq!(otp.text(), "OTP:");
        assert!(otp.visible());
    }

    #[test]
    fn late_callbacks_from_a_cancelled_attempt_do_not_replace_the_prompt() {
        let mut state = State::new();
        state.begin(pending(1).0);
        let old = state.generation;
        state.generation += 1;
        state.prompt(state.generation, "New password:".into(), false);
        state.prompt(old, "Old password:".into(), true);
        assert_eq!(
            state.view.flow().unwrap().prompt().unwrap().text(),
            "New password:"
        );
    }

    #[test]
    fn cancelling_a_queued_request_does_not_cancel_the_current_conversation() {
        let mut state = State::new();
        let (first, first_reply) = pending(1);
        let (second, second_reply) = pending(2);
        state.begin(first);
        state.begin(second);
        state.prompt(state.generation, "Password:".into(), false);
        assert!(!state.cancel(2));
        assert!(matches!(
            second_reply.try_recv().unwrap(),
            Err(super::super::agent::AgentError::Cancelled(_))
        ));
        assert_eq!(
            state.view.flow().unwrap().prompt().unwrap().text(),
            "Password:"
        );
        assert!(first_reply.try_recv().is_err());
        assert!(!state.finish(PolkitResult::Success));
    }

    #[test]
    fn rejection_allows_retry_without_completing_the_authority_request() {
        let mut state = State::new();
        let (request, reply) = pending(1);
        state.begin(request);
        state.prompt(state.generation, "Password:".into(), false);
        let old_generation = state.generation;
        state.failed();
        assert!(state.view.flow().unwrap().failed());
        assert!(state.view.flow().unwrap().can_retry());
        assert!(
            state
                .view
                .flow()
                .unwrap()
                .supplementary()
                .unwrap()
                .is_error()
        );
        assert!(state.view.flow().unwrap().prompt().is_none());
        assert!(reply.try_recv().is_err());
        assert!(state.restart(1, None));
        assert!(!state.view.flow().unwrap().can_retry());
        assert!(!state.restart(1, None));
        state.prompt(old_generation, "stale".into(), true);
        assert!(state.view.flow().unwrap().prompt().is_none());
        state.prompt(state.generation, "Password:".into(), false);
        assert!(state.view.flow().unwrap().failed());
        state.finish(PolkitResult::Success);
        assert!(reply.try_recv().unwrap().is_ok());
        assert_eq!(
            state.view.completion().unwrap().result(),
            &PolkitResult::Success
        );
    }

    #[test]
    fn switching_identity_invalidates_the_previous_conversation() {
        let mut state = State::new();
        let (mut request, reply) = pending(1);
        request.flow.identities.push(PolkitIdentity {
            uid: 1001,
            name: "bob".into(),
        });
        state.begin(request);
        state.prompt(state.generation, "Alice password:".into(), false);
        let prompt = state.view.flow().unwrap().prompt().unwrap().id();
        let generation = state.generation;
        assert!(!state.restart(1, Some(3)));
        assert!(!state.restart(1, Some(0)));
        assert!(state.restart(1, Some(1)));
        assert_eq!(state.view.flow().unwrap().selected_identity(), 1);
        assert!(!state.take_response(1, prompt));
        state.prompt(generation, "Alice password:".into(), false);
        assert!(state.view.flow().unwrap().prompt().is_none());
        assert!(reply.try_recv().is_err());
    }

    #[test]
    fn stopping_resolves_all_pending_calls_without_starting_the_queue() {
        let mut state = State::new();
        let (first, first_reply) = pending(1);
        let (second, second_reply) = pending(2);
        state.begin(first);
        state.begin(second);
        state.cancel_all(PolkitResult::Cancelled);
        assert!(first_reply.try_recv().unwrap().is_err());
        assert!(second_reply.try_recv().unwrap().is_err());
        assert!(state.view.flow().is_none());
        assert_eq!(
            state.view.completion().unwrap().result(),
            &PolkitResult::Cancelled
        );
        assert!(state.active.is_none());
        assert!(state.queue.is_empty());
    }

    #[test]
    fn cancellation_before_begin_does_not_open_a_late_authentication_dialog() {
        let mut state = State::new();
        state.cancel_cookie("cookie-1").unwrap();
        let (request, reply) = pending(1);
        assert!(!state.begin(request));
        assert!(state.view.flow().is_none());
        assert!(matches!(
            reply.try_recv().unwrap(),
            Err(super::super::agent::AgentError::Cancelled(_))
        ));
    }

    #[test]
    fn cancellation_history_stays_bounded_without_forgetting_early_cancellations() {
        let mut state = State::new();
        for id in 0..100 {
            let (request, _) = pending(id);
            state.begin(request);
            state.finish(PolkitResult::Success);
            state.cancel_cookie(&format!("cookie-{id}")).unwrap();
        }
        assert!(state.cancelled.is_empty());
        assert_eq!(state.finished_cookies.len(), COOKIE_HISTORY_LIMIT);
        for id in 0..COOKIE_HISTORY_LIMIT {
            state.cancel_cookie(&format!("early-{id}")).unwrap();
        }
        assert!(matches!(
            state.cancel_cookie("overflow"),
            Err(PolkitError::Disconnected(_))
        ));
        state.cancel_all(PolkitResult::Cancelled);
        assert!(state.cancelled.is_empty());
        assert!(state.finished_cookies.is_empty());
    }
}
