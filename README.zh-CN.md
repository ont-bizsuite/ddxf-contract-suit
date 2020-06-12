# ddxf合约族

ddxf是一个通用的去中心化数据交易框架，解决的痛点是数据自治。

[这里](https://ont-bizsuite.github.io/ddxf-contract-suite/doc/ddxf/)是ddxf合约的开发文档。

该框架中有以下角色：卖家、MP、买家、Registry、Browser。

对MP、Registry、Browser作出解释。

这里的MP是个抽象概念，其实就是我们日常使用的各类APP。不同的是，在ddxf框架中MP主要的作用只是提供商品数据和交易担保，从而收取一定的手续费。

Registry是MP的注册中心，所有的MP都会注册到Registry。

而Browser则是能够跟Registry和MP交互的通用浏览器。

交易流程是：

1. 卖家通过Browser发布商品到MP。
2. 买家通过Browser浏览到该商品，并购买，拿到链上的dtoken和endpoint。
3. 买家通过Browser到endpoint指定的“网点”核销该dtoken，并拿到商品。

ddxf把整个交易过程都记录到了链上，一旦产生纠纷，便能以链上的数据作为仲裁证据。

为了以统一的方式描述商品，我们定义了data_meta、token_meta和item_meta。

data_meta即数据本身的元信息，token_meta即权限的元信息，item_meta即商品的元信息。

通过将上述3个meta元信息的上链，我们便把商品的核心特性以不可篡改的方式存了下来。另外由于链上资源宝贵，实际链上存的是元信息的可验哈希和endpoint，endpoint是链外的查询接口，通过可验哈希能查到对应的元信息。

