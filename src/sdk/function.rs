use crate::sdk::*;
use anchor_client::anchor_lang::prelude::*;
use anchor_client::anchor_lang::{
    AnchorDeserialize, AnchorSerialize,
};
use bytemuck::{Pod, Zeroable};
use chrono::DateTime;
use chrono::NaiveDateTime;
use chrono::Utc;
use cron::Schedule;
use solana_sdk::{
    pubkey::Pubkey,
};
use std::str::FromStr;
use std::{env};

#[repr(packed)]
#[derive(Copy, Clone, Debug)]
pub struct FunctionAccountData {
    pub name: [u8; 64],
    pub metadata: [u8; 256],
    pub authority: Pubkey,
    ///
    pub container_registry: [u8; 64],
    pub container: [u8; 64],
    pub version: [u8; 32],
    ///
    pub attestation_queue: Pubkey,
    pub queue_idx: u32,
    pub last_execution_timestamp: i64,
    pub next_allowed_timestamp: i64,
    pub schedule: [u8; 64],
    pub escrow: Pubkey,
    pub status: FunctionStatus,
    pub created_at: i64,
    pub _ebuf: [u8; 1024],
}
unsafe impl Pod for FunctionAccountData {}
unsafe impl Zeroable for FunctionAccountData {}
#[repr(u8)]
#[derive(Debug, Copy, Clone, Eq, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum FunctionStatus {
    None = 0,
    Active = 1 << 0,
    NonExecutable = 1 << 1,
    Expired = 1 << 2,
    OutOfFunds = 1 << 3,
    InvalidPermissions = 1 << 4,
}
impl FunctionAccountData {
    pub fn get_schedule(&self) -> Option<Schedule> {
        if self.schedule[0] == 0 {
            return None;
        }
        let every_second = Schedule::try_from("* * * * * *").unwrap();
        let schedule = std::str::from_utf8(&self.schedule)
            .unwrap_or("* * * * * *")
            .trim_end_matches('\0');
        let schedule = Schedule::try_from(schedule);
        Some(schedule.unwrap_or(every_second.clone()))
    }

    pub fn get_last_execution_datetime(&self) -> DateTime<Utc> {
        DateTime::from_utc(
            NaiveDateTime::from_timestamp_opt(self.last_execution_timestamp, 0).unwrap(),
            Utc,
        )
    }

    pub fn next_execution_timestamp(&self) -> Option<DateTime<Utc>> {
        let schedule = self.get_schedule();
        if schedule.is_none() {
            return None;
        }
        let dt = self.get_last_execution_datetime();
        schedule.unwrap().after(&dt).next()
    }
}

pub fn fn_accounts() -> (Pubkey, Pubkey) {
    let fn_key = Pubkey::from_str(&env::var("FUNCTION_KEY").unwrap()).unwrap();
    let (fn_quote, _) =
        Pubkey::find_program_address(&[b"QuoteAccountData", &fn_key.to_bytes()], &ATTESTATION_PID);
    (fn_key, fn_quote)
}
