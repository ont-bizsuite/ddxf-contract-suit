use super::{Address, Decoder, Encoder, Error, Sink, Source, U128};
use common::Fee;

#[derive(Encoder, Decoder, Clone)]
pub struct FeeSplitModel {
    pub percentage: u16,
}

#[derive(Encoder, Decoder, Default)]
pub struct SettleInfo {
    pub split_contract_addr: Address,
    pub fee: Fee,
    pub n: U128,
}
