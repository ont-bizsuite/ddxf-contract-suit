use super::ostd::abi::{Decoder, Encoder};
use super::ostd::types::Address;
use common::TokenTemplate;

#[derive(Encoder, Decoder)]
pub struct TokenTemplateInfo {
    // note: `creator` must be the first field, since we only decode it in `update_token_template`
    pub creator: Address,
    pub token_template: TokenTemplate,
}
