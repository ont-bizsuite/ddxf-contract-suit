use super::check_witness;
use super::ostd::abi::EventBuilder;
use super::ostd::abi::{Decoder, Encoder, Error, Sink, Source};
use super::ostd::database;
use super::ostd::prelude::*;
use super::ostd::types::{Address, U128};

pub struct TrMulParam<'a> {
    pub from: &'a Address,
    pub to: &'a Address,
    pub id: &'a [u8],
    pub amt: U128,
}

impl<'a> Encoder for TrMulParam<'a> {
    fn encode(&self, sink: &mut Sink) {
        sink.write(self.from);
        sink.write(self.to);
        sink.write(self.id);
        sink.write(self.amt);
    }
}

impl<'a> Decoder<'a> for TrMulParam<'a> {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let from: &Address = source.read().unwrap();
        let to: &Address = source.read().unwrap();
        let id: &[u8] = source.read().unwrap();
        let amt: U128 = source.read().unwrap();
        Ok(TrMulParam { from, to, id, amt })
    }
}

pub struct TrFromMulParam<'a> {
    pub spender: &'a Address,
    pub from: &'a Address,
    pub to: &'a Address,
    pub id: &'a [u8],
    pub amt: U128,
}

impl<'a> Encoder for TrFromMulParam<'a> {
    fn encode(&self, sink: &mut Sink) {
        sink.write(self.spender);
        sink.write(self.from);
        sink.write(self.to);
        sink.write(self.id);
        sink.write(self.amt);
    }
}

impl<'a> Decoder<'a> for TrFromMulParam<'a> {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let spender = source.read().unwrap();
        let from = source.read().unwrap();
        let to = source.read().unwrap();
        let id = source.read().unwrap();
        let amt = source.read().unwrap();
        Ok(TrFromMulParam {
            spender,
            from,
            to,
            id,
            amt,
        })
    }
}

pub struct AppMulParam<'a> {
    pub owner: &'a Address,
    pub spender: &'a Address,
    pub id: &'a [u8],
    pub amt: U128,
}

impl<'a> Encoder for AppMulParam<'a> {
    fn encode(&self, sink: &mut Sink) {
        sink.write(self.owner);
        sink.write(self.spender);
        sink.write(self.id);
        sink.write(self.amt);
    }
}

impl<'a> Decoder<'a> for AppMulParam<'a> {
    fn decode(source: &mut Source<'a>) -> Result<Self, Error> {
        let owner = source.read().unwrap();
        let spender = source.read().unwrap();
        let id = source.read().unwrap();
        let amt = source.read().unwrap();
        Ok(AppMulParam {
            owner,
            spender,
            id,
            amt,
        })
    }
}

#[derive(Encoder, Decoder)]
pub struct Balance {
    pub id: Vec<u8>,
    pub amt: U128,
}

const PRE_BALANCE: &[u8] = b"91";
const PRE_APPROVE: &[u8] = b"92";
const KEY_TOKEN_COUNTER: &[u8] = b"93";
const PRE_NAME: &[u8] = b"94";
const PRE_SYMBOL: &[u8] = b"95";
const PRE_SUPPLY: &[u8] = b"96";

pub fn name(id: &[u8]) -> Vec<u8> {
    database::get::<_, Vec<u8>>(gen_key(PRE_NAME, id)).unwrap_or(vec![])
}

pub fn symbol(id: &[u8]) -> Vec<u8> {
    database::get::<_, Vec<u8>>(gen_key(PRE_SYMBOL, id)).unwrap_or(vec![])
}

pub fn total_supply(id: &[u8]) -> U128 {
    database::get::<_, U128>(gen_key(PRE_SUPPLY, id)).unwrap_or(0)
}

pub fn balance_of(acct: &Address, id: &[u8]) -> U128 {
    database::get::<_, U128>(gen_balance_key(id, acct.as_ref())).unwrap_or(0)
}

pub fn destroy_token(acct: &Address, id: &[u8], n: U128) {
    let key = gen_balance_key(id, acct.as_ref());
    let ba = database::get::<_, U128>(key.as_slice()).unwrap_or(0);
    assert!(ba >= n);
    if ba == n {
        database::delete(key.as_slice());
    } else {
        database::put(key.as_slice(), ba - n);
    }
}

pub fn balances_of(acct: &Address) -> Vec<Balance> {
    let id = get_next_id();
    let mut res: Vec<Balance> = vec![];
    for i in 0..id {
        let id_str = i.to_string();
        let i_bs = id_str.as_bytes().to_vec();
        let ba = balance_of(acct, i_bs.as_slice());
        if ba > 0 {
            res.push(Balance { id: i_bs, amt: ba });
        }
    }
    res
}

pub fn generate_token(name: &[u8], symbol: &[u8], supply: U128, admin: &Address) -> Vec<u8> {
    let id = get_next_id();
    let token_id = id.to_string();
    database::put(gen_key(PRE_NAME, token_id.as_bytes()), name);
    database::put(gen_key(PRE_SYMBOL, token_id.as_bytes()), symbol);
    database::put(gen_key(PRE_SUPPLY, token_id.as_bytes()), supply);
    database::put(gen_balance_key(token_id.as_bytes(), admin.as_ref()), supply);
    database::put(KEY_TOKEN_COUNTER, id + 1);
    EventBuilder::new()
        .string("generate_token")
        .bytearray(token_id.as_bytes())
        .notify();
    token_id.as_bytes().to_vec()
}

