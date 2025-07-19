//! # rust-bitcoincore-rpc integration test
//!
//! The test methods are named to mention the methods tested.
//! Individual test methods don't use any methods not tested before or
//! mentioned in the test method name.
//!
//! The goal of this test is not to test the correctness of the server, but
//! to test the serialization of arguments and deserialization of responses.
//!

#![deny(unused)]

#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;

use dogecoincore_rpc::json;
use dogecoincore_rpc::json::import;
use dogecoincore_rpc::jsonrpc::error::Error as JsonRpcError;
use dogecoincore_rpc::{Auth, Client, Error, RpcApi};

use crate::json::BlockStatsFields as BsFields;
use dogecoincore_rpc::json::{GetBlockTemplateModes, GetBlockTemplateRules};
use import::consensus::encode::{deserialize, serialize_hex};
use import::hashes::hex::FromHex;
use import::hashes::Hash;
use import::{
    address::{Address, AddressType},
    secp256k1, Amount, Network, OutPoint, SignedAmount, Txid,
};

lazy_static! {
    static ref SECP: secp256k1::Secp256k1<secp256k1::All> = secp256k1::Secp256k1::new();
    /// A random address not owned by the node.
    static ref RANDOM_ADDRESS: Address = Address::p2pkh("162c5ea71c0b23f5b9022ef047c4a86470a5b070".parse::<import::PubkeyHash>().unwrap(), Network::Regtest);
    /// The default fee amount to use when needed.
    static ref FEE: Amount = Amount::from_btc(0.001).unwrap();
}

struct StdLogger;

impl log::Log for StdLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.target().contains("jsonrpc") || metadata.target().contains("bitcoincore_rpc")
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            println!("[{}][{}]: {}", record.level(), record.metadata().target(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: StdLogger = StdLogger;

#[allow(unused)]
/// Assert that the call returns a "method not found" error.
macro_rules! assert_not_found {
    ($call:expr) => {
        match $call.unwrap_err() {
            Error::JsonRpc(JsonRpcError::Rpc(ref e)) if e.code == -32601 => {}
            e => panic!("expected method not found error for {}, got: {}", stringify!($call), e),
        }
    };
}

/// Assert that the call returns the specified error message.
macro_rules! assert_error_message {
    ($call:expr, $code:expr, $msg:expr) => {
        match $call.unwrap_err() {
            Error::JsonRpc(JsonRpcError::Rpc(ref e))
                if e.code == $code && e.message.contains($msg) => {}
            e => panic!("expected '{}' error for {}, got: {}", $msg, stringify!($call), e),
        }
    };
}

static mut VERSION: usize = 0;
/// Get the version of the node that is running.
fn version() -> usize {
    unsafe { VERSION }
}

/// Quickly create a BTC amount.
fn btc<F: Into<f64>>(btc: F) -> Amount {
    Amount::from_btc(btc.into()).unwrap()
}
/// Quickly create a signed BTC amount.
fn sbtc<F: Into<f64>>(btc: F) -> SignedAmount {
    SignedAmount::from_btc(btc.into()).unwrap()
}

fn get_rpc_url() -> String {
    return std::env::var("RPC_URL").expect("RPC_URL must be set");
}

fn get_auth() -> dogecoincore_rpc::Auth {
    if let Ok(cookie) = std::env::var("RPC_COOKIE") {
        return Auth::CookieFile(cookie.into());
    } else if let Ok(user) = std::env::var("RPC_USER") {
        return Auth::UserPass(user, std::env::var("RPC_PASS").unwrap_or_default());
    } else {
        panic!("Either RPC_COOKIE or RPC_USER + RPC_PASS must be set.");
    };
}

#[allow(unused)]
fn new_wallet_client(wallet_name: &str) -> Client {
    let url = get_rpc_url();
    Client::new(Network::Regtest, &url, get_auth()).unwrap()
}

fn main() {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::max())).unwrap();

    let cl = new_wallet_client("testwallet");

    test_get_network_info(&cl);
    unsafe { VERSION = cl.version().unwrap() };
    println!("Version: {}", version());

    test_get_mining_info(&cl);
    test_get_blockchain_info(&cl);
    test_get_new_address(&cl);
    test_get_raw_change_address(&cl);
    test_generate(&cl);
    test_get_balance_generate_to_address(&cl);
    test_get_best_block_hash(&cl);
    test_get_block_count(&cl);
    test_get_block_hash(&cl);
    test_get_block(&cl);
    test_get_block_header_get_block_header_info(&cl);
    test_get_block_stats(&cl);
    test_get_block_stats_fields(&cl);
    test_send_to_address(&cl);
    test_get_received_by_address(&cl);
    test_list_unspent(&cl);
    test_get_difficulty(&cl);
    test_get_connection_count(&cl);
    test_get_raw_transaction(&cl);
    test_get_raw_mempool(&cl);
    test_get_raw_mempool_verbose(&cl);
    test_get_transaction(&cl);
    test_list_transactions(&cl);
    test_list_since_block(&cl);
    test_get_tx_out(&cl);
    test_get_tx_out_proof(&cl);
    test_get_mempool_entry(&cl);
    test_lock_unspent_unlock_unspent(&cl);
    test_invalidate_block_reconsider_block(&cl);
    test_key_pool_refill(&cl);
    test_create_raw_transaction(&cl);
    test_decode_raw_transaction(&cl);
    test_fund_raw_transaction(&cl);
    test_list_received_by_address(&cl);
    test_estimate_smart_fee(&cl);
    test_ping(&cl);
    test_get_peer_info(&cl);
    test_get_tx_out_set_info(&cl);
    test_get_chain_tips(&cl);
    test_get_net_totals(&cl);
    test_getblocktemplate(&cl);
    test_get_mempool_info(&cl);
    // test_add_multisig_address(&cl);
    //TODO import_multi(
    //TODO verify_message(
    //TODO encrypt_wallet(&self, passphrase: &str) -> Result<()> {
    //TODO get_by_id<T: queryable::Queryable<Self>>(
    test_add_node(&cl);
    test_get_added_node_info(&cl);
    test_disconnect_node(&cl);
    test_set_network_active(&cl);
    test_stop(cl);
}

