use super::oep8::*;
use super::ostd::mock::build_runtime;
use super::Address;

#[test]
fn test_name() {
    let admin = Address::repeat_byte(1);
    let handle = build_runtime();
    let token_id = generate_token(b"name", b"symbol", 1000, &admin);
    let nam = name(token_id.as_slice());
    assert_eq!(nam, b"name");
    let sym = symbol(token_id.as_slice());
    assert_eq!(sym, b"symbol");
    let supply = total_supply(token_id.as_slice());
    assert_eq!(supply, 1000);

    let ba = balance_of(&admin, token_id.as_slice());
    assert_eq!(ba, 1000);

    handle.witness(&[admin]);
    let to = Address::repeat_byte(2);
    assert!(transfer(&admin, &to, token_id.as_slice(), 10));
    let ba = balance_of(&admin, token_id.as_slice());
    assert_eq!(ba, 990);
    let ba = balance_of(&to, token_id.as_slice());
    assert_eq!(ba, 10);

    let to2 = Address::repeat_byte(3);
    let tp = TrMulParam {
        from: &admin,
        to: &to,
        id: token_id.as_slice(),
        amt: 10,
    };
    let tp2 = TrMulParam {
        from: &admin,
        to: &to2,
        id: token_id.as_slice(),
        amt: 10,
    };
    assert!(transfer_multi(&[tp, tp2]));
    let admin_ba = balance_of(&admin, token_id.as_slice());
    assert_eq!(admin_ba, 990 - 10 - 10);
    let to_ba = balance_of(&to, token_id.as_slice());
    assert_eq!(to_ba, 10 + 10);
    let to2_ba = balance_of(&to2, token_id.as_slice());
    assert_eq!(to2_ba, 10);

    assert!(approve(&admin, &to, token_id.as_slice(), 10));
    let all = allowance(&admin, &to, token_id.as_slice());
    assert_eq!(all, 10);

    handle.witness(&[to]);
    assert!(transfer_from(&to, &admin, &to, token_id.as_slice(), 5));

    let all = allowance(&admin, &to, token_id.as_slice());
    assert_eq!(all, 10 - 5);

    let admin_ba = balance_of(&admin, token_id.as_slice());
    assert_eq!(admin_ba, 990 - 10 - 10 - 5);
    let to_ba = balance_of(&to, token_id.as_slice());
    assert_eq!(to_ba, 10 + 10 + 5);

    let tfp = TrFromMulParam {
        spender: &to,
        from: &admin,
        to: &to,
        id: token_id.as_slice(),
        amt: 1,
    };
    let tfp2 = TrFromMulParam {
        spender: &to,
        from: &admin,
        to: &to2,
        id: token_id.as_slice(),
        amt: 1,
    };
    assert!(transfer_from_multi(&[tfp, tfp2]));
    let all = allowance(&admin, &to, token_id.as_slice());
    assert_eq!(all, 10 - 5 - 1 - 1);

    let to_ba = balance_of(&to, token_id.as_slice());
    assert_eq!(to_ba, 10 + 10 + 5 + 1);

    let to2_ba = balance_of(&to2, token_id.as_slice());
    assert_eq!(to2_ba, 10 + 1);
}
