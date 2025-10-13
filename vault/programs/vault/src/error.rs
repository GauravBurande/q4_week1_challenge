use anchor_lang::prelude::*;

#[error_code]
pub enum VaultError {
    #[msg("You don't have this much of amount deposited in the vault!")]
    AmountExceededUrDeposit,
}