fn test_get_network_info(cl: &Client) {
    let _ = cl.get_network_info().unwrap();
}

fn test_get_mining_info(cl: &Client) {
    let _ = cl.get_mining_info().unwrap();
}

fn test_get_blockchain_info(cl: &Client) {
    let info = cl.get_blockchain_info().unwrap();
    assert_eq!(info.chain, Network::Regtest);
}

fn test_get_new_address(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    assert_eq!(addr.address_type(), Some(AddressType::P2pkh));

    let addr = cl.get_new_address(None, Some(json::AddressType::Legacy)).unwrap();
    assert_eq!(addr.address_type(), Some(AddressType::P2pkh));
}

fn test_get_raw_change_address(cl: &Client) {
    let addr = cl.get_raw_change_address(Some(json::AddressType::Legacy)).unwrap();
    assert_eq!(addr.address_type(), Some(AddressType::P2pkh));
}

fn test_generate(cl: &Client) {
    let blocks = cl.generate(4, None).unwrap();
    assert_eq!(blocks.len(), 4);
    let blocks = cl.generate(6, Some(45)).unwrap();
    assert_eq!(blocks.len(), 6);
}

fn test_get_balance_generate_to_address(cl: &Client) {
    let initial = cl.get_balance(None, None).unwrap();

    let blocks = cl.generate_to_address(500, &cl.get_new_address(None, None).unwrap()).unwrap();
    assert_eq!(blocks.len(), 500);
    assert_ne!(cl.get_balance(None, None).unwrap(), initial);
}

fn test_get_best_block_hash(cl: &Client) {
    let _ = cl.get_best_block_hash().unwrap();
}

fn test_get_block_count(cl: &Client) {
    let height = cl.get_block_count().unwrap();
    assert!(height > 0);
}

fn test_get_block_hash(cl: &Client) {
    let h = cl.get_block_count().unwrap();
    assert_eq!(cl.get_block_hash(h).unwrap(), cl.get_best_block_hash().unwrap());
}

