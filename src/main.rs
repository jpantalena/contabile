use std::collections::HashMap;
use std::error::Error;
use std::{env, process};

use csv::{ReaderBuilder, Trim};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    id: u32,
    amount: Option<f64>,
}

impl Transaction {
    fn amount(&self) -> f64 {
        return self.amount.unwrap_or(0f64)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

impl TransactionType {
    fn as_str(&self) -> &'static str {
        match self {
            TransactionType::Deposit => "deposit",
            TransactionType::Withdrawal => "withdrawal",
            TransactionType::Dispute => "dispute",
            TransactionType::Resolve => "resolve",
            TransactionType::Chargeback => "chargeback",
        }
    }
}

#[derive(Debug)]
struct Account {
    client_id: u16,
    available: f64,
    held: f64,
    total: f64,
}

impl Account {
    fn new(client_id: &u16) -> Account {
        Account {
            client_id: client_id.to_owned(),
            available: 0f64,
            held: 0f64,
            total: 0f64,
        }
    }

    fn sum_total(&mut self) {
        self.total = self.available + self.held
    }
}

// From a filepath, read a .csv file for transaction data
// Deserialize data to Transaction structs
fn get_csv_transactions_from_filepath(path: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;
    let mut transactions: Vec<Transaction> = Vec::new();
    for item in reader.deserialize() {
        let transaction: Transaction = item?;
        println!("{:?}", transaction);
        transactions.push(transaction);
    }
    Ok(transactions)
}

fn main() {
    // TODO: add logging crate
    println!("*** Contabile *** \nStarting up!");
    let args: Vec<String> = env::args().collect();

    // TODO: add input validation and error handling
    let transactions_filepath = args.get(1).unwrap();
    println!("{:?}", transactions_filepath);

    // Read csv input file
    let transactions: Vec<Transaction> =
        match get_csv_transactions_from_filepath(transactions_filepath) {
            Ok(transactions) => {
                println!(
                    "Success reading csv input file csv file, {:?} transactions",
                    transactions.len()
                );
                transactions
            }
            Err(err) => {
                println!("Error reading csv input file: {:?}", err);
                process::exit(1);
            }
        };

    let mut account_map: HashMap<u16, Account> = HashMap::new();
    let mut transaction_history: HashMap<u32, Transaction> = HashMap::new();

    let mut apply_transaction = |acct: &mut Account, tx: Transaction| {
        match tx.transaction_type {
            TransactionType::Deposit => acct.available += tx.amount(),
            TransactionType::Withdrawal => {
                if acct.available >= tx.amount() {
                    acct.available -= tx.amount()
                }
            }
            TransactionType::Dispute => {
                if let Some(transaction) = &transaction_history.get(&tx.id) {
                    // assume the transaction is a deposit
                    acct.available -= transaction.amount();
                    acct.held += transaction.amount();
                }
            }
            TransactionType::Resolve => {
                if let Some(transaction) = &transaction_history.get(&tx.id) {
                    acct.available += transaction.amount();
                    acct.held -= transaction.amount();
                }
            }
            _ => {}
        };

        transaction_history.entry(tx.id).or_insert(tx);
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

    // For debugging purposes
    for item in account_map {
        println!("{:?}", item.1);
    }
}
