use std::str::FromStr;

use anyhow::{Ok, anyhow};
use solana_cli_config::Config;
use solana_client::{
    rpc_client::RpcClient,
    rpc_config::{CommitmentConfig, RpcTransactionConfig, UiTransactionEncoding},
    rpc_response::transaction::Transaction,
};
use solana_compute_budget_interface::ComputeBudgetInstruction;
use solana_sdk::{
    message::Instruction,
    native_token::LAMPORTS_PER_SOL,
    pubkey::Pubkey,
    signature::{Keypair, Signature, Signer},
    signer::EncodableKey,
};

use solana_system_interface::instruction::transfer;

fn send_tx(
    client: &RpcClient,
    sender: &Keypair,
    instructions: &[Instruction],
) -> anyhow::Result<Signature> {
    let recent_blockhash = client.get_latest_blockhash()?;

    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&sender.pubkey()),
        &[sender],
        recent_blockhash,
    );

    client.send_and_confirm_transaction(&tx).map_err(Into::into)
}

fn print_tx_result(client: &RpcClient, signature: &Signature) -> anyhow::Result<()> {
    let tx_details = client.get_transaction_with_config(
        signature,
        RpcTransactionConfig {
            encoding: Some(UiTransactionEncoding::JsonParsed),
            commitment: Some(CommitmentConfig::confirmed()),
            ..Default::default()
        },
    )?;

    println!("\nSignature: {}", signature);
    println!("Slot: {}", tx_details.slot);
    println!("Block time: {:?}", tx_details.block_time);

    if let Some(meta) = &tx_details.transaction.meta {
        println!("Fee: {} lamports", meta.fee);
        println!("Pre-balances:  {:?}", meta.pre_balances);
        println!("Post-balances: {:?}", meta.post_balances);

        if meta.err.is_none() {
            println!("Статус: Success");
        } else {
            println!("Статус: Error — {:?}", meta.err);
        }
    }

    Ok(())
}

fn get_cli_and_signer() -> anyhow::Result<(RpcClient, Keypair)> {
    let json_rpc_url =
        std::env::var("SOLANA_RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".into());

    // Стягуємо з конфігу клієнта:
    let config_file = solana_cli_config::CONFIG_FILE
        .as_ref()
        .ok_or_else(|| anyhow!("unable to get config file path"))?;
    let cfg = Config::load(config_file)?;

    // Ініціалізуємо клієнта
    let client =
        RpcClient::new_with_commitment(json_rpc_url.clone(), CommitmentConfig::confirmed());

    let wallet = Keypair::read_from_file(&cfg.keypair_path)
        .map_err(|e| anyhow!("Failed to read keypair: {}", e))?;

    Ok((client, wallet))
}

fn main() -> anyhow::Result<()> {
    let (client, wallet) = get_cli_and_signer()?;
    let receiver = Keypair::new();

    // 1. Відправка SOL
    println!("\n========================================");
    println!("Виконання інструкції трансферу SOL");
    println!("========================================");

    println!("Відправник: {}", wallet.pubkey());
    println!("Отримувач:  {}", receiver.pubkey());

    let transfer_amount = LAMPORTS_PER_SOL / 100; // 0.01 SOL
    let transfer_ix = transfer(&wallet.pubkey(), &receiver.pubkey(), transfer_amount);

    let signature = send_tx(&client, &wallet, &[transfer_ix])?;
    print_tx_result(&client, &signature)?;

    let sender_balance = client.get_balance(&wallet.pubkey())?;
    let receiver_balance = client.get_balance(&receiver.pubkey())?;

    println!(
        "Баланс відправника після: {} SOL",
        sender_balance as f64 / LAMPORTS_PER_SOL as f64
    );
    println!(
        "Баланс отримувача після:  {} SOL",
        receiver_balance as f64 / LAMPORTS_PER_SOL as f64
    );

    // 2. Виконання інструкції Memo Program + Compute Budget Program
    println!("\n========================================");
    println!("Виконання інструкції Memo Program + Compute Budget Program");
    println!("========================================");

    // Вносимо список адрес, які будуть writable в транзакції - і беремо результат за останній слот
    let binding = client.get_recent_prioritization_fees(&[wallet.pubkey(), receiver.pubkey()])?;

    // Ціна за compute unit може бути відсутня, якщо в останніх слотах не було транзакцій з пріоритетністю, тому додаємо запас
    // Плата за пріоритетність виражається в мікро лампортах (0,000001 лампорта) за обчислювальну одиницю (CU)
    let cu_price = binding
        .last()
        .ok_or(anyhow!(
            "Цей метод має повертати результат хоча б за один слот!"
        ))?
        .prioritization_fee
        + 1_000_000;

    println!("Ціна за CU: {:?} microlamports", cu_price);
    let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);
    let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(cu_price);

    let memo_program_id = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")?;
    let msg = "Solana workshop: transactions";
    let memo_ix = Instruction::new_with_bytes(memo_program_id, msg.as_bytes(), vec![]);

    let signature = send_tx(&client, &wallet, &[cu_limit_ix, cu_price_ix, memo_ix])?;
    print_tx_result(&client, &signature)?;

    Ok(())
}
