
#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;

extern crate sp_runtime;

pub use sp_consensus_aura::sr25519::AuthorityId as AuraId;
pub use ares_oracle_provider_support::crypto::sr25519::AuthorityId as AresId;

use sp_runtime::{
    create_runtime_str, generic, impl_opaque_keys,
    traits::{AccountIdLookup, BlakeTwo256, Block as BlockT, ConvertInto},
    transaction_validity::{TransactionSource, TransactionValidity},
    ApplyExtrinsicResult, Percent,
};
use constants::currency::CurrencyBalance;

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic<TCall, TSignedExtra> = generic::UncheckedExtrinsic<Address, TCall, Signature, TSignedExtra>;
/// Extrinsic type that has already been checked.
pub type CheckedExtrinsic<TCall, TSignedExtra> = generic::CheckedExtrinsic<AccountId, TCall, TSignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive<TRuntime, TAllPallets, TCall, TSignedExtra> =
frame_executive::Executive<TRuntime, Block<TCall, TSignedExtra>, frame_system::ChainContext<TRuntime>, TRuntime, TAllPallets>;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = sp_runtime::MultiSignature;
/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId =
<<Signature as sp_runtime::traits::Verify>::Signer as sp_runtime::traits::IdentifyAccount>::AccountId;
/// Balance of an account.
// pub type Balance = u128;
pub type Balance = CurrencyBalance;
/// Index of a transaction in the chain.
pub type Index = u32;
/// A hash of some data.will.del used by the chain.
pub type Hash = sp_core::H256;
/// An index to a block.
pub type BlockNumber = u32;
/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block<TCall, TSignedExtra> = generic::Block<Header, UncheckedExtrinsic<TCall, TSignedExtra>>;
/// A Block signed with a Justification
pub type SignedBlock<TCall, TSignedExtra> = generic::SignedBlock<Block<TCall, TSignedExtra>>;
/// BlockId type as expected by this runtime.
pub type BlockId<TCall, TSignedExtra> = generic::BlockId<Block<TCall, TSignedExtra>>;

// pub mod opaque {
//     use super::*;
//     use sp_runtime::{generic, traits::BlakeTwo256};
//
//     pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;
//     /// Opaque block header type.
//     pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
//     /// Opaque block type.
//     pub type Block = generic::Block<Header, UncheckedExtrinsic>;
//     /// Opaque block identifier type.
//     pub type BlockId = generic::BlockId<Block>;
// }