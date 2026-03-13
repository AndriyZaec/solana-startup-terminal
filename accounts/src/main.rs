use anyhow::anyhow;
use borsh::BorshDeserialize;
use clock::Clock;
use mpl_token_metadata::accounts::Metadata;
use solana_cli_config::Config;
use solana_client::{rpc_client::RpcClient, rpc_config::CommitmentConfig};
use solana_sdk::{
    clock,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    signer::EncodableKey,
    sysvar::SysvarId,
    transaction::Transaction,
};
use solana_system_interface::instruction::create_account;
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

    let sender = Keypair::read_from_file("local_wallet.json")
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
    let usdc_id = Pubkey::from_str("4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU")?;
    let mint_pub = create_nft(&client, sender)?;

    let metadata_seeds = &[
        b"metadata",
        mpl_token_metadata::ID.as_ref(),
        mint_pub.as_ref(),
    ];

    let (metadata_key, metadata_bump) =
        Pubkey::find_program_address(metadata_seeds, &metaplex_program_id);

    let metadata_acc = client
        .get_account(&metadata_key)
        .map_err(|e| anyhow!("No PDA acc for this {e}"))?;
    let decoded = Metadata::deserialize(&mut &metadata_acc.data[..])?;

    println!("\n========================================");
    println!("Metadata PDA: {}", metadata_key);
    println!("Metadata bump: {}", metadata_bump);
    println!("Metadata account: {}-{}", decoded.name, decoded.symbol);

    Ok(())
}

fn create_nft(client: &RpcClient, wallet: Keypair) -> anyhow::Result<Pubkey> {
    let mint = Keypair::new();
    print!("mint: {0}", mint.pubkey());

    let recent_blockhash = client.get_latest_blockhash()?;
    let rent = client.get_minimum_balance_for_rent_exemption(spl_token::state::Mint::LEN)?;

    let create_mint_account_ix = create_account(
        &wallet.pubkey(),
        &mint.pubkey(),
        rent,
        spl_token::state::Mint::LEN as u64,
        &spl_token::id(),
    );

    let create_mint_tx = Transaction::new_signed_with_payer(
        &[create_mint_account_ix],
        Some(&wallet.pubkey()),
        &[&wallet, &mint],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&create_mint_tx)?;

    let init_mint_ix = spl_token::instruction::initialize_mint(
        &spl_token::id(),
        &mint.pubkey(),
        &wallet.pubkey(),
        Some(&wallet.pubkey()),
        0,
    )?;

    let init_mint_tx = Transaction::new_signed_with_payer(
        &[init_mint_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&init_mint_tx)?;

    let ata = spl_associated_token_account::get_associated_token_address(
        &wallet.pubkey(),
        &mint.pubkey(),
    );

    let create_ata_ix = spl_associated_token_account::instruction::create_associated_token_account(
        &wallet.pubkey(),
        &wallet.pubkey(),
        &mint.pubkey(),
        &spl_token::id(),
    );

    let create_ata_tx = Transaction::new_signed_with_payer(
        &[create_ata_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&create_ata_tx)?;

    let mint_to_ix = spl_token::instruction::mint_to(
        &spl_token::id(),
        &mint.pubkey(),
        &ata,
        &wallet.pubkey(),
        &[],
        1,
    )?;

    let mint_to_tx = Transaction::new_signed_with_payer(
        &[mint_to_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&mint_to_tx)?;

    let mint_pubkey = mint.pubkey();
    let metadata_seeds = &[
        b"metadata",
        mpl_token_metadata::ID.as_ref(),
        mint_pubkey.as_ref(),
    ];

    let (metadata_pda, _metadata_bump) =
        Pubkey::find_program_address(metadata_seeds, &mpl_token_metadata::ID);

    println!("\n========================================");
    println!("NFT Mint key: {mint_pubkey}");
    println!("\n========================================");

    let data = mpl_token_metadata::types::DataV2 {
        name: String::from("My Lovely NFT"),
        symbol: String::from("MLNG"),
        uri: "https://example.com/metadata.json".to_string(),
        seller_fee_basis_points: 0,
        creators: None,
        collection: None,
        uses: None,
    };

    let create_metadata_ix =
        mpl_token_metadata::instructions::CreateMetadataAccountV3Builder::new()
            .metadata(metadata_pda)
            .mint(mint_pubkey)
            .mint_authority(wallet.pubkey())
            .payer(wallet.pubkey())
            .update_authority(wallet.pubkey(), true)
            .data(data)
            .is_mutable(true)
            .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_metadata_ix],
        Some(&wallet.pubkey()),
        &[&wallet],
        recent_blockhash,
    );
    client.send_and_confirm_transaction(&tx)?;

    Ok(mint_pubkey)
}
