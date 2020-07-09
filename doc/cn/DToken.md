# DToken接口设计

DToken包含用于表示物的jwtToken和用来表示资产的Token两部分组成, 下面会分别介绍这两部分的接口逻辑。

## jwtToken逻辑

TokenTemplate结构体设计
```rust
pub struct TokenTemplate {
    pub data_id: Option<Vec<u8>>,
    pub token_hash: Vec<Vec<u8>>,
    pub endpoint: Vec<u8>,
}
```
参数解释:
* `data_id`是物的`ontid`, 可以为空
* `token_hash`是链下存的`TokenMeta`的hash值数组
* `endpoint`是一个服务器的接口，通过该接口可以根据`token_hash`可以查询到链下的`TokenMeta`

1. 创建TokenTemplate

接口设计
```
fn create_token_template(creator:&Address, token_template_bs:&[u8]) -> bool
```
参数解释:
* 任何人都可以创建，也就是`creator`不受限制
* 校验`creator`签名
* 该方法会推出event, 客户端需要监听该事件获得`token_template_id`(数据格式是bytearray) 
* 存储设计`creator+token_template_id -> TokenTemplate`

Event设计:
```
["create_token_template",creator, token_template_bs, token_template_id]
```

2. 授权其他地址可以根据该TokenTemplate生成DToken   (被授权的地址 可以生成token的数量有限制吗)

接口设计
```
fn authorize_token_template(creator:&Address, token_template_id:&[u8], authorized_addr: &Address) -> bool
```
参数解释:
* `creator`是`token_template_id`的创建者地址
* 校验`creator`签名， 只有创建者可以授权给其他人
* 存储设计 `token_template_id+authorized_addr -> bool` 这样可以节省空间，但是不能遍历已经被授权的所有地址信息，
如果需要遍历已经被授权的地址,存储应该采用下面的设计
`token_template_id -> [Address,]`
   

3. generateDToken 生成DToken

接口设计
```
pub fn generate_token(acc:&Address, token_template_id:&[u8], n:U128) -> bool 
```

参数解释:
* `acc`是`TokenTemplate`的创建者或者被授权的地址
* 校验 `acc`签名
* `n` 发行的Token的总量
* 该接口会生成`TokenId`,会通过Event推送出去, 客户端需要监听该event
* 存储设计 `token_template_id -> token_id`

Event设计:
```
["generate_token",acc, token_template_id, n, token_id]
```


4. useDToken 使用DToken

接口设计
```
fn use_token(acc:&Address, token_id:&[u8], n:U128) -> bool
```
* `acc`是DToken的持有者
* 校验 `acc`签名
* `n`使用DToken的数量

## Token设计

1. DToken transfer(OEP8标准)

接口设计
```
fn transfer(from: &Address, to: &Address, id: &[u8], amt: u128) -> bool
```

* `from` DToken的转出方地址，会校验其签名
* `to` DToken的转入方地址
* `id` 是`TokenId`
* `amt`要转移的数量



## 遇到的问题

1. 被授权的地址 可以生成DToken的数量有限制吗
2. 使用Token即UseToken接口， 输入的参数是TokenId 还是TokenTemplateId
