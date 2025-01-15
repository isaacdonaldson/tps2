use crate::{
    clients::{ClientId, ClientList},
    transactions::{
        manager::TransactionManager, Transaction, TransactionError, TransactionId, TransactionType,
    },
};

#[derive(Debug)]
pub struct Chargeback {
    tx_id: TransactionId,
    client_id: ClientId,
}

impl Chargeback {
    pub fn new(transaction: &Transaction) -> Result<Self, TransactionError> {
        Ok(Self {
            tx_id: transaction.tx_id,
            client_id: transaction.client_id,
        })
    }

    pub fn process(
        &self,
        clients: &mut ClientList,
        transactions: &mut TransactionManager,
    ) -> Result<(), TransactionError> {
        // Doing the sanity checks before making any changes
        let client = clients
            .get_client_mut(&self.client_id)
            // Client is actually needed here
            .ok_or(TransactionError::MissingClient(self.client_id))?;

        if client.locked {
            return Err(TransactionError::LockedClient(self.client_id));
        }

        // Making the changes
        let transaction = transactions
            .get_mut(&self.tx_id)
            .ok_or(TransactionError::MissingTransactionId)?;

        let chargeback_amount = transaction.amount.ok_or(TransactionError::InvalidAmount)?;

        // Like the dispute, and resolve, only deposits can be charged back
        if transaction.tx_type == TransactionType::Deposit {
            if client.held < chargeback_amount {
                return Err(TransactionError::InsufficientFunds(self.client_id));
            }

            if !transaction.in_dispute {
                return Err(TransactionError::InvalidTransaction);
            }

            // Chargeback the amount and lock the account
            client.held -= chargeback_amount;
            client.total -= chargeback_amount;
            // Chargeback locks the client account
            client.locked = true;
            transaction.in_dispute = false;

            // Reverting the changes if the transaction is incorrect
            if !client.is_valid() {
                client.held += chargeback_amount;
                client.total += chargeback_amount;
                client.locked = false;
                transaction.in_dispute = true;

                return Err(TransactionError::RevertInvalidTransaction);
            }
        }

        Ok(())
    }
}
