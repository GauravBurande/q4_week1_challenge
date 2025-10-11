#[cfg(test)]
mod tests {

    use anchor_lang::{system_program, InstructionData, ToAccountMetas};
    use anchor_spl::associated_token::{self, spl_associated_token_account};
    use litesvm::LiteSVM;
    use litesvm_token::spl_token::ID as TOKEN_PROGRAM;
    use solana_instruction::Instruction;
    use solana_keypair::Keypair;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::{pubkey, Pubkey};
    use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM;
    use solana_signer::Signer;
    use std::str::FromStr;

    use crate::vault;

    pub struct TestEnv {
        pub svm: LiteSVM,
        pub admin: Keypair,
        pub mint2022: Keypair,
        pub config: Pubkey,
        pub vault: Pubkey,
    }
    static PROGRAM_ID: Pubkey = crate::ID;

    const ASSOCIATED_TOKEN_PROGRAM: Pubkey = spl_associated_token_account::ID;

    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let admin = Keypair::new();
        let mint2022 = Keypair::new();

        svm.airdrop(&admin.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to admin.");

        let config = Pubkey::find_program_address(&[b"config"], &PROGRAM_ID).0;
        let vault = associated_token::get_associated_token_address(&config, &mint2022.pubkey());

        TestEnv {
            svm,
            admin,
            mint2022,
            config,
            vault,
        }
    }

    fn build_init_instruction() -> Instruction {
        let TestEnv {
            svm,
            admin,
            mint2022,
            config,
            vault,
        } = setup();
        let transfer_hook_program =
            Pubkey::from_str("Fhtnxy2v3DLuLxDMPMCWkNY1Qk4Nfp7HMCYxJvLjwzQp").unwrap(); // todo: enter the correct tf hook program, this one is dummy
        let init_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Initialize {
                admin: admin.pubkey(),
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM,
                config,
                vault,
                mint: mint2022.pubkey(),
                transfer_hook_program,
                system_program: SYSTEM_PROGRAM,
                token_program: TOKEN_PROGRAM,
            }
            .to_account_metas(None),
            data: crate::instruction::InitializeVault {}.data(),
        };

        init_ix
    }

    #[test]
    fn test_init_vault() {}
}
