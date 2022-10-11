
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

// ------

// Common constants used in all runtimes.
// parameter_types! {
// 	pub const BlockHashCount: BlockNumber = 2400;
// 	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
//     /// than this will decrease the weight and more will increase.
// 	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
// 	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
//     /// change the fees more rapidly.
// 	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(3, 100_000);
// 	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
//     /// that combined with `AdjustmentVariable`, we can recover from the minimum.
//     /// See `multiplier_can_grow_from_zero`.
// 	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 1_000_000u128);
// 	/// Maximum length of block. Up to 5MB.
// 	pub BlockLength: limits::BlockLength =
// 		limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
// 	/// Block weights base values and limits.
// 	pub BlockWeights: limits::BlockWeights = limits::BlockWeights::builder()
// 		.base_block(BlockExecutionWeight::get())
// 		.for_class(DispatchClass::all(), |weights| {
// 			weights.base_extrinsic = ExtrinsicBaseWeight::get();
// 		})
// 		.for_class(DispatchClass::Normal, |weights| {
// 			weights.max_total = Some(NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT);
// 		})
// 		.for_class(DispatchClass::Operational, |weights| {
// 			weights.max_total = Some(MAXIMUM_BLOCK_WEIGHT);
// 			// Operational transactions have an extra reserved space, so that they
// 			// are included even if block reached `MAXIMUM_BLOCK_WEIGHT`.
// 			weights.reserved = Some(
// 				MAXIMUM_BLOCK_WEIGHT - NORMAL_DISPATCH_RATIO * MAXIMUM_BLOCK_WEIGHT,
// 			);
// 		})
// 		.avg_block_initialization(AVERAGE_ON_INITIALIZE_RATIO)
// 		.build_or_panic();
// }
//
// /// A source of random balance for the NPoS Solver, which is meant to be run by the off-chain worker
// /// election miner.
// pub struct OffchainRandomBalancing;
// impl Get<Option<BalancingConfig>> for OffchainRandomBalancing {
//     fn get() -> Option<BalancingConfig> {
//         use sp_runtime::traits::TrailingZeroInput;
//         let iterations = match MINER_MAX_ITERATIONS {
//             0 => 0,
//             max => {
//                 let seed = sp_io::offchain::random_seed();
//                 let random = <u32>::decode(&mut TrailingZeroInput::new(&seed))
//                     .expect("input is padded with zeroes; qed") %
//                     max.saturating_add(1);
//                 random as usize
//             },
//         };
//
//         let config = BalancingConfig { iterations, tolerance: 0 };
//         Some(config)
//     }
// }
//
// /// Logic for the author to get a portion of fees.
// pub struct ToAuthor<R>(sp_std::marker::PhantomData<R>);
// impl<R> OnUnbalanced<NegativeImbalance<R>> for ToAuthor<R>
//     where
//         R: pallet_balances::Config + pallet_authorship::Config,
//         <R as frame_system::Config>::AccountId: From<AccountId>,
//         <R as frame_system::Config>::AccountId: Into<AccountId>,
//         <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
// {
//     fn on_nonzero_unbalanced(amount: NegativeImbalance<R>) {
//         if let Some(author) = <pallet_authorship::Pallet<R>>::author() {
//             <pallet_balances::Pallet<R>>::resolve_creating(&author, amount);
//         }
//     }
// }
//
// pub struct DealWithFees<R>(sp_std::marker::PhantomData<R>);
// impl<R> OnUnbalanced<NegativeImbalance<R>> for DealWithFees<R>
//     where
//         R: pallet_balances::Config + pallet_treasury::Config + pallet_authorship::Config,
//         pallet_treasury::Pallet<R>: OnUnbalanced<NegativeImbalance<R>>,
//         <R as frame_system::Config>::AccountId: From<AccountId>,
//         <R as frame_system::Config>::AccountId: Into<AccountId>,
//         <R as frame_system::Config>::Event: From<pallet_balances::Event<R>>,
// {
//     fn on_unbalanceds<B>(mut fees_then_tips: impl Iterator<Item = NegativeImbalance<R>>) {
//         if let Some(fees) = fees_then_tips.next() {
//             // for fees, 80% to treasury, 20% to author
//             let mut split = fees.ration(80, 20);
//             if let Some(tips) = fees_then_tips.next() {
//                 // for tips, if any, 100% to author
//                 tips.merge_into(&mut split.1);
//             }
//             use pallet_treasury::Pallet as Treasury;
//             <Treasury<R> as OnUnbalanced<_>>::on_unbalanced(split.0);
//             <ToAuthor<R> as OnUnbalanced<_>>::on_unbalanced(split.1);
//         }
//     }
// }
