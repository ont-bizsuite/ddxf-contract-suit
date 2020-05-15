use super::*;

#[test]
fn token_template_test() {
    let token_hash = vec![1u8, 32];
    let template = TokenTemplate::new(None, token_hash);

    let mut sink = Sink::new(16);
    sink.write(&template);

    let mut source = Source::new(sink.bytes());
    let template2: TokenTemplate = source.read().unwrap();
    assert_eq!(template.token_hash, template2.token_hash);
}