fn test_get_block(cl: &Client) {
    let tip = cl.get_best_block_hash().unwrap();
    let block = cl.get_block(&tip).unwrap();
    let hex = cl.get_block_hex(&tip).unwrap();
    assert_eq!(block, deserialize(&Vec::<u8>::from_hex(&hex).unwrap()).unwrap());
    assert_eq!(hex, serialize_hex(&block));

    let tip = cl.get_best_block_hash().unwrap();
    let info = cl.get_block_info(&tip).unwrap();
    assert_eq!(info.hash, tip);
    assert_eq!(info.confirmations, 1);
}

fn test_get_block_header_get_block_header_info(cl: &Client) {
    let tip = cl.get_best_block_hash().unwrap();
    let header = cl.get_block_header(&tip).unwrap();
    let info = cl.get_block_header_info(&tip).unwrap();
    assert_eq!(header.block_hash(), info.hash);
    assert_eq!(header.version, info.version);
    assert_eq!(header.merkle_root, info.merkle_root);
    assert_eq!(info.confirmations, 1);
    assert_eq!(info.next_block_hash, None);
    assert!(info.previous_block_hash.is_some());
}

fn test_get_block_stats(cl: &Client) {
    let tip = cl.get_block_count().unwrap();
    let tip_hash = cl.get_best_block_hash().unwrap();
    let header = cl.get_block_header(&tip_hash).unwrap();
    let stats = cl.get_block_stats(tip_hash).unwrap();
    assert_eq!(header.block_hash(), stats.block_hash);
    assert_eq!(header.time, stats.time as u32);
    assert_eq!(tip, stats.height);
}

fn test_get_block_stats_fields(cl: &Client) {
    let tip = cl.get_block_count().unwrap();
    let tip_hash = cl.get_best_block_hash().unwrap();
    let header = cl.get_block_header(&tip_hash).unwrap();
    let fields = [BsFields::BlockHash, BsFields::Height, BsFields::TotalFee];
    let stats = cl.get_block_stats_fields(tip_hash, &fields).unwrap();
    assert_eq!(header.block_hash(), stats.block_hash.unwrap());
    assert_eq!(tip, stats.height.unwrap());
    assert!(stats.total_fee.is_some());
    assert!(stats.avg_fee.is_none());
}

fn test_send_to_address(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), Some("cc"), None, None, None, None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, Some("tt"), None, None, None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, None, Some(true), None, None, None).unwrap();
}

fn test_get_received_by_address(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let _ = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();
    assert_eq!(cl.get_received_by_address(&addr, Some(0)).unwrap(), btc(1));
    assert_eq!(cl.get_received_by_address(&addr, Some(1)).unwrap(), btc(0));
    let _ = cl.generate_to_address(7, &addr).unwrap();
    let _ = cl.generate_to_address(100, &cl.get_new_address(None, None).unwrap()).unwrap();
    assert_eq!(cl.get_received_by_address(&addr, Some(6)).unwrap(), btc(1));
    assert_eq!(cl.get_received_by_address(&addr, None).unwrap(), btc(1));
    println!("test_get_received_by_addrss {}", addr);
    panic!("err");
}

fn test_list_unspent(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let addr_checked = addr.clone();
    let txid =
        cl.send_to_address(&addr.clone(), btc(1), None, None, None, None, None, None).unwrap();
    let unspent = cl.list_unspent(Some(0), None, Some(&[&addr_checked]), None, None).unwrap();
    assert_eq!(unspent[0].txid, txid);
    assert_eq!(
        unspent[0].address.as_ref().map(|addr| addr.clone().assume_checked()),
        Some(addr.clone())
    );
    assert_eq!(unspent[0].amount, btc(1));

    let txid =
        cl.send_to_address(&addr_checked, btc(7), None, None, None, None, None, None).unwrap();
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(7)),
        maximum_amount: Some(btc(7)),
        ..Default::default()
    };
    let unspent =
        cl.list_unspent(Some(0), None, Some(&[&addr_checked]), None, Some(options)).unwrap();
    assert_eq!(unspent.len(), 1);
    assert_eq!(unspent[0].txid, txid);
    assert_eq!(
        unspent[0].address.as_ref().map(|addr| addr.clone().assume_checked()),
        Some(addr.clone())
    );
    assert_eq!(unspent[0].amount, btc(7));
}

fn test_get_difficulty(cl: &Client) {
    let _ = cl.get_difficulty().unwrap();
}

fn test_get_connection_count(cl: &Client) {
    let _ = cl.get_connection_count().unwrap();
}

