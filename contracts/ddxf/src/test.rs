use super::*;
use hexutil::to_hex;
use hexutil::{read_hex, to_hex};
use ostd::abi::{Decoder, Encoder};
use ostd::mock::build_runtime;
use ostd::mock::contract_mock::Command;
use ostd::prelude::String;

#[test]
fn test() {
    let data = read_hex("0e746573745265736f757263654964f20001000000012a6469643a6f6e743a41626b35725255794a53636e6d5045645264567934693769666955377967433853682096cae35ce8a9b0244178bf28e4966c2ce1b8385723a96a6b838858cdd6ca0a1e00675478ea7368fd9579c00a8a749d29c2b82f2aef10687474703a2f2f64656d6f2e7465737401000000012a6469643a6f6e743a41626b35725255794a53636e6d5045645264567934693769666955377967433853682096cae35ce8a9b0244178bf28e4966c2ce1b8385723a96a6b838858cdd6ca0a1e10687474703a2f2f64656d6f2e7465737400012fee6d8699c9b8f992a6bd54753cf84cb3aae87400002d000000000000000000000000000000000000000201c80000000000000000943577000000006400000000000000").unwrap_or_default();
    let mut source = Source::new(&data);
    let (resource_id, ddo_bytes, item_bytes): (Vec<u8>, &[u8], &[u8]) = source.read().unwrap();
    println!("resource_id:{:?}", String::from_utf8(resource_id));
    let ddo = ResourceDDO::from_bytes(ddo_bytes);
    println!("ddo:{:?}", ddo.manager);
    let item = DTokenItem::from_bytes(item_bytes);
    println!("item:{:?}", item.expired_date);
}

#[test]
fn test2() {
    let data = read_hex("0001000000012a6469643a6f6e743a41626b35725255794a53636e6d5045645264567934693769666955377967433853682096cae35ce8a9b0244178bf28e4966c2ce1b8385723a96a6b838858cdd6ca0a1e00675478ea7368fd9579c00a8a749d29c2b82f2aef10687474703a2f2f64656d6f2e7465737401000000012a6469643a6f6e743a41626b35725255794a53636e6d5045645264567934693769666955377967433853682096cae35ce8a9b0244178bf28e4966c2ce1b8385723a96a6b838858cdd6ca0a1e10687474703a2f2f64656d6f2e7465737400012fee6d8699c9b8f992a6bd54753cf84cb3aae8740000").unwrap_or_default();
    let ddo = ResourceDDO::from_bytes(&data);
    println!("{}", ddo.manager);
}

#[test]
fn serialize() {
    let mut bmap: BTreeMap<TokenTemplate, RT> = BTreeMap::new();
    let token_hash = read_hex("96cae35ce8a9b0244178bf28e4966c2ce1b8385723a96a6b838858cdd6ca0a1e")
        .unwrap_or_default();
    let token_template = TokenTemplate::new(
        Some(b"did:ont:Abk5rRUyJScnmPEdRdVy4i7ifiU7ygC8Sh".to_vec()),
        token_hash,
    );
    bmap.insert(token_template.clone(), RT::RTStaticFile);

    let mut token_endpoint = BTreeMap::new();
    token_endpoint.insert(token_template.clone(), "http://demo.test".to_string());
    let manager = ostd::macros::base58!("ARCESVnP8Lbf6S7FuTei3smA35EQYog4LR");

    let mp_contract_address = Address::repeat_byte(3);

    let dtoken_contract_hex =
        read_hex("2fee6d8699c9b8f992a6bd54753cf84cb3aae874").unwrap_or_default();
    let mut temp: [u8; 20] = [0; 20];
    for i in 0..20 {
        temp[i] = dtoken_contract_hex[i]
    }
    let dtoken_contract = Address::new(temp);
    let ddo = ResourceDDO {
        resource_type: RT::RTStaticFile,
        token_resource_type: bmap,
        manager: manager.clone(),
        endpoint: "endpoint".to_string(),
        token_endpoint,
        desc_hash: None,
        dtoken_contract_address: Some(dtoken_contract.clone()),
        mp_contract_address: None,
        split_policy_contract_address: None,
    };

    let mut sink = Sink::new(16);
    sink.write(ddo);
    println!("{}", to_hex(sink.bytes()));
    panic!("");
}

#[test]
fn publish() {
    let resource_id = b"resource_id";
    let mut bmap: BTreeMap<TokenTemplate, RT> = BTreeMap::new();
    let temp = vec![0u8; 36];
    let token_template = TokenTemplate::new(None, temp);
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
        dtoken_contract_address: Some(dtoken_contract_address.clone()),
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
    assert!(dtoken_seller_publish(
        resource_id,
        &ddo.to_bytes(),
        &dtoken_item.to_bytes()
    ));

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
