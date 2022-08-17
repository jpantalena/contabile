use std::collections::HashMap;
use crate::{Account, Transaction, TransactionType};

pub fn process_transactions(transactions: Vec<Transaction>) -> HashMap<u16, Account> {
    let mut account_map: HashMap<u16, Account> = HashMap::new();
    let mut transaction_history: HashMap<u32, Transaction> = HashMap::new();
    let mut dispute_history: HashMap<u32, Transaction> = HashMap::new();

    let mut apply_transaction = |acct: &mut Account, tx: Transaction| {
        match tx.transaction_type {
            TransactionType::Deposit => {
                acct.available += tx.amount();
                transaction_history.entry(tx.id).or_insert(tx);
            }
            TransactionType::Withdrawal => {
                if acct.available >= tx.amount() {
                    acct.available -= tx.amount()
                }
                transaction_history.entry(tx.id).or_insert(tx);
            }
            TransactionType::Dispute => {
                if let Some(transaction) = &transaction_history.get(&tx.id) {
                    // assume the transaction is a deposit
                    acct.available -= transaction.amount();
                    acct.held += transaction.amount();
                }
                dispute_history.entry(tx.id).or_insert(tx);
            }
            TransactionType::Resolve => {
                if let Some(transaction) = &transaction_history.get(&tx.id) {
                    if dispute_history.contains_key(&tx.id) {
                        acct.available += transaction.amount();
                        acct.held -= transaction.amount();
                        dispute_history.remove(&tx.id);
                    }
                }
            }
            TransactionType::Chargeback => {
                if let Some(transaction) = &transaction_history.get(&tx.id) {
                    if dispute_history.contains_key(&tx.id) {
                        acct.held -= transaction.amount();
                        acct.locked = true;
                        dispute_history.remove(&tx.id);
                    }
                }
            }
        };

        acct.sum_total();
    };
    // Iterate over transactions and apply to accounts
    for transaction in transactions {
        if let Some(account) = account_map.get_mut(&transaction.client_id) {
            apply_transaction(account, transaction);
        } else {
            let mut account = Account::new(&transaction.client_id);
            let client_id = transaction.client_id;
            apply_transaction(&mut account, transaction);
            account_map.insert(client_id, account);
        }
    }
    return account_map;
}
