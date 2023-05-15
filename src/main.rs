pub mod ipfs;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::signer::keypair::keypair_from_seed;
use anchor_client::solana_sdk::signer::keypair::Keypair;
use anchor_client::Client;
use anchor_client::Cluster;
pub use ipfs::*;
use sbac::sgx::Sgx;
use sbac::solana::*;
use serde::{Deserialize, Serialize};
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::keypair::read_keypair_file;
use std::sync::Arc;
use switchboard_attestation_client as sbac;
use tokio;
use sgx_quote::Quote;
type AnchorClient = Client<Arc<Keypair>>;

const VERIFIER_QUEUE: Pubkey = pubkey!("4AnQSCo6YJmmYVq2BUHx5vfxoqktpBmTbDh1NurxShgN");
const PID: Pubkey = pubkey!("Hxfwq7cxss4Ef9iDvaLb617dhageGyNWbDLLrg2sdQgT");

pub async fn solana_init_quote(anchor_client: &AnchorClient, payer: Arc<Keypair>) -> Arc<Keypair> {
    let mut randomness = [0; 32];
    Sgx::read_rand(&mut randomness).unwrap();
    let quote_kp = Arc::new(keypair_from_seed(&randomness).unwrap());
    let quote = Sgx::gramine_generate_quote(&quote_kp.pubkey().to_bytes()).unwrap();
    let quote_init_ixs = QuoteInitSimple::build(
        &anchor_client,
        QuoteInitSimpleArgs {
            quote: quote_kp.pubkey(),
            verifier_queue: VERIFIER_QUEUE,
            authority: quote_kp.pubkey(),
            data: quote.clone(),
        },
        vec![&payer, &quote_kp],
    )
    .unwrap();
    quote_kp
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct MyObject {
    value: i32,
}

#[tokio::main(worker_threads = 12)]
async fn main() {
    let quote_kp = Arc::new(keypair_from_seed(&randomness).unwrap());
    let quote = Sgx::gramine_generate_quote(&quote_kp.pubkey().to_bytes()).unwrap();
    let quote = Quote::parse(&quote).unwrap();
    println!("{:#?}", quote);
}
