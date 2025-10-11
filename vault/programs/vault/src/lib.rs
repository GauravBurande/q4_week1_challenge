pub mod constants;
pub mod error;
pub mod instructions;
pub mod state;
pub mod tests;

use anchor_lang::prelude::*;

pub use constants::*;
pub use instructions::*;
pub use state::*;

declare_id!("6Ttxuh7AqpLQMkVS9xCAu6ncbsyFmq8f3EZVhskoQ3m4");

#[program]
pub mod vault {
    use super::*;

    pub fn initialize_vault(ctx: Context<Initialize>) -> Result<()> {
        ctx.accounts.init_vault(&ctx.bumps)
    }

    pub fn mint(ctx: Context<MintToken>, amount: u64) -> Result<()> {
        ctx.accounts.mint_token(amount)
    }
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        ctx.accounts.withdraw(amount)
    }
    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        ctx.accounts.deposit(amount, &ctx.bumps)
    }
}
