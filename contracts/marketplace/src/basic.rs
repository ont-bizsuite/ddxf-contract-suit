use super::{Address, Decoder, Encoder, Error, Sink, Source, U128};

#[derive(Encoder, Decoder, Clone)]
pub struct FeeSplitModel {
    pub percentage: u16,
}

pub struct TokenBalance {
    pub token_type: TokenType,
    pub contract_address: Option<Address>,
    pub balance: U128,
}

impl Encoder for TokenBalance {
    fn encode(&self, sink: &mut Sink) {
        sink.write(&self.token_type);
        if let Some(addr) = &self.contract_address {
            sink.write(true);
            sink.write(addr);
        } else {
            sink.write(false);
        }
        sink.write(self.balance);
    }
}

impl<'a> Decoder<'a> for TokenBalance {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let token_type = source.read()?;
        let is_val: bool = source.read()?;
        let contract_address = match is_val {
            true => {
                let temp: Address = source.read()?;
                Some(temp)
            }
            false => None,
        };
        let balance: U128 = source.read()?;
        Ok(TokenBalance {
            token_type,
            contract_address,
            balance,
        })
    }
}

impl TokenBalance {
    pub fn new(token_type: TokenType) -> Self {
        TokenBalance {
            token_type,
            contract_address: None,
            balance: 0,
        }
    }
}
