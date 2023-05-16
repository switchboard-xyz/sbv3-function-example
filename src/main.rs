pub mod ipfs;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::signer::keypair::keypair_from_seed;
use anchor_client::solana_sdk::signer::keypair::Keypair;
use anchor_client::Client;
use anchor_client::Cluster;
pub use ipfs::*;
use serde::{Deserialize, Serialize};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::read_keypair_file;
use std::sync::Arc;
use tokio;
use sgx_quote::Quote;
use getrandom::getrandom;
use sha2::{Digest, Sha256};
use std::fs;
use bincode;
use hex;
use anchor_client::solana_sdk::instruction::Instruction;
type AnchorClient = Client<Arc<Keypair>>;

#[derive(Clone, Debug, PartialEq)]
pub enum Err {
    Generic,
    SgxError,
    SgxWriteError,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionResult {
    pub version: u32,
    pub chain: Chain,
    pub key: [u8; 32],
    pub serialized_tx: Vec<u8>,
    pub quote: Vec<u8>,
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

pub async fn secure_sign_ix(sgx_kp: &Keypair) -> Instruction {
    Instruction {
        program_id: Pubkey::default(),
        accounts: vec![],
        data: vec![],
    }
}

#[tokio::main(worker_threads = 12)]
async fn main() {
    println!("START");
    let mut randomness = [0; 32];
    let quote_kp = Arc::new(keypair_from_seed(&randomness).unwrap());
    let ix = secure_sign_ix(&quote_kp).await;
    let mut result = FunctionResult {
        version: 1,
        chain: Chain::Solana,
        key: quote_kp.pubkey().to_bytes(),
        serialized_tx: bincode::serialize(&ix).unwrap(),
        quote: vec![],
    };
    let quote_raw = Sgx::gramine_generate_quote(&bincode::serialize(&result).unwrap()).unwrap();
    result.quote = quote_raw;
    println!("{:#?}", hex::encode(&bincode::serialize(&result).unwrap()));
}
