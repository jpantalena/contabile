# Contabile

Contabile is a program written in Rust that processes transactions to determine account status and financials.

## How to run

```
cargo run -- transactions.csv
```

## How to run tests

```
cargo test
```

## Input File

See example file in `transactions.csv`

You can assume the transactions occur chronologically in the file, so if transaction b appears after a in the input file then 
you can assume b occurred chronologically after a.


