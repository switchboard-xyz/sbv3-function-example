use anchor_client::anchor_lang::prelude::*;

use anchor_client::anchor_lang::AnchorDeserialize;
use anchor_client::anchor_lang::AnchorSerialize;
use anchor_client::anchor_lang::Discriminator;
use anchor_client::anchor_lang::InstructionData;
use anchor_client::anchor_lang::ToAccountMetas;

use anchor_client::solana_sdk::instruction::Instruction;

use anchor_client::solana_sdk::signer::keypair::Keypair;

use bytemuck;
use bytemuck::{Pod, Zeroable};
use getrandom::getrandom;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use sha2::{Digest, Sha256};

use solana_sdk::instruction::AccountMeta;

use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;

use spl_token;
use std::fs;
use std::result::Result;

use std::sync::Arc;

lazy_static! {
    static ref ATTESTATION_PID: Pubkey = pubkey!("2No5FVKPAAYqytpkEoq93tVh33fo4p6DgAnm4S6oZHo7");
}

#[derive(Clone, Debug, PartialEq)]
pub enum Err {
    Generic,
    SgxError,
    SgxWriteError,
    AnchorParseError,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct FunctionResult {
    pub version: u32,
    pub chain: Chain,
    pub key: [u8; 32],
    pub signer: [u8; 32],
    pub serialized_tx: Vec<u8>,
    pub quote: Vec<u8>,
    pub program: Vec<u8>,
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Chain {
    Solana,
    Arbitrum,
    Bsc,
    Coredao,
    Aptos,
    Sui,
}
impl Default for Chain {
    fn default() -> Self {
        Self::Solana
    }
}

pub struct Sgx {}
impl Sgx {
    pub fn gramine_generate_quote(user_data: &[u8]) -> std::result::Result<Vec<u8>, Err> {
        match fs::metadata("/dev/attestation/quote") {
            Ok(_) => (),
            Err(_) => return Err(Err::SgxError),
        }
        let mut hasher = Sha256::new();
        hasher.update(user_data);
        let hash_result = &hasher.finalize()[..32];

        let mut data = [0u8; 64];
        data[..32].copy_from_slice(hash_result);

        let user_report_data_path = "/dev/attestation/user_report_data";
        if fs::write(user_report_data_path, &data[..]).is_err() {
            return Err(Err::SgxWriteError);
        }

        fs::read("/dev/attestation/quote").map_err(|_| Err::SgxError)
    }

    pub fn read_rand(buf: &mut [u8]) -> std::result::Result<(), Err> {
        // https://gramine.readthedocs.io/en/latest/devel/features.html#randomness
        getrandom(buf).map_err(|_| Err::SgxError)
    }
}

pub struct FunctionVerify {
    pub function: Pubkey,
    pub fn_signer: Pubkey,
    pub fn_quote: Pubkey,
    pub verifier_quote: Pubkey,
    pub attestation_queue: Pubkey,
    pub escrow: Pubkey,
    pub receiver: Pubkey,
    pub permission: Pubkey,
    pub state: Pubkey,
    pub token_program: Pubkey,
    pub payer: Pubkey,
    pub system_program: Pubkey,
}
#[derive(Clone, AnchorSerialize, AnchorDeserialize)]
pub struct FunctionVerifyParams {
    pub observed_time: i64,
    pub next_allowed_timestamp: i64,
    pub is_failure: bool,
    pub mr_enclave: [u8; 32],
}
pub struct FunctionVerifyArgs {
    pub function: Pubkey,
    pub fn_signer: Pubkey,
    pub reward_receiver: Pubkey,
    pub verifier: Pubkey,
    pub payer: Pubkey,
    pub timestamp: i64,
    pub next_allowed_timestamp: i64,
    pub is_failure: bool,
    pub mr_enclave: [u8; 32],
}
impl Discriminator for FunctionVerifyParams {
    const DISCRIMINATOR: [u8; 8] = [0; 8];
    fn discriminator() -> [u8; 8] {
        ix_discriminator("function_verify")
    }
}
impl InstructionData for FunctionVerifyParams {}
impl ToAccountMetas for FunctionVerify {
    fn to_account_metas(&self, _: Option<bool>) -> Vec<AccountMeta> {
        vec![
            AccountMeta {
                pubkey: self.function,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.fn_signer,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.fn_quote,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.verifier_quote,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.attestation_queue,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.escrow,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.receiver,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.permission,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.state,
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.token_program,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: self.payer,
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: self.system_program,
                is_signer: false,
                is_writable: false,
            },
        ]
    }
}

impl FunctionVerify {
    pub async fn build(
        client: &anchor_client::Client<Arc<Keypair>>,
        args: FunctionVerifyArgs,
    ) -> Result<Instruction, Err> {
        let fn_data: FunctionAccountData = load(client, args.function).await?;
        let queue = fn_data.attestation_queue;
        let queue_data: AttestationQueueAccountData = load(client, queue).await?;
        let escrow = fn_data.escrow;
        let (fn_quote, _) = Pubkey::find_program_address(
            &[b"QuoteAccountData", &args.function.to_bytes()],
            &ATTESTATION_PID,
        );
        let (permission, _) = Pubkey::find_program_address(
            &[
                b"PermissionccountData",
                &queue_data.authority.to_bytes(),
                &queue.to_bytes(),
                &args.function.to_bytes(),
            ],
            &ATTESTATION_PID,
        );
        let (state, _) = Pubkey::find_program_address(
            &[
                b"STATE",
                &queue_data.authority.to_bytes(),
                &queue.to_bytes(),
                &args.function.to_bytes(),
            ],
            &ATTESTATION_PID,
        );
        let accounts = Self {
            function: args.function,
            fn_signer: args.fn_signer,
            fn_quote,
            verifier_quote: args.verifier,
            attestation_queue: queue,
            escrow,
            receiver: args.reward_receiver,
            permission,
            state,
            token_program: spl_token::ID,
            payer: args.payer,
            system_program: solana_sdk::system_program::ID,
        };
        Ok(build_ix(
            accounts,
            FunctionVerifyParams {
                observed_time: args.timestamp,
                next_allowed_timestamp: args.next_allowed_timestamp,
                is_failure: args.is_failure,
                mr_enclave: args.mr_enclave,
            },
        ))
    }
}

pub fn ix_discriminator(name: &str) -> [u8; 8] {
    let preimage = format!("global:{}", name);
    let mut sighash = [0u8; 8];
    sighash.copy_from_slice(&solana_sdk::hash::hash(preimage.as_bytes()).to_bytes()[..8]);
    sighash
}

pub fn build_ix<A: ToAccountMetas, I: InstructionData + Discriminator>(
    accounts: A,
    params: I,
) -> Instruction {
    Instruction {
        program_id: *ATTESTATION_PID,
        accounts: accounts.to_account_metas(None),
        data: params.data(),
    }
}

pub async fn load<T: bytemuck::Pod>(
    client: &anchor_client::Client<Arc<Keypair>>,
    key: Pubkey,
) -> Result<T, Err> {
    let data = client
        .program(*ATTESTATION_PID)
        .async_rpc()
        .get_account_data(&key)
        .await
        .map_err(|_| Err::AnchorParseError)?;
    Ok(*bytemuck::try_from_bytes::<T>(&data[8..]).map_err(|_| Err::AnchorParseError)?)
}
#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct AttestationQueueAccountData {
    // Authority controls adding/removing allowed enclave measurements
    pub authority: Pubkey,
    // allowed enclave measurements
    pub mr_enclaves: [[u8; 32]; 32],
    pub mr_enclaves_len: u32,
    pub data: [Pubkey; 128],
    pub data_len: u32,
    // Allow authority to force add a node after X seconds with no heartbeat
    pub allow_authority_override_after: i64,
    // Even if a heartbeating machine quote verifies with proper measurement,
    // require authority signoff.
    pub require_authority_heartbeat_permission: bool,
    pub require_usage_permissions: bool,
    pub max_quote_verification_age: i64,
    pub reward: u32, //TODO
    pub last_heartbeat: i64,
    pub node_timeout: i64,
    pub curr_idx: u32,
    pub gc_idx: u32,
    pub _ebuf: [u8; 1024],
}
unsafe impl Pod for AttestationQueueAccountData {}
unsafe impl Zeroable for AttestationQueueAccountData {}

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct FunctionAccountData {
    pub name: [u8; 64],
    pub metadata: [u8; 256],
    pub authority: Pubkey,
    ///
    pub container_registry: [u8; 64],
    pub container: [u8; 64],
    pub version: [u8; 32],
    ///
    pub attestation_queue: Pubkey,
    pub queue_idx: u32,
    pub last_execution_timestamp: i64,
    pub next_allowed_timestamp: i64,
    pub schedule: [u8; 64],
    pub escrow: Pubkey,
    pub status: FunctionStatus,
    pub _ebuf: [u8; 1024],
}
unsafe impl Pod for FunctionAccountData {}
unsafe impl Zeroable for FunctionAccountData {}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum FunctionStatus {
    None = 0,
    Active = 1 << 0,
    NonExecutable = 1 << 1,
    Expired = 1 << 2,
    OutOfFunds = 1 << 3,
    InvalidPermissions = 1 << 4,
}
