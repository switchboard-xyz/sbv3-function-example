use crate::sdk::*;
use anchor_client::solana_sdk::signer::keypair::Keypair;
use solana_sdk::pubkey::Pubkey;
use std::result::Result;
use std::sync::Arc;

pub async fn load<T: bytemuck::Pod>(
    client: &anchor_client::Client<Arc<Keypair>>,
    key: Pubkey,
) -> Result<T, Err> {
    let data = client
        .program(ATTESTATION_PID)
        .async_rpc()
        .get_account_data(&key)
        .await
        .map_err(|_| Err::AnchorParseError)?;
    Ok(*bytemuck::try_from_bytes::<T>(&data[8..]).map_err(|_| Err::AnchorParseError)?)
}
