use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::AnchorDeserialize;
use anchor_lang::AnchorSerialize;
use anchor_lang::Discriminator;
use anchor_lang::InstructionData;
use reqwest;
use sb_functions_sdk::*;
use serde::Deserialize;
use solana_sdk::instruction::AccountMeta;
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signer;
use tokio;

const DEMO_PID: Pubkey = pubkey!("8kjszBCEgkzAsU6QySHSZvr9yFaboau2RnarCQFFvasS");

#[derive(Clone, AnchorSerialize, AnchorDeserialize, Debug, Default)]
pub struct PingParams {
    pub prices: Vec<BorshDecimal>,
    pub volumes: Vec<BorshDecimal>,
    pub twaps: Vec<BorshDecimal>,
}
impl Discriminator for PingParams {
    const DISCRIMINATOR: [u8; 8] = [0; 8];
    fn discriminator() -> [u8; 8] {
        ix_discriminator("ping")
    }
}
impl InstructionData for PingParams {}

#[allow(non_snake_case)]
#[derive(Deserialize, Clone, Debug)]
struct Ticker {
    symbol: String,
    weightedAvgPrice: String,
    lastPrice: String,
    volume: String,
}

#[tokio::main(worker_threads = 12)]
async fn main() {
    let symbols = ["BTCUSDC", "ETHUSDC", "SOLUSDT"];

    let symbols = symbols.map(|x| format!("\"{}\"", x)).join(",");
    let tickers = reqwest::get(format!(
        "https://api.binance.com/api/v3/ticker?symbols=[{}]&windowSize=1h",
        symbols
    ))
    .await
    .unwrap()
    .json::<Vec<Ticker>>()
    .await
    .unwrap();
    println!("{:#?}", tickers);

    let enclave_signer = generate_signer();
    let (fn_key, fn_quote) = fn_accounts();
    let ix = Instruction {
        program_id: DEMO_PID,
        accounts: vec![
            AccountMeta {
                pubkey: fn_key,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: fn_quote,
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: enclave_signer.pubkey(),
                is_signer: true,
                is_writable: false,
            },
        ],
        data: PingParams {
            prices: tickers
                .iter()
                .map(|x| BorshDecimal::from(&x.lastPrice))
                .collect(),
            volumes: tickers
                .iter()
                .map(|x| BorshDecimal::from(&x.volume))
                .collect(),
            twaps: tickers
                .iter()
                .map(|x| BorshDecimal::from(&x.weightedAvgPrice))
                .collect(),
        }
        .data(),
    };
    FunctionResult::generate_verifiable_solana_tx(enclave_signer, vec![ix])
        .await
        .unwrap()
        .emit();
}
