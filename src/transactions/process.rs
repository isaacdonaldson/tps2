use crate::{clients::ClientList, errors::TpsError};

use super::{
    logic::{
        chargeback::Chargeback, deposit::Deposit, dispute::Dispute, resolve::Resolve,
        withdrawal::Withdrawal,
    },
    manager::TransactionManager,
    Transaction, TransactionType,
};

pub fn process_transactions(
    transactions: Vec<Transaction>,
    clients: &mut ClientList,
    transaction_manager: &mut TransactionManager,
) -> Result<(), TpsError> {
    for transaction in transactions {
        let operation_result = match &transaction.tx_type {
            TransactionType::Deposit | TransactionType::Withdrawal
                if transaction_manager.contains(&transaction.tx_id) =>
            {
                eprintln!(
                    "Duplicate transaction id found: {:?}, skipping",
                    transaction.tx_id
                );
                Ok(())
            }

            TransactionType::Deposit => Deposit::new(&transaction)?.process(clients),
            TransactionType::Withdrawal => Withdrawal::new(&transaction)?.process(clients),
            TransactionType::Dispute => {
                Dispute::new(&transaction)?.process(clients, transaction_manager)
            }
            TransactionType::Resolve => {
                Resolve::new(&transaction)?.process(clients, transaction_manager)
            }
            TransactionType::Chargeback => {
                Chargeback::new(&transaction)?.process(clients, transaction_manager)
            }
        };

        match operation_result {
            Ok(_) => (),
            // All errors can continue processing
            Err(e) => {
                eprintln!("Error processing transaction: {transaction:?}, error: {e}");
            }
        };

        // only store the transaction if it is a deposit or withdrawal
        if transaction.tx_type == TransactionType::Deposit
            || transaction.tx_type == TransactionType::Withdrawal
        {
            transaction_manager.insert(transaction);
        }
    }

    Ok(())
}
