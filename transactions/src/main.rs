use std::{
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::{anyhow, Ok};
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

fn tree_reciver_tx(client: &RpcClient, signer: &Keypair) -> anyhow::Result<Signature> {
    let reciver1_keys_path = format!("{}/local_wallet.json", env!("CARGO_MANIFEST_DIR"));
    let reciver1 = Keypair::read_from_file(&reciver1_keys_path).map_err(|e| {
        anyhow!(
            "Failed to read keypair from file: {} and path {}",
            e,
            &reciver1_keys_path
        )
    })?;
    let reciver2 = Pubkey::from_str("devwuNsNYACyiEYxRNqMNseBpNnGfnd4ZwNHL7sphqv")?;
    let reciver3 = Keypair::new();

    let transfer_amount = LAMPORTS_PER_SOL / 100;
    let ix_1 = transfer(&signer.pubkey(), &reciver1.pubkey(), transfer_amount);
    let ix_2 = transfer(&signer.pubkey(), &reciver2, transfer_amount);
    let ix_3 = transfer(&signer.pubkey(), &reciver3.pubkey(), transfer_amount);

    let recent_blockhash = client
        .get_latest_blockhash()
        .map_err(|e| anyhow!("Failed to fetcu blockhash: {e}"))?;
    let tx = Transaction::new_signed_with_payer(
        &[ix_1, ix_2, ix_3],
        Some(&signer.pubkey()),
        &[signer],
        recent_blockhash,
    );

    let sim = client
        .simulate_transaction_with_config(
            &tx,
            solana_client::rpc_config::RpcSimulateTransactionConfig {
                sig_verify: true,
                replace_recent_blockhash: false,
                commitment: Some(CommitmentConfig::processed()),
                ..Default::default()
            },
        )
        .map_err(|e| anyhow!("Failed to simulate: {e}"))?;

    println!("err: {:?}", sim.value.err);
    println!("logs: {:?}", sim.value.logs);
    println!("units consumed: {:?}", sim.value.units_consumed);

    client
        .send_and_confirm_transaction(&tx)
        .map_err(|e| anyhow!("Failed to send tx: {e}"))
}

fn send_priority_fee_trans(priority_fee_lamports: u64) -> anyhow::Result<Signature> {
    let (client, signer) = get_cli_and_signer()?;

    let recent_blockhash = client
        .get_latest_blockhash()
        .map_err(|e| anyhow!("Failed to fetcu blockhash: {e}"))?;

    let cu_budget: u64 = 400_000; // same as set_compute_unit_limit
    let micro_lamports_per_cu = priority_fee_lamports.saturating_mul(1_000_000) / cu_budget.max(1);
    let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(cu_budget as u32);
    let cu_price_ix = ComputeBudgetInstruction::set_compute_unit_price(micro_lamports_per_cu);

    let memo_program_id = Pubkey::from_str("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr")?;
    let msg = "Solana workshop: transactions 0xDecadance";
    let memo_ix = Instruction::new_with_bytes(memo_program_id, msg.as_bytes(), vec![]);

    let tx = Transaction::new_signed_with_payer(
        &[cu_price_ix, cu_limit_ix, memo_ix],
        Some(&signer.pubkey()),
        &[signer],
        recent_blockhash,
    );
    let start = Instant::now();
    let sig = client
        .send_transaction(&tx)
        .map_err(|e| anyhow!("Error sending tx {e}"))?;
    client.confirm_transaction(&sig)?;
    let elapsed_ms = start.elapsed().as_millis();
    println!("\n===========tx: {}===========", &sig);
    println!("tx confirm in {elapsed_ms} ms");
    Ok(sig)
}

fn main() -> anyhow::Result<()> {
    let (client, wallet) = get_cli_and_signer()?;

    // Homework 1-2 with sim: 3 reciver tx
    let tree_reciver_sig = tree_reciver_tx(&client, &wallet)?;
    print_tx_result(&client, &tree_reciver_sig)?;

    // Homework 3: priority fee
    let no_fee_tx = send_priority_fee_trans(0)?;
    std::thread::sleep(Duration::from_secs(2));
    print_tx_result(&client, &no_fee_tx)?;

    let fee_tx = send_priority_fee_trans(1000)?;
    std::thread::sleep(Duration::from_secs(2));
    print_tx_result(&client, &fee_tx)?;

    return Ok(());

    let receiver = Keypair::read_from_file("local_wallet.json")
        .map_err(|e| anyhow!("Failed to read keypair from file: {}", e))?;

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
