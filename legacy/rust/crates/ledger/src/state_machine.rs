use crate::state_machine::PaymentEvent::{Fail, Settle, StartProcessing};
use crate::state_machine::TransactionState::{Failed, Initiated, Processing, Settled};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionState {
    Initiated,
    Processing,
    Settled,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaymentEvent {
    StartProcessing,
    Settle,
    Fail,
}

pub struct Transition {
    pub next: TransactionState,
    pub post_money: bool,
}

pub enum TransitionError {
    Illegal {
        transaction_state: TransactionState,
        payment_event: PaymentEvent,
    },
}

pub trait PostFunds {
    type State;
    type Event;
    fn can_post_funds(state: &Self::State, event: &Self::Event) -> bool;
}

impl TransactionState {
    pub fn apply(
        &self,
        transaction_state: TransactionState,
        payment_event: PaymentEvent,
    ) -> Result<Transition, TransitionError> {
        if TransactionState::is_illegal_edge(transaction_state, payment_event) {
            Err(TransitionError::Illegal {
                transaction_state,
                payment_event,
            })
        } else if !Self::can_post_funds(&transaction_state, &payment_event) {
            Ok(Self::transition_to_next_state(
                transaction_state,
                payment_event,
            ))
        } else {
            Ok(Transition {
                next: Settled,
                post_money: true,
            })
        }
    }

    fn transition_to_next_state(
        starting_state: TransactionState,
        event: PaymentEvent,
    ) -> Transition {
        let mut result = Transition {
            next: Initiated,
            post_money: false,
        };

        if starting_state == Initiated && event == StartProcessing {
            result.next = Processing;
            result.post_money = false;
        } else if starting_state == Processing && event == Fail {
            result.next = Failed;
            result.post_money = false
        }

        if Self::can_post_funds(&starting_state, &event) {
            result.next = Settled;
            result.post_money = true;
        }
        result
    }
    fn is_illegal_edge(transaction_state: TransactionState, payment_event: PaymentEvent) -> bool {
        matches!(
            (transaction_state, payment_event),
            (Initiated, Settle)
                | (Failed, Settle)
                | (Settled, Fail)
                | (Failed, StartProcessing)
                | (Settled, StartProcessing)
        )
    }
}

impl PostFunds for TransactionState {
    type State = TransactionState;
    type Event = PaymentEvent;
    fn can_post_funds(state: &Self::State, event: &Self::Event) -> bool {
        matches!((state, event), (Processing, Settle))
    }
}

#[cfg(test)]
mod tests {
    use super::PaymentEvent::{Fail, Settle, StartProcessing};
    use super::TransactionState::{Failed, Initiated, Processing, Settled};
    use super::{PaymentEvent, TransactionState, TransitionError};

    fn expect_illegal(state: TransactionState, event: PaymentEvent) {
        match state.apply(state, event) {
            Err(TransitionError::Illegal {
                transaction_state,
                payment_event,
            }) => {
                assert_eq!(transaction_state, state);
                assert_eq!(payment_event, event);
            }
            Ok(transition) => panic!(
                "expected {state:?} + {event:?} to be illegal, got next={:?} post_money={}",
                transition.next, transition.post_money
            ),
        }
    }

    fn expect_transition(
        state: TransactionState,
        event: PaymentEvent,
        next: TransactionState,
        post_money: bool,
    ) {
        match state.apply(state, event) {
            Ok(transition) => {
                assert_eq!(transition.next, next);
                assert_eq!(transition.post_money, post_money);
            }
            Err(TransitionError::Illegal {
                transaction_state,
                payment_event,
            }) => panic!("expected {transaction_state:?} + {payment_event:?} to be legal"),
        }
    }

    #[test]
    fn initiated_and_settle_is_illegal() {
        expect_illegal(Initiated, Settle);
    }

    #[test]
    fn failed_and_settle_is_illegal() {
        expect_illegal(Failed, Settle);
    }

    #[test]
    fn settled_and_fail_is_illegal() {
        expect_illegal(Settled, Fail);
    }

    #[test]
    fn failed_and_start_processing_is_illegal() {
        expect_illegal(Failed, StartProcessing);
    }

    #[test]
    fn settled_and_start_processing_is_illegal() {
        expect_illegal(Settled, StartProcessing);
    }

    #[test]
    fn initiated_and_start_processing_moves_to_processing_without_posting() {
        expect_transition(Initiated, StartProcessing, Processing, false);
    }

    #[test]
    fn processing_and_settle_moves_to_settled_and_posts_funds() {
        expect_transition(Processing, Settle, Settled, true);
    }

    #[test]
    fn processing_and_fail_moves_to_failed_without_posting() {
        expect_transition(Processing, Fail, Failed, false);
    }

    #[test]
    fn initiated_and_fail_stays_initiated_without_posting() {
        expect_transition(Initiated, Fail, Initiated, false);
    }

    #[test]
    fn processing_and_start_processing_returns_initiated_without_posting() {
        expect_transition(Processing, StartProcessing, Initiated, false);
    }

    #[test]
    fn settled_and_settle_returns_initiated_without_posting() {
        expect_transition(Settled, Settle, Initiated, false);
    }

    #[test]
    fn failed_and_fail_returns_initiated_without_posting() {
        expect_transition(Failed, Fail, Initiated, false);
    }
}
