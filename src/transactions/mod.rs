use crate::clients::ClientId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod logic;
pub mod manager;
pub mod process;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TransactionId(u32);

impl From<u32> for TransactionId {
    fn from(id: u32) -> Self {
        Self(id)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    #[serde(rename = "client")]
    pub client_id: ClientId,
    #[serde(rename = "tx")]
    pub tx_id: TransactionId,
    pub amount: Option<Decimal>, // using this Decimal type allows for desired precision
    #[serde(default)] // useful for seeing disputes, defaults to false
    pub in_dispute: bool,
}

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Missing Transaction ID")]
    MissingTransactionId,

    #[error("Client {0} is locked")]
    LockedClient(ClientId),

    #[error("Transaction is missing a client")]
    MissingClient(ClientId),

    #[error("Transaction is missing an amount")]
    MissingAmount,

    #[error("Transaction amount is invalid")]
    InvalidAmount,

    #[error("Invalid transaction")]
    InvalidTransaction,

    #[error("Reverting invalid transaction")]
    RevertInvalidTransaction,

    #[error("Insufficient funds for client {0}")]
    InsufficientFunds(ClientId),
}
