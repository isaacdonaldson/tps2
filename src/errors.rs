use crate::transactions::TransactionError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TpsError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("CSV error: {0}")]
    CsvError(#[from] csv::Error),

    #[error("Transaction error: {0}")]
    TransactionError(#[from] TransactionError),
}
