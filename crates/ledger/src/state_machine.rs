use crate::state_machine::PaymentEvent::{Fail, Settle, StartProcessing};
use crate::state_machine::TransactionState::{Failed, Initiated, Processing, Settled};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransactionState {
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
