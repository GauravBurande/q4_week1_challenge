#[cfg(test)]
mod tests {

    use anchor_lang::{
        prelude::msg,
        solana_program::hash::{hash, Hash},
        InstructionData, ToAccountMetas,
    };
    use anchor_spl::associated_token::{self, spl_associated_token_account};
    use litesvm::LiteSVM;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_pubkey::Pubkey;
    use solana_sdk_ids::system_program::ID as SYSTEM_PROGRAM;
    use solana_signer::Signer;
    use solana_transaction::Transaction;
    use spl_token_2022::{extension::StateWithExtensions, state::Account};
    use std::{fs::read, path::PathBuf, str::FromStr};
    pub struct TestEnv {
        pub svm: LiteSVM,
        pub admin: Keypair,
        pub mint2022: Keypair,
        pub token_program: Pubkey,
        pub config: Pubkey,
        pub vault: Pubkey,
        pub user_ata: Pubkey,
    }

    // TODO: ADD IT IN A SINGLE WORKSPACE, ALL THE PROGRAMS!
    static PROGRAM_ID: Pubkey = crate::ID;

    const ASSOCIATED_TOKEN_PROGRAM: Pubkey = spl_associated_token_account::ID;

    fn get_tf_hook_program_address() -> Pubkey {
        let transfer_hook_program =
            Pubkey::from_str("E6mxgYTtMfqneSJHxBZ9sP7VdJjW9FQsz1Dff8TsSN9p").unwrap();
        transfer_hook_program
    }

    fn setup() -> TestEnv {
        let mut svm = LiteSVM::new();
        let admin = Keypair::new();
        let mint2022 = Keypair::new();
        let token_program =
            Pubkey::from_str("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb").unwrap();

        let vault_so_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/deploy/vault.so");
        let vault_program_data = read(vault_so_path).expect("Failed to read the program SO file!");

        let whitelist_tf_hook_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/deploy/whitelist_transfer_hook.so");
        let whitelist_tf_hook_program_data =
            read(whitelist_tf_hook_path).expect("Failed to read the program SO file!");

        svm.add_program(PROGRAM_ID, &vault_program_data);
        svm.add_program(
            get_tf_hook_program_address(),
            &whitelist_tf_hook_program_data,
        );

        svm.airdrop(&admin.pubkey(), 10 * LAMPORTS_PER_SOL)
            .expect("Failed to airdrop SOL to admin.");

        let config = Pubkey::find_program_address(&[b"config"], &PROGRAM_ID).0;
        let vault = associated_token::get_associated_token_address_with_program_id(
            &config,
            &mint2022.pubkey(),
            &token_program,
        );
        let user_ata = associated_token::get_associated_token_address_with_program_id(
            &admin.pubkey(),
            &mint2022.pubkey(),
            &token_program,
        );

        TestEnv {
            svm,
            admin,
            mint2022,
            token_program,
            config,
            vault,
            user_ata,
        }
    }

    fn build_init_transaction(
        admin: &Keypair,
        mint2022: &Keypair,
        token_program: Pubkey,
        config: Pubkey,
        vault: Pubkey,
        recent_blockhash: Hash,
    ) -> Transaction {
        // this one is correct
        let init_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Initialize {
                admin: admin.pubkey(),
                config: config,
                vault: vault,
                mint: mint2022.pubkey(),
                transfer_hook_program: get_tf_hook_program_address(),
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM,
                system_program: SYSTEM_PROGRAM,
                token_program,
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::InitializeVault {}.data(),
        };

        let message = Message::new(&[init_ix], Some(&admin.pubkey()));

        Transaction::new(&[&admin, &mint2022], message, recent_blockhash)
    }

    fn build_mint_transaction(
        admin: &Keypair,
        mint2022: &Keypair,
        token_program: Pubkey,
        user_ata: Pubkey,
        amount: u64,
        recent_blockhash: Hash,
    ) -> Transaction {
        let mint_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::MintToken {
                admin: admin.pubkey(),
                user: admin.pubkey(),
                mint: mint2022.pubkey(),
                user_ata,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM,
                token_program,
                system_program: SYSTEM_PROGRAM,
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::Mint { amount }.data(),
        };

        let message = Message::new(&[mint_ix], Some(&admin.pubkey()));

        Transaction::new(&[&admin], message, recent_blockhash)
    }

