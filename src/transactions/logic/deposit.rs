use rust_decimal::Decimal;

use crate::{
    clients::{ClientId, ClientList},
    transactions::{Transaction, TransactionError},
};

#[derive(Debug)]
pub struct Deposit {
    client_id: ClientId,
    amount: Decimal,
}

impl Deposit {
    pub fn new(transaction: &Transaction) -> Result<Self, TransactionError> {
        let amount = transaction.amount.ok_or(TransactionError::MissingAmount)?;

        Ok(Self {
            client_id: transaction.client_id,
            amount,
        })
    }

    pub fn process(&self, clients: &mut ClientList) -> Result<(), TransactionError> {
        // Doing the sanity checks before making any changes
        let client = clients.get_or_create_client(&self.client_id);

        let deposit_amount = self.amount;
        if deposit_amount < Decimal::from(0) {
            return Err(TransactionError::InvalidAmount);
        }

        if client.locked {
            return Err(TransactionError::LockedClient(self.client_id));
        }

        // Making the changes
        client.available += deposit_amount;
        client.total += deposit_amount;

        // Reverting the changes if the transaction is incorrect
        if !client.is_valid() {
            client.available -= deposit_amount;
            client.total -= deposit_amount;
            return Err(TransactionError::RevertInvalidTransaction);
        }

        Ok(())
    }
}
