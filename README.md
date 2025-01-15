# Transaction Processing System 2

- push to git

This is a simple transaction processing system that processing 5 transaction types: Deposits, Withdrawals, Disputes, Resolutions, and Chargebacks.

## Installation

This project uses version `1.83.0` of rust. To install the project, run the following command:

```bash
git clone https://github.com/isaacdonaldson/tps2.git
cd tps2
cargo run -- transactions.csv > output.csv
```

## Testing

To test the project, run the following command:

```bash
cargo test
```

## Error Handling

There are 2 error types that are implemented using the `thiserror` crate: one if for the main running of the program, and the other encodes all the various transaction failure states that could be encountered. The benefit of using `thiserror` is that it is trivial to implement the `From<TransactionError>` for the higher level error enum. This allows all the errors I defined (and the others interfacing with like io::Error and csv::Error) to work nicely together.

There are some errors that one can encounter that make further progress impossible, but these are all contained in the `fn main()` and are quite obvious (file not existing, parsing error, etc...). All other "errors" are intended to allow continuation of transaction processing while being logged using `eprintln!`. That means they will not be shown when the output is piped into a file, but will still be printed to stdout.

## Decimal Type

I use the `rust_decimal` crate because using floats for money is famously a bad idea. I use the decimal crate to prevent the precision errors floats have, and can easily use the proper precision of 4 decimal places without having to use much extra overhead. There is a nice builtin deserialization ability that works well with the `csv` crate. There is one function I implemented that converts the `Decimal` type to the proper precision string, and it would be trivial to change the precision because of that.

## CSV Reading

The `csv` crate provides a very nice interface to read the csv file by providing the ability to deserialize directly into the defined struct, as well as using a `BufRead` internally for buffered reading (better on memory consumption). I took this 1-step further by wrapping the CSV reader iterator in a new srtuct, and implemeting the `Iterator` trait on that. This allowed me to chunk the input data, and save on memory consumption even more by only reading (and then freeing) 100 rows at a time (this is trivially configurable). Benchmarking showed no change in performance, while consuming ~30% less memory.

Besides saving memory, converting the `Iterator` into a `future::Stream` would be relatively straightforward if we decide to add support for asynchronous processing. Collecting data from other sources like TCP, file I/O, or databases would be quite simple since the main processing loop operates on chunks of data. We could easily pass CSV data through an asynchronous channel from multiple sources without much difficulty.

## Design Considerations

The following are some design considerations I thought are worth mentioning.

### Implementing a Transaction Trait

I thought about implementing a trait that would provide methods to check validity, apply the forward action , and revert in case it was incorrect. This would also open the door to implementing a write ahead log based system that could provide some database like transactions.

The time constraint meant this was pretty unfeasable, and for a project this small it would have made the code less readable so I opted to not do this and do the validity checks, the actions, and the reverts inline to be clearer.

### Transaction Types

I found the main processing file grew quite quickly with all the processin logic for the transaction types included, so I opted to seperate each transaction type's processing logic into their own files. This is cleaner and allows for better future maintenence.

### `BTreeMap`

Because we know that client ID's and transaction ID's are ordered numbers (by nature of being integers), we can use this fact to speed up the operations, and print ordered results, both are benefits when compared with a `HashMap`. This structure is also suitable when wrapped in other types like a `Mutex` or `RwLock`.

Another option for multi-threaded environments is to mutex shard the `BTreeMap` by converting the `Arc<Mutex<BTreeMap<T>>>` into `Arc<Vec<Mutex<BTreeMap<T>>>>` which would ideally alleviate some of the locking in a high contention system. (The operations would happen on `let tree = sharded_trees[accessing_number % num_shards]` in this scenario)

### Using the Type System

I used the type system quite a bit to provide clarity of intentions, but as well to limit the program to the defined behavior. For example, using an `Enum` for all possible transaction types allows the compiler to ensure that every transaction type is handled when using the `match` expression. In addition, creating types like `ClientList` and `TransactionManager` allows me to control what functionality is available to the programmer, restricting behaviours I don't want or adding functionality I do. This allows for better maintainability, but also allows for more correctness in the program.

### Validity Checks and Faux Atomicity

I added some validity checks to the program to ensure that the program is behaving as expected. Beyond the basic ones like checking a client or transaction exists, after each transaction I check if the client's balances are valid as explained in the document. Becuase the transactions are mutable, if the transaction is invalid, the transaction will be 'reversed' and the balances will be returned to their pre-transaction amounts. This is the most basic implementation of Atomicity.

### Serde Serialization and Deserialization

Using `serde` allows me to avoid some error prone areas with data ingestion and outputting. The serialization capability allows me to define the data type, and allow serde to handle the edge cases, where errors can easily occur. This allows me to focus on designing proper types, and a more correct system. It also integrates really well with the `csv` and `rust_decimal` crates to skip a lot of hard programming.

## Possible Extensions

### Scale and Concurreny

There was some thought put into the ease of extension should the need arise for larger scale or more concurrent processing. One is to process in chunks, whether from a file or a channel this allows the processing to be agnostic to the data ingestion & delivery. As well, implementing the Iterator myself allows me to extend it to a `future::Stream` async iterator should the need arise. It would be straigtforward to read from many TCP sockets or open connections and process the list as it comes instead of waiting for it all to arrive at once. Like mentioned before, the data structures used are very compatible with being put inside a `Arc<Mutex<T>>` so they can then be used in a multi-threaded/async environment, as well as other improvements like mutex sharding. If there is gonna be high contention, then the mutable `ClientList` and `TransactionManager` could be converted to an async task/thread that communicates through a channel quite easily.

### Basic Atomicity

Since this is financial data, data integrity is very important. The first extension I would add is atomic operations. Exchanging money with another account is a great example of an operation that is all or none, and I think that it would not be super hard to get some basic functionality. Since I already implemented the revert ability on the transactions, all that would be needed (alongside refactoring of the code to split the revert action into a function/trait impl) would be to have a log of operations for an ongoing "atomic transaction", and at the time it completes, it gets commited to the main data.

## Assumptions

### Failed Transactions continue program

If a transaction fails (ex: client has insufficient funds for a withdrawal), then the program should continue to process the rest of the transactions, only the failed transaction is skipped.

### Dispute, Resolve, and Chargebacks only occur on Deposit transactions

I assumed that disputes, resolutions, and chargebacks only occur on deposits. This is a reasonable assumption, as there is no clear way to handle these on other transaction types.

### Frozen Account Prevents Activity

I assumed that a frozen account prevents anymore transactions from being processed on it. So all 5 transaction types would be ignored for that account.

### Accounts can be created

I assumed that on failed transactions, the account can still be created and it will have no effect on the output as long as the transaction did not effect the account at all (i.e. Account exists but balances are all 0).
