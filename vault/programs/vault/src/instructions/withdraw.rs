use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke_signed},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::spl_token_2022::instruction::transfer_checked,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{error::VaultError, Amount, Config};

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
    #[account[mut]]
    pub extra_account_meta_list: UncheckedAccount<'info>,
    /// CHECK: ExtraAccountMetalist Account
    #[account[mut]]
    pub whitelist: UncheckedAccount<'info>,
    /// CHECK: this will be the program created for the whitelist tf hook
    #[account[mut]]
    pub transfer_hook_program: UncheckedAccount<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

impl Withdraw<'_> {
    pub fn withdraw(&mut self, amount: u64) -> Result<()> {
        let user_deposited_amount = self.amount_pda.amount;

        require!(
            user_deposited_amount <= amount,
            VaultError::AmountExceededUrDeposit
        );

        let seeds: &[&[u8]] = &[b"vault", &[self.config.bump]];
        let singer_seeds = &[seeds];

        // TODO: deduct the amount withdrawn from the pda
        Ok(())
    }
}

pub fn token_transfer_with_extra_and_signer_seeds<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    extra_account_meta_list: &AccountInfo<'info>,
    hook_program: &AccountInfo<'info>,
    whitelist: &AccountInfo<'info>,
    signer_seeds: &[&[&[u8]]],
    amount: u64,
    decimals: u8,
) -> Result<()> {
    // Create the list of accounts in order
    let mut accounts = vec![
        AccountMeta::new(*from.key, false),
        AccountMeta::new_readonly(*mint.key, false),
        AccountMeta::new(*to.key, false),
        AccountMeta::new_readonly(*authority.key, true),
    ];
    accounts.push(AccountMeta::new(*extra_account_meta_list.key, false));
    accounts.push(AccountMeta::new(*whitelist.key, false));
    accounts.push(AccountMeta::new(hook_program.key(), false));

    // Build the transfer_checked instruction
    let ix = transfer_checked(
        token_program.key,
        from.key,
        mint.key,
        to.key,
        authority.key,
        &[], // multisigners if any
        amount,
        decimals,
    )?;

    // Manually override accounts of the instruction with full list including extras
    let instruction = Instruction {
        program_id: *token_program.key,
        accounts,
        data: ix.data,
    };

    invoke_signed(
        &instruction,
        &[
            from.clone(),
            mint.clone(),
            to.clone(),
            authority.clone(),
            extra_account_meta_list.clone(),
            whitelist.clone(),
            hook_program.clone(),
        ],
        signer_seeds,
    )?;

    Ok(())
}
