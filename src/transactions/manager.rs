use crate::transactions::{Transaction, TransactionId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// This is very similar to the ClientList struct in clients.rs
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct TransactionManager(HashMap<TransactionId, Transaction>);

impl TransactionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, transaction: Transaction) {
        self.0.insert(transaction.tx_id, transaction);
    }

    pub fn contains(&self, tx_id: &TransactionId) -> bool {
        self.0.contains_key(tx_id)
    }

    pub fn get(&self, tx_id: &TransactionId) -> Option<&Transaction> {
        self.0.get(tx_id)
    }

    pub fn get_mut(&mut self, tx_id: &TransactionId) -> Option<&mut Transaction> {
        self.0.get_mut(tx_id)
    }
}