fn test_get_raw_transaction(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let txid = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();
    let tx = cl.get_raw_transaction(&txid, None).unwrap();
    let hex = cl.get_raw_transaction_hex(&txid, None).unwrap();
    assert_eq!(tx, deserialize(&Vec::<u8>::from_hex(&hex).unwrap()).unwrap());
    assert_eq!(hex, serialize_hex(&tx));

    let info = cl.get_raw_transaction_info(&txid, None).unwrap();
    assert_eq!(info.txid, txid);
}

fn test_get_raw_mempool(cl: &Client) {
    let _ = cl.get_raw_mempool().unwrap();
}

fn test_get_raw_mempool_verbose(cl: &Client) {
    cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let _ = cl.get_raw_mempool_verbose().unwrap();

    // cleanup mempool transaction
    cl.generate_to_address(2, &RANDOM_ADDRESS).unwrap();
}

fn test_get_transaction(cl: &Client) {
    let txid =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let tx = cl.get_transaction(&txid, None).unwrap();
    assert_eq!(tx.amount, sbtc(-1.0));
    assert_eq!(tx.info.txid, txid);

    let fake = Txid::hash(&[1, 2]);
    assert!(cl.get_transaction(&fake, Some(true)).is_err());
}

fn test_list_transactions(cl: &Client) {
    let _ = cl.list_transactions(None, None, None, None).unwrap();
    let _ = cl.list_transactions(Some("l"), None, None, None).unwrap();
    let _ = cl.list_transactions(None, Some(3), None, None).unwrap();
    let _ = cl.list_transactions(None, None, Some(3), None).unwrap();
    let _ = cl.list_transactions(None, None, None, Some(true)).unwrap();
}

fn test_list_since_block(cl: &Client) {
    let r = cl.list_since_block(None, None, None, None).unwrap();
    assert_eq!(r.lastblock, cl.get_best_block_hash().unwrap());
    assert!(!r.transactions.is_empty());
}

fn test_get_tx_out(cl: &Client) {
    let txid =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let out = cl.get_tx_out(&txid, 0, Some(false)).unwrap();
    assert!(out.is_none());
    let out = cl.get_tx_out(&txid, 0, Some(true)).unwrap();
    assert!(out.is_some());
    let _ = cl.get_tx_out(&txid, 0, None).unwrap();
}

fn test_get_tx_out_proof(cl: &Client) {
    let txid1 =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let txid2 =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let blocks = cl.generate_to_address(7, &cl.get_new_address(None, None).unwrap()).unwrap();
    let proof = cl.get_tx_out_proof(&[txid1, txid2], Some(&blocks[0])).unwrap();
    assert!(!proof.is_empty());
}

#[allow(unused)]
fn test_get_mempool_entry(cl: &Client) {
    let txid =
        cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();
    let entry = cl.get_mempool_entry(&txid).unwrap();
    let fake = Txid::hash(&[1, 2]);
    assert!(cl.get_mempool_entry(&fake).is_err());
}

fn test_lock_unspent_unlock_unspent(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let txid = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();

    assert!(cl.lock_unspent(&[OutPoint::new(txid, 0)]).unwrap());
    assert!(cl.unlock_unspent(&[OutPoint::new(txid, 0)]).unwrap());

    assert!(cl.lock_unspent(&[OutPoint::new(txid, 0)]).unwrap());
    assert!(cl.unlock_unspent_all().unwrap());
}

fn test_invalidate_block_reconsider_block(cl: &Client) {
    let hash = cl.get_best_block_hash().unwrap();
    cl.invalidate_block(&hash).unwrap();
    cl.reconsider_block(&hash).unwrap();
}

fn test_key_pool_refill(cl: &Client) {
    cl.key_pool_refill(Some(100)).unwrap();
    cl.key_pool_refill(None).unwrap();
}

fn test_create_raw_transaction(cl: &Client) {
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(2)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(6), None, None, None, Some(options)).unwrap();
    let unspent = unspent.into_iter().nth(0).unwrap();

    let input = json::CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: None,
    };
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), btc(1));

    let tx =
        cl.create_raw_transaction(&[input.clone()], &output, Some(500_000), Some(true)).unwrap();
    let hex = cl.create_raw_transaction_hex(&[input], &output, Some(500_000), Some(true)).unwrap();
    assert_eq!(tx, deserialize(&Vec::<u8>::from_hex(&hex).unwrap()).unwrap());
    assert_eq!(hex, serialize_hex(&tx));
}

