use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::sysvar;
use solana_sdk::address_lookup_table::{
    instruction::{create_lookup_table, extend_lookup_table},
    state::AddressLookupTable,
};
use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey,
    signature::Keypair, signer::Signer, transaction::Transaction,
};
use std::error::Error;
use std::sync::Arc;
use tokio;

use solana_sdk::{
    message::{ VersionedMessage, v0},
    transaction::VersionedTransaction,
};

use solana_sdk::address_lookup_table::AddressLookupTableAccount;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Call the async function and unwrap the result to get the Pubkey
    let lookup_table_address = create_atl_address().await?;

    println!(
        "\nSuccessfully created and extended ALT at address: {}",
        lookup_table_address
    );
    Ok(())
}

pub async fn create_atl_address() -> Result<Pubkey, Box<dyn Error + Send + Sync>> {
    let dev_b58 = std::env::var("DEV_KEY").unwrap_or_default();
    let payer = Keypair::from_base58_string(dev_b58.clone().as_str());
    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "".to_string());

    let client_connection = Arc::new(RpcClient::new_with_commitment(
        rpc_url.clone(),
        CommitmentConfig::processed(),
    ));

    let recent_blockhash = client_connection.get_latest_blockhash().await?;
    let program_id = Pubkey::new_unique(); // Mock program ID for the example

    let addresses_to_add: Vec<Pubkey> = vec![
        solana_program::system_program::id(),
        sysvar::rent::id(),
        program_id,
        Pubkey::new_unique(),
        Pubkey::new_unique(),
        Pubkey::new_unique(),
    ];

    let slot = client_connection.get_slot().await?;

    let (create_ix, lookup_table_address) =
        create_lookup_table(payer.pubkey(), payer.pubkey(), slot);

    println!("Creating lookup table at address: {}", lookup_table_address);

    let raw_account = client_connection
        .clone()
        .get_account(&lookup_table_address)
        .await?;

    let address_lookup_table = AddressLookupTable::deserialize(&raw_account.data)?;

    let address_lookup_table_account = AddressLookupTableAccount {
        key: lookup_table_address,
        addresses: address_lookup_table.addresses.to_vec(),
    };

    let recent_blockhash = client_connection.get_latest_blockhash().await.unwrap();

    let message = v0::Message::try_compile(
        &payer.pubkey(),
        &[create_ix],
        &[address_lookup_table_account.clone()],
        recent_blockhash.clone(),
    )
    .unwrap();

    let create_txn = VersionedTransaction::try_new(VersionedMessage::V0(message), &[payer])?;
    let create_sig = client_connection
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &create_txn,
            CommitmentConfig::processed(),
        )
        .await?;
    println!("Create transaction confirmed: {}", create_sig);

    // --- Step 2: Extend the Address Lookup Table ---

    let payer_arc = Arc::from(Keypair::from_base58_string(dev_b58.as_str().clone()));
    let recent_blockhash = client_connection.get_latest_blockhash().await.unwrap();

    let extend_ix = extend_lookup_table(
        lookup_table_address,
        payer_arc.clone().pubkey(),
        Some(payer_arc.clone().pubkey()),
        addresses_to_add,
    );

    let extend_msg = v0::Message::try_compile(
        &payer_arc.pubkey(),
        &[extend_ix],
        &[address_lookup_table_account.clone()],
        recent_blockhash.clone(),
    )
    .unwrap();

    let extend_txn =
        VersionedTransaction::try_new(VersionedMessage::V0(extend_msg), &[&*payer_arc.clone()])?;

    let extend_sig = client_connection
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &extend_txn,
            CommitmentConfig::processed(),
        )
        .await
        .unwrap();

    println!("Extend transaction confirmed: {}", extend_sig);

    // Fetch the account to get the latest addresses and table state.
    let table_account = client_connection
        .get_account(&lookup_table_address)
        .await
        .unwrap();

    let lookup_table = AddressLookupTable::deserialize(&table_account.data)?;

    let recent_blockhash = client_connection.get_latest_blockhash().await.unwrap();

    let addresses_to_lookup: Vec<Pubkey> = lookup_table.addresses.to_vec();
    let mut instructions: Vec<Instruction> = vec![];
    instructions.push(solana_sdk::system_instruction::transfer(
        &payer_arc.clone().pubkey(),
        &addresses_to_lookup[2], // Using the program_id from our table
        10_000,
    ));

    // Pass the lookup table to `try_compile`!
    let message = v0::Message::try_compile(
        &payer_arc.clone().pubkey(),
        &instructions,
        &[address_lookup_table_account.clone()],
        recent_blockhash,
    )?;

    let final_txn =
        VersionedTransaction::try_new(VersionedMessage::V0(message), &[&payer_arc.clone()])?;
    let final_sig = client_connection
        .send_and_confirm_transaction_with_spinner_and_commitment(
            &final_txn,
            CommitmentConfig::processed(),
        )
        .await
        .unwrap();

    println!("Final transaction using ALT sent! Signature: {}", final_sig);

    Ok(lookup_table_address)
}
