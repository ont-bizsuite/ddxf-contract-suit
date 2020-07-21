use super::*;
use alloc::collections::btree_map::BTreeMap;
use hexutil::{read_hex, to_hex};
use ostd::abi::{Decoder, Encoder};
use ostd::mock::build_runtime;
use ostd::mock::contract_mock::Command;
use ostd::prelude::String;
use ostd::types::u128_from_neo_bytes;

#[test]
fn test67() {
    #[derive(Encoder, Decoder, Debug)]
    struct Fee {
        pub contract_addr: Address,
        pub count: U128,
    }
    #[derive(Debug)]
    struct DTokenItem {
        pub fee: Fee,
        pub expired_date: U128,
        pub stocks: U128,
        pub sold: U128,
        pub token_template_ids: Vec<Vec<u8>>,
    }
    impl<'a> ontio_std::abi::Decoder<'a> for DTokenItem {
        fn decode(source: &mut ontio_std::abi::Source) -> Result<Self, ontio_std::abi::Error> {
            let fee = source.read()?;
            let expired_date_bs = source.read()?;
            let expired_date = u128_from_neo_bytes(expired_date_bs);
            return Ok(DTokenItem {
                fee,
                expired_date,
                stocks: source.read()?,
                sold: source.read()?,
                token_template_ids: source.read()?,
            });
        }
    }

    let bs = read_hex("0a6d6574686f644e616d650200000000000000000000000000000000000000001027000000000000000000000000000010270000000000000000000000000000102700000000000000000000000000000000000000000000000000000000000002036161610362626210270000000000000000000000000000").unwrap();
    let mut source = Source::new(bs.as_slice());
    let method: &[u8] = source.read().unwrap();
    let (item, aaa): (DTokenItem, U128) = source.read().unwrap();

    println!("{}", item.expired_date);
    println!("{}", item.stocks);
    println!("{:?}", item);
    println!("{}", aaa);
}

#[test]
fn test_token_template() {
    let tt = TokenTemplate {
        data_id: None,
        token_hash: vec![vec![0u8; 32]],
        endpoint: vec![],
        token_name: vec![],
        token_symbol: vec![],
    };
    let mut sink = Sink::new(16);
    sink.write(tt.clone());
    let mut source = Source::new(sink.bytes());
    let tt2: TokenTemplate = source.read().unwrap();

    let bs = read_hex("012c646174615f69645f63316235663139352d623431342d343535632d393464332d6466303565366563373635300120e2a740fa12bd94f0e242688e29f6d803f7671eb1f81bcfbdc1c3e213878e7dd4").unwrap_or_default();
    let tt = TokenTemplate::from_bytes(bs.as_slice());
}

#[test]
fn test() {
    let data = read_hex("00010000000000017a0842016023031e8c24c7ea90cac9aa52f3b7da000100000000000001207d479be9ae1b65d3f0e98327c2eafc5f2e0e0693e15d175198735e0a8eec8f91000000").unwrap_or_default();
    let mut source = Source::new(&data);

    let ddo = ResourceDDO::from_bytes(data.as_slice());

    let method: &[u8] = source.read().unwrap();
    let (resource_id, ddo_bytes, item_bytes): (Vec<u8>, &[u8], &[u8]) = source.read().unwrap();
    println!("resource_id:{:?}", String::from_utf8(resource_id));
    let ddo = ResourceDDO::from_bytes(ddo_bytes);
    println!("ddo:{:?}", ddo.manager);
    let item = DTokenItem::from_bytes(item_bytes);
    println!("item:{:?}", item.expired_date);
}

#[test]
fn dtoken_test() {
    let addr = Address::repeat_byte(1);
    let item = DTokenItem {
        fee: Fee {
            contract_addr: addr,
            contract_type: TokenType::ONG,
            count: 1000000,
        },
        expired_date: 10000,
        stocks: 10000,
        sold: 1000,
        token_template_ids: vec![],
    };

    let mut sink = Sink::new(16);
    sink.write(&item);

    let mut source = Source::new(sink.bytes());
    let item2: DTokenItem = source.read().unwrap();
    assert_eq!(item.sold, item2.sold);
}

#[test]
fn test2() {
    let data = read_hex("0001000000012a6469643a6f6e743a41626b35725255794a53636e6d5045645264567934693769666955377967433853682096cae35ce8a9b0244178bf28e4966c2ce1b8385723a96a6b838858cdd6ca0a1e00675478ea7368fd9579c00a8a749d29c2b82f2aef10687474703a2f2f64656d6f2e7465737401000000012a6469643a6f6e743a41626b35725255794a53636e6d5045645264567934693769666955377967433853682096cae35ce8a9b0244178bf28e4966c2ce1b8385723a96a6b838858cdd6ca0a1e10687474703a2f2f64656d6f2e7465737400012fee6d8699c9b8f992a6bd54753cf84cb3aae8740000").unwrap_or_default();
    let ddo = ResourceDDO::from_bytes(&data);
    println!("{}", ddo.manager);
}

