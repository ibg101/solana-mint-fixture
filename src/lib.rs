#[allow(deprecated)]
use solana_sdk::{
    rent::Rent,
    hash::Hash,
    pubkey::Pubkey,
    program_pack::Pack,
    program_error::ProgramError,
    message::Message,
    system_program::ID as SYSTEM_PROGRAM_ID,
    system_instruction,
    transaction::Transaction,
    instruction::{Instruction, AccountMeta},
    signer::keypair::Keypair,
    signature::Signer,
};
use spl_token_2022::{
    state::Mint,
    instruction as spl_token_2022_ix,
    ID as SPL_TOKEN_2022_ID
};
#[cfg(feature = "banks")]
use solana_program_test::{
    BanksClient, 
    BanksClientError
};
#[cfg(feature = "rpc")]
use solana_client::{
    client_error::ClientError,
    nonblocking::rpc_client::RpcClient
};


/// If `spl_associated_token_account_client` will be added, this must be removed.
/// 
/// Since i've implemented ix crafting & ATA derivation manually, importing `spl_associated_token_account_client` only for ID is an overkill 
const ATA_PROGRAM_ID: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

#[derive(Debug)]
pub enum MintFixtureError {
    #[cfg(feature = "rpc")]
    Client(ClientError),

    #[cfg(feature = "banks")]
    Banks(BanksClientError),
    
    Program(ProgramError)
}

#[cfg(feature = "rpc")]
impl From<ClientError> for MintFixtureError {
    fn from(value: ClientError) -> Self {
        Self::Client(value)
    }
}

#[cfg(feature = "banks")]
impl From<BanksClientError> for MintFixtureError {
    fn from(value: BanksClientError) -> Self {
        Self::Banks(value)
    }
}

impl From<ProgramError> for MintFixtureError {
    fn from(value: ProgramError) -> Self {
        Self::Program(value)
    }
}

impl std::error::Error for MintFixtureError {}
impl std::fmt::Display for MintFixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            #[cfg(feature = "banks")]
            Self::Banks(err) => write!(f, "{}", err),
            
            #[cfg(feature = "rpc")]
            Self::Client(err) => write!(f, "{}", err),
            
            Self::Program(err) => write!(f, "{}", err)
        }
    }
}

pub enum MintFixtureClient<'a> {
    #[cfg(feature = "rpc")]
    Rpc(&'a RpcClient),

    #[cfg(feature = "banks")]
    Banks(&'a BanksClient),
}

pub struct MintFixture<'a> {
    client: MintFixtureClient<'a>,
    payer: &'a Keypair,
    payer_pkey: &'a Pubkey,
    rent: &'a Rent
}

impl<'a> MintFixture<'a> {
    pub fn new(
        client: MintFixtureClient<'a>,
        payer: &'a Keypair,
        payer_pkey: &'a Pubkey,
        rent: &'a Rent 
    ) -> Self {        
        Self { client, payer, payer_pkey, rent }
    }
    
    /// Returns created Mint Account Pubkey.
    /// 
    /// If `freeze_authority` is None, consider using `create_and_initialize_mint_without_freeze` instead.
    pub async fn create_and_initialize_mint(
        &self, 
        mint_decimals: u8, 
        freeze_authority: Option<&Pubkey>,
        latest_blockhash: &Hash
    ) -> Result<Pubkey, MintFixtureError> {
        // 1. Craft Mint Account ix using System Program
        let mint_keypair: Keypair = Keypair::new();
        let mint_pkey: Pubkey = mint_keypair.pubkey();

        let create_mint_ix: Instruction = system_instruction::create_account(
            self.payer_pkey, 
            &mint_pkey, 
            self.rent.minimum_balance(Mint::LEN), 
            Mint::LEN as u64, 
            &SPL_TOKEN_2022_ID
        );

        // 2. Craft Initialize Mint Account ix using SPL program
        let initialize_mint_ix: Instruction = spl_token_2022_ix::initialize_mint(
            &SPL_TOKEN_2022_ID, 
            &mint_pkey, 
            self.payer_pkey, 
            freeze_authority, 
            mint_decimals
        )?;

        // 3. Craft atomic tx & sign and send it
        let message: Message = Message::new(
            &[
                create_mint_ix,
                initialize_mint_ix
            ], 
            Some(self.payer_pkey)
        );
        let mut create_and_initiliaze_tx: Transaction = Transaction::new_unsigned(message);

        create_and_initiliaze_tx.sign(&[self.payer], *latest_blockhash);
        self.process_transaction(create_and_initiliaze_tx).await?;

        Ok(mint_pkey)
    }

