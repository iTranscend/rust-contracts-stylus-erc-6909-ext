use alloy::{
    network::{AnyNetwork, EthereumWallet},
    primitives::Address,
    providers::ProviderBuilder,
    sol,
    sol_types::SolCall,
    uint,
};
use e2e::{receipt, Account};

use crate::{
    report::{ContractReport, FunctionReport},
    Opt,
};

sol!(
    #[sol(rpc)]
    contract Erc6909Supply {
        function mint(address to, uint256 id, uint256 amount) external;
        function totalSupply(uint256 id) external view returns (uint256);
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc6909Supply", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_addr = alice.address();
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc6909Supply::new(contract_addr, &alice_wallet);

    let token = uint!(1_U256);
    let value = uint!(100_U256);

    // IMPORTANT: Order matters!
    use Erc6909Supply::*;
    #[rustfmt::skip]
    let receipts = vec![
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, token, value))?),
        (totalSupplyCall::SIGNATURE, receipt!(contract.totalSupply(token))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "erc6909-supply", None, cache_opt).await
}
