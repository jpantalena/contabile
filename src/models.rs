use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub id: u32,
    pub amount: Option<f64>,
}

impl Transaction {
    pub fn amount(&self) -> f64 {
        return self.amount.unwrap_or(0f64);
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug)]
pub struct Account {
    pub client_id: u16,
    pub available: f64,
    pub held: f64,
    pub total: f64,
    pub locked: bool,
}

impl Account {
    pub fn new(client_id: &u16) -> Account {
        Account {
            client_id: client_id.to_owned(),
            available: 0f64,
            held: 0f64,
            total: 0f64,
            locked: false,
        }
    }

    pub fn sum_total(&mut self) {
        self.total = self.available + self.held
    }
}
