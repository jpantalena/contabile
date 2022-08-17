use std::error::Error;
use std::{env, process};

use csv::{ReaderBuilder, Trim, WriterBuilder};
use models::{Account, Transaction, TransactionType};
use processor::process_transactions;

mod models;
mod processor;

#[macro_use]
extern crate log;

// From a filepath, read a .csv file for transaction data
// Deserialize data to Transaction structs
fn get_csv_transactions_from_filepath(path: &str) -> Result<Vec<Transaction>, Box<dyn Error>> {
    let mut reader = ReaderBuilder::new().trim(Trim::All).from_path(path)?;
    let mut transactions: Vec<Transaction> = Vec::new();
    for item in reader.deserialize() {
        let transaction: Transaction = item?;
        debug!("{:?}", transaction);
        transactions.push(transaction);
    }
    Ok(transactions)
}

fn main() {
    env_logger::init();
    debug!("Contabile starting up");
    let args: Vec<String> = env::args().collect();

    // TODO: add input validation and error handling
    let transactions_filepath = args.get(1).unwrap();
    debug!("{:?}", transactions_filepath);

    // Read csv input file
    let transactions: Vec<Transaction> =
        match get_csv_transactions_from_filepath(transactions_filepath) {
            Ok(transactions) => {
                debug!(
                    "Success reading csv input file csv file, {:?} transactions",
                    transactions.len()
                );
                transactions
            }
            Err(err) => {
                error!("Error reading csv input file: {:?}", err);
                process::exit(1);
            }
        };

    let account_map = process_transactions(transactions);

    // Print output to std out in csv format
    println!("{}", "client,available,held,total,locked");
    for item in account_map {
        println!("{}", item.1.to_csv());
    }
}