#[test]
fn test3() {
    let data = read_hex("077075626c697368133538343238373838383234323030353432343837fbe02b027e61a6d7602f26cfa9487fa58ef9ee72e43abcf3375244839c012f9633f95862d232a95b00d5bc7348b3098b9fed7f32000000380000000000000000000000000000000000000000000100000000000000fea9165f00000000102700000000000000000000000000000101303502fbe02b027e61a6d7602f26cfa9487fa58ef9ee728813000000baa22722bfaba589eed9ef9760581de679fb251b88130000000100").unwrap_or_default();
    let mut source = Source::new(&data);
    let mthod_name: &str = source.read().unwrap();
    println!("method_name:{}", mthod_name);
    let (resource_id, ddo, item, sp): (&[u8], &[u8], &[u8], &[u8]) = source.read().unwrap();
    //        println!("resource_id:{}", to_hex(resource_id));
    //        let ddo = ResourceDDO::from_bytes(ddo);
    //        println!("manager:{}", ddo.manager);
    //        let item = DTokenItem::from_bytes(item);
    //        println!("item:{}", item.stocks);

    let build = build_runtime();
    let addr = ostd::macros::base58!("Aejfo7ZX5PVpenRj23yChnyH64nf8T1zbu");
    build.witness(&[addr]);
    assert!(dtoken_seller_publish(resource_id, ddo, item, sp));
}

#[test]
fn test5() {
    let data = read_hex("0962757944746f6b656e03067265736f5f3501000000000000000000000000000000151dd3ecff3994999739bee170e6f490437248a7").unwrap_or_default();
    let mut source = Source::new(&data);

    let method_name: &str = source.read().unwrap();
    println!("method_name:{}", method_name);
    let (resource_id, n, buyer): (&[u8], U128, &Address) = source.read().unwrap();

    let build = build_runtime();
    let addr = ostd::macros::base58!("AHhXa11suUgVLX1ZDFErqBd3gskKqLfa5N");
    build.witness(&[buyer.clone()]);
    assert!(buy_dtoken(resource_id, n, buyer, buyer));
}

#[test]
fn serialize() {
    let token_hash = read_hex("96cae35ce8a9b0244178bf28e4966c2ce1b8385723a96a6b838858cdd6ca0a1e")
        .unwrap_or_default();
    let token_template = TokenTemplate::new(
        vec![],
        vec![],
        Some(b"did:ont:Abk5rRUyJScnmPEdRdVy4i7ifiU7ygC8Sh".to_vec()),
        vec![token_hash],
        vec![],
    );

    let manager = ostd::macros::base58!("ARCESVnP8Lbf6S7FuTei3smA35EQYog4LR");

    let mp_contract_address = Address::repeat_byte(3);

    let dtoken_contract_hex =
        read_hex("2fee6d8699c9b8f992a6bd54753cf84cb3aae874").unwrap_or_default();
    let mut temp: [u8; 20] = [0; 20];
    for i in 0..20 {
        temp[i] = dtoken_contract_hex[i]
    }
    let dtoken_contract = Address::new(temp);
    let h = H256::repeat_byte(1);
    let ddo = ResourceDDO {
        manager: manager.clone(),
        item_meta_hash: h,
        dtoken_contract_address: vec![dtoken_contract.clone()],
        accountant_contract_address: None,
        split_policy_contract_address: None,
    };

    let mut sink = Sink::new(16);
    sink.write(ddo);
    println!("{}", to_hex(sink.bytes()));
}

#[test]
fn publish() {
    let resource_id = b"resource_id";
    let temp = vec![0u8; 36];
    let token_template = TokenTemplate::new(
        b"name".to_vec(),
        b"symbol".to_vec(),
        None,
        vec![temp],
        vec![],
    );
    let manager = Address::repeat_byte(1);
    let dtoken_contract_address = Address::repeat_byte(2);
    let mp_contract_address = Address::repeat_byte(3);

    let ddo = ResourceDDO {
        item_meta_hash: H256::repeat_byte(1),
        manager: manager.clone(),
        dtoken_contract_address: vec![dtoken_contract_address.clone()],
        accountant_contract_address: None,
        split_policy_contract_address: None,
    };

    let mut sink_temp = Sink::new(64);
    sink_temp.write(&ddo);
    let mut source = Source::new(sink_temp.bytes());
    let ddo_temp: ResourceDDO = source.read().unwrap();
    assert_eq!(&ddo.manager, &ddo_temp.manager);

    let contract_addr = Address::repeat_byte(4);
    let fee = Fee {
        contract_addr,
        contract_type: TokenType::ONG,
        count: 0,
    };
    let mut templates = vec![b"template_id".to_vec()];
    let dtoken_item = DTokenItem {
        fee,
        expired_date: 1,
        stocks: 1000,
        sold: 1,
        token_template_ids: templates,
    };

    let handle = build_runtime();

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

    handle.witness(&[manager.clone(), CONTRACT_COMMON.admin().clone()]);
    assert!(set_dtoken_contract(&dtoken_contract_address));

    let split_param = b"test";
    assert!(dtoken_seller_publish(
        resource_id,
        &ddo.to_bytes(),
        &dtoken_item.to_bytes(),
        split_param
    ));

    handle.witness(&[buyer.clone()]);
    //    assert!(buy_dtoken(resource_id, 1, &buyer));
    assert!(buy_dtokens(vec![resource_id], vec![1], &buyer, &buyer));

    handle.witness(&[buyer.clone(), buyer2.clone()]);
    assert!(buy_dtoken_from_reseller(resource_id, 1, &buyer2, &buyer));
    let token_template_bytes = token_template.to_bytes();
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
    token_template: Vec<u8>,
    n: U128,
}

#[derive(Encoder, Decoder)]
struct GenerateDToken {
    account: Address,
    templates: Vec<u8>,
    n: U128,
}

#[derive(Encoder, Decoder)]
struct TransferDToken {
    from_account: Address,
    to_account: Address,
    templates: Vec<u8>,
    n: U128,
}

fn mock_dtoken_contract(
    _data: &[u8],
    ong_balance_map: &mut BTreeMap<Address, U128>,
) -> Option<Vec<u8>> {
    let mut sink = Sink::new(12);
    let mut source = Source::new(_data);
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
