# Call one token balance

```rust
let provider = Arc::new(Provider::<Http>::connect(rpc).await);
let balances = provider.get_erc20_balances(vec![Address::zero()], vec![Address::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap()).await.unwrap();
```

result:
```json
0x0000000000000000000000000000000000000000: {
    0xdac17f958d2ee523a2206206994597c13d831ec7: 17962155817,
}
```

# Call many tokens balance

```rust
let tokens = vec![
    Address::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap(), // USDT
    Address::from_str("0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(), // DAI
]
let provider = Arc::new(Provider::<Http>::connect(rpc).await);
let balances = provider.get_erc20_balances(vec![Address::zero()], tokens).await.unwrap();
```

result:
```json
0x0000000000000000000000000000000000000000: {
    0xdac17f958d2ee523a2206206994597c13d831ec7: 17962155817,
    0x6b175474e89094c44da98b954eedeac495271d0f: 9234114788971643034663,
},
```

# Call many tokens balance with many addresses

```rust
let tokens = vec![
    Address::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap(), // USDT
    Address::from_str("0x6B175474E89094C44Da98b954EedeAC495271d0F").unwrap(), // DAI
];
let wallets = vec![
    Address::zero(),
    Address::from_str("0xF977814e90dA44bFA03b6295A0616a897441aceC").unwrap(),
];
let provider = Arc::new(Provider::<Http>::connect(rpc).await);
let balances = provider.get_erc20_balances(wallets, tokens).await.unwrap();
```

result:
```json
0x0000000000000000000000000000000000000000: {
    0x6b175474e89094c44da98b954eedeac495271d0f: 9234114788971643034663,
    0xdac17f958d2ee523a2206206994597c13d831ec7: 17962155817,
},
0xf977814e90da44bfa03b6295a0616a897441acec: {
    0x6b175474e89094c44da98b954eedeac495271d0f: 7000000000000000000000000,
    0xdac17f958d2ee523a2206206994597c13d831ec7: 1429615184000000,
},
```