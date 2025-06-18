#![cfg(feature = "e2e")]

use abi::Erc6909;
use alloy::primitives::U256;
use e2e::Account;

mod abi;

fn random_token_ids(size: usize) -> Vec<U256> {
    (0..size).map(U256::from).collect()
}

// ============================================================================
// Integration Tests: ERC-6909 Token
// ============================================================================

#[e2e::test]
async fn balance_of_zero_balance(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909::new(contract_addr, &alice.wallet);
    let token_ids = random_token_ids(1);

    let Erc6909::balanceOfReturn { balance } =
        contract.balanceOf(alice.address(), token_ids[0]).call().await?;
    assert_eq!(U256::ZERO, balance);

    Ok(())
}
