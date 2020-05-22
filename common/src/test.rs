use super::*;

#[test]
fn token_template_test() {
    let token_hash = vec![1u8, 32];
    let template = TokenTemplate::new(None, token_hash.clone());

    let mut sink = Sink::new(16);
    sink.write(&template);

    let mut source = Source::new(sink.bytes());
    let template2: TokenTemplate = source.read().unwrap();
    assert_eq!(template.token_hash, template2.token_hash);

    let data_ids = vec![1u8, 2u8];
    let template = TokenTemplate::new(Some(data_ids), token_hash);
    let mut sink = Sink::new(16);
    sink.write(&template);

    let mut source = Source::new(sink.bytes());
    let template2: TokenTemplate = source.read().unwrap();
    assert_eq!(template.token_hash, template2.token_hash);
}

#[test]
fn test_fee() {
    let contract_addr = Address::repeat_byte(1);
    let fee = Fee {
        contract_addr,
        contract_type: TokenType::ONG,
        count: 10,
    };

    let mut sink = Sink::new(16);
    sink.write(&fee);

    let mut source = Source::new(sink.bytes());
    let fee2: Fee = source.read().unwrap();
    assert_eq!(fee.count, fee2.count);
}