fn test_decode_raw_transaction(cl: &Client) {
    let options = json::ListUnspentQueryOptions {
        minimum_amount: Some(btc(2)),
        ..Default::default()
    };
    let unspent = cl.list_unspent(Some(6), None, None, None, Some(options)).unwrap();
    let unspent = unspent.into_iter().nth(0).unwrap();

    let input = json::CreateRawTransactionInput {
        txid: unspent.txid,
        vout: unspent.vout,
        sequence: None,
    };
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), btc(1));

    let tx =
        cl.create_raw_transaction(&[input.clone()], &output, Some(500_000), Some(true)).unwrap();
    let hex = cl.create_raw_transaction_hex(&[input], &output, Some(500_000), Some(true)).unwrap();

    let decoded_transaction = cl.decode_raw_transaction(hex, None).unwrap();

    assert_eq!(tx.compute_txid(), decoded_transaction.txid);
    assert_eq!(500_000, decoded_transaction.locktime);

    assert_eq!(decoded_transaction.vin[0].txid.unwrap(), unspent.txid);
    assert_eq!(decoded_transaction.vout[0].clone().value, btc(1));
}

fn test_fund_raw_transaction(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let mut output = HashMap::new();
    output.insert(RANDOM_ADDRESS.to_string(), btc(1));

    let options = json::FundRawTransactionOptions {
        add_inputs: None,
        change_address: Some(addr),
        change_position: Some(0),
        change_type: None,
        include_watching: Some(true),
        lock_unspents: Some(true),
        fee_rate: Some(*FEE),
        subtract_fee_from_outputs: Some(vec![0]),
        conf_target: None,
        estimate_mode: None,
    };
    let tx = cl.create_raw_transaction_hex(&[], &output, Some(500_000), Some(true)).unwrap();
    let funded = cl.fund_raw_transaction(tx, Some(&options), Some(false)).unwrap();
    let _ = funded.transaction().unwrap();

    let options = json::FundRawTransactionOptions {
        add_inputs: None,
        change_address: None,
        change_position: Some(0),
        change_type: None,
        include_watching: Some(true),
        lock_unspents: Some(true),
        fee_rate: None,
        subtract_fee_from_outputs: Some(vec![0]),
        conf_target: None,
        estimate_mode: None,
    };
    let tx = cl.create_raw_transaction_hex(&[], &output, Some(500_000), Some(true)).unwrap();
    let funded = cl.fund_raw_transaction(tx, Some(&options), Some(false)).unwrap();
    let _ = funded.transaction().unwrap();
}

fn test_list_received_by_address(cl: &Client) {
    let addr = cl.get_new_address(None, None).unwrap();
    let txid = cl.send_to_address(&addr, btc(1), None, None, None, None, None, None).unwrap();

    let _ = cl.list_received_by_address(None, Some(200), None, None).unwrap();

    let res = cl.list_received_by_address(None, Some(0), None, None).unwrap();
    assert_eq!(
        res.iter().find(|x| x.address.clone().assume_checked() == addr).map(|x| x.txids.clone()),
        Some(vec![txid])
    );
}

fn test_estimate_smart_fee(cl: &Client) {
    // let mode = json::EstimateMode::Unset;
    let res = cl.estimate_smart_fee(3, None).unwrap();

    // With a fresh node, we can't get fee estimates.
    if let Some(errors) = res.errors {
        if errors == &["Insufficient data or no feerate found"] {
            println!("Cannot test estimate_smart_fee because no feerate found!");
            return;
        } else {
            panic!("Unexpected error(s) for estimate_smart_fee: {:?}", errors);
        }
    }

    assert!(res.fee_rate.is_some(), "no fee estimate available: {:?}", res.errors);
    assert!(res.fee_rate.unwrap() >= btc(0));
}

fn test_ping(cl: &Client) {
    let _ = cl.ping().unwrap();
}

fn test_get_peer_info(cl: &Client) {
    let info = cl.get_peer_info().unwrap();
    if info.is_empty() {
        panic!("No peers are connected so we can't test get_peer_info");
    }
}

