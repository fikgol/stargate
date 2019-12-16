// Copyright (c) The Starcoin Core Contributors
// SPDX-License-Identifier: Apache-2.0
mod wallet_utils;

use std::sync::Arc;

use crate::wallet_utils::WalletLibrary;
use anyhow::{Error, Result};
use futures_01::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use libra_crypto::{
    ed25519::{Ed25519PrivateKey, Ed25519PublicKey},
    test_utils::KeyPair,
};
use libra_logger::prelude::*;
use libra_types::account_address::AccountAddress;
use network::{build_network_service, NetworkMessage, NetworkService};
use node::client;
use node_internal::node::Node;
use node_service::setup_node_service;
use router::Router;
use sg_config::config::{load_from, NodeConfig, WalletConfig};
use sgchain::star_chain_client::StarChainClient;
use sgwallet::wallet::*;
use structopt::StructOpt;
use tokio::runtime::{Handle, Runtime};

#[derive(Debug, StructOpt)]
#[structopt(
    name = "stargate",
    author = "star-team",
    about = "stargate local node "
)]
struct Args {
    #[structopt(short = "l", long = "enable_logging")]
    pub enable_logging: bool,
    #[structopt(short = "s", long = "start_client")]
    pub start_client: bool,
    #[structopt(short = "c", long = "config_dir", default_value = "wallet")]
    pub config_dir: String,
    #[structopt(short = "f", long = "faucet_key_path", default_value = "wallet/key")]
    pub faucet_key_path: String,
    #[structopt(short = "n", long = "child_number", default_value = "1")]
    pub child_num: u64,
}

pub struct Swarm {
    pub config: NodeConfig,
    _tee_logs: bool,
}

fn launch_swarm(args: &Args) -> Result<Swarm> {
    let node_config = load_from(&(args.config_dir.to_string() + "/node.toml"))?;
    Ok(Swarm {
        config: node_config,
        _tee_logs: true,
    })
}

fn load_from_keyfile(
    faucet_account_file: &str,
    child_num: u64,
) -> KeyPair<Ed25519PrivateKey, Ed25519PublicKey> {
    let wallet_library = WalletLibrary::recover(faucet_account_file).unwrap();
    wallet_library.get_keypair(child_num).unwrap()
}

fn gen_node(
    rt: &mut Runtime,
    executor: Handle,
    keypair: Arc<KeyPair<Ed25519PrivateKey, Ed25519PublicKey>>,
    wallet_config: &WalletConfig,
    network_service: NetworkService,
    sender: UnboundedSender<NetworkMessage>,
    receiver: UnboundedReceiver<NetworkMessage>,
    close_tx: futures_01::sync::oneshot::Sender<()>,
    timeout: u64,
    auto_approve: bool,
) -> Result<Node> {
    let account_address = AccountAddress::from_public_key(&keypair.public_key);
    let client = StarChainClient::new(
        &wallet_config.chain_address,
        wallet_config.chain_port as u32,
    );

    let client = Arc::new(client);
    let mut router = Router::new(client.clone(), executor.clone());
    router.start().unwrap();

    info!("account addr is {:?}", hex::encode(account_address));
    let mut wallet = Wallet::new_with_client(
        account_address,
        keypair.clone(),
        client,
        &wallet_config.store_dir,
    )
    .unwrap();

    wallet.start(&executor).unwrap();

    let f = async {
        let enabled: bool = wallet.is_channel_feature_enabled().await?;
        if !enabled {
            wallet.enable_channel().await?;
        }
        Ok::<_, Error>(())
    };
    rt.block_on(f)?;

    info!("account resource is {:?}", wallet.account_resource());
    Ok(Node::new(
        executor.clone(),
        wallet,
        network_service,
        sender,
        receiver,
        close_tx,
        auto_approve,
        timeout,
        router,
    ))
}

fn main() {
    let _g = libra_logger::set_default_global_logger(false /* async */, Some(25600));
    env_logger::init();

    let args = Args::from_args();
    let swarm = launch_swarm(&args).unwrap();

    info!("swarm is {:?}", swarm.config);
    let mut rt = Runtime::new().unwrap();
    let executor = rt.handle().clone();

    let keypair = Arc::new(load_from_keyfile(&args.faucet_key_path, args.child_num));
    let (network_service, tx, rx, close_tx) =
        build_network_service(&swarm.config.net_config, keypair.clone());

    let mut node = gen_node(
        &mut rt,
        executor.clone(),
        keypair,
        &swarm.config.wallet,
        network_service,
        tx,
        rx,
        close_tx,
        swarm.config.rpc_config.timeout,
        swarm.config.rpc_config.auto_approve,
    )
    .unwrap();

    node.start_server();
    let api_node = Arc::new(node);
    let mut node_server = setup_node_service(&swarm.config, api_node.clone());
    node_server.start();

    if args.start_client {
        let client = client::InteractiveClient::new_with_inherit_io(
            swarm.config.rpc_config.port, //Path::new(&faucet_key_file_path),
        );
        println!("Loading client...");
        let _output = client.output().expect("Failed to wait on child");
        println!("Exit client.");
    } else {
        let (tx, rx) = std::sync::mpsc::channel();
        ctrlc::set_handler(move || {
            tx.send(())
                .expect("failed to send unit when handling CTRL-C");
        })
        .expect("failed to set CTRL-C handler");
        println!("CTRL-C to exit.");
        rx.recv()
            .expect("failed to receive unit when handling CTRL-C");
    }
}
