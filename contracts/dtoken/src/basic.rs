use super::ostd::abi::{Decoder, Encoder};
use super::ostd::types::Address;
use alloc::vec::Vec;

#[derive(Encoder, Decoder)]
pub struct TokenTemplateInfo {
    pub creator: Address,
    pub tt: Vec<u8>,
}

impl TokenTemplateInfo {
    pub fn new(creator: Address, tt: Vec<u8>) -> Self {
        TokenTemplateInfo { creator, tt }
    }
}
