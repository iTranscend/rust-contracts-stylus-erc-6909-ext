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
    contract Erc6909 {
        function transfer(address receiver, uint256 id, uint256 amount) external returns (bool status);
        function transferFrom(address sender, address receiver, uint256 id, uint256 amount) external returns (bool status);
        function approve(address spender, uint256 id, uint256 amount) external returns (bool status);
        function setOperator(address spender, bool approved) external returns (bool status);
        function balanceOf(address owner, uint256 id) external view returns (uint256 balance);
        function allowance(address owner, address spender, uint256 id) external view returns (uint256 balance);
        function isOperator(address owner, address spender) external returns (bool status);
        function mint(address to, uint256 id, uint256 amount) external;
        function mintBatch(address to, uint256[] memory ids, uint256[] memory amounts) external;
        function burn(address from, uint256 id, uint256 amount) external;
        function burnBatch(address from, uint256[] memory ids, uint256[] memory amounts) external;
    }
);

pub async fn bench() -> eyre::Result<ContractReport> {
    ContractReport::generate("Erc6909", run).await
}

pub async fn run(cache_opt: Opt) -> eyre::Result<Vec<FunctionReport>> {
    let alice = Account::new().await?;
    let alice_addr = alice.address();
    let alice_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(alice.signer.clone()))
        .on_http(alice.url().parse()?);

    let bob = Account::new().await?;
    let bob_addr = bob.address();
    let bob_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(bob.signer.clone()))
        .on_http(bob.url().parse()?);

    let charlie = Account::new().await?;
    let charlie_addr = charlie.address();
    let charlie_wallet = ProviderBuilder::new()
        .network::<AnyNetwork>()
        .with_recommended_fillers()
        .wallet(EthereumWallet::from(charlie.signer.clone()))
        .on_http(charlie.url().parse()?);

    let contract_addr = deploy(&alice, cache_opt).await?;

    let contract = Erc6909::new(contract_addr, &alice_wallet);
    let contract_bob = Erc6909::new(contract_addr, &bob_wallet);
    let contract_charlie = Erc6909::new(contract_addr, &charlie_wallet);

    let token_1 = uint!(1_U256);
    let token_2 = uint!(2_U256);
    let token_3 = uint!(3_U256);
    let token_4 = uint!(4_U256);

    let value_1 = uint!(100_U256);
    let value_2 = uint!(200_U256);
    let value_3 = uint!(300_U256);
    let value_4 = uint!(400_U256);

    let ids = vec![token_1, token_2, token_3, token_4];
    let values = vec![value_1, value_2, value_3, value_4];

    // IMPORTANT: Order matters!
    use Erc6909::*;
    #[rustfmt::skip]
    let receipts = vec![
        (mintCall::SIGNATURE, receipt!(contract.mint(alice_addr, token_1, value_1))?),
        (mintBatchCall::SIGNATURE, receipt!(contract.mintBatch(alice_addr, ids.clone(), values.clone()))?),
        (balanceOfCall::SIGNATURE, receipt!(contract.balanceOf(alice_addr, token_1))?),
        (approveCall::SIGNATURE, receipt!(contract.approve(bob_addr, token_1, value_1))?),
        (allowanceCall::SIGNATURE, receipt!(contract.allowance(alice_addr, bob_addr, token_1))?),
        (setOperatorCall::SIGNATURE, receipt!(contract.setOperator(charlie_addr, true))?),
        (isOperatorCall::SIGNATURE, receipt!(contract.isOperator(alice_addr, charlie_addr))?),
        (transferCall::SIGNATURE, receipt!(contract.transfer(bob_addr, token_1, value_1))?),
        (transferFromCall::SIGNATURE, receipt!(contract_charlie.transferFrom(alice_addr, bob_addr, token_1, value_1))?),
        (burnCall::SIGNATURE, receipt!(contract_bob.burn(bob_addr, token_1, value_1))?),
        (burnBatchCall::SIGNATURE, receipt!(contract_bob.burnBatch(bob_addr, ids, values))?),
    ];

    receipts
        .into_iter()
        .map(FunctionReport::new)
        .collect::<eyre::Result<Vec<_>>>()
}

async fn deploy(account: &Account, cache_opt: Opt) -> eyre::Result<Address> {
    crate::deploy(account, "erc6909", None, cache_opt).await
}
