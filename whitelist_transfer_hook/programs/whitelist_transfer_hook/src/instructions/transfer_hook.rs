use std::cell::RefMut;

use anchor_lang::prelude::*;
use anchor_spl::token_2022::spl_token_2022::extension::transfer_hook::TransferHookAccount;
use anchor_spl::token_2022::spl_token_2022::extension::{
    BaseStateWithExtensionsMut, PodStateWithExtensionsMut,
};
use anchor_spl::token_2022::spl_token_2022::pod::PodAccount;
use anchor_spl::token_interface::{Mint, TokenAccount};

use crate::error::WhitelistError;
use crate::Whitelist;

#[derive(Accounts)]
pub struct TransferHook<'info> {
    #[account(
        token::mint = mint,
        token::authority = owner
    )]
    pub source_token: InterfaceAccount<'info, TokenAccount>,

    #[account(mint::decimals = 6)]
    pub mint: InterfaceAccount<'info, Mint>,

    #[account(
        token::mint = mint
    )]
    pub destination_token: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: source token account owner, can be SystemAccount or PDA owned by another program
    pub owner: UncheckedAccount<'info>,

    /// CHECK: ExtraAccountMetalist Account
    #[account(
        seeds=[b"extra-account-metas", mint.key().as_ref()],
        bump
    )]
    pub extra_account_meta_list: UncheckedAccount<'info>,

    #[account(
        seeds=[b"whitelist", source_token.key().as_ref()],
        bump=whitelist.bump
    )]
    pub whitelist: Account<'info, Whitelist>,
}

impl TransferHook<'_> {
    /// This function is called when the transfer hook is executed.
    pub fn transfer_hook(&mut self, _amount: u64) -> Result<()> {
        // Entry log for debugging
        msg!(
            "transfer_hook: invoked. source={}, destination={}, owner={}, mint={}, amount={}",
            self.source_token.key(),
            self.destination_token.key(),
            self.owner.key(),
            self.mint.key(),
            _amount
        );

        // Fail this instruction if it is not called from within a transfer hook
        self.check_is_transferring()?;
        msg!("transfer_hook: passed check_is_transferring");

        if self.whitelist.address != self.source_token.key() {
            msg!(
                "transfer_hook: owner {} is not whitelisted (whitelist.address={})",
                self.source_token.key(),
                self.whitelist.address
            );
            return err!(WhitelistError::NotWhitelisted);
        }

        msg!("transfer_hook: whitelist check passed for owner {}", self.owner.key());

        Ok(())
    }

    /// Checks if the transfer hook is being executed during a transfer operation.
    pub fn check_is_transferring(&mut self) -> Result<()> {
        // Ensure that the source token account has the transfer hook extension enabled
        let source_token_info = self.source_token.to_account_info();

        msg!("check_is_transferring: source_token_account={}", source_token_info.key());

        let mut account_data_ref: RefMut< &mut [u8]> = match source_token_info.try_borrow_mut_data() {
            Ok(d) => {
                msg!("check_is_transferring: borrowed account data, len={}", d.len());
                d
            }
            Err(e) => {
                msg!("check_is_transferring: failed to borrow account data: {:?}", e);
                return Err(e.into());
            }
        };

        let mut account = PodStateWithExtensionsMut::<PodAccount>::unpack(*account_data_ref)?;
        msg!("check_is_transferring: unpacked PodAccount with extensions");

        let account_extension = account.get_extension_mut::<TransferHookAccount>()?;
        msg!(
            "check_is_transferring: transfer hook extension transferring={}",
            bool::from(account_extension.transferring)
        );

        if !bool::from(account_extension.transferring) {
            msg!("check_is_transferring: NOT transferring - returning NotTransferring error");
            return err!(crate::error::WhitelistError::NotTransferring);
        }
        msg!("check_is_transferring: transferring flag set, continuing");
        Ok(())
    }
}
