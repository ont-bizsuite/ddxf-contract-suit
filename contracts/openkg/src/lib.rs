

fn freeze_and_publish(
    old_resource_id: &[u8],
    new_resource_id: &[u8],
    resource_ddo_bytes: &[u8],
    item_bytes: &[u8],
    split_policy_param_bytes: &[u8],
) -> bool {
    assert!(freeze(old_resource_id));
    assert!(dtoken_seller_publish(
        new_resource_id,
        resource_ddo_bytes,
        item_bytes,
        split_policy_param_bytes
    ));
    true
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
