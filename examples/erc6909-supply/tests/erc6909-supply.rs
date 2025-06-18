#![cfg(feature = "e2e")]

use abi::Erc6909Supply;
use e2e::Account;

mod abi;

// ============================================================================
// Integration Tests: ERC-6909 Supply Extension
// ============================================================================

// TODO

// ============================================================================
// Integration Tests: ERC-165 Support Interface
// ============================================================================

#[e2e::test]
async fn supports_interface(alice: Account) -> eyre::Result<()> {
    let contract_addr = alice.as_deployer().deploy().await?.contract_address;
    let contract = Erc6909Supply::new(contract_addr, &alice.wallet);
    let invalid_interface_id: u32 = 0xffffffff;
    let supports_interface = contract
        .supportsInterface(invalid_interface_id.into())
        .call()
        .await?
        ._0;

    assert!(!supports_interface);

    let erc6909_interface_id: u32 = 0xbd85b039;
    let supports_interface = contract
        .supportsInterface(erc6909_interface_id.into())
        .call()
        .await?
        ._0;

    assert!(supports_interface);

    let erc165_interface_id: u32 = 0x01ffc9a7;
    let supports_interface =
        contract.supportsInterface(erc165_interface_id.into()).call().await?._0;

    assert!(supports_interface);

    Ok(())
}
