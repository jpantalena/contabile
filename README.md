# Contabile

Contabile is a program written in Rust that processes transactions to determine account status and financials.

## Types of transactions

### Deposit

A deposit credits the client's account. It should increase the available and total funds.

### Withdrawal

A withdrawal debits a client's account. It should decrease the available and total funds.
If a client has insufficient funds, the withdrawal should fail.

### Dispute

A dispute is a claim that the transaction was made in error. The funds from the transaction
should be held. The available funds should decrease, the held funds should increase and the
total funds should stay the same. If the disputed transaction doesn't exist, it should be ignored.

### Resolve

A resolve is a resolution to a dispute. The available funds should increase, the held funds should
decrease, and the total funds should stay the same. If the disputed transaction doesn't exist or
the transaction isn't under dispute, it should be ignored.

### Chargeback

A chargeback is when the client reverses the disputed transaction. The held funds should
decrease, and the total funds should decrease. The client's account should be locked.
If the disputed transaction doesn't exist or the transaction isn't under dispute, it should be ignored.

## How to run

Output to std out
```
cargo run -- transactions.csv
```

Output to file
```
cargo run -- transactions.csv > accounts.csv
```

## How to run tests

```
cargo test
```

This project contains unit tests which can be found in the `.rs` files in the `src` directory.

This project also contains integration testing that can be found in the `tests` directory.


## Input File

See example file in `transactions.csv`

```
type, client, tx, amount
deposit, 1, 1, 5.0
deposit, 2, 2, 6.0
```

It can be assumed that the transaction occur chronologically in the file.

## Assumptions

* Each client only has 1 account
* There are multiple clients
* Disputes only reference deposit transactions
* Disputes, resolves, and chargeback will reference a tx with a matching client id