pub fn delete_token(token_id: &[u8]) {
    database::delete(gen_key(PRE_NAME, token_id));
    database::delete(gen_key(PRE_SYMBOL, token_id));
    database::delete(gen_key(PRE_SUPPLY, token_id));
    //TODO
    // database::delete(gen_balance_key(token_id, admin.as_ref()));
}

pub fn transfer(from: &Address, to: &Address, id: &[u8], amt: u128) -> bool {
    assert!(check_witness(from));
    transfer_inner(from, to, id, amt)
}

pub fn transfer_inner(from: &Address, to: &Address, id: &[u8], amt: u128) -> bool {
    let from_ba = balance_of(from, id);
    assert!(from_ba >= amt);
    let from_ba = from_ba.checked_sub(amt).unwrap();
    let to_ba = balance_of(to, id);
    let to_ba = to_ba.checked_add(amt).unwrap();
    if from_ba == 0 {
        database::delete(gen_balance_key(id, from.as_ref()));
    } else {
        database::put(gen_balance_key(id, from.as_ref()), from_ba);
    }
    database::put(gen_balance_key(id, to.as_ref()), to_ba);
    EventBuilder::new()
        .bytearray(b"transfer")
        .bytearray(from.as_ref())
        .bytearray(to.as_ref())
        .bytearray(id)
        .bytearray(amt.to_string().as_bytes())
        .notify();
    true
}

///
pub fn transfer_multi(param: &[TrMulParam]) -> bool {
    for item in param.iter() {
        assert!(transfer(item.from, item.to, item.id, item.amt));
    }
    true
}

pub fn approve(owner: &Address, spender: &Address, token_id: &[u8], amt: U128) -> bool {
    assert!(check_witness(owner));
    let owner_ba = balance_of(owner, token_id);
    assert!(amt > 0);
    assert!(owner_ba >= amt);
    let key = gen_approve_key(token_id, owner.as_ref(), spender.as_ref());
    database::put(key, amt);
    EventBuilder::new()
        .bytearray(b"approval")
        .bytearray(owner.as_ref())
        .bytearray(spender.as_ref())
        .bytearray(token_id)
        .number(amt)
        .notify();
    true
}

pub fn transfer_from(
    spender: &Address,
    from: &Address,
    to: &Address,
    id: &[u8],
    amt: U128,
) -> bool {
    assert!(check_witness(spender));
    let from_key = gen_balance_key(id, from.as_ref());
    let from_ba: U128 = database::get::<_, U128>(from_key.as_slice()).unwrap_or(0);
    assert!(amt > 0);
    assert!(from_ba >= amt);

    let approve_key = gen_approve_key(id, from.as_ref(), spender.as_ref());
    let approve_amt: U128 = database::get::<_, U128>(approve_key.as_slice()).unwrap_or(0);
    if approve_amt < amt {
        panic!("you are not allowed to withdraw too many tokens")
    } else if approve_amt == amt {
        database::delete(approve_key.as_slice());
        if from_ba == amt {
            database::delete(from_key.as_slice());
        } else {
            let from_ba = from_ba.checked_sub(amt).unwrap();
            database::put(from_key.as_slice(), from_ba);
        }
    } else {
        let approve_amt = approve_amt.checked_sub(amt).unwrap();
        database::put(approve_key, approve_amt);
        let from_ba = from_ba.checked_sub(amt).unwrap();
        database::put(from_key, from_ba);
    }
    let to_key = gen_balance_key(id, to.as_ref());
    let to_ba: U128 = database::get::<_, U128>(to_key.as_slice()).unwrap_or(0);
    let to_ba = to_ba.checked_add(amt).unwrap();
    database::put(to_key, to_ba);
    EventBuilder::new()
        .bytearray(b"transfer")
        .bytearray(from.as_ref())
        .bytearray(to.as_ref())
        .bytearray(id)
        .number(amt)
        .notify();
    true
}

pub fn transfer_from_multi(param: &[TrFromMulParam]) -> bool {
    for item in param.iter() {
        assert!(transfer_from(
            item.spender,
            item.from,
            item.to,
            item.id,
            item.amt
        ));
    }
    true
}

pub fn approve_multi(param: &[AppMulParam]) -> bool {
    for item in param.iter() {
        assert!(approve(item.owner, item.spender, item.id, item.amt));
    }
    true
}

pub fn allowance(owner: &Address, spender: &Address, id: &[u8]) -> U128 {
    let key = gen_approve_key(id, owner.as_ref(), spender.as_ref());
    database::get::<_, U128>(key).unwrap_or(0)
}

fn get_next_id() -> u64 {
    database::get::<_, u64>(KEY_TOKEN_COUNTER).unwrap_or(0)
}

fn gen_key(pre: &[u8], k: &[u8]) -> Vec<u8> {
    [pre, k].concat()
}

fn gen_balance_key(id: &[u8], addr: &[u8]) -> Vec<u8> {
    [PRE_BALANCE, id, addr].concat()
}

fn gen_approve_key(id: &[u8], owner: &[u8], spender: &[u8]) -> Vec<u8> {
    [PRE_APPROVE, id, owner, spender].concat()
}
