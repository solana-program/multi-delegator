use crate::errors::MultiDelegatorError;

use solana_instruction::error::InstructionError;
use solana_transaction_error::TransactionError;

/// Helper function to assert that a transaction failed with the expected error
pub fn assert_error(
    result: litesvm::types::TransactionResult,
    expected_error: MultiDelegatorError,
) {
    match result {
        Ok(_) => panic!("Expected transaction to fail with {:?}", expected_error),
        Err(failed_tx) => {
            assert_eq!(
                failed_tx.err,
                TransactionError::InstructionError(
                    0,
                    InstructionError::Custom(expected_error as u32)
                ),
                "Expected error {:?}, got {:?}",
                expected_error,
                failed_tx.err
            );
        }
    }
}
