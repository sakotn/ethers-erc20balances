const RPCS: [&str; 4] = [
    "https://rpc.ankr.com/eth",
    "https://rpc.ankr.com/polygon",
    "https://rpc.ankr.com/bsc",
    "https://rpc.ankr.com/avalanche",
];

#[cfg(test)]
mod tests {
    use crate::RPCS;
    use ethers::core::rand;
    use ethers::middleware::SignerMiddleware;
    use ethers::prelude::{abigen, Http, Lazy, LocalWallet, Provider, ProviderExt};
    use ethers::types::Address;
    use ethers_erc20balances::Erc20BalancesMiddleware;
    use std::str::FromStr;
    use std::sync::Arc;
    use std::time::Duration;

    abigen! {
        Erc20,
        r#"[
            function balanceOf(address tokenOwner) public constant returns (uint balance)
        ]"#
    }

    static ADDRESS: Lazy<Address> = Lazy::new(|| Address::zero());

    static TOKEN_ADDRESS: Lazy<[Address; 4]> = Lazy::new(|| {
        [
            Address::from_str("0xdAC17F958D2ee523a2206206994597C13D831ec7").unwrap(),
            Address::from_str("0xc2132D05D31c914a87C6611C10748AEb04B58e8F").unwrap(),
            Address::from_str("0x55d398326f99059fF775485246999027B3197955").unwrap(),
            Address::from_str("0xc7198437980c041c805A1EDcbA50c1Ce5db95118").unwrap(),
        ]
    });

    #[tokio::test]
    async fn test_send_request_provider() {
        for (rpc, token_address) in RPCS.into_iter().zip(TOKEN_ADDRESS.iter()) {
            let provider = Arc::new(Provider::<Http>::connect(rpc).await);
            let balance_contract = Erc20::new(token_address.clone(), provider.clone());
            let balance = balance_contract.balance_of(ADDRESS.clone()).await.unwrap();
            dbg!(&balance);
            tokio::time::sleep(Duration::from_secs(1)).await;
            let balances = provider
                .get_erc20_balances(vec![ADDRESS.clone()], vec![token_address.clone()])
                .await
                .unwrap();
            dbg!(&balances);

            assert_eq!(balance, balances[&ADDRESS][token_address]);
        }
    }

    #[tokio::test]
    async fn test_send_request_signer() {
        for (rpc, token_address) in RPCS.into_iter().zip(TOKEN_ADDRESS.iter()) {
            let provider = Arc::new(Provider::<Http>::connect(rpc).await);
            let wallet = LocalWallet::new(&mut rand::thread_rng());
            let signer = Arc::new(SignerMiddleware::new(provider.clone(), wallet));
            let balance_contract = Erc20::new(token_address.clone(), signer.clone());
            let balance = balance_contract.balance_of(ADDRESS.clone()).await.unwrap();
            tokio::time::sleep(Duration::from_secs(1)).await;
            let balances = signer
                .get_erc20_balances(vec![ADDRESS.clone()], vec![token_address.clone()])
                .await
                .unwrap();

            assert_eq!(balance, balances[&ADDRESS][token_address]);
        }
    }
}
