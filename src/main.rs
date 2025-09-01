use solana_sdk::{
    commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey,
    signature::Keypair, signer::Signer, transaction::Transaction,
};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::address_lookup_table::{
    instruction::{create_lookup_table, extend_lookup_table},
    state::{AddressLookupTable},
};
use solana_program::{system_instruction, sysvar};
use std::error::Error;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{Duration, sleep};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Call the async function and unwrap the result to get the Pubkey
    let lookup_table_address = create_atl_address().await?;
    
    println!("\nSuccessfully created and extended ALT at address: {}", lookup_table_address);
    Ok(())
}

/// Creates and extends a Solana Address Lookup Table on chain
/// Requires the `DEV_B58` and `RPC_URL` environment variables to be set.
/// Returns the Pubkey of the created lookup table.
pub async fn create_atl_address() -> Result<Pubkey, Box<dyn Error + Send + Sync>> {
    let dev_key = std::env::var("DEV_B58").unwrap();
    let payer = Keypair::from_base58_string(&dev_key.as_str());

    let rpc_url = std::env::var("RPC_URL").unwrap_or_else(|_| "".to_string());

    let client_connection = Arc::new(
        RpcClient::new_with_commitment(rpc_url.clone(), CommitmentConfig::processed())
    );

    // === STEP 1: CREATE THE LOOKUP TABLE ===
    println!("🏗️ Step 1: Creating Address Lookup Table...");

    let slot = client_connection.get_slot().await?;
    let (create_ix, lookup_table_address) = create_lookup_table(
        payer.pubkey(),
        payer.pubkey(),
        slot
    );

    println!("Creating lookup table at address: {}", lookup_table_address);

    let recent_blockhash = client_connection.get_latest_blockhash().await?;
    let create_transaction = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );

    let create_sig = client_connection.send_and_confirm_transaction(&create_transaction).await?;
    println!("✅ ALT created! Signature: {}", create_sig);

    println!("⏳ Waiting for ALT to be available...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // === STEP 2: ADD WSOL,PROGRAM,WALLET,PDAs,ATA AND REPEATED ADDRESSES ===
    let addresses_to_add: Vec<Pubkey> = vec![
        // Core Solana programs (most frequently used)
        solana_program::system_program::id(), // System Program
        spl_token::id(), // Token Program
        spl_associated_token_account::id(), // ATA Program
        sysvar::rent::id(), // Rent Sysvar
        // Token mints
        Pubkey::from_str("So11111111111111111111111111111111111111112")?, // WSOL
    ];

    println!("Adding {} addresses to ALT:", addresses_to_add.len());
    for (i, addr) in addresses_to_add.iter().enumerate() {
        println!("  [{}] {}", i, addr);
    }

    let extend_ix = extend_lookup_table(
        lookup_table_address,
        payer.pubkey(),
        Some(payer.pubkey()),
        addresses_to_add
    );

    let recent_blockhash = client_connection.get_latest_blockhash().await?;
    let extend_transaction = Transaction::new_signed_with_payer(
        &[extend_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash
    );

    let extend_sig = client_connection.send_and_confirm_transaction(&extend_transaction).await?;
    println!("✅ ALT extended! Signature: {}", extend_sig);

    println!("⏳ Waiting for addresses to be added...");
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // === STEP 3: VERIFY THE LOOKUP TABLE ===
    println!("🧪 Step 3: Verifying lookup table...");

    let table_account = client_connection.get_account(&lookup_table_address).await?;

    let lookup_table = AddressLookupTable::deserialize(&table_account.data)?;
    println!("✅ ALT now contains {} addresses", lookup_table.addresses.len());

    Ok(lookup_table_address)
}
