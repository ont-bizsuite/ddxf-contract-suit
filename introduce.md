# ddxf合约

## marketplace 合约

* 合约地址
`e01d500ed0c1719b7750367ae59b4b2d308d1ceb`

该合约主要负责商品的发布、更新、删除、买卖功能。

* 商品发布
卖家可以将自己的商品信息通过调用该合约的 `dtokenSellerPublish`方法将商品信息保存到链上,
该方法会校验卖家和marketplace双方的交易签名

* 商品更新

卖家可以将自己的商品信息通过调用该合约的 `update`方法更新链上的商品信息,
该方法会校验卖家和marketplace双方的交易签名

* 删除
卖家和marketplace均可以删除失效的商品

* 买商品
买家可以通过调用该合约的 `buyDToken`方法购买商品，当用户调用该方法购买商品时，`marketplace`合约会调用
`DToken`合约生成DToken, DToken支持转移和使用功能。

## dtoken 合约

* 合约地址
`466b94488bf2ad1b1eec0ae7e49e40708e71a35d`
该合约提供DToken的生成、转移、使用、设置代理等功能。将来DToken会提供`OEP8`属性
DToken的生成依赖于 `TokenTemplate`, `TokenTemplate`由 `DataId`、`TokenHash`、`endpoint`、`token_name`和`token_symbol`组成。

`DataId`是物的ontid, 也是物的链上的唯一标志
`TokenHash` 是TokenMeta的Hash, `TokenMeta`表示权限
`endpoint` 


## splitpolicy合约
* 合约地址
`f024034fe7e5ea69c53cede4774bd1dad566234f`

合约描述
该合约用来给某个商品的多个所有者进行分润, 某个商品的每个所有者的利益分配参数是在卖家调用 `marketplace`合约发布商品的时候指定好的。

## openkg合约

* 合约地址
`5f16f2985bba3f02f9e6783dda8542983e3c32b1`

该合约是为了方便openkg 调用ddxf协议的 `marketplace`和 `dtoken`合约中的方法专门提供的, 主要提供的方法是购买使用DToken

## dataId 合约

* 合约地址
`3a0eee3f373a5b43d4e5be3b055d77b8f1310bb8`

ontid是链上的身份标志，为了方便调用链上的ontid合约封装了该合约，该合约主要提供了注册ontid和为该ontid添加属性的功能



## oep4合约

* 合约地址
`195d72da6725e8243a52803f6de4cd93df48fc1f`

