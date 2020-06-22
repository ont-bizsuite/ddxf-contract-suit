use super::{Address, Decoder, Encoder, Error, Sink, Source, U128};
use common::Fee;

#[derive(Encoder, Decoder, Clone)]
pub struct FeeSplitModel {
    pub weight: u16,
}

#[derive(Encoder, Decoder)]
pub struct SettleInfo {
    pub split_contract_addr: Address,
    pub fee: Fee,
    pub n: U128,
}

impl SettleInfo {
    pub fn default() -> Self {
        SettleInfo {
            split_contract_addr: Address::new([0u8; 20]),
            fee: Fee::default(),
            n: 0,
        }
    }
}
