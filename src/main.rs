use std::error::Error;
use std::{env, io, process};

use csv::{ReaderBuilder, Trim};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Transaction {
    #[serde(rename = "type")]
    transaction_type: TransactionType,
    #[serde(rename = "client")]
    client_id: u16,
    #[serde(rename = "tx")]
    transaction_id: u32,
    amount: f64,
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
    let transactionss: Vec<Transaction> = match get_csv_transactions_from_filepath(transactions_filepath) {
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


}
