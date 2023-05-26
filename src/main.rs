pub mod sdk;


use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::signer::keypair::keypair_from_seed;

use anchor_client::Cluster;
use bincode;
use hex;
pub use sdk::*;
use serde_json;
use sgx_quote::Quote;
use solana_sdk::commitment_config::CommitmentConfig;

use solana_sdk::pubkey::Pubkey;
use solana_sdk::transaction::Transaction;
use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio;

#[tokio::main(worker_threads = 12)]
async fn main() {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    println!("START");
    let randomness = [0; 32];
    let enclave_signer = Arc::new(keypair_from_seed(&randomness).unwrap());
    let client = anchor_client::Client::new_with_options(
        Cluster::Devnet,
        enclave_signer.clone(),
        CommitmentConfig::processed(),
    );
    let quote_raw = Sgx::gramine_generate_quote(&enclave_signer.pubkey().to_bytes()).unwrap();
    let quote = Quote::parse(&quote_raw).unwrap();

    let blockhash = client
        .program(ATTESTATION_PID)
        .rpc()
        .get_latest_blockhash()
        .unwrap();
    let function = Pubkey::from_str(&env::var("FUNCTION_KEY").unwrap()).unwrap();
    let _ix = FunctionVerify::build(
        &client,
        FunctionVerifyArgs {
            function,
            fn_signer: enclave_signer.pubkey(),
            reward_receiver: Pubkey::from_str(&env::var("REWARD_RECEIVER").unwrap()).unwrap(),
            verifier: Pubkey::from_str(&env::var("VERIFIER").unwrap()).unwrap(),
            payer: Pubkey::from_str(&env::var("PAYER").unwrap()).unwrap(),
            timestamp: current_time,
            next_allowed_timestamp: current_time,
            is_failure: false,
            mr_enclave: quote.isv_report.mrenclave.try_into().unwrap(),
        },
    )
    .await
    .unwrap();
    let mut tx = Transaction::default();
    tx.partial_sign_unchecked(&[enclave_signer.as_ref()], vec![2], blockhash);
    let result = FunctionResult {
        version: 1,
        chain: Chain::Solana,
        key: function.to_bytes(),
        signer: enclave_signer.pubkey().to_bytes(),
        serialized_tx: bincode::serialize(&tx).unwrap(),
        quote: quote_raw,
        ..Default::default()
    };
    println!(
        "{:#?}",
        hex::encode(&serde_json::to_string(&result).unwrap())
    );
}
