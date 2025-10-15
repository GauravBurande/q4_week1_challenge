use anchor_lang::{
    prelude::*,
    solana_program::{instruction::Instruction, program::invoke},
};
use anchor_spl::{
    associated_token::AssociatedToken,
    token_2022::spl_token_2022::instruction::transfer_checked,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::{Amount, Config};

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        payer=user,
        seeds=[b"amount", user.key().as_ref()],
        bump,
        space = 8 + Amount::INIT_SPACE
    )]
    pub amount_pda: Account<'info, Amount>,

    #[account( seeds = [b"config"], bump=config.bump)]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        mint::decimals = 6,
        mint::token_program = token_program,
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        associated_token::mint=mint,
        associated_token::authority=user,
        associated_token::token_program=token_program
    )]
    pub user_ata: InterfaceAccount<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = config,
        associated_token::token_program=token_program
    )]
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

impl Deposit<'_> {
    pub fn deposit(&mut self, amount: u64, bumps: &DepositBumps) -> Result<()> {
        self.amount_pda.set_inner(Amount {
            amount: self
                .amount_pda
                .amount
                .checked_add(amount)
                .expect("Failed to add amount, account Overflow!"),
            bump: bumps.amount_pda,
        });

        token_transfer_with_extra(
            &self.token_program.to_account_info(),
            &self.user_ata.to_account_info(),
            &self.mint.to_account_info(),
            &self.vault.to_account_info(),
            &self.user.to_account_info(),
            &self.extra_account_meta_list.to_account_info(),
            &self.transfer_hook_program.to_account_info(),
            &self.whitelist.to_account_info(),
            amount,
            self.mint.decimals,
        )?;
        Ok(())
    }
}

pub fn token_transfer_with_extra<'info>(
    token_program: &AccountInfo<'info>,
    from: &AccountInfo<'info>,
    mint: &AccountInfo<'info>,
    to: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    extra_account_meta_list: &AccountInfo<'info>,
    transfer_hook_program: &AccountInfo<'info>,
    whitelist: &AccountInfo<'info>,
    amount: u64,
    decimals: u8,
) -> Result<()> {
    // Create the list of accounts in order
    let mut accounts = vec![
        AccountMeta::new(from.key(), false),
        AccountMeta::new_readonly(mint.key(), false),
        AccountMeta::new(to.key(), false),
        AccountMeta::new_readonly(authority.key(), true),
    ];
    accounts.push(AccountMeta::new(extra_account_meta_list.key(), false));
    // accounts.push(AccountMeta::new(whitelist.key(), false));
    accounts.push(AccountMeta::new(transfer_hook_program.key(), false));

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
        program_id: token_program.key(),
        accounts,
        data: ix.data,
    };

    invoke(
        &instruction,
        &[
            from.clone(),
            mint.clone(),
            to.clone(),
            authority.clone(),
            extra_account_meta_list.clone(),
            whitelist.clone(),
            transfer_hook_program.clone(),
        ],
    )?;

    Ok(())
}
