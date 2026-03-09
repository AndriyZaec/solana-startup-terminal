use anyhow::Context;
use borsh::BorshDeserialize;

use solana_sdk::{
    message::Instruction, program_pack::Pack, pubkey::Pubkey, signature::Keypair, signer::Signer,
};

use spl_token::{
    instruction::{freeze_account, mint_to},
    state::Mint,
};

use mpl_token_metadata::{
    accounts::Metadata, instructions::CreateMetadataAccountV3Builder, types::DataV2,
};

use crate::cli::SolanaCli;

const SYSTEM_PROGRAM: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");

impl SolanaCli {
    pub fn create_mint(
        &self,
        mint_authority: &Pubkey,
        freeze_authority: Option<&Pubkey>,
        decimals: u8,
    ) -> anyhow::Result<Keypair> {
        let mint = Keypair::new();

        let min_rent = self
            .client
            .get_minimum_balance_for_rent_exemption(Mint::LEN)
            .context("Fetch rent exemption")?;

        let create_account_instruction: Instruction =
            solana_system_interface::instruction::create_account(
                &self.payer.pubkey(),
                &mint.pubkey(),
                min_rent,
                Mint::LEN as u64,
                &spl_token::ID,
            );

        let initialize_mint_instruction: Instruction = spl_token::instruction::initialize_mint(
            &spl_token::ID,
            &mint.pubkey(),
            mint_authority,
            freeze_authority,
            decimals,
        )?;

        self.send_tx(
            &[create_account_instruction, initialize_mint_instruction],
            &self.payer.pubkey(),
            &[&self.payer, &mint],
        )?;

        println!("Created mint: {}", mint.pubkey());

        Ok(mint)
    }

    pub fn mint_tokens(
        &self,
        mint: &Pubkey,
        owner_ata: &Pubkey,
        amount: u64,
    ) -> anyhow::Result<()> {
        let instruction: Instruction = mint_to(
            &spl_token::ID,
            mint,
            owner_ata,
            &self.payer.pubkey(),
            &[&self.payer.pubkey()],
            amount,
        )?;

        self.send_tx(&[instruction], &self.payer.pubkey(), &[&self.payer])?;

        println!("Minted tokens to: {}", owner_ata);

        Ok(())
    }

    pub fn fetch_mint(&self, mint: &Pubkey) -> anyhow::Result<()> {
        let mint_state = self
            .fetch_account::<Mint>(mint)
            .context("fetch mint state")?;

        println!("\nMint state");
        println!("decimals: {}", mint_state.decimals);
        println!("supply: {}", mint_state.supply);
        println!("mint_authority: {:?}", mint_state.mint_authority);
        println!("freeze_authority: {:?}", mint_state.freeze_authority);
        println!("is_initialized: {}", mint_state.is_initialized);
        Ok(())
    }

    pub fn freeze_tokens(&self, mint: &Pubkey, owner_ata: &Pubkey) -> anyhow::Result<()> {
        let instruction: Instruction = freeze_account(
            &spl_token::ID,
            owner_ata,
            mint,
            &self.payer.pubkey(),
            &[&self.payer.pubkey()],
        )?;

        self.send_tx(&[instruction], &self.payer.pubkey(), &[&self.payer])?;

        println!("Account frozen: {}", owner_ata);

        Ok(())
    }

    pub fn create_metaplex_metadata(
        &self,
        mint: &Pubkey,
        name: String,
        symbol: String,
        uri: String,
    ) -> anyhow::Result<Pubkey> {
        let (metadata_pda, _) = Pubkey::find_program_address(
            &[b"metadata", mpl_token_metadata::ID.as_ref(), mint.as_ref()],
            &mpl_token_metadata::ID,
        );

        let ix = CreateMetadataAccountV3Builder::new()
            .metadata(metadata_pda)
            .mint(*mint)
            .mint_authority(self.payer.pubkey())
            .payer(self.payer.pubkey())
            .update_authority(self.payer.pubkey(), true)
            .system_program(SYSTEM_PROGRAM)
            .data(DataV2 {
                name,
                symbol,
                uri,
                seller_fee_basis_points: 0,
                creators: None,
                collection: None,
                uses: None,
            })
            .is_mutable(true)
            .instruction();

        self.send_tx(&[ix], &self.payer.pubkey(), &[&self.payer])?;

        println!("Created metadata PDA: {}", metadata_pda);

        Ok(metadata_pda)
    }

    pub fn fetch_metadata(&self, mint: &Pubkey) -> anyhow::Result<()> {
        let (metadata_pda, _) = Pubkey::find_program_address(
            &[b"metadata", mpl_token_metadata::ID.as_ref(), mint.as_ref()],
            &mpl_token_metadata::ID,
        );

        let account = self
            .client
            .get_account(&metadata_pda)
            .context("fetch metadata account")?;
        let metadata =
            Metadata::deserialize(&mut &account.data[..]).context("deserialize metadata")?;

        println!("\nMetaplex Metadata:");
        println!("Name: {}", metadata.name);
        println!("Symbol: {}", metadata.symbol);
        println!("URI: {}", metadata.uri);
        println!("Update Authority: {}", metadata.update_authority);
        println!("Mint: {}", metadata.mint);
        println!("Is Mutable: {}", metadata.is_mutable);
        println!(
            "Creators: {:?}",
            metadata.creators.as_ref().map(|c| c.len()).unwrap_or(0)
        );

        Ok(())
    }
}
