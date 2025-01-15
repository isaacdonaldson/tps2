use rust_decimal::Decimal;

pub mod clients;
pub mod errors;
pub mod transactions;

use errors::TpsError;
use transactions::Transaction;

const DECIMAL_PRECISION: u32 = 4;

pub struct CsvChunkedReader {
    entries: csv::DeserializeRecordsIntoIter<std::fs::File, Transaction>,
    chunk_size: usize,
}

impl CsvChunkedReader {
    pub fn new(filename: &str, chunk_size: usize) -> Result<Self, TpsError> {
        let file = std::fs::File::open(filename)?;
        let reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .has_headers(true)
            .from_reader(file);

        Ok(Self {
            entries: reader.into_deserialize(),
            chunk_size,
        })
    }
}

// This allows the caller to iterate over the chunks of transactions
// in a more memory efficient way. Also it's easier to adopt for
// async/multi-threaded processing if needed
impl Iterator for CsvChunkedReader {
    type Item = Result<Vec<Transaction>, TpsError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chunk = Vec::with_capacity(self.chunk_size);

        for record in self.entries.by_ref() {
            let txn = match record {
                Ok(txn) => txn,
                Err(err) => return Some(Err(TpsError::CsvError(err))),
            };

            chunk.push(txn);

            if chunk.len() == self.chunk_size {
                return Some(Ok(chunk));
            }
        }

        if chunk.is_empty() {
            None
        } else {
            Some(Ok(chunk))
        }
    }
}

// This reads the entire CSV file into memory, which is not ideal for large files
// but is a better API for the user when reading smaller files.
// This is not currently being used
pub fn read_whole_csv(filename: &str) -> Result<Vec<Transaction>, TpsError> {
    let reader = CsvChunkedReader::new(filename, 100)?;

    let mut transactions: Vec<Transaction> = Vec::new();

    for txn in reader {
        let txn = txn?;
        transactions.extend(txn);
    }

    Ok(transactions)
}

// I am choosing to have this function instead of creating a NewType for Decimal
// because I would need to implement a lot of the std::ops traits for the NewType to be useful
// and that is not worth it to just get the one easier displaying function
pub fn decimal_to_string(decimal: Decimal) -> String {
    format!("{:.4}", decimal.round_dp(DECIMAL_PRECISION))
}
