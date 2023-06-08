use anchor_client::anchor_lang::prelude::*;
use anchor_client::anchor_lang::{
    AnchorDeserialize, AnchorSerialize,
};
use rust_decimal::prelude::*;
use std::str::FromStr;

#[derive(Default, Eq, PartialEq, Copy, Clone, AnchorSerialize, AnchorDeserialize, Debug)]
pub struct BorshDecimal {
    pub mantissa: i128,
    pub scale: u32,
}
impl From<Decimal> for BorshDecimal {
    fn from(val: Decimal) -> Self {
        BorshDecimal {
            mantissa: val.mantissa(),
            scale: val.scale(),
        }
    }
}
impl From<&String> for BorshDecimal {
    fn from(val: &String) -> Self {
        let val = Decimal::from_str(val.as_ref()).unwrap();
        BorshDecimal::from(val)
    }
}
