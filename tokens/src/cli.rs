use anyhow::{Context, anyhow};
use solana_cli_config::Config;
use solana_client::{
    rpc_client::RpcClient, rpc_config::CommitmentConfig, rpc_response::transaction::Transaction,
};
use solana_sdk::{
    message::Instruction,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{Keypair, Signature},
    signer::EncodableKey,
};

pub struct SolanaCli {
    pub client: RpcClient,
    pub payer: Keypair,
}

impl SolanaCli {
    pub fn new() -> anyhow::Result<Self> {
        let json_rpc_url = std::env::var("SOLANA_RPC_URL")
            .unwrap_or_else(|_| "https://api.devnet.solana.com".into());

        let config_file = solana_cli_config::CONFIG_FILE
            .as_ref()
            .ok_or_else(|| anyhow!("unable to get config file path"))?;
        let cfg = Config::load(config_file)?;

        let client =
            RpcClient::new_with_commitment(json_rpc_url.clone(), CommitmentConfig::confirmed());

        let payer = Keypair::read_from_file(cfg.keypair_path)
            .map_err(|e| anyhow!("Failed to read keypair from file: {}", e))?;

        Ok(Self { client, payer })
    }

    pub fn send_tx(
        &self,
        instructions: &[Instruction],
        payer: &Pubkey,
        signers: &[&Keypair],
    ) -> anyhow::Result<Signature> {
        let recent_blockhash = self.client.get_latest_blockhash()?;
        let tx = Transaction::new_signed_with_payer(
            instructions,
            Some(payer),
            signers,
            recent_blockhash,
        );

        let signature = self
            .client
            .send_and_confirm_transaction(&tx)
            .context("Send tx")?;

        println!("Transaction signature: {}", signature);

        Ok(signature)
    }

    pub fn fetch_account<T: Pack>(&self, account: &Pubkey) -> anyhow::Result<T> {
        let ata_account = self.client.get_account(account).context("Fetch account")?;
        T::unpack_unchecked(&ata_account.data).context("Decode ATA state")
    }
}