    fn get_extra_account_metalist_pubkey(
        mint2022: &Keypair,
        transfer_hook_program: Pubkey,
    ) -> Pubkey {
        let (extra_account_meta_list, _) = Pubkey::find_program_address(
            &[b"extra-account-metas", mint2022.pubkey().as_ref()],
            &transfer_hook_program,
        );
        extra_account_meta_list
    }

    fn build_init_tf_transaction(
        admin: &Keypair,
        mint2022: &Keypair,
        recent_blockhash: Hash,
    ) -> Transaction {
        let transfer_hook_program = get_tf_hook_program_address();

        let extra_account_meta_list =
            get_extra_account_metalist_pubkey(&mint2022, transfer_hook_program);
        let account_metas = vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(extra_account_meta_list, false),
            AccountMeta::new(mint2022.pubkey(), false),
            AccountMeta::new(SYSTEM_PROGRAM, false),
        ];

        let mut data = vec![];

        let discriminator =
            anchor_lang::solana_program::hash::hash(b"global:initialize_transfer_hook");
        data.extend_from_slice(&discriminator.to_bytes()[..8]);

        let instruction = Instruction {
            program_id: transfer_hook_program,
            accounts: account_metas,
            data,
        };

        let message = Message::new(&[instruction], Some(&admin.pubkey()));

