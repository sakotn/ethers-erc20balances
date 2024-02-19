use async_trait::async_trait;
use auto_impl::auto_impl;
use ethers::prelude::{
    abigen, Address, JsonRpcClient, Lazy, Middleware, Provider, ProviderError, Signer,
    SignerMiddleware, U256,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

pub static CONTRACTS: Lazy<HashMap<U256, Address>> = Lazy::new(|| {
    HashMap::from([
        (
            1.into(),
            Address::from_str("0xb1f8e55c7f64d203c1400b9d8555d050f94adf39")
                .expect("Failed to parse address"),
        ), // mainnet
        (
            3.into(),
            Address::from_str("0x8D9708f3F514206486D7E988533f770a16d074a7")
                .expect("Failed to parse address"),
        ), // ropsten
        (
            4.into(),
            Address::from_str("0x3183B673f4816C94BeF53958BaF93C671B7F8Cf2")
                .expect("Failed to parse address"),
        ), // rinkeby
        (
            69.into(),
            Address::from_str("0x55ABBa8d669D60A10c104CC493ec5ef389EC92bb")
                .expect("Failed to parse address"),
        ), // kovan
        (
            5.into(),
            Address::from_str("0x9788C4E93f9002a7ad8e72633b11E8d1ecd51f9b")
                .expect("Failed to parse address"),
        ), // goerli
        (
            56.into(),
            Address::from_str("0x2352c63A83f9Fd126af8676146721Fa00924d7e4")
                .expect("Failed to parse address"),
        ), // binance smart chain mainnet
        (
            97.into(),
            Address::from_str("0x2352c63A83f9Fd126af8676146721Fa00924d7e4")
                .expect("Failed to parse address"),
        ), // binance smart chain testnet
        (
            137.into(),
            Address::from_str("0x2352c63A83f9Fd126af8676146721Fa00924d7e4")
                .expect("Failed to parse address"),
        ), // polygon
        (
            80001.into(),
            Address::from_str("0x2352c63A83f9Fd126af8676146721Fa00924d7e4")
                .expect("Failed to parse address"),
        ), // mumbai
        (
            10.into(),
            Address::from_str("0xB1c568e9C3E6bdaf755A60c7418C269eb11524FC")
                .expect("Failed to parse address"),
        ), // optimism
        (
            69.into(),
            Address::from_str("0xB1c568e9C3E6bdaf755A60c7418C269eb11524FC")
                .expect("Failed to parse address"),
        ), // optimism kovan
        (
            42161.into(),
            Address::from_str("0x151E24A486D7258dd7C33Fb67E4bB01919B7B32c")
                .expect("Failed to parse address"),
        ), // arbitrum
        (
            43114.into(),
            Address::from_str("0xD023D153a0DFa485130ECFdE2FAA7e612EF94818")
                .expect("Failed to parse address"),
        ), // avax
        (
            250.into(),
            Address::from_str("0x07f697424ABe762bB808c109860c04eA488ff92B")
                .expect("Failed to parse address"),
        ), // fantom
        (
            25.into(),
            Address::from_str("0x56a4420cb0ef5b0d14ce1bbe380992fa31d6a907")
                .expect("Failed to parse address"),
        ), // cronos
        (
            66.into(),
            Address::from_str("0x25B3584f4799F788c0189dd6496b0AA02cBA4605")
                .expect("Failed to parse address"),
        ), // okt
        (
            1666600000.into(),
            Address::from_str("0x549b6A5A3027F9B73A23Db4bb95701bAcb9b9573")
                .expect("Failed to parse address"),
        ), // harmony
        (
            17000.into(),
            Address::from_str("0x437DF28584e878948aE2417E86e15690cCf822F4")
                .expect("Failed to parse address"),
        ), // holesky
    ])
});

abigen!(BalanceChecker, "abi/BalanceChecker.abi.json",);

#[async_trait]
#[auto_impl(&, Arc, Box)]
pub trait Erc20BalancesMiddleware {
    type Error;

    async fn get_erc20_balances(
        &self,
        address: Vec<Address>,
        token_addresses: Vec<Address>,
    ) -> Result<HashMap<Address, HashMap<Address, U256>>, Self::Error>;
    async fn get_erc20_balances_with_chain_id(
        &self,
        address: Vec<Address>,
        token_addresses: Vec<Address>,
        chain_id: U256,
    ) -> Result<HashMap<Address, HashMap<Address, U256>>, Self::Error>;
}

#[async_trait]
impl<P> Erc20BalancesMiddleware for Provider<P>
where
    P: JsonRpcClient,
    Self: Clone,
{
    type Error = ProviderError;

    async fn get_erc20_balances(
        &self,
        address: Vec<Address>,
        token_addresses: Vec<Address>,
    ) -> Result<HashMap<Address, HashMap<Address, U256>>, Self::Error> {
        let chain_id = self.get_chainid().await?;
        self.get_erc20_balances_with_chain_id(address, token_addresses, chain_id)
            .await
    }

    async fn get_erc20_balances_with_chain_id(
        &self,
        address: Vec<Address>,
        token_addresses: Vec<Address>,
        chain_id: U256,
    ) -> Result<HashMap<Address, HashMap<Address, U256>>, Self::Error> {
        let contract_address = CONTRACTS
            .get(&chain_id)
            .ok_or_else(|| {
                ProviderError::CustomError(format!("Chain id {} not supported", chain_id))
            })?
            .clone();
        let contract = BalanceChecker::new(contract_address, Arc::new(self.clone()));
        let balances = contract.balances(address.clone(), token_addresses.clone()).call().await.map_err(|e|
            ProviderError::CustomError(format!("Failed to get balances for addresses {:?} and tokens {:?} on chain id {}; {:?}", address, token_addresses, chain_id, e))
        )?;
        Ok(reformat(address, token_addresses, balances))
    }
}

#[async_trait]
impl<M, S> Erc20BalancesMiddleware for SignerMiddleware<M, S>
where
    M: Middleware,
    S: Signer,
    Self: Clone,
{
    type Error = ProviderError;

    async fn get_erc20_balances(
        &self,
        address: Vec<Address>,
        token_addresses: Vec<Address>,
    ) -> Result<HashMap<Address, HashMap<Address, U256>>, Self::Error> {
        let chain_id = self
            .get_chainid()
            .await
            .map_err(|e| ProviderError::CustomError(format!("Failed to get chain id; {:?}", e)))?;
        Self::get_erc20_balances_with_chain_id(self, address, token_addresses, chain_id).await
    }

    async fn get_erc20_balances_with_chain_id(
        &self,
        address: Vec<Address>,
        token_addresses: Vec<Address>,
        chain_id: U256,
    ) -> Result<HashMap<Address, HashMap<Address, U256>>, Self::Error> {
        let contract_address = CONTRACTS
            .get(&chain_id)
            .ok_or_else(|| {
                ProviderError::CustomError(format!("Chain id {} not supported", chain_id))
            })?
            .clone();
        let contract = BalanceChecker::new(contract_address, Arc::new(self.clone()));
        let balances = contract.balances(address.clone(), token_addresses.clone()).call().await.map_err(|e|
            ProviderError::CustomError(format!("Failed to get balances for addresses {:?} and tokens {:?} on chain id {}; {:?}", address, token_addresses, chain_id, e))
        )?;
        Ok(reformat(address, token_addresses, balances))
    }
}

fn reformat(
    addresses: Vec<Address>,
    token_addresses: Vec<Address>,
    balances: Vec<U256>,
) -> HashMap<Address, HashMap<Address, U256>> {
    balances
        .chunks(token_addresses.len())
        .enumerate()
        .map(|(i, balances)| {
            let address = addresses[i];
            let balances = token_addresses
                .iter()
                .zip(balances.iter())
                .map(|(token_address, balance)| (token_address.clone(), balance.clone()))
                .collect();
            (address, balances)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reformat() {
        // Create sample addresses
        let addresses = vec![
            Address::from_str("0xb1f8e55c7f64d203c1400b9d8555d050f94adf39").unwrap(),
            Address::from_str("0x8D9708f3F514206486D7E988533f770a16d074a7").unwrap(),
        ];

        // Create sample token addresses
        let token_addresses = vec![
            Address::from_str("0x3183B673f4816C94BeF53958BaF93C671B7F8Cf2").unwrap(),
            Address::from_str("0x55ABBa8d669D60A10c104CC493ec5ef389EC92bb").unwrap(),
        ];

        // Create sample balances
        let balances = vec![
            U256::from(10), // balance for address[0] and token[0]
            U256::from(7),  // balance for address[0] and token[1]
            U256::from(5),  // balance for address[1] and token[0]
            U256::from(7),  // balance for address[1] and token[1]
        ];

        // Get result from reformat function
        let result = reformat(addresses.clone(), token_addresses.clone(), balances);

        // Create expected result
        let mut expected = HashMap::new();
        let mut address_0_balances = HashMap::new();
        address_0_balances.insert(token_addresses[0], U256::from(10));

        let mut address_1_balances = HashMap::new();
        address_1_balances.insert(token_addresses[0], U256::from(5));
        address_1_balances.insert(token_addresses[1], U256::from(7));

        expected.insert(addresses[0], address_0_balances);
        expected.insert(addresses[1], address_1_balances);

        assert_eq!(result, expected);
    }
}
