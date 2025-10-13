use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::{transfer_checked, TransferChecked},
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{Amount, Config};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        close=user,
        seeds=[b"amount", user.key().as_ref()],
        bump=amount_pda.bump,
    )]
    pub amount_pda: Account<'info, Amount>,

    #[account( seeds = [b"config"], bump=config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mint::decimals = 6,
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint=mint,
        associated_token::authority=user
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(associated_token::mint = mint, associated_token::authority = config)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: ExtraAccountMetalist Account
    #[account(
        seeds=[b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    /// CHECK: this will be the program created for the whitelist tf hook
    pub transfer_hook_program: UncheckedAccount<'info>,
    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Withdraw<'_> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let cpi_program = self.token_program.to_account_info();

        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            to: self.user_ata.to_account_info(),
            mint: self.mint.to_account_info(),
            authority: self.user.to_account_info(),
        };

        let seeds: &[&[u8]] = &[b"vault", &[self.config.bump]];
        let singer_seeds = &[seeds];

        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, singer_seeds);

        transfer_checked(cpi_ctx, amount, self.mint.decimals)?;
        Ok(())
    }
}
