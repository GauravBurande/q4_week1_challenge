use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::Config;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(init, payer = admin, space = 8 + Config::INIT_SPACE, seeds = [b"config"], bump)]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = admin,
        mint::authority = admin,
        mint::decimals = 6,
        mint::token_program = token_program,
        extensions::transfer_hook::authority = admin,
        extensions::transfer_hook::program_id = transfer_hook_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: this will be the program created for the whitelist tf hook
    pub transfer_hook_program: UncheckedAccount<'info>,

    #[account(init, payer = admin, associated_token::mint = mint, associated_token::authority = config)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Initialize<'_> {
    pub fn init_vault(&mut self, bumps: &InitializeBumps) -> Result<()> {
        self.config.set_inner(Config {
            admin: self.admin.key(),
            vault: self.vault.key(),
            mint: self.mint.key(),
            bump: bumps.config,
        });
        Ok(())
    }
}
