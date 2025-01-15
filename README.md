# Transaction Processing System 2

This is a simple transaction processing system that processes five transaction types:
Deposits, Withdrawals, Disputes, Resolutions, and Chargebacks.

## Installation

This project used version `1.83.0` of rust, so other versions are untested. To install the project, run the following command:

```bash
git clone https://github.com/isaacdonaldson/tps2.git
cd tps2
cargo run -- transactions.csv > accounts.csv
```

## Testing

To test the project, run the following command:

```bash
cargo test
```

## Error Handling

There are two error types implemented using the `thiserror` crate. One covers the main program runtime, and the other encodes all possible transaction failure states. By using `thiserror`, it becomes trivial to implement `From<TransactionError>` for the higher-level error enumeration, allowing seamless interplay with other errors like `io::Error` and `csv::Error`.

Some errors (e.g., missing files or parsing problems) make further processing impossible and are handled in `fn main()`. All other errors are logged to stderr using `eprintln!` without affecting standard output.

## Decimal Type

The `rust_decimal` crate is used instead of floating-point numbers because floating-point precision issues make them unsuitable for monetary values. The `Decimal` type avoids precision errors and can easily provide the required four decimal places with minimal overhead. Serialization and deserialization are straightforward with the `csv` crate (because the provided compatibility functionality), and precision can be centrally managed when converting `Decimal` to a string.

## CSV Reading

The `csv` crate provides an easy way to deserialize CSV data directly into a defined struct. It uses `BufRead` for buffered reading, improving memory usage. To optimize further, I wrapped the CSV reader in a custom iterator that processes data in chunks of 100 rows (configurable), freeing memory after each chunk. Benchmarking showed reduced memory usage by roughly 30%.

This approach also makes it simpler to convert the iterator into a `future::Stream` if we decide to add asynchronous processing. The chunk-based design allows data from multiple sources (e.g., TCP, file I/O, databases) to be collected in an asynchronous pipeline without changing the core processing flow.

## Design Considerations

### Implementing a Transaction Trait

I considered creating a common trait for every transaction type, including methods to validate, apply, and revert transactions. This would allow the logic for rolling back changes to be centralized (and abstracted for a future atomic transaction extension), but for a project of this scope and under a time constraint I opted for inline checks, actions, and reversions to keep the code more straightforward and manageable.

### Transaction Types

Originally, the main processing file grew rapidly with all the transaction logic. To improve maintainability and clarity, I separated each transaction type’s logic into its own file. These are found under the `src/transactions/logic` directory.

### `HashMap` over `BTreeMap`

A `HashMap` is used for storing client and transaction IDs because order is irrelevant and `HashMap` generally provides O(1) lookups compared to the O(log n) lookups of a `BTreeMap`. This approach also works well when combined with threading primitives like `RwLock` or `Arc<Mutex>`.

For multi-threaded scenarios with high contention, another strategy is "mutex sharding" by splitting the map into multiple smaller locks (`Arc<Vec<Mutex<HashMap<T>>>>`). This can reduce locking overhead in highly concurrent environments.

### Utilizing the Type System

Enums for transaction types ensure that every case is handled in `match` expressions. Custom data types like `ClientList` and `TransactionManager` offer clearer organization and control over permitted operations, improving code clarity and reducing potential errors. Using the new type pattern like this helps clarify a developers intentions and when used more extensively, can resulting in more correct software.

### Validity Checks and Basic Atomicity

After each transaction, the system checks whether the client’s balances remain valid. If a transaction is invalid, it is reverted immediately, returning balances to their values before the transaction. This provides minimal atomicity for the operations in the system.

### Serde Serialization and Deserialization

Using `serde` helps avoid common pitfalls with data ingestion and output by automatically handling both serialization and deserialization. This integration with the `csv` and `rust_decimal` crates saves effort and reduces the likelihood of errors, letting me focus on designing correct types and logic.

## Possible Extensions

### Scale and Concurrency

The system is designed to support incremental scaling and concurrency. Chunk-based data processing that is source-agnostic, along with custom iterator functionality (would become `future::Stream` when async), can be easily adapted to many asynchronous source streams. Current data structures can be extended (using `Arc<Mutex<T>>`), and mutex sharding can reduce lock contention for high-traffic scenarios, while using channels to message pass to different tasks/threads would remove most lock contention if further extension steps were needed.

### Basic Atomicity

Financial data requires strong data integrity. By expanding on the existing revert functionality, we could implement more robust atomic operations. For example, transferring funds across two accounts would require both changes to succeed or neither to apply; essentially a transaction rollback if any step fails like the all or none approach seen often in ACID databases. A write-ahead log or similar mechanism would be a logical next step.

## Assumptions

### Failed Transactions and Program Continuation

When a transaction (e.g., withdrawal with insufficient funds) fails, the program continues to process subsequent transactions. Only the failed transaction is skipped.

### Dispute, Resolve, and Chargebacks only occur on Deposit transactions

I assumed that Disputes, resolutions, and chargebacks apply strictly to deposits. How to handle these operations for other transaction types is not clear, and thus it makes sense to ignore them.

### Frozen Account Prevents Activity

I assumed that if an account becomes frozen, no transactions of any type are processed for that account thereafter.

### Accounts can be created

Even if a transaction fails, I assumed the account still gets created (with zero balances) so it appears in the final output.
