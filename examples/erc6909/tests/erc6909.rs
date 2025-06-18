#![cfg(feature = "e2e")]

use abi::Erc6909;
use alloy::primitives::{Address, U256};
use e2e::{receipt, watch, Account, EventExt};

mod abi;

fn random_values(size: usize) -> Vec<U256> {
    (1..=size).map(U256::from).collect()
}

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

#[e2e::test]
async fn mint(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let token_id = random_token_ids(1)[0];
    let amount = random_values(1)[0];

    let receipt = receipt!(contract.mint(alice_addr, token_id, amount))?;

    assert!(receipt.emits(Erc6909::TransferSingle {
        caller: alice_addr,
        from: Address::ZERO,
        to: alice_addr,
        id: token_id,
        amount
    }));

    let Erc6909::balanceOfReturn { balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(amount, balance);

    Ok(())
}

#[e2e::test]
async fn transfer_from(alice: Account, bob: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909::new(contract_addr, &alice.wallet);

    let alice_addr = alice.address();
    let bob_addr = bob.address();
    let token_id = random_token_ids(1)[0];
    let value = random_values(1)[0];
    watch!(contract.mint(alice_addr, token_id, value,))?;

    let Erc6909::balanceOfReturn { balance: initial_alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    let Erc6909::balanceOfReturn { balance: initial_bob_balance } =
        contract.balanceOf(bob_addr, token_id).call().await?;

    let receipt = receipt!(
        contract.transferFrom(alice_addr, bob_addr, token_id, value,)
    )?;

    assert!(receipt.emits(Erc6909::TransferSingle {
        caller: alice_addr,
        from: alice_addr,
        to: bob_addr,
        id: token_id,
        amount: value
    }));

    let Erc6909::balanceOfReturn { balance: alice_balance } =
        contract.balanceOf(alice_addr, token_id).call().await?;
    assert_eq!(initial_alice_balance - value, alice_balance);

    let Erc6909::balanceOfReturn { balance: bob_balance } =
        contract.balanceOf(bob_addr, token_id).call().await?;
    assert_eq!(initial_bob_balance + value, bob_balance);

    Ok(())
}
