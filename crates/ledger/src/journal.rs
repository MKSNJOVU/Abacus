#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountSide {
    Debit,
    Credit,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub account_id: String,
    pub side: AccountSide,
    pub amount: i64,
}

#[derive(Debug)]
pub struct LedgerTransaction {
    pub transactions: Vec<Entry>,
    pub transaction_id: String,
}

pub struct TransactionError {
    pub error_code: u16,
    pub body: String,
    pub amount: i64,
}

impl LedgerTransaction {
    pub async fn post_transaction_async(self) -> Result<LedgerTransaction, TransactionError> {
        let balance = self.calculate_net_balance();
        if balance != 0 {
            return Err(TransactionError {
                error_code: 422,
                body: "The entry does not balance out to zero".to_string(),
                amount: balance,
            });
        }

        Ok(self)
    }
    fn calculate_net_balance(&self) -> i64 {
        let mut balance: i64 = 0;
        for entry in &self.transactions {
            match entry.side {
                AccountSide::Debit => balance += entry.amount,
                AccountSide::Credit => balance -= entry.amount,
            }
        }
        balance
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(account_id: &str, side: AccountSide, amount: i64) -> Entry {
        Entry {
            account_id: account_id.to_string(),
            side,
            amount,
        }
    }

    fn transaction(transaction_id: &str, entries: Vec<Entry>) -> LedgerTransaction {
        LedgerTransaction {
            transactions: entries,
            transaction_id: transaction_id.to_string(),
        }
    }

    async fn assert_posted(tx: LedgerTransaction, transaction_id: &str) {
        assert_eq!(tx.calculate_net_balance(), 0);
        let posted = match tx.post_transaction_async().await {
            Ok(posted) => posted,
            Err(_) => panic!("zero-balance transaction {transaction_id} should post"),
        };
        assert_eq!(posted.transaction_id, transaction_id);
        assert_eq!(posted.calculate_net_balance(), 0);
    }

    async fn assert_unbalanced(tx: LedgerTransaction, expected_net: i64) {
        assert_ne!(expected_net, 0);
        assert_eq!(tx.calculate_net_balance(), expected_net);

        let err = tx.post_transaction_async().await.unwrap_err();
        assert_eq!(err.error_code, 422);
        assert_eq!(err.body, "The entry does not balance out to zero");
        assert_eq!(err.amount, expected_net);
    }

    #[tokio::test]
    async fn posts_when_debit_and_credit_minor_units_are_equal() {
        let tx = transaction(
            "txn-equal",
            vec![
                entry("cash", AccountSide::Debit, 1_500),
                entry("revenue", AccountSide::Credit, 1_500),
            ],
        );

        assert_posted(tx, "txn-equal").await;
    }

    #[tokio::test]
    async fn posts_when_several_legs_net_to_zero_regardless_of_order() {
        let tx = transaction(
            "txn-multi",
            vec![
                entry("revenue", AccountSide::Credit, 15_000),
                entry("accounts_receivable", AccountSide::Debit, 10_000),
                entry("fees_payable", AccountSide::Credit, 25),
                entry("cash", AccountSide::Debit, 5_025),
            ],
        );

        assert_posted(tx, "txn-multi").await;
    }

    #[tokio::test]
    async fn posts_when_both_sides_are_i64_max_minor_units() {
        let tx = transaction(
            "txn-max",
            vec![
                entry("cash", AccountSide::Debit, i64::MAX),
                entry("revenue", AccountSide::Credit, i64::MAX),
            ],
        );

        assert_posted(tx, "txn-max").await;
    }

    #[tokio::test]
    async fn posts_when_every_leg_is_zero_minor_units() {
        let tx = transaction(
            "txn-zero",
            vec![
                entry("cash", AccountSide::Debit, 0),
                entry("revenue", AccountSide::Credit, 0),
            ],
        );

        assert_posted(tx, "txn-zero").await;
    }

    #[tokio::test]
    async fn posts_when_there_are_no_entries() {
        let tx = transaction("txn-empty", Vec::new());

        assert_posted(tx, "txn-empty").await;
    }

    #[tokio::test]
    async fn rejects_a_debit_with_no_matching_credit() {
        let tx = transaction(
            "txn-debit-only",
            vec![entry("cash", AccountSide::Debit, 42)],
        );

        assert_unbalanced(tx, 42).await;
    }

    #[tokio::test]
    async fn rejects_a_credit_with_no_matching_debit() {
        let tx = transaction(
            "txn-credit-only",
            vec![entry("revenue", AccountSide::Credit, 42)],
        );

        assert_unbalanced(tx, -42).await;
    }

    #[tokio::test]
    async fn rejects_an_imbalance_of_one_minor_unit() {
        let tx = transaction(
            "txn-off-by-one",
            vec![
                entry("cash", AccountSide::Debit, 100),
                entry("revenue", AccountSide::Credit, 99),
            ],
        );

        assert_unbalanced(tx, 1).await;
    }

    #[tokio::test]
    async fn rejects_two_debits_that_share_a_magnitude_but_not_a_side() {
        let tx = transaction(
            "txn-same-side",
            vec![
                entry("cash", AccountSide::Debit, 100),
                entry("revenue", AccountSide::Debit, 100),
            ],
        );

        assert_unbalanced(tx, 200).await;
    }

    #[tokio::test]
    async fn rejects_when_credits_exceed_debits() {
        let tx = transaction(
            "txn-credit-heavy",
            vec![
                entry("cash", AccountSide::Debit, 100),
                entry("revenue", AccountSide::Credit, 40),
                entry("liability", AccountSide::Credit, 250),
            ],
        );

        assert_unbalanced(tx, -190).await;
    }

    #[tokio::test]
    async fn rejects_an_i64_max_debit_against_a_one_minor_unit_credit() {
        let tx = transaction(
            "txn-max-gap",
            vec![
                entry("cash", AccountSide::Debit, i64::MAX),
                entry("revenue", AccountSide::Credit, 1),
            ],
        );

        assert_unbalanced(tx, i64::MAX - 1).await;
    }
}
