use std::process;

use tps2::{
    clients::ClientList,
    errors::TpsError,
    transactions::{manager::TransactionManager, process::process_transactions},
    CsvChunkedReader,
};

// This is how many transactions we will read at a time,
// the ideal configuration will depend on the characteristics of the problem
const CHUNK_SIZE: usize = 100;

fn main() {
    let mut args = std::env::args();
    let _program_name = args.next();

    let Some(filename) = args.next() else {
        eprintln!("Usage: cargo run -- <input_file.csv>");
        process::exit(1);
    };

    let incoming_transactions = match CsvChunkedReader::new(&filename, CHUNK_SIZE) {
        Ok(transactions) => transactions,
        Err(TpsError::IoError(err)) => {
            eprintln!("Error occurred when reading {}: {}", filename, err);
            process::exit(1);
        }
        Err(TpsError::CsvError(err)) => {
            eprintln!("CSV parsing error encountered in {}: {}", filename, err);
            process::exit(1);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            process::exit(1);
        }
    };

    let mut clients = ClientList::new();
    let mut transactions = TransactionManager::new();

    for chunk in incoming_transactions {
        let chunk = match chunk {
            Ok(chunk) => chunk,
            Err(err) => {
                eprintln!("Error reading chunk: {}", err);
                process::exit(1);
            }
        };

        if let Err(err) = process_transactions(chunk, &mut clients, &mut transactions) {
            eprintln!("Error processing transactions: {}", err);
            process::exit(1);
        }
    }

    println!("{clients}");
}
