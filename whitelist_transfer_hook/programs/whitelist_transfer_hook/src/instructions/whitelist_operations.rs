use anchor_lang::prelude::*;

use crate::error::WhitelistError;
use crate::Whitelist;

#[derive(Accounts)]
#[instruction(token_account: Pubkey)]
pub struct WhitelistOperations<'info> {
    #[account(
        mut,
        //address = <the admin address> (to be more strict)
    )]
    pub admin: Signer<'info>,

    #[account(
        init_if_needed,
        payer = admin,
        space= 8 + Whitelist::INIT_SPACE,
        seeds=[b"whitelist", token_account.key().as_ref()],
        bump
    )]
    pub whitelist: Account<'info, Whitelist>,

    pub system_program: Program<'info, System>,
}

impl WhitelistOperations<'_> {
    pub fn add_to_whitelist(
        &mut self,
        token_account: Pubkey,
        bumps: &WhitelistOperationsBumps,
    ) -> Result<()> {
        if self.whitelist.address == token_account {
            return err!(WhitelistError::AlreadyWhitelisted);
        }

        self.whitelist.set_inner(Whitelist {
            address: token_account,
            bump: bumps.whitelist,
        });
        Ok(())
    }

    pub fn remove_from_whitelist(&mut self, token_account: Pubkey) -> Result<()> {
        if self.whitelist.address != token_account {
            return err!(WhitelistError::AlreadyWhitelisted);
        }

        self.whitelist.close(self.admin.to_account_info())?;
        Ok(())
    }
}
