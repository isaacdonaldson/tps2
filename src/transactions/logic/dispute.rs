use crate::{
    clients::{ClientId, ClientList},
    transactions::{
        manager::TransactionManager, Transaction, TransactionError, TransactionId, TransactionType,
    },
};

#[derive(Debug)]
pub struct Dispute {
    tx_id: TransactionId,
    client_id: ClientId,
}

impl Dispute {
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

        let dispute_amount = transaction.amount.ok_or(TransactionError::InvalidAmount)?;

        // It does not make sense to dispute anything other than a deposit
        if transaction.tx_type == TransactionType::Deposit {
            if client.available < dispute_amount {
                return Err(TransactionError::InsufficientFunds(self.client_id));
            }

            client.available -= dispute_amount;
            client.held += dispute_amount;

            transaction.in_dispute = true;

            // Reverting the changes if the transaction is incorrect
            if !client.is_valid() {
                client.available += dispute_amount;
                client.held -= dispute_amount;
                transaction.in_dispute = false;

                return Err(TransactionError::RevertInvalidTransaction);
            }
        }

        Ok(())
    }
}
