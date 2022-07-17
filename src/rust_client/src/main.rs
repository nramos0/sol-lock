#![allow(dead_code)]

use chrono::prelude::*;
use solana_client::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use std::thread::sleep;
use std::time::Duration;
use std::{error::Error, str::FromStr};
use transaction::{check_balance, create_and_lock, unlock};

mod transaction;

const URL_TESTNET: &str = "https://api.testnet.solana.com";
const URL_DEVNET: &str = "https://api.devnet.solana.com";
const URL_LOCAL: &str = "http://127.0.0.1:8899";

fn main() -> Result<(), Box<dyn Error>> {
    let rpc_client = RpcClient::new(URL_LOCAL);

    let keypair_secret_json: serde_json::Value =
        serde_json::from_str(include_str!("../keys/key.json"))?;
    let keypair_secret = keypair_secret_json
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_u64().unwrap() as u8)
        .collect::<Vec<_>>();

    let sender = Keypair::from_bytes(&keypair_secret[..])?;
    let sender_key = sender.pubkey();

    println!("Sender: {:?}", sender_key);

    let program_id = Pubkey::from_str("DBqu2qa8B43uzVqrNJJcXeFW2y91os6xwpraoN5D43rP").unwrap();
    let system_program = Pubkey::from_str("11111111111111111111111111111111").unwrap();

    let acc_index = 100u64;
    let sol_lock_account = Pubkey::find_program_address(
        &[sender_key.as_ref(), &acc_index.to_le_bytes()],
        &program_id,
    )
    .0;

    let now = Utc::now().timestamp();
    let deadline = now + 20;

    println!("creating and locking...");
    create_and_lock(
        &rpc_client,
        &program_id,
        &sender_key,
        &sol_lock_account,
        &system_program,
        acc_index,
        deadline.try_into().unwrap(),
        10000000000,
        &sender,
    )?;

    println!("Account: {:?} created successfully.", sol_lock_account);
    let pda_balance = check_balance(&rpc_client, &sol_lock_account)?;
    println!("SolLock account balance: {:?}", pda_balance);

    let balance = check_balance(&rpc_client, &sender_key)?;
    println!("Sender balance: {:?}", balance);

    let now = Utc::now().timestamp();
    let diff = deadline.saturating_sub(now);
    if diff > 0 {
        println!("Waiting for deadline in {} seconds...", diff + 30);
        sleep(Duration::from_secs((diff + 30).try_into().unwrap()));
    }

    println!("Unlocking Sol.");
    unlock(
        &rpc_client,
        &program_id,
        &sender_key,
        &sol_lock_account,
        acc_index,
        &sender,
    )?;

    println!("Unlocked successfully!");

    let pda_balance = check_balance(&rpc_client, &sol_lock_account)?;
    println!("SolLock account balance: {:?}", pda_balance);

    let balance = check_balance(&rpc_client, &sender_key)?;
    println!("Sender balance: {:?}", balance);

    Ok(())
}
