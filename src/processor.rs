use crate::{Account, ProcessorError, Transaction, TransactionType};
use std::collections::HashMap;

pub fn process_transactions(transactions: Vec<Transaction>) -> HashMap<u16, Account> {
    let mut account_map: HashMap<u16, Account> = HashMap::new();
    let mut tx_history: HashMap<u32, Transaction> = HashMap::new();
    let mut dispute_history: HashMap<u32, Transaction> = HashMap::new();

    for tx in transactions {
        if let Some(account) = account_map.get_mut(&tx.client_id) {
            if let Err(e) = apply_transaction(account, &tx, &mut tx_history, &mut dispute_history) {
                error!("Error processing transaction: {:?}. {}", tx, e.message);
            }
        } else {
            let mut account = Account::new(&tx.client_id);
            let client_id = tx.client_id;
            match apply_transaction(&mut account, &tx, &mut tx_history, &mut dispute_history) {
                Ok(()) => {
                    account_map.insert(client_id, account);
                }
                Err(e) => error!("Error processing transaction: {:?}. {}", tx, e.message),
            }
        }
    }
    account_map
}

fn apply_transaction(
    acct: &mut Account,
    tx: &Transaction,
    tx_history: &mut HashMap<u32, Transaction>,
    dispute_history: &mut HashMap<u32, Transaction>,
) -> Result<(), ProcessorError> {
    match tx.transaction_type {
        TransactionType::Deposit => {
            acct.available += tx.amount();
            tx_history.entry(tx.id).or_insert_with(|| tx.clone());
        }
        TransactionType::Withdrawal => {
            if acct.available >= tx.amount() {
                acct.available -= tx.amount();
                tx_history.entry(tx.id).or_insert_with(|| tx.clone());
            } else {
                return Err(ProcessorError::new(
                    "Insufficient funds for withdrawal".into(),
                ));
            }
        }
        TransactionType::Dispute => {
            let historical_tx_amount = find_transaction_amount(&tx.id, tx_history)?;
            acct.available -= historical_tx_amount;
            acct.held += historical_tx_amount;
            dispute_history.entry(tx.id).or_insert_with(|| tx.clone());
        }
        TransactionType::Resolve => {
            let historical_tx_amount = find_transaction_amount(&tx.id, tx_history)?;
            if disputed_transaction_exists(&tx.id, dispute_history)? {
                acct.available += historical_tx_amount;
                acct.held -= historical_tx_amount;
                dispute_history.remove(&tx.id);
            }
        }
        TransactionType::Chargeback => {
            let historical_tx_amount = find_transaction_amount(&tx.id, tx_history)?;
            if disputed_transaction_exists(&tx.id, dispute_history)? {
                acct.held -= historical_tx_amount;
                acct.locked = true;
                dispute_history.remove(&tx.id);
            }
        }
    };

    acct.sum_total();
    Ok(())
}

fn find_transaction_amount(
    tx_id: &u32,
    tx_history: &mut HashMap<u32, Transaction>,
) -> Result<f64, ProcessorError> {
    match &tx_history.get(tx_id) {
        Some(historical_tx) => Ok(historical_tx.amount()),
        None => Err(ProcessorError::new(
            "Unable to find historical transaction".into(),
        )),
    }
}

fn disputed_transaction_exists(
    tx_id: &u32,
    dispute_history: &mut HashMap<u32, Transaction>,
) -> Result<bool, ProcessorError> {
    match dispute_history.contains_key(tx_id) {
        true => Ok(true),
        false => Err(ProcessorError::new(
            "Unable to find disputed transaction".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TransactionType::{Chargeback, Deposit, Dispute, Resolve, Withdrawal};

    #[test]
    fn test_process_transactions_deposit_withdrawal() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                client_id: 1,
                transaction_type: Deposit,
                amount: Some(3.0123),
            },
            Transaction {
                id: 2,
                client_id: 1,
                transaction_type: Withdrawal,
                amount: Some(1.8761),
            },
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
            },
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
            },
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

    #[test]
    fn test_process_transactions_withdrawal_without_account() {
        let transactions: Vec<Transaction> = vec![Transaction {
            id: 1,
            client_id: 1,
            transaction_type: Withdrawal,
            amount: Some(3.0000),
        }];

        let actual = process_transactions(transactions);
        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn test_process_transactions_withdrawal_too_large() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                client_id: 1,
                transaction_type: Deposit,
                amount: Some(5.0000),
            },
            Transaction {
                id: 2,
                client_id: 1,
                transaction_type: Withdrawal,
                amount: Some(7.0000),
            },
        ];

        let actual = process_transactions(transactions);
        assert_eq!(actual.len(), 1);

        let account = actual.get(&1).unwrap();
        assert_eq!(account.client_id, 1);
        assert_eq!(account.locked, false);
        assert_eq!(account.available, 5.0000);
        assert_eq!(account.held, 0f64);
        assert_eq!(account.total, 5.0000);
    }

    #[test]
    fn test_process_transactions_dispute_transaction_not_found() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                client_id: 1,
                transaction_type: Deposit,
                amount: Some(5.0000),
            },
            Transaction {
                id: 2,
                client_id: 1,
                transaction_type: Dispute,
                amount: None,
            },
        ];

        let actual = process_transactions(transactions);
        assert_eq!(actual.len(), 1);

        let account = actual.get(&1).unwrap();
        assert_eq!(account.client_id, 1);
        assert_eq!(account.locked, false);
        assert_eq!(account.available, 5.0000);
        assert_eq!(account.held, 0f64);
        assert_eq!(account.total, 5.0000);
    }

    #[test]
    fn test_process_transactions_resolve_disputed_transaction_not_found() {
        let transactions: Vec<Transaction> = vec![
            Transaction {
                id: 1,
                client_id: 1,
                transaction_type: Deposit,
                amount: Some(5.0000),
            },
            Transaction {
                id: 1,
                client_id: 1,
                transaction_type: Resolve,
                amount: None,
            },
        ];

        let actual = process_transactions(transactions);
        assert_eq!(actual.len(), 1);

        let account = actual.get(&1).unwrap();
        assert_eq!(account.client_id, 1);
        assert_eq!(account.locked, false);
        assert_eq!(account.available, 5.0000);
        assert_eq!(account.held, 0f64);
        assert_eq!(account.total, 5.0000);
    }
}
