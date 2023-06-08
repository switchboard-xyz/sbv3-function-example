use crate::sdk::*;
use anchor_client::solana_sdk::signer::keypair::Keypair;
use getrandom::getrandom;
use sha2::{Digest, Sha256};
use solana_sdk::{
    signer::keypair::keypair_from_seed,
};
use std::sync::Arc;
use std::{fs};

pub struct Sgx;
impl Sgx {
    pub fn gramine_generate_quote(user_data: &[u8]) -> std::result::Result<Vec<u8>, Err> {
        fs::metadata("/dev/attestation/quote").map_err(|_| Err::SgxError)?;
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

pub fn generate_signer() -> Arc<Keypair> {
    let mut randomness = [0; 32];
    Sgx::read_rand(&mut randomness).unwrap();
    Arc::new(keypair_from_seed(&randomness).unwrap())
}