    pub async fn create_and_initialize_mint_without_freeze(&self, mint_decimals: u8, latest_blockhash: &Hash) -> Result<Pubkey, MintFixtureError> {
        Self::create_and_initialize_mint(&self, mint_decimals, None, latest_blockhash).await
    }

    /// Returns created ATA Pubkey.
    pub async fn create_and_initialize_ata(&self, mint_pkey: &Pubkey, latest_blockhash: &Hash) -> Result<Pubkey, MintFixtureError> {
        // If `spl_associated_token_account_client` will be added, this must be removed.
        let ata_pda: Pubkey = Pubkey::find_program_address(
            &[
                self.payer_pkey.as_ref(),
                SPL_TOKEN_2022_ID.as_ref(),
                mint_pkey.as_ref()
            ], 
            &ATA_PROGRAM_ID
        ).0;

        // If `spl_associated_token_account_client` will be added, this must be removed.
        let create_ata_ix: Instruction = Instruction::new_with_bytes(
            ATA_PROGRAM_ID, 
            &[0],  // AssociatedTokenAccount::Create ix
            vec![
                AccountMeta::new(*self.payer_pkey, true),
                AccountMeta::new(ata_pda, false),
                AccountMeta::new_readonly(*self.payer_pkey, false),
                AccountMeta::new_readonly(*mint_pkey, false),
                AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
                AccountMeta::new_readonly(SPL_TOKEN_2022_ID, false)
            ]
        );

        let message: Message = Message::new(&[create_ata_ix], Some(self.payer_pkey));
        let mut create_ata_tx: Transaction = Transaction::new_unsigned(message); 

        create_ata_tx.sign(&[self.payer], *latest_blockhash);
        self.process_transaction(create_ata_tx).await?;

        Ok(ata_pda)
    }
    
    /// All this fn does is **minting** new **tokens** with the given Mint Account Pubkey **to** the provided **ATA**.
    pub async fn mint_to_ata(
        &self, 
        mint_pkey: &Pubkey, 
        ata_pda: &Pubkey, 
        mint_amount: u64,
        latest_blockhash: &Hash
    ) -> Result<(), MintFixtureError> {
        let mint_to_ix: Instruction = spl_token_2022_ix::mint_to(
            &SPL_TOKEN_2022_ID, 
            mint_pkey, 
            ata_pda, 
            self.payer_pkey, 
            &[], 
            mint_amount
        )?;
        let message: Message = Message::new(&[mint_to_ix], Some(self.payer_pkey));
        let mut mint_to_tx: Transaction = Transaction::new_unsigned(message);

        mint_to_tx.sign(&[self.payer], *latest_blockhash);
        self.process_transaction(mint_to_tx).await?;

        Ok(())
    }

    async fn process_transaction(&self, tx: Transaction) -> Result<(), MintFixtureError> {
        match self.client {
            #[cfg(feature = "rpc")]
            MintFixtureClient::Rpc(client) => {
                client.send_and_confirm_transaction(&tx).await?;
            },

            #[cfg(feature = "banks")]
            MintFixtureClient::Banks(client) => {
                client.process_transaction(tx).await?;
            }
        }

        Ok(())
    }
}