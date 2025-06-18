//! # Rust Client for Bitcoin Core API
//!
//! This is a client library for the Bitcoin Core JSON-RPC API.
//!

#![crate_name = "bitcoincore_rpc_json"]
#![crate_type = "rlib"]

#[cfg(not(feature = "dogecoin"))]
mod bitcoin;

#[cfg(feature = "dogecoin")]
mod dogecoin;

#[cfg(not(feature = "dogecoin"))]
pub use crate::bitcoin::*;

#[cfg(feature = "dogecoin")]
pub use crate::dogecoin::*;

pub mod import {
    #[cfg(not(feature = "dogecoin"))]
    pub mod address {
        pub use bitcoin::address::AddressType;
        pub type Address = bitcoin::address::Address<bitcoin::address::NetworkChecked>;
        pub type AddressUnchecked = bitcoin::address::Address<bitcoin::address::NetworkUnchecked>;
        pub type AddressParseError = bitcoin::address::ParseError;
    }

    #[cfg(feature = "dogecoin")]
    pub mod address {
        pub use bitcoin::address::AddressType;
        pub type Address = bitcoin::dogecoin::address::Address<bitcoin::address::NetworkChecked>;
        pub type AddressUnchecked =
            bitcoin::dogecoin::address::Address<bitcoin::address::NetworkUnchecked>;
        pub type AddressParseError = bitcoin::dogecoin::address::ParseError;
    }

    pub mod transaction {
        pub use bitcoin::transaction::Version;
    }

    #[cfg(not(feature = "dogecoin"))]
    pub use bitcoin::{Block, Network};

    #[cfg(feature = "dogecoin")]
    pub use bitcoin::dogecoin::{Block, Network};

    pub use bitcoin::{
        absolute::LockTime, amount, block::Header as BlockHeader, consensus, hashes, hex,
        secp256k1, sighash, Amount, BlockHash, CompressedPublicKey, OutPoint, PrivateKey,
        PubkeyHash, PublicKey, Script, ScriptBuf, Sequence, SignedAmount, Transaction, TxIn,
        TxMerkleNode, TxOut, Txid, Witness, block::Version
    };
}

extern crate serde;
extern crate serde_json;

/// A module used for serde serialization of bytes in hexadecimal format.
///
/// The module is compatible with the serde attribute.
pub mod serde_hex {
    pub use bitcoin::hex;

    use hex::{DisplayHex, FromHex};
    use serde::de::Error;
    use serde::{Deserializer, Serializer};

    pub fn serialize<S: Serializer>(b: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&b.to_lower_hex_string())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let hex_str: String = ::serde::Deserialize::deserialize(d)?;
        Ok(FromHex::from_hex(&hex_str).map_err(D::Error::custom)?)
    }

    pub mod opt {
        use super::*;

        pub fn serialize<S: Serializer>(b: &Option<Vec<u8>>, s: S) -> Result<S::Ok, S::Error> {
            match *b {
                None => s.serialize_none(),
                Some(ref b) => s.serialize_str(&b.to_lower_hex_string()),
            }
        }

        pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Vec<u8>>, D::Error> {
            let hex_str: String = ::serde::Deserialize::deserialize(d)?;
            Ok(Some(FromHex::from_hex(&hex_str).map_err(D::Error::custom)?))
        }
    }
}
