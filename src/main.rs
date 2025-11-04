use solana_address::Address;
use solana_commitment_config::CommitmentConfig;
use solana_rpc_client::rpc_client::RpcClient;
use solana_rpc_client_api::{
    config::{RpcAccountInfoConfig, RpcProgramAccountsConfig},
    filter::{Memcmp, MemcmpEncodedBytes, RpcFilterType},
};
use std::{env, time::Duration};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();

    let rpc_url = env::var("RPC_URL").expect("RPC_URL must be set in environment or .env file");
    let client = RpcClient::new_with_timeout_and_commitment(
        rpc_url,
        Duration::from_secs(180),
        CommitmentConfig::confirmed(),
    );

    // Meteora DLMM program ID
    let program_id = Address::from_str_const("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo");
    let bin_array_discriminator: [u8; 8] = [92, 142, 92, 220, 5, 148, 70, 181];

    println!("Fetching all Meteora DLMM BinArray accounts...");

    // Create filter for BinArray discriminator
    let filters = vec![RpcFilterType::Memcmp(Memcmp::new(
        0, // offset 0 (start of account data)
        MemcmpEncodedBytes::Bytes(bin_array_discriminator.to_vec()),
    ))];

    // Configure to return 0 data (just addresses)
    let config = RpcProgramAccountsConfig {
        filters: Some(filters),
        account_config: RpcAccountInfoConfig {
            encoding: Some(solana_account_decoder::UiAccountEncoding::Base64),
            data_slice: None,
            // data_slice: Some(UiDataSliceConfig {
            //     offset: 0,
            //     length: 0, // Return 0 data, just the addresses
            // }),
            commitment: Some(CommitmentConfig::confirmed()),
            min_context_slot: None,
        },
        with_context: Some(false),
        sort_results: None,
    };

    // Fetch all bin array accounts
    let accounts = client.get_program_accounts_with_config(&program_id, config)?;

    println!("\n=== Results ===");
    println!("Total BinArray accounts found: {}", accounts.len());

    let mut bin_arrays_with_zero_price = 0;
    for (address, account) in accounts {
        let data = account.data[8..].to_vec(); // Alignment issue workaround
        let bin_array = bytemuck::from_bytes::<BinArray>(&data);
        for bin in bin_array.bins {
            if bin.price == 0 {
                println!("Found 0 price in bin array {address}");
                bin_arrays_with_zero_price += 1;
                break;
            }
        }
    }
    println!("bin_arrays_with_zero_price: {bin_arrays_with_zero_price}");

    Ok(())
}

pub const MAX_BIN_PER_ARRAY: usize = 70;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct BinArray {
    pub index: i64, // Larger size to make bytemuck "safe" (correct alignment)
    /// Version of binArray
    pub version: u8,
    pub _padding: [u8; 7],
    pub lb_pair: Address,
    pub bins: [Bin; MAX_BIN_PER_ARRAY],
}

pub const NUM_REWARDS: usize = 2;

#[repr(C)]
#[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone)]
pub struct Bin {
    /// Amount of token X in the bin. This already excluded protocol fees.
    pub amount_x: u64,
    /// Amount of token Y in the bin. This already excluded protocol fees.
    pub amount_y: u64,
    /// Bin price
    pub price: u128,
    /// Liquidities of the bin. This is the same as LP mint supply. q-number
    pub liquidity_supply: u128,
    /// reward_a_per_token_stored
    pub reward_per_token_stored: [u128; NUM_REWARDS],
    /// Swap fee amount of token X per liquidity deposited.
    pub fee_amount_x_per_token_stored: u128,
    /// Swap fee amount of token Y per liquidity deposited.
    pub fee_amount_y_per_token_stored: u128,
    /// Total token X swap into the bin. Only used for tracking purpose.
    pub amount_x_in: u128,
    /// Total token Y swap into he bin. Only used for tracking purpose.
    pub amount_y_in: u128,
}
