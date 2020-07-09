# Marketplace接口设计

marketplace提供商品的发布，交易等功能

## 接口设计

1. 发布商品 publish

```
fn dtoken_seller_publish(
     item_id: &[u8],
     basic_bytes: &[u8],
     item_bytes: &[u8],
     split_policy_param_bytes: &[u8],
 ) -> bool
```

发布参数
```
struct BasicParam {
    pub manager: Address, // data owner
    pub item_meta_hash: H256,
    pub item_meta_endpoint:&[u8],
    pub dtoken_contract_address: Option<Vec<Address>>, // can be empty
    pub mp_contract_address: Option<Address>,          // can be empty
    pub split_policy_contract_address: Option<Address>, //can be empty
}
struct DTokenItem {
    pub fee: Fee,
    pub expired_date: u64,
    pub stocks: u32,
    pub sold: u32,
    pub token_template_ids: Vec<Vec<u8>>,
}
```

* `item_id`用来标识链上唯一的商品
* `BasicParam` 包含`item_meta_hash`和`item_meta_endpoint`(当前版本缺失), 通过这两个参数可以查询到链下的item_meta信息
* `DTokenItem` 包含`TokenTemplateId`数组`token_template_ids`,
* 发布商品需要卖家 和mp双方的签名

问题：
* 分润逻辑怎么设计


2. 更新商品 update 

```
fn dtoken_seller_update(
     item_id: &[u8],
     basic_bytes: &[u8],
     item_bytes: &[u8],
     split_policy_param_bytes: &[u8],
 ) -> bool
```

* 需要卖家和mp双方的签名


问题 有哪些字段会更新？

3. 删除商品
```
fn delete(item_id: &[u8]) -> bool
```

* 只需要卖家签名就可以删除商品

4. buyDToken 购买DToken

```
fn buy_dtoken(item_id: &[u8], n: U128, buyer_account: &Address, payer: &Address) -> bool
```

问题
购买的时候，需要生成DToken, 但是买家没有被授权生成DToken怎么办？