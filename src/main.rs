pub mod sdk;
pub use sdk::*;

use tokio;

#[tokio::main(worker_threads = 12)]
async fn main() {
    let enclave_signer = generate_signer();
    let instructions = vec![];
    let function_output =
        FunctionResult::generate_verifiable_solana_tx(enclave_signer, instructions)
            .await
            .unwrap();
    function_output.emit();
}
