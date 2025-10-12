#[cfg(test)]
mod tests {

    use anchor_lang::{prelude::msg, InstructionData, ToAccountMetas};
    use anchor_spl::associated_token::{self, spl_associated_token_account};
    use litesvm::LiteSVM;
    use litesvm_token::spl_token::ID as TOKEN_PROGRAM;
    use solana_instruction::Instruction;
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use std::{fs::read, path::PathBuf, str::FromStr};
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

        let vault_so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/vault.so");
        let vault_program_data = read(vault_so_path).expect("Failed to read the program SO file!");

        svm.add_program(PROGRAM_ID, &vault_program_data);

        svm.airdrop(&admin.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to admin.");

        // let mint2022 = CreateMint::new(&mut svm, &admin)
        // .authority(&admin.pubkey())
        // .decimals(6)
        // .send()
        // .unwrap();

        let config = Pubkey::find_program_address(&[b"config"], &PROGRAM_ID).0;
        let vault = associated_token::get_associated_token_address_with_program_id(
            &config,
            &mint2022.pubkey(),
            &TOKEN_PROGRAM,
        );

        TestEnv {
            svm,
            admin,
            mint2022,
            config,
            vault,
        }
    }

    fn build_init_instruction(
        admin: &Keypair,
        mint2022: &Keypair,
        config: Pubkey,
        vault: Pubkey,
    ) -> Instruction {
        let transfer_hook_program =
            Pubkey::from_str("E6mxgYTtMfqneSJHxBZ9sP7VdJjW9FQsz1Dff8TsSN9p").unwrap(); // this one is correct
        let init_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Initialize {
                admin: admin.pubkey(),
                config: config,
                vault: vault,
                mint: mint2022.pubkey(),
                transfer_hook_program,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM,
                system_program: SYSTEM_PROGRAM,
                token_program: TOKEN_PROGRAM,
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::InitializeVault {}.data(),
        };

        init_ix
    }

    #[test]
    fn test_init_vault() {
        let TestEnv {
            mut svm,
            admin,
            mint2022,
            config,
            vault,
        } = setup();
        let init_ix = build_init_instruction(&admin, &mint2022, config, vault);
        msg!("program id {}", PROGRAM_ID);

        let message = Message::new(&[init_ix], Some(&admin.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&admin, &mint2022], message, recent_blockhash);

        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);
    }
}
