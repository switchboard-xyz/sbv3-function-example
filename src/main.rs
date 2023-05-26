pub mod sdk;
pub use sdk::*;
use anchor_client::solana_client::rpc_client::RpcClient;
use anchor_client::solana_sdk::signature::Signer;
use anchor_client::solana_sdk::signer::keypair::keypair_from_seed;
use anchor_client::solana_sdk::signer::keypair::Keypair;





use solana_sdk::pubkey::Pubkey;


use tokio;




use bincode;
use hex;
use serde_json;
use anchor_client::solana_sdk::instruction::Instruction;
use solana_sdk::transaction::Transaction;
use solana_sdk::message::Message;
use std::str::FromStr;


pub async fn secure_sign_ix(_sgx_kp: &Keypair) -> Instruction {
    Instruction {
        program_id: Pubkey::from_str("2No5FVKPAAYqytpkEoq93tVh33fo4p6DgAnm4S6oZHo7").unwrap(),
        accounts: vec![],
        data: vec![],
    }
}

#[tokio::main(worker_threads = 12)]
async fn main() {
    println!("START");
    let url = "https://api.devnet.solana.com".to_string();
    let client = RpcClient::new(url);
    let randomness = [0; 32];
    let quote_kp = keypair_from_seed(&randomness).unwrap();
    let quote_raw = Sgx::gramine_generate_quote(&quote_kp.pubkey().to_bytes()).unwrap();

    let blockhash = client.get_latest_blockhash().unwrap();
    let msg = Message::default();
    let mut tx = Transaction::new_unsigned(msg);
    tx.partial_sign_unchecked(&[&quote_kp], vec![2], blockhash);
    let result = FunctionResult {
        version: 1,
        chain: Chain::Solana,
        key: quote_kp.pubkey().to_bytes(),
        serialized_tx: bincode::serialize(&tx).unwrap(),
        quote: quote_raw,
        ..Default::default()
    };
    println!("{:#?}", hex::encode(&serde_json::to_string(&result).unwrap()));
}