fn test_get_tx_out_set_info(cl: &Client) {
    cl.get_tx_out_set_info(None, None, None).unwrap();
}

fn test_get_chain_tips(cl: &Client) {
    let tips = cl.get_chain_tips().unwrap();
    assert_eq!(tips.len(), 1);
}

fn test_add_node(cl: &Client) {
    cl.add_node("127.0.0.1:1234").unwrap();
    assert_error_message!(cl.add_node("127.0.0.1:1234"), -23, "Error: Unable to add node");
    cl.remove_node("127.0.0.1:1234").unwrap();
    cl.onetry_node("127.0.0.1:1234").unwrap();
}

fn test_get_added_node_info(cl: &Client) {
    cl.add_node("127.0.0.1:1234").unwrap();
    cl.add_node("127.0.0.1:4321").unwrap();

    assert!(cl.get_added_node_info(Some("127.0.0.1:1111")).is_err());
    assert_eq!(cl.get_added_node_info(None).unwrap().len(), 2);
    assert_eq!(cl.get_added_node_info(Some("127.0.0.1:1234")).unwrap().len(), 1);
    assert_eq!(cl.get_added_node_info(Some("127.0.0.1:4321")).unwrap().len(), 1);
}

fn test_disconnect_node(cl: &Client) {
    assert_error_message!(
        cl.disconnect_node("127.0.0.1:1234"),
        -29,
        "Node not found in connected nodes"
    );
}

fn test_set_network_active(cl: &Client) {
    cl.set_network_active(false).unwrap();
    cl.set_network_active(true).unwrap();
}

fn test_get_net_totals(cl: &Client) {
    cl.get_net_totals().unwrap();
}

fn test_getblocktemplate(cl: &Client) {
    // We want to have a transaction in the mempool so the GetBlockTemplateResult
    // contains an entry in the vector of GetBlockTemplateResultTransaction.
    // Otherwise the GetBlockTemplateResultTransaction deserialization wouldn't
    // be tested.
    cl.send_to_address(&RANDOM_ADDRESS, btc(1), None, None, None, None, None, None).unwrap();

    cl.get_block_template(GetBlockTemplateModes::Template, &[GetBlockTemplateRules::SegWit], &[])
        .unwrap();

    // cleanup mempool transaction
    cl.generate_to_address(2, &RANDOM_ADDRESS).unwrap();
}

#[allow(unused)]
// Error: "Only legacy wallets are supported by this command"
fn test_add_multisig_address(cl: &Client) {
    let addr1 = cl.get_new_address(None, Some(json::AddressType::Bech32)).unwrap();
    let addr2 = cl.get_new_address(None, Some(json::AddressType::Bech32)).unwrap();
    let addresses =
        [json::PubKeyOrAddress::Address(&addr1), json::PubKeyOrAddress::Address(&addr2)];

    assert!(cl.add_multisig_address(addresses.len(), &addresses, None, None).is_ok());
    assert!(cl.add_multisig_address(addresses.len() - 1, &addresses, None, None).is_ok());
    assert!(cl.add_multisig_address(addresses.len() + 1, &addresses, None, None).is_err());
    assert!(cl.add_multisig_address(0, &addresses, None, None).is_err());
    assert!(cl.add_multisig_address(addresses.len(), &addresses, Some("test_label"), None).is_ok());
    assert!(cl
        .add_multisig_address(addresses.len(), &addresses, None, Some(json::AddressType::Legacy))
        .is_ok());
    assert!(cl
        .add_multisig_address(
            addresses.len(),
            &addresses,
            None,
            Some(json::AddressType::P2shSegwit)
        )
        .is_ok());
    assert!(cl
        .add_multisig_address(addresses.len(), &addresses, None, Some(json::AddressType::Bech32))
        .is_ok());
}

fn test_get_mempool_info(cl: &Client) {
    let res = cl.get_mempool_info().unwrap();

    assert!(res.loaded.is_none());
    assert!(res.unbroadcast_count.is_none());
    assert!(res.total_fee.is_none());
    assert!(res.incremental_relay_fee.is_none());
    assert!(res.full_rbf.is_none());
}

fn test_stop(cl: Client) {
    println!("Stopping: '{}'", cl.stop().unwrap());
}
