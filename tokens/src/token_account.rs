use anyhow::Context;
use solana_client::{
    client_error::{ClientError, ClientErrorKind},
    rpc_request::RpcError,
};
use solana_sdk::{message::Instruction, pubkey::Pubkey, signer::Signer};
use spl_associated_token_account::{
    get_associated_token_address, instruction::create_associated_token_account,
};
use spl_token::{instruction::burn, state::Account as TokenAccount};

use crate::cli::SolanaCli;

impl SolanaCli {
    pub fn create_token_account(&self, mint: &Pubkey, owner: &Pubkey) -> anyhow::Result<Pubkey> {
        let ata = get_associated_token_address(owner, mint);

        let instruction =
            create_associated_token_account(&self.payer.pubkey(), owner, mint, &spl_token::ID);

        match self.send_tx(&[instruction], &self.payer.pubkey(), &[&self.payer]) {
            Ok(_) => println!("Created ATA: {}", ata),
            Err(e) => Self::handle_create_ata_error(e)?,
        }

        Ok(ata)
    }

    fn handle_create_ata_error(e: anyhow::Error) -> anyhow::Result<()> {
        let err = e.downcast::<ClientError>()?;

        let already_exists = matches!(
            err.kind(),
            ClientErrorKind::RpcError(RpcError::RpcResponseError { message, .. })
                if message == "Provided owner is not allowed"
        );

        if already_exists {
            println!("ATA already exists");
            Ok(())
        } else {
            Err(err.into())
        }
    }
    pub fn close_token_account(&self, owner_ata: &Pubkey) -> anyhow::Result<()> {
        let instruction: Instruction = spl_token::instruction::close_account(
            &spl_token::ID,
            owner_ata,
            &self.payer.pubkey(),
            &self.payer.pubkey(),
            &[&self.payer.pubkey()],
        )?;

        self.send_tx(&[instruction], &self.payer.pubkey(), &[&self.payer])?;

        println!("Closed associated token account: {}", owner_ata);
        Ok(())
    }

    pub fn wrap_sol(&self, owner_ata: &Pubkey, amount: u64) -> anyhow::Result<()> {
        let native_transfer_ix: Instruction =
            solana_system_interface::instruction::transfer(&self.payer.pubkey(), owner_ata, amount);

        let sync_native_ix: Instruction =
            spl_token::instruction::sync_native(&spl_token::ID, owner_ata)?;

        self.send_tx(
            &[native_transfer_ix, sync_native_ix],
            &self.payer.pubkey(),
            &[&self.payer],
        )?;

        println!(
            "Synced and transferred {} lamports to wrapped SOL account: {}",
            amount, owner_ata
        );

        Ok(())
    }

    pub fn fetch_ata(&self, ata: &Pubkey) -> anyhow::Result<()> {
        let ata_state = self
            .fetch_account::<TokenAccount>(ata)
            .context("fetch ATA state")?;

        println!("\nToken account state:");
        println!("mint: {}", ata_state.mint);
        println!("owner: {}", ata_state.owner);
        println!("amount: {}", ata_state.amount);
        println!("delegate: {:?}", ata_state.delegate);
        println!("state: {:?}", ata_state.state);
        println!("is_native: {:?}", ata_state.is_native);
        println!("delegated_amount: {}", ata_state.delegated_amount);
        println!("close_authority: {:?}", ata_state.close_authority);

        Ok(())
    }

    pub fn burn_tokens(
        &self,
        mint: &Pubkey,
        owner_ata: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<()> {
        let instruction: Instruction = burn(
            &spl_token::ID,
            owner_ata,
            mint,
            &self.payer.pubkey(),
            &[&self.payer.pubkey()],
            amount,
        )?;

        self.send_tx(&[instruction], &self.payer.pubkey(), &[&self.payer])?;

        println!("Burned tokens from: {}", owner_ata);

        Ok(())
    }

    pub fn transfer_tokens(
        &self,
        owner_ata: &Pubkey,
        receiver_ata: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<()> {
        let instruction: Instruction = spl_token::instruction::transfer(
            &spl_token::ID,
            owner_ata,
            receiver_ata,
            &self.payer.pubkey(),
            &[&self.payer.pubkey()],
            amount,
        )?;

        self.send_tx(&[instruction], &self.payer.pubkey(), &[&self.payer])?;

        println!(
            "Transferred {} tokens from {} to {}",
            amount, owner_ata, receiver_ata
        );

        Ok(())
    }
}
