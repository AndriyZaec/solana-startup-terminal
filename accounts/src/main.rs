use anyhow::anyhow;
use borsh::BorshDeserialize;
use clock::Clock;
use mpl_token_metadata::accounts::Metadata;
use solana_cli_config::Config;
use solana_client::{rpc_client::RpcClient, rpc_config::CommitmentConfig};
use solana_sdk::{
    clock,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    signer::EncodableKey,
    sysvar::SysvarId,
};
use std::str::FromStr;

fn main() -> anyhow::Result<()> {
    // Стягуємо з конфігу клієнта:
    let config_file = solana_cli_config::CONFIG_FILE
        .as_ref()
        .ok_or_else(|| anyhow!("unable to get config file path"))?;
    let cfg = Config::load(config_file)?;

    // Ініціалізуємо клієнта
    let client =
        RpcClient::new_with_commitment(cfg.json_rpc_url.clone(), CommitmentConfig::confirmed());

    let sender = Keypair::read_from_file(cfg.keypair_path)
        .map_err(|e| anyhow!("Failed to read keypair from file: {}", e))?;

    // 1. Fetch sender
    let sender_data = client.get_account(&sender.pubkey())?;
    println!("\n========================================");
    println!("Sender account: {:?}", sender_data);

    // 2. Fetch sysvar account
    let clock_account = client.get_account(&Clock::id())?;
    let clock_data: Clock = bincode::deserialize(&clock_account.data)?;
    println!("\n========================================");
    println!("Clock account data: {:?}", clock_data);

    // 3. Fetching metaplex metadata pda (mainnet address! - спробуйте запустина на devnet)
    let metaplex_program_id = mpl_token_metadata::ID;
    let usdc_id = Pubkey::from_str("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB")?;
    let metadata_seeds = &[b"metadata", metaplex_program_id.as_ref(), usdc_id.as_ref()];

    let (metadata_key, metadata_bump) =
        Pubkey::find_program_address(metadata_seeds, &metaplex_program_id);

    let metadata_acc = client.get_account(&metadata_key)?;
    let decoded = Metadata::deserialize(&mut &metadata_acc.data[..])?;

    println!("\n========================================");
    println!("Metadata PDA: {}", metadata_key);
    println!("Metadata bump: {}", metadata_bump);
    println!("Metadata account: {}-{}", decoded.name, decoded.symbol);

    Ok(())
}
