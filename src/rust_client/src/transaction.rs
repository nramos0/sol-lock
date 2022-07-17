use sol_lock::instruction::*;
use solana_client::rpc_client::RpcClient;
use solana_program::{clock::UnixTimestamp, instruction::Instruction, pubkey::Pubkey};
use solana_sdk::{
    instruction::AccountMeta,
    signature::{Keypair, Signature},
    transaction::Transaction,
};
use std::error::Error;

const LAMPORTS_PER_SOL: f64 = 1000000000.0;

pub fn check_balance(rpc_client: &RpcClient, public_key: &Pubkey) -> Result<f64, Box<dyn Error>> {
    Ok(rpc_client.get_balance(&public_key)? as f64 / LAMPORTS_PER_SOL)
}

pub fn request_air_drop(
    rpc_client: &RpcClient,
    pub_key: &Pubkey,
    amount_sol: f64,
) -> Result<Signature, Box<dyn Error>> {
    let sig = rpc_client.request_airdrop(&pub_key, (amount_sol * LAMPORTS_PER_SOL) as u64)?;
    loop {
        let confirmed = rpc_client.confirm_transaction(&sig)?;
        if confirmed {
            break;
        }
    }
    Ok(sig)
}

pub fn unlock(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
    sender_key: &Pubkey,
    sol_lock_account: &Pubkey,
    acc_index: u64,
    sender: &Keypair,
) -> core::result::Result<(), Box<dyn Error>> {
    let mut transaction = Transaction::new_with_payer(
        &[Instruction::new_with_borsh(
            *program_id,
            &SolLockInstruction::Unlock(Unlock {
                acc_index,
                has_receiver: false,
            }),
            vec![
                AccountMeta::new(sender_key.clone(), true),
                AccountMeta::new(sol_lock_account.clone(), false),
            ],
        )],
        Some(&sender_key),
    );

    let blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[sender], blockhash);

    if let Err(err) = rpc_client.send_and_confirm_transaction(&transaction) {
        println!("{:#?}", err);
        panic!();
    };

    Ok(())
}

pub fn create_and_lock(
    rpc_client: &RpcClient,
    program_id: &Pubkey,
    sender_key: &Pubkey,
    sol_lock_account: &Pubkey,
    system_program: &Pubkey,
    acc_index: u64,
    deadline: UnixTimestamp,
    lamports: u64,
    sender: &Keypair,
) -> core::result::Result<(), Box<dyn Error>> {
    let mut transaction = Transaction::new_with_payer(
        &[
            Instruction::new_with_borsh(
                *program_id,
                &SolLockInstruction::CreateAccount(CreateAccount { acc_index }),
                vec![
                    AccountMeta::new(sender_key.clone(), true),
                    AccountMeta::new(sol_lock_account.clone(), false),
                    AccountMeta::new(system_program.clone(), false),
                ],
            ),
            Instruction::new_with_borsh(
                *program_id,
                &SolLockInstruction::SetDeadline(SetDeadline {
                    acc_index,
                    deadline,
                }),
                vec![
                    AccountMeta::new(sender_key.clone(), true),
                    AccountMeta::new(sol_lock_account.clone(), false),
                ],
            ),
            Instruction::new_with_borsh(
                *program_id,
                &SolLockInstruction::AddSol(AddSol {
                    acc_index,
                    lamports,
                    has_payer: false,
                }),
                vec![
                    AccountMeta::new(sender_key.clone(), true),
                    AccountMeta::new(sol_lock_account.clone(), false),
                    AccountMeta::new(system_program.clone(), false),
                ],
            ),
            Instruction::new_with_borsh(
                *program_id,
                &SolLockInstruction::RemoveSol(RemoveSol {
                    acc_index,
                    lamports: 6000000000,
                    has_receiver: false,
                }),
                vec![
                    AccountMeta::new(sender_key.clone(), true),
                    AccountMeta::new(sol_lock_account.clone(), false),
                ],
            ),
            Instruction::new_with_borsh(
                *program_id,
                &SolLockInstruction::Lock(Lock { acc_index }),
                vec![
                    AccountMeta::new(sender_key.clone(), true),
                    AccountMeta::new(sol_lock_account.clone(), false),
                ],
            ),
        ],
        Some(&sender_key),
    );

    let blockhash = rpc_client.get_latest_blockhash()?;
    transaction.sign(&[sender], blockhash);

    if let Err(err) = rpc_client.send_and_confirm_transaction(&transaction) {
        println!("{:#?}", err);
        panic!();
    };

    Ok(())
}
