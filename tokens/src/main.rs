mod cli;
mod mint;
mod token_account;

use solana_sdk::{
    native_token::LAMPORTS_PER_SOL, pubkey::Pubkey, signature::Keypair, signer::Signer,
};

fn wrap_unwrap_sol(cli: &cli::SolanaCli) -> anyhow::Result<()> {
    let mint = Pubkey::from_str_const("So11111111111111111111111111111111111111112");
    let amount = LAMPORTS_PER_SOL; // 1 SOL

    println!("\n========================================");
    println!("Wsol mint:");
    cli.fetch_mint(&mint)?;
    let balance_before = cli.client.get_balance(&cli.payer.pubkey())?;

    println!("\n========================================");
    println!("Врапаємо SOL:");
    let ata = cli.create_token_account(&mint, &cli.payer.pubkey())?;
    cli.fetch_ata(&ata)?;
    cli.wrap_sol(&ata, amount)?;
    cli.fetch_ata(&ata)?;

    println!("\n========================================");
    println!("Aнврапаємо SOL:");
    cli.close_token_account(&ata)?;

    let balance_after = cli.client.get_balance(&cli.payer.pubkey())?;

    println!("Balance before: {balance_before}");
    println!("Balance after: {balance_after}");
    println!("Delta: {}", balance_after as i64 - balance_before as i64);

    Ok(())
}

fn token_operations(cli: &cli::SolanaCli) -> anyhow::Result<()> {
    println!("\n========================================");
    println!("Створюємо новий mint:");
    let mint = cli.create_mint(&cli.payer.pubkey(), Some(&cli.payer.pubkey()), 6)?;
    cli.fetch_mint(&mint.pubkey())?;

    println!("\n========================================");
    println!("Створюємо ATA:");
    let ata = cli.create_token_account(&mint.pubkey(), &cli.payer.pubkey())?;
    cli.fetch_ata(&ata)?;

    println!("\n========================================");
    println!("Мінтимо токени:");
    let amount = 5_000_000;
    cli.mint_tokens(&mint.pubkey(), &ata, amount)?;
    cli.fetch_mint(&mint.pubkey())?;
    cli.fetch_ata(&ata)?;

    println!("\n========================================");
    println!("Спалюємо токени:");
    cli.burn_tokens(&mint.pubkey(), &ata, amount / 2)?;
    cli.fetch_mint(&mint.pubkey())?;
    cli.fetch_ata(&ata)?;

    println!("\n========================================");
    println!("Трансферимо токени:");
    let receiver = Keypair::new();
    let receiver_ata = cli.create_token_account(&mint.pubkey(), &receiver.pubkey())?;

    cli.transfer_tokens(&ata, &receiver_ata, amount / 4)?;
    cli.fetch_ata(&ata)?;

    println!("\n========================================");
    println!("Заморожуємо токени:");
    cli.freeze_tokens(&mint.pubkey(), &ata)?;
    cli.fetch_mint(&mint.pubkey())?;
    cli.fetch_ata(&ata)?;

    println!("\n========================================");
    let res = cli.transfer_tokens(&ata, &receiver_ata, amount / 4);
    println!(
        "Спроба трансферу з замороженого мінта: 
        \nERROR: {:?}",
        res.unwrap_err()
    );

    cli.fetch_mint(&mint.pubkey())?;
    cli.fetch_ata(&ata)?;

    Ok(())
}

fn mint_with_metadata(cli: &cli::SolanaCli) -> anyhow::Result<()> {
    println!("\n========================================");
    println!("Створюємо mint:");
    let mint = cli.create_mint(&cli.payer.pubkey(), Some(&cli.payer.pubkey()), 6)?;

    println!("\n========================================");
    println!("Створюємо Metaplex metadata:");
    cli.create_metaplex_metadata(
        &mint.pubkey(),
        "Workshop Token".to_string(),
        "WRK".to_string(),
        "https://raw.githubusercontent.com/solana-developers/program-examples/new-examples/tokens/tokens/.assets/spl-token.json".to_string(),
    )?;

    println!("\n========================================");
    println!("Зчитуємо metadata:");
    cli.fetch_metadata(&mint.pubkey())?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let example = std::env::args()
        .nth(1)
        .ok_or(anyhow::anyhow!("Вкажіть номер завдання"))?;

    let cli = cli::SolanaCli::new()?;

    match example.as_str() {
        "1" => token_operations(&cli),
        "2" => wrap_unwrap_sol(&cli),
        "3" => mint_with_metadata(&cli),
        _ => anyhow::bail!("Запустіть завдання: 1, 2, або 3"),
    }
}
