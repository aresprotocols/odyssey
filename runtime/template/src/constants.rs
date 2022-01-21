// Copyright (C) 2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub mod currency {
	// use node_primitives::Balance;
	pub type Balance = u128;
	/// The existential deposit. Set to 1/10 of its parent Relay Chain (v9010).
	pub const EXISTENTIAL_DEPOSIT: Balance = 10 * CENTS;

	pub const UNITS: Balance = 10_000_000_000;
	pub const DOLLARS: Balance = UNITS;
	pub const CENTS: Balance = UNITS / 100;        // 100_000_000
	pub const MILLICENTS: Balance = CENTS / 1_000; // 100_000

	pub const fn deposit(items: u32, bytes: u32) -> Balance {
		// 1/10 of Polkadot v9010
		(items as Balance * 20 * DOLLARS + (bytes as Balance) * 100 * MILLICENTS) / 10
	}
}

pub mod types {
	use sp_runtime::traits::{Verify, IdentifyAccount, BlakeTwo256};

	/// An index to a block.
	pub type BlockNumber = u32;

	/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
	pub type Signature = sp_runtime::MultiSignature;

	/// Some way of identifying an account on the chain. We intentionally make it equivalent
	/// to the public key of our transaction signing scheme.
	pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	/// The type for looking up accounts. We don't expect more than 4 billion of them, but you
	/// never know...
	pub type AccountIndex = u32;

	/// Balance of an account.
	pub type Balance = u128;

	/// Index of a transaction in the chain.
	pub type Index = u32;

	/// A hash of some data used by the chain.
	pub type Hash = sp_core::H256;

	/// Block header type as expected by this runtime.
	pub type Header = sp_runtime::generic::Header<BlockNumber, BlakeTwo256>;

	/// Digest item type.
	pub type DigestItem = sp_runtime::generic::DigestItem<Hash>;

	// Aura consensus authority.
	pub type AuraId = sp_consensus_aura::sr25519::AuthorityId;
}

/// Common constants of statemint and statemine
pub mod constants {
	use super::types::BlockNumber;
	use sp_runtime::Perbill;
	use frame_support::weights::{Weight, constants::WEIGHT_PER_SECOND};
	/// This determines the average expected block time that we are targeting. Blocks will be
	/// produced at a minimum duration defined by `SLOT_DURATION`. `SLOT_DURATION` is picked up by
	/// `pallet_timestamp` which is in turn picked up by `pallet_aura` to implement `fn
	/// slot_duration()`.
	///
	/// Change this to adjust the block time.
	pub const MILLISECS_PER_BLOCK: u64 = 3000;
	pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

	// Time is measured by number of blocks.
	pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber);
	pub const HOURS: BlockNumber = MINUTES * 60;
	pub const DAYS: BlockNumber = HOURS * 24;

	/// We assume that ~5% of the block weight is consumed by `on_initialize` handlers. This is
	/// used to limit the maximal weight of a single extrinsic.
	pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(5);
	/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used by
	/// Operational  extrinsics.
	pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

	/// We allow for 0.5 seconds of compute with a 6 second average block time.
	pub const MAXIMUM_BLOCK_WEIGHT: Weight = WEIGHT_PER_SECOND / 2;
}



/// Fee-related.
pub mod fee {
	type Balance = u128;
	pub use sp_runtime::Perbill;
	use frame_support::weights::{
		constants::ExtrinsicBaseWeight, WeightToFeeCoefficient, WeightToFeeCoefficients,
		WeightToFeePolynomial,
	};
	use smallvec::smallvec;

	/// The block saturation level. Fees will be updates based on this value.
	pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

	/// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
	/// node's balance type.
	///
	/// This should typically create a mapping between the following ranges:
	///   - [0, MAXIMUM_BLOCK_WEIGHT]
	///   - [Balance::min, Balance::max]
	///
	/// Yet, it can be used for any other sort of change to weight-fee. Some examples being:
	///   - Setting it to `0` will essentially disable the weight fee.
	///   - Setting it to `1` will cause the literal `#[weight = x]` values to be charged.
	pub struct WeightToFee;
	impl WeightToFeePolynomial for WeightToFee {
		type Balance = Balance;
		fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
			// in Polkadot, extrinsic base weight (smallest non-zero weight) is mapped to 1/10 CENT:
			// in Statemint, we map to 1/10 of that, or 1/100 CENT
			let p = super::currency::CENTS;
			let q = 100 * Balance::from(ExtrinsicBaseWeight::get());
			smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
		}
	}
}