        Transaction::new(&[&admin], message, recent_blockhash)
    }

    fn build_whitelist_transaction(
        admin: &Keypair,
        token_account: Pubkey,
        operation: &str,
        recent_blockhash: Hash,
    ) -> Transaction {
        let transfer_hook_program = get_tf_hook_program_address();

        let whitelist = Pubkey::find_program_address(
            &[b"whitelist", token_account.as_ref()],
            &transfer_hook_program,
        )
        .0;

        let account_metas = vec![
            AccountMeta::new(admin.pubkey(), true),
            AccountMeta::new(whitelist, false),
            AccountMeta::new(SYSTEM_PROGRAM, false),
        ];

        let mut data = vec![];

        let string = format!("global:{}", operation);
        let discriminator = hash(string.as_bytes());
        data.extend_from_slice(&discriminator.to_bytes()[..8]);
        data.extend_from_slice(&token_account.as_ref());

        let instruction = Instruction {
            program_id: transfer_hook_program,
            accounts: account_metas,
            data,
        };

        let message = Message::new(&[instruction], Some(&admin.pubkey()));

        Transaction::new(&[&admin], message, recent_blockhash)
    }

    fn build_deposit_transaction(
        admin: &Keypair,
        mint2022: &Keypair,
        token_program: Pubkey,
        config: Pubkey,
        vault: Pubkey,
        user_ata: Pubkey,
        recent_blockhash: Hash,
    ) -> Transaction {
        let amount_pda =
            Pubkey::find_program_address(&[b"amount", admin.pubkey().as_ref()], &PROGRAM_ID).0;
        let transfer_hook_program = get_tf_hook_program_address();
        let extra_account_meta_list =
            get_extra_account_metalist_pubkey(&mint2022, transfer_hook_program);
        let user_whitelist = Pubkey::find_program_address(
            &[b"whitelist", user_ata.as_ref()],
            &transfer_hook_program,
        )
        .0;
        // this one is correct
        let deposit_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Deposit {
                user: admin.pubkey(),
                user_ata,
                config: config,
                amount_pda,
                vault: vault,
                mint: mint2022.pubkey(),
                transfer_hook_program,
                whitelist: user_whitelist,
                extra_account_meta_list,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM,
                token_program,
                system_program: SYSTEM_PROGRAM,
            }
            .to_account_metas(Some(true)),
            data: crate::instruction::Deposit { amount: 100 }.data(),
        };

        let message = Message::new(&[deposit_ix], Some(&admin.pubkey()));

        Transaction::new(&[&admin], message, recent_blockhash)
    }

    fn build_withdraw_transaction(
        admin: &Keypair,
        mint2022: &Keypair,
        token_program: Pubkey,
        config: Pubkey,
        vault: Pubkey,
        user_ata: Pubkey,
        whitelist: Pubkey,
        recent_blockhash: Hash,
    ) -> Transaction {
        let amount_pda =
            Pubkey::find_program_address(&[b"amount", admin.pubkey().as_ref()], &PROGRAM_ID).0;
        let transfer_hook_program = get_tf_hook_program_address();
        let extra_account_meta_list =
            get_extra_account_metalist_pubkey(&mint2022, transfer_hook_program);
        let withdraw_ix = Instruction {
            program_id: PROGRAM_ID,
            accounts: crate::accounts::Withdraw {
                user: admin.pubkey(),
                user_ata,
                config: config,
                amount_pda,
                vault: vault,
                mint: mint2022.pubkey(),
                transfer_hook_program,
                whitelist,
                extra_account_meta_list,
                associated_token_program: ASSOCIATED_TOKEN_PROGRAM,
                token_program,
                system_program: SYSTEM_PROGRAM,
            }
            .to_account_metas(None),
            data: crate::instruction::Withdraw { amount: 100 }.data(),
        };

        let message = Message::new(&[withdraw_ix], Some(&admin.pubkey()));

        Transaction::new(&[&admin], message, recent_blockhash)
    }

    #[test]
    fn test_init_vault() {
        let TestEnv {
            mut svm,
            admin,
            mint2022,
            token_program,
            config,
            vault,
            user_ata: _,
        } = setup();
        let recent_blockhash = svm.latest_blockhash();
        let transaction = build_init_transaction(
            &admin,
            &mint2022,
            token_program,
            config,
            vault,
            recent_blockhash,
        );

        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nInit transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);
    }

    #[test]
    fn test_mint() {
        let TestEnv {
            mut svm,
            admin,
            mint2022,
            token_program,
            config,
            vault,
            user_ata,
        } = setup();

        let recent_blockhash = svm.latest_blockhash();
        let transaction1 = build_init_transaction(
            &admin,
            &mint2022,
            token_program,
            config,
            vault,
            recent_blockhash,
        );
        let _tx1 = svm.send_transaction(transaction1).unwrap();

        let amount = 1_000_000;

        let transaction2 = build_mint_transaction(
            &admin,
            &mint2022,
            token_program,
            user_ata,
            amount,
            recent_blockhash,
        );
        let tx2 = svm
            .send_transaction(transaction2)
            .expect("Failed to send mint token transaction!");

        // Log transaction details
        msg!("\n\n Mint transaction sucessfull");
        msg!("CUs Consumed: {}", tx2.compute_units_consumed);
        msg!("Tx Signature: {}", tx2.signature);

        let token_account = svm.get_account(&user_ata).unwrap();
        let token_state = StateWithExtensions::<Account>::unpack(&token_account.data)
            .expect("Failed to deserialize token account data");
        msg!("token state: {:?}", token_state.base);
        assert_eq!(token_state.base.amount, amount);
    }

    #[test]
    fn test_initialize_transfer_hook() {
        let TestEnv {
            mut svm,
            admin,
            mint2022,
            token_program,
            config,
            vault,
            user_ata: _,
        } = setup();
        let recent_blockhash = svm.latest_blockhash();
        let transaction1 = build_init_transaction(
            &admin,
            &mint2022,
            token_program,
            config,
            vault,
            recent_blockhash,
        );
        let _tx1 = svm
            .send_transaction(transaction1)
            .expect("Failed to send init vault tx");

        let transaction2 = build_init_tf_transaction(&admin, &mint2022, recent_blockhash);

        let tx2 = svm
            .send_transaction(transaction2)
            .expect("Failed to send init tf hoook tx");

        // Log transaction details
        msg!("\n\n Init tf hook transaction sucessfull");
        msg!("CUs Consumed: {}", tx2.compute_units_consumed);
        msg!("Tx Signature: {}", tx2.signature);
    }

    #[test]
    fn test_add_to_whitelist() {
        let TestEnv {
            mut svm,
            admin,
            mint2022: _,
            token_program: _,
            config: _,
            vault,
            user_ata: _,
        } = setup();
        let recent_blockhash = svm.latest_blockhash();
        let transaction =
            build_whitelist_transaction(&admin, vault, "add_to_whitelist", recent_blockhash);
        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send init tf hoook tx");

        // Log transaction details
        msg!("\n\n Add to whitelist transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);
    }

    #[test]
    fn test_remove_from_whitelist() {
        let TestEnv {
            mut svm,
            admin,
            mint2022: _,
            token_program: _,
            config: _,
            vault,
            user_ata: _,
        } = setup();
        let recent_blockhash = svm.latest_blockhash();

        let transaction1 =
            build_whitelist_transaction(&admin, vault, "add_to_whitelist", recent_blockhash);
        let _tx1 = svm
            .send_transaction(transaction1)
            .expect("Failed to send init tf hoook tx");

        let transaction =
            build_whitelist_transaction(&admin, vault, "remove_from_whitelist", recent_blockhash);
        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send remove from whitelist tx");

        // Log transaction details
        msg!("\n\n Remove from whitelist transaction successful");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);
    }

    #[test]
    fn test_deposit() {
        let TestEnv {
            mut svm,
            admin,
            mint2022,
            token_program,
            config,
            vault,
            user_ata,
        } = setup();

        let recent_blockhash = svm.latest_blockhash();
        let transaction1 = build_init_transaction(
            &admin,
            &mint2022,
            token_program,
            config,
            vault,
            recent_blockhash,
        );
        let _tx1 = svm
            .send_transaction(transaction1)
            .expect("Failed to send vault init tx");

        let transaction2 = build_init_tf_transaction(&admin, &mint2022, recent_blockhash);
        let _tx2 = svm
            .send_transaction(transaction2)
            .expect("Failed to send init tf hoook tx");

        let amount = 1_000_000;
        let transaction3 = build_mint_transaction(
            &admin,
            &mint2022,
            token_program,
            user_ata,
            amount,
            recent_blockhash,
        );
        let _tx3 = svm
            .send_transaction(transaction3)
            .expect("Failed to send mint txn");

        let token_account = svm.get_account(&user_ata).unwrap();
        let token_state = StateWithExtensions::<Account>::unpack(&token_account.data)
            .expect("Failed to deserialize token account data");
        msg!("token state: {:?}", token_state.base);

        let transaction4 =
            build_whitelist_transaction(&admin, user_ata, "add_to_whitelist", recent_blockhash);
        let _tx4 = svm
            .send_transaction(transaction4)
            .expect("Failed to send whitelist txn");

        let transaction = build_deposit_transaction(
            &admin,
            &mint2022,
            token_program,
            config,
            vault,
            user_ata,
            recent_blockhash,
        );
        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send Deposit txn");

        // Log transaction details
        msg!("\n\n Desposit transaction successful");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);
    }

    #[test]
    fn test_withdraw() {
        let TestEnv {
            mut svm,
            admin,
            mint2022,
            token_program,
            config,
            vault,
            user_ata,
        } = setup();

        let recent_blockhash = svm.latest_blockhash();
        let transfer_hook_program = get_tf_hook_program_address();
        let vault_whitelist =
            Pubkey::find_program_address(&[b"whitelist", vault.as_ref()], &transfer_hook_program).0;
        let transaction1 = build_init_transaction(
            &admin,
            &mint2022,
            token_program,
            config,
            vault,
            recent_blockhash,
        );
        let _tx1 = svm
            .send_transaction(transaction1)
            .expect("Failed to send vault init tx");

        let transaction2 = build_init_tf_transaction(&admin, &mint2022, recent_blockhash);
        let _tx2 = svm
            .send_transaction(transaction2)
            .expect("Failed to send init tf hoook tx");

        let amount = 1_000_000;
        let transaction3 = build_mint_transaction(
            &admin,
            &mint2022,
            token_program,
            user_ata,
            amount,
            recent_blockhash,
        );
        let _tx3 = svm
            .send_transaction(transaction3)
            .expect("Failed to send mint txn");

        let transaction4 =
            build_whitelist_transaction(&admin, user_ata, "add_to_whitelist", recent_blockhash);
        let _tx4 = svm
            .send_transaction(transaction4)
            .expect("Failed to send whitelist it txn");

        let transaction5 = build_deposit_transaction(
            &admin,
            &mint2022,
            token_program,
            config,
            vault,
            user_ata,
            recent_blockhash,
        );
        let _tx5 = svm
            .send_transaction(transaction5)
            .expect("Failed to send Deposit txn");

        let transaction6 =
            build_whitelist_transaction(&admin, vault, "add_to_whitelist", recent_blockhash);
        let _tx6 = svm
            .send_transaction(transaction6)
            .expect("Failed to send whitelist it txn");

        let transaction = build_withdraw_transaction(
            &admin,
            &mint2022,
            token_program,
            config,
            vault,
            user_ata,
            vault_whitelist,
            recent_blockhash,
        );

        let tx = svm
            .send_transaction(transaction)
            .expect("Failed to send withdraw txn");
        // Log transaction details
        msg!("\n\n Withdraw transaction successful");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);
        msg!("Tx Signature: {}", tx.signature);
    }
}
