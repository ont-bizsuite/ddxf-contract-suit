use super::*;

use hexutil::read_hex;

#[test]
fn test() {
    let data = "0331323301033132330100000001033132337b00000001036b65790576616c7565027479";
    let d = read_hex(data).unwrap_or_default();
    let rp = RegIdAddAttributesParam::from_bytes(d.as_slice());
    assert_eq!(rp.ont_id.as_slice(), b"123");

    let data = read_hex("1a7265675f69645f6164645f6174747269627574655f61727261799b01992a6469643a6f6e743a5459354a626369646445664d73684536544261324e42466964335447523459543572012a6469643a6f6e743a544247646354684a5056756f6946776f67625354784e724874754839756a54466e5001000000012a6469643a6f6e743a544247646354684a5056756f6946776f67625354784e724874754839756a54466e500100000001036b65790576616c7565027479").unwrap_or_default();

    let mut source = Source::new(data.as_slice());
    let action: &[u8] = source.read().unwrap();
    assert_eq!(action, b"reg_id_add_attribute_array");

    let pa: Vec<Vec<u8>> = source.read().unwrap();
    let a = 1;
}
