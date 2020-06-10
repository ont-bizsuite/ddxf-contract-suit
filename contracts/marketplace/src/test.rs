use super::*;

use ostd::contract::ong;
use ostd::mock::build_runtime;
const ONG_CONTRACT_ADDRESS: Address = ostd::macros::base58!("AFmseVrdL9f9oyCzZefL9tG6UbvhfRZMHJ");
use ostd::mock::contract_mock::Command;
use std::collections::btree_map::BTreeMap;

#[test]
fn test() {
    let build = build_runtime();
    build.witness(&[ADMIN]);
    let mp = Address::repeat_byte(1);
    assert!(set_mp(&mp));

    assert_eq!(get_mp_account(), mp);

    let seller = Address::repeat_byte(2);
    let fee_split = FeeSplitModel { percentage: 10 };
    build.witness(&[seller.clone(), mp.clone()]);
    assert!(set_fee_split_model(&seller, fee_split.clone()));
    let fee_split2 = get_fee_split_model(&seller);
    assert_eq!(fee_split.percentage, fee_split2.percentage);

    let buyer = Address::repeat_byte(3);
    let fee = Fee {
        contract_addr: buyer.clone(),
        contract_type: TokenType::ONG,
        count: 1,
    };

    let mut ong_balance_map: BTreeMap<Address, U128> = BTreeMap::new();
    ong_balance_map.insert(buyer.clone(), 10000);

    let call_contract = move |_addr: &Address, _data: &[u8]| -> Option<Vec<u8>> {
        if _addr == &ONG_CONTRACT_ADDRESS {
            mock_ong_contract(_data, &mut ong_balance_map)
        } else {
            mock_ong_contract(_data, &mut ong_balance_map)
        }
    };
    build.on_contract_call(call_contract);

    build.witness(&[buyer.clone()]);

    let self_addr = Address::repeat_byte(4);
    build.address(&self_addr);
    oi = OrderId{
        item_id:vec![0u8,1u8],
        tx_hash:H256::new([0u8;32]),
    }

    assert!(transfer_amount(&buyer, &seller, fee, 1));

    let seller_balance = get_settle_info(&seller, &TokenType::ONG);
    assert_eq!(seller_balance.balance, 1);

    build.witness(&[seller.clone()]);
    assert!(settle(&seller));

    let seller_balance = balance_of(&seller, &TokenType::ONG);
    assert_eq!(seller_balance.balance, 0);
}

fn mock_ong_contract(
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
