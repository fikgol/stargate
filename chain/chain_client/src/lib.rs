use failure::prelude::*;
use types::{account_address::AccountAddress, access_path::AccessPath, transaction::SignedTransaction};
use types::proto::{transaction::SignedTransaction as SignedTransactionProto, access_path::AccessPath as AccessPathProto};
use core::borrow::Borrow;
use std::str::FromStr;
use crypto::HashValue;
use star_types::{proto::{star_account::AccountState, chain::{GetTransactionByHashRequest, WatchEventRequest, EventKey as EventKeyProto}}, channel::SgChannelStream};
use types::transaction::Version;
use star_types::{proto::{chain_grpc, chain::{FaucetRequest, LeastRootRequest, GetAccountStateWithProofByStateRootRequest, SubmitTransactionRequest, WatchTransactionRequest, WatchTransactionResponse}}, resource::Resource};
use grpcio::{Channel, EnvBuilder, ChannelBuilder};
use std::{sync::Arc, thread};
use proto_conv::IntoProto;
use futures::{Future, Stream};
use watch_transaction_stream::WatchTransactionStream;
use protobuf::RepeatedField;
use types::event::EventKey;
use proto_conv::FromProto;

pub mod watch_transaction_stream;

pub trait ChainClient {
    type WatchResp: Stream<Item=WatchTransactionResponse, Error=grpcio::Error>;

    fn least_state_root(&self) -> Result<HashValue>;
    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<Vec<u8>>>;
    fn get_state_by_access_path(&self, access_path: &AccessPath) -> Result<Option<Vec<u8>>>;
    fn faucet(&self, address: AccountAddress, amount: u64) -> Result<()>;
    fn submit_transaction(&self, signed_transaction: SignedTransaction) -> Result<()>;
    fn watch_transaction(&self, address: &AccountAddress, ver: Version) -> Result<WatchTransactionStream<Self::WatchResp>>;
    fn watch_event(&self, address: &AccountAddress, event_keys: Vec<EventKey>);
    fn get_transaction_by_hash(&self, hash: HashValue) -> Result<SignedTransaction>;
}

#[derive(Clone)]
pub struct RpcChainClient {
    conn_addr: String,
    client: chain_grpc::ChainClient,
}

impl RpcChainClient {
    pub fn new(host: &str, port: u32) -> RpcChainClient {
        let conn_addr = format!("{}:{}", host, port);

        // Create a GRPC client
        let env = Arc::new(EnvBuilder::new().name_prefix("grpc-client-").build());
        let ch = ChannelBuilder::new(env).connect(&conn_addr);
        Self {
            conn_addr,
            client: chain_grpc::ChainClient::new(ch),
        }
    }

    pub fn get_account_state_with_proof_by_state_root(&self, address: &AccountAddress, state_root_hash: HashValue) -> Result<Option<Vec<u8>>> {
        self.get_account_state(address)
    }
}

impl ChainClient for RpcChainClient {
    type WatchResp = grpcio::ClientSStreamReceiver<WatchTransactionResponse>;

    fn least_state_root(&self) -> Result<HashValue> {
        let req = LeastRootRequest::new();
        let resp = self.client.least_state_root(&req)?;
        HashValue::from_slice(resp.state_root_hash.as_slice())
    }

    fn get_account_state(&self, address: &AccountAddress) -> Result<Option<Vec<u8>>> {
        let mut req = GetAccountStateWithProofByStateRootRequest::new();
        req.set_address(address.to_vec());
        self.client.get_account_state_with_proof_by_state_root(&req).map_err(|e| {
            format_err!("{:?}", e)
        }).and_then(|resp| {
            let tmp = match resp.account_state_blob.into_option() {
                Some(blob) => {
                    Some(blob.blob)
                }
                _ => {
                    None
                }
            };
            Ok(tmp)
        })
    }

    fn get_state_by_access_path(&self, path: &AccessPath) -> Result<Option<Vec<u8>>> {
        let mut req = AccessPathProto::new();
        req.set_address(path.address.to_vec());
        req.set_path(path.path.to_vec());
        self.client.state_by_access_path(&req).map_err(|e| {
            format_err!("{:?}", e)
        }).and_then(|resp| {
            let a_r = resp.account_resource.into_option();
            let result = match a_r {
                Some(resource) => { Some(resource.resource) }
                None => { None }
            };
            Ok(result)
        })
    }

    fn faucet(&self, address: AccountAddress, amount: u64) -> Result<()> {
        let mut req = FaucetRequest::new();
        req.set_address(address.to_vec());
        req.set_amount(amount);
        let resp = self.client.faucet(&req)?;
        Ok(())
    }

    fn submit_transaction(&self, signed_transaction: SignedTransaction) -> Result<()> {
        let mut req = SubmitTransactionRequest::new();
        req.set_signed_txn(signed_transaction.into_proto());
        let resp = self.client.submit_transaction(&req);
        Ok(())
    }

    fn watch_transaction(&self, address: &AccountAddress, ver: Version) -> Result<WatchTransactionStream<Self::WatchResp>> {
        let watch_channel = ChannelBuilder::new(Arc::new(EnvBuilder::new().build())).connect(&self.conn_addr);
        let watch_client = chain_grpc::ChainClient::new(watch_channel);

        //        let print_data = move || {
        let mut req = WatchTransactionRequest::new();
        req.set_address(address.to_vec());
        let items_stream = watch_client.watch_transaction(&req).unwrap();
        Ok(WatchTransactionStream::new(items_stream))
        //            let f = items_stream.for_each(|item| {
        //                println!("received sign {:?}", item);
        //                Ok(())
        //            });
        //            f.wait().unwrap();
        //        };

        //thread::spawn(print_data);
    }

    fn watch_event(&self, address: &AccountAddress, event_keys: Vec<EventKey>) {
//        let keys = event_keys.iter().map(|key| -> EventKeyProto {
//            let mut event_key = EventKeyProto::new();
//            event_key.set_key(key.into_proto());
//            event_key
//        }).collect();
//        let mut req = WatchEventRequest::new();
//        req.set_address(address.to_vec());
//        req.set_keys(RepeatedField::from_vec(keys));
//        let event_stream = self.client.watch_event(&req).unwrap();
//        Ok(WatchTransactionStream::new(tx_stream))
        unimplemented!()
    }

    fn get_transaction_by_hash(&self, hash: HashValue) -> Result<SignedTransaction> {
        let mut req = GetTransactionByHashRequest::new();
        req.set_state_root_hash(hash.to_vec());
        let resp = self.client.get_transaction_by_hash(&req);
        match resp {
            Ok(tx) => { Ok(SignedTransaction::from_proto(tx.get_signed_tx().clone()).unwrap()) }
            Err(err) => { bail_err!(err) }
        }
    }
}
