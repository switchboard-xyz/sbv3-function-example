pub mod sdk;
pub use sdk::*;
use tokio;
use solana_sdk::system_instruction;
use solana_sdk::signature::Signer;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main(worker_threads = 12)]
async fn main() {
    let enclave_signer = generate_signer();
    let payer = Pubkey::from_str(&std::env::var("PAYER").unwrap()).unwrap();
    let instructions = vec![
        system_instruction::transfer(
            &enclave_signer.pubkey(),
            &enclave_signer.pubkey(),
            1,
        )
    ];
    let function_output =
        FunctionResult::generate_verifiable_solana_tx(enclave_signer, instructions)
            .await
            .unwrap();
    function_output.emit();
}
