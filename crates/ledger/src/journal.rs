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
