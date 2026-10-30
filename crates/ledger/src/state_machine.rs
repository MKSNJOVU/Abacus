use crate::state_machine::PaymentEvent::{Fail, Settle, StartProcessing};
use crate::state_machine::TransactionState::{Failed, Initiated, Settled};

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

impl TransactionState {
    pub fn apply(
        &self,
        transaction_state: TransactionState,
        payment_event: PaymentEvent,
    ) -> Result<Transition, TransitionError> {
        if TransactionState::is_illegal_edge(transaction_state.clone(), payment_event.clone()) {
            Err(TransitionError::Illegal {
                transaction_state,
                payment_event,
            })
        } else {
            Ok(Transition {
                next: transaction_state,
                post_money: true,
            })
        }
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
