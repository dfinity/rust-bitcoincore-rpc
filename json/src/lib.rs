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
        pub use dogecoin::script::{Address, AddressType};
        pub type AddressUnchecked = dogecoin::script::Address;
        pub use dogecoin::script::AddressParseError;
    }

    #[cfg(not(feature = "dogecoin"))]
    pub mod transaction {
        pub use bitcoin::transaction::Version;
    }

    #[cfg(feature = "dogecoin")]
    pub mod transaction {
        pub struct Version;
        impl Version {
            pub const ONE: u32 = 1;
        }
    }

    #[cfg(not(feature = "dogecoin"))]
    pub use bitcoin::{
        block::Header as BlockHeader, Block, BlockHash, Network, OutPoint, Transaction, TxIn,
        TxOut, Txid, Witness,
    };

    #[cfg(feature = "dogecoin")]
    pub use dogecoin::{
        block::{Block, BlockHash, BlockHeader},
        network::Network,
        transaction::{OutPoint, Transaction, TxIn, TxOut, Txid, Witness},
    };

    pub use bitcoin::{
        absolute::LockTime, amount, consensus, hashes, hex, secp256k1, sighash, Amount,
        CompressedPublicKey, PrivateKey, PublicKey, Script, ScriptBuf, Sequence, SignedAmount,
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
