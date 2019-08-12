use std::sync::Arc;

use rand::prelude::*;

use chain_client::{ChainClient, RpcChainClient};
use mock_chain_client::MockChainClient;
use nextgen_crypto::test_utils::KeyPair;
use nextgen_crypto::Uniform;
use types::account_address::AccountAddress;

use super::wallet::*;
use types::account_config::coin_struct_tag;
use logger::prelude::*;

#[test]
fn test_wallet() {
    ::logger::init_for_e2e_testing();
    let amount: u64 = 1_000_000_000;
    let mut rng: StdRng = SeedableRng::from_seed([0; 32]);
    let keypair = KeyPair::generate_for_testing(&mut rng);
    let client = Arc::new(MockChainClient::new());
    let account_address = AccountAddress::from_public_key(&keypair.public_key);
    debug!("account_address: {}", account_address);
    client.faucet(account_address, amount).unwrap();
    let mut wallet = Wallet::new_with_client(account_address, keypair, client).unwrap();
    assert_eq!(amount, wallet.balance());

    let account_address2 = AccountAddress::random();
    let transfer_amount = 1_000_000;
    let offchain_txn = wallet.transfer(coin_struct_tag(), account_address2, transfer_amount).unwrap();
    debug!("txn:{:#?}", offchain_txn);
    wallet.apply_txn(&offchain_txn);
    assert_eq!(amount - transfer_amount - offchain_txn.output().gas_used(), wallet.balance());
}
