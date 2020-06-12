# ddxf contract suite

ddxf is a decentralized data exchange framework, aimed to solve the data self sovereign problem.

For the impatient, [here](https://ont-bizsuite.github.io/ddxf-contract-suite/doc/ddxf/) is the contract document for ddxf. 

There're 5 roles in this framework: Seller, Marketplace, Buyer, Registry and Browser.

Some explanations for Marketplace, Registry and Browser.

Marketplace is an abstract concept, it's just those apps we use daily. What's different is, in ddxf, the main function of Marketplace is to provide item data and endorsement.

Registry is the central place for Marketplace, each Marketplace will register to the Registry.

Browser is a general tool to interact with Registry and Marketplace.


The trading process is :

1. Seller publishes an item to Marketplace via Browser.
2. Buyer finds an item via Browser, then buys it, and gets a dtoken and endpoint, the dtoken is recorded on blockchain.
3. Buyer uses the above dtoken via Browser through the endpoint, and gets the data he wanted.

ddxf records the whole process on blockchain, which will be the evidence if legal issue happens.

In order to decribe an item generally, we define 3 metas, namely data_meta, token_meta and item_meta。

data_meta is meta info about the data itself, token_meta is the meta info about the permissions, item_meta is the meta info about items.

By recording the above 3 metas on blockchain, we've saved the item info in a tamperless way. In fact, because the resource on blockchain is expensive, we only record the verifiable hashes of those metas and corresponding endpoint, the endpoint is used to look up the original meta info by the verifiable hashes.

