Solana Address Lookup Table (ALT) Creator

This Rust utility provides a simple, standalone function to create and extend a Solana Address Lookup Table (ALT) on the Mainnet/Devnet. It's designed to be used in projects that require a consistent lookup table to reduce transaction size and avoid the "transaction too large" error.

How to Use
The core of this project is the create_atl_address function, which handles the entire process of creating an ALT from scratch and extending it with a predefined set of addresses.

Prerequisites
You must have the following installed to run this code:

Rust and Cargo

A funded Solana Mainnet/Devnet wallet.

Environment Variables
Before running, you need to set two environment variables:

DEV_B58: The Base58-encoded private key of your Solana wallet. This wallet will be the payer and authority for the lookup table.

RPC_URL: The RPC endpoint URL you want to use. The Devnet endpoint is recommended for this example.

Example for a Unix-like shell:

export DEV_B58="<YOUR_BASE58_PRIVATE_KEY_HERE>"
export RPC_URL="[https://api.devnet.solana.com](https://api.devnet.solana.com)"

Running the Code
Clone the repository and navigate into the project directory.

Ensure your Cargo.toml dependencies are compatible. The solana-sdk and related crates must be on the same version. A good starting point is 2.0.0 for all of them.

Run the utility:

cargo run

The output will be the public key of the newly created lookup table. You can then use this address in your main application's code to build versioned transactions.


🧠 Functionality Overview
The create_atl_address function performs the following three steps:

Creation: Sends a transaction with a create_lookup_table instruction. This initializes a new, empty lookup table on the network.

Extension: After confirming the table's creation, it sends a second transaction with an extend_lookup_table instruction. This instruction populates the table with a list of the most common program IDs, as well as any other addresses your application will use repeatedly.

Verification: It fetches and deserializes the newly created lookup table to confirm that all addresses were added successfully. The function then returns the lookup table's public key.
