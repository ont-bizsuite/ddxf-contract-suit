use super::*;
use hexutil::to_hex;
use ostd::abi::{Decoder, Encoder};
use ostd::mock::build_runtime;
use ostd::mock::contract_mock::Command;

#[test]
fn publish() {
    let resource_id = b"resource_id";
    let mut bmap: BTreeMap<TokenTemplate, RT> = BTreeMap::new();
    let temp = vec![0u8; 36];
    let token_template = TokenTemplate::new(temp);
    bmap.insert(token_template.clone(), RT::RTStaticFile);

    let manager = Address::repeat_byte(1);
    let dtoken_contract_address = Address::repeat_byte(2);
    let mp_contract_address = Address::repeat_byte(3);

    let ddo = ResourceDDO {
        resource_type: RT::RTStaticFile,
        token_resource_type: bmap,
        manager: manager.clone(),
        endpoint: "endpoint".to_string(),
        token_endpoint: BTreeMap::new(),
        desc_hash: None,
        dtoken_contract_address: dtoken_contract_address.clone(),
        mp_contract_address: None,
        split_policy_contract_address: None,
    };
    let contract_addr = Address::repeat_byte(4);
    let fee = Fee {
        contract_addr,
        contract_type: TokenType::ONG,
        count: 0,
    };
    let mut templates = vec![];
    templates.push(token_template.clone());
    let dtoken_item = DTokenItem {
        fee,
        expired_date: 1,
        stocks: 1,
        templates,
    };

    let handle = build_runtime();
    handle.witness(&[manager.clone()]);
    assert!(dtoken_seller_publish(resource_id, &ddo, &dtoken_item));

    let buyer = Address::repeat_byte(4);

    let mut ong_balance_map = BTreeMap::<Address, U128>::new();
    ong_balance_map.insert(buyer.clone(), 10000);
    ong_balance_map.insert(manager.clone(), 10000);
    let buyer2 = Address::repeat_byte(5);
    ong_balance_map.insert(buyer2.clone(), 10000);

    let call_contract = move |_addr: &Address, _data: &[u8]| -> Option<Vec<u8>> {
        if _addr == &dtoken_contract_address {
            mock_dtoken_contract(_data, &mut ong_balance_map)
        } else {
            mock_mp_contract(_data, &mut ong_balance_map)
        }
    };
    handle.on_contract_call(call_contract);

    handle.witness(&[buyer.clone()]);
    assert!(buy_dtoken(resource_id, 1, &buyer));

    handle.witness(&[buyer.clone(), buyer2.clone()]);
    assert!(buy_dtoken_from_reseller(resource_id, 1, &buyer2, &buyer));
    assert!(use_token(resource_id, &buyer2, token_template.clone(), 1));
}

fn mock_mp_contract(
    _data: &[u8],
    ong_balance_map: &mut BTreeMap<Address, U128>,
) -> Option<Vec<u8>> {
    let mut sink = Sink::new(12);
    let mut source = Source::new(_data);
    let command = Command::decode(&mut source).ok().unwrap();
    match command {
        Command::Transfer { from, to, value } => {
            let mut from_ba = ong_balance_map.get(from).map(|val| val.clone()).unwrap();
            let mut to_ba = ong_balance_map
                .get(to)
                .map(|val| val.clone())
                .unwrap_or_default();
            from_ba -= value;
            to_ba += value;
            ong_balance_map.insert(from.clone(), from_ba);
            ong_balance_map.insert(to.clone(), to_ba);
            sink.write(true);
        }
        Command::BalanceOf { addr } => {
            let ba = ong_balance_map.get(addr).map(|val| val.clone()).unwrap();
            sink.write(ba);
        }
        _ => {}
    }
    return Some(sink.bytes().to_vec());
}

#[derive(Encoder, Decoder)]
enum DtokenCommand {
    GenerateDToken(GenerateDToken),
    TransferDToken(TransferDToken),
    UseToken(UseToken),
}

#[derive(Encoder, Decoder)]
struct UseToken {
    account: Address,
    resource_id: Vec<u8>,
    token_template: TokenTemplate,
    n: U128,
}

#[derive(Encoder, Decoder)]
struct GenerateDToken {
    account: Address,
    resource_id: Vec<u8>,
    templates: Vec<u8>,
    n: U128,
}

#[derive(Encoder, Decoder)]
struct TransferDToken {
    from_account: Address,
    to_account: Address,
    resource_id: Vec<u8>,
    templates: Vec<u8>,
    n: U128,
}

fn mock_dtoken_contract(
    _data: &[u8],
    ong_balance_map: &mut BTreeMap<Address, U128>,
) -> Option<Vec<u8>> {
    let mut sink = Sink::new(12);
    let mut source = Source::new(_data);
    println!("{}", to_hex(_data));
    let command = DtokenCommand::decode(&mut source).ok().unwrap();
    match command {
        DtokenCommand::GenerateDToken(generate) => {
            sink.write(true);
        }
        DtokenCommand::TransferDToken(transfer) => {
            sink.write(true);
        }
        DtokenCommand::UseToken(useToken) => {
            sink.write(true);
        }
    }
    return Some(sink.bytes().to_vec());
}
