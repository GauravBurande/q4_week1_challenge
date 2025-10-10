use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Amount {
    pub amount: u64,
    pub bump: u8,
}
