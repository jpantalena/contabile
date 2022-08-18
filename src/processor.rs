use crate::{Account, Transaction, TransactionType};
use std::collections::HashMap;

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
    account_map
}

#[cfg(test)]
mod tests {
    use crate::TransactionType::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};
    use super::*;

    #[test]
    fn test_process_transactions_deposit_withdrawal() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                client_id: 1,
                transaction_type: Deposit,
                amount: Some(3.0123)
            },
            Transaction {
                id: 2,
                client_id: 1,
                transaction_type: Withdrawal,
                amount: Some(1.8761)
            }
        ];

        let actual = process_transactions(transactions);
        assert_eq!(actual.len(), 1);

        let account = actual.get(&1).unwrap();
        assert_eq!(account.client_id, 1);
        assert_eq!(account.locked, false);
        assert_eq!(account.available, 1.1362);
        assert_eq!(account.held, 0f64);
        assert_eq!(account.total, 1.1362);
    }

    #[test]
    fn test_process_transactions_dispute() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                transaction_type: Deposit,
                client_id: 1,
                id: 1,
                amount: Some(5.5),
            },
            Transaction {
                transaction_type: Deposit,
                client_id: 1,
                id: 2,
                amount: Some(2.5),
            },
            Transaction {
                transaction_type: Dispute,
                client_id: 1,
                id: 2,
                amount: None,
            },
        ];

        let actual = process_transactions(transactions);
        assert_eq!(actual.len(), 1);

        let account = actual.get(&1).unwrap();
        assert_eq!(account.client_id, 1);
        assert_eq!(account.locked, false);
        assert_eq!(account.available, 5.5);
        assert_eq!(account.held, 2.5);
        assert_eq!(account.total, 8.0);
    }

    #[test]
    fn test_process_transactions_dispute_resolve() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                transaction_type: Deposit,
                client_id: 1,
                id: 1,
                amount: Some(5.5),
            },
            Transaction {
                transaction_type: Deposit,
                client_id: 1,
                id: 2,
                amount: Some(2.5),
            },
            Transaction {
                transaction_type: Dispute,
                client_id: 1,
                id: 2,
                amount: None,
            },
            Transaction {
                transaction_type: Resolve,
                client_id: 1,
                id: 2,
                amount: None,
            }
        ];

        let actual = process_transactions(transactions);
        assert_eq!(actual.len(), 1);

        let account = actual.get(&1).unwrap();
        assert_eq!(account.client_id, 1);
        assert_eq!(account.locked, false);
        assert_eq!(account.available, 8.0);
        assert_eq!(account.held, 0f64);
        assert_eq!(account.total, 8.0);
    }

    #[test]
    fn test_process_transactions_dispute_chargeback() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                transaction_type: Deposit,
                client_id: 1,
                id: 1,
                amount: Some(5.5),
            },
            Transaction {
                transaction_type: Deposit,
                client_id: 1,
                id: 2,
                amount: Some(2.5),
            },
            Transaction {
                transaction_type: Dispute,
                client_id: 1,
                id: 2,
                amount: None,
            },
            Transaction {
                transaction_type: Chargeback,
                client_id: 1,
                id: 2,
                amount: None,
            }
        ];

        let actual = process_transactions(transactions);
        assert_eq!(actual.len(), 1);

        let account = actual.get(&1).unwrap();
        assert_eq!(account.client_id, 1);
        assert_eq!(account.locked, true);
        assert_eq!(account.available, 5.5);
        assert_eq!(account.held, 0f64);
        assert_eq!(account.total, 5.5);
    }
}
