#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccountSide {
    Debit,
    Credit,
}

#[derive(Debug, Clone)]
pub struct Entry{
    pub account_id: String,
    pub side: AccountSide,
    pub amount: i64,
}

pub struct LedgerTransaction{
    pub transaction: Vec<Entry>,
    pub transaction_id: String
}
