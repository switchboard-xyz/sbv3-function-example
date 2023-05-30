pub mod sdk;
pub use sdk::*;
use tokio;
use solana_sdk::system_instruction;
use solana_sdk::signature::Signer;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use anchor_lang::solana_program::instruction::Instruction;
use solana_sdk::instruction::AccountMeta;
use std::env;
use solana_sdk::pubkey;
use anchor_lang::AnchorSerialize;
use anchor_lang::AnchorDeserialize;
use anchor_lang::Discriminator;
use anchor_lang::InstructionData;
use anchor_lang::prelude::*;

const DEMO_PID: Pubkey = pubkey!("8kjszBCEgkzAsU6QySHSZvr9yFaboau2RnarCQFFvasS");

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct PingParams {
    pub value: BorshDecimal,
}
impl Discriminator for PingParams {
    const DISCRIMINATOR: [u8; 8] = [0; 8];
    fn discriminator() -> [u8; 8] {
        ix_discriminator("ping")
    }
}
impl InstructionData for PingParams {}
#[derive(Default, Eq, PartialEq, Copy, Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct BorshDecimal {
    pub mantissa: i128,
    pub scale: u32,
}

#[tokio::main(worker_threads = 12)]
async fn main() {
    let enclave_signer = generate_signer();
    let (fn_key, fn_quote) = fn_accounts();
    let ix = Instruction {
        program_id: DEMO_PID,
        accounts: vec![
            AccountMeta {
                pubkey: fn_key,
                is_signer: false,
                is_writable: false
            },
            AccountMeta {
                pubkey: fn_quote,
                is_signer: false,
                is_writable: false
            },
            AccountMeta {
                pubkey: enclave_signer.pubkey(),
                is_signer: true,
                is_writable: false
            }],
        data: PingParams::default().data(),
    };
    let function_output =
        FunctionResult::generate_verifiable_solana_tx(enclave_signer, vec![ix])
            .await
            .unwrap();
    function_output.emit();
}
