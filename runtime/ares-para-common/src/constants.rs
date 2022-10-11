
use frame_support::{
    parameter_types,
    traits::{ConstU32, Currency, OneSessionHandler},
    weights::{constants::WEIGHT_PER_SECOND, Weight},
};
use frame_support::sp_runtime::Perbill;
use pallet_transaction_payment::{Multiplier, TargetedFeeAdjustment};
use crate::BlockNumber;
use sp_runtime::Perquintill;
use frame_system::limits;
use crate::sp_runtime::FixedPointNumber;

/// We assume that an on-initialize consumes 1% of the weight on average, hence a single extrinsic
/// will not be allowed to consume more than `AvailableBlockRatio - 1%`.
pub const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(1);
/// We allow `Normal` extrinsics to fill up the block up to 75%, the rest can be used
/// by  Operational  extrinsics.
pub const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);
/// We allow for 2 seconds of compute with a 6 second average block time.
pub const MAXIMUM_BLOCK_WEIGHT: Weight = 2 * WEIGHT_PER_SECOND;

// Common constants used in all runtimes.
parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	/// The portion of the `NORMAL_DISPATCH_RATIO` that we adjust the fees with. Blocks filled less
    /// than this will decrease the weight and more will increase.
	pub const TargetBlockFullness: Perquintill = Perquintill::from_percent(25);
	/// The adjustment variable of the runtime. Higher values will cause `TargetBlockFullness` to
    /// change the fees more rapidly.
	pub AdjustmentVariable: Multiplier = Multiplier::saturating_from_rational(75, 1000_000);
	/// Minimum amount of the multiplier. This value cannot be too low. A test case should ensure
    /// that combined with `AdjustmentVariable`, we can recover from the minimum.
    /// See `multiplier_can_grow_from_zero`.
	pub MinimumMultiplier: Multiplier = Multiplier::saturating_from_rational(1, 10u128);
	/// Maximum length of block. Up to 5MB.
	pub BlockLength: limits::BlockLength =
	limits::BlockLength::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
}


pub mod currency {
    // use node_primitives::Balance;
    pub type CurrencyBalance = u128;

    /// The existential deposit. Set to 1/10 of its parent Relay Chain.
    pub const EXISTENTIAL_DEPOSIT: CurrencyBalance = 10000 * AMAS_UNITS;

    pub const AMAS_UNITS: CurrencyBalance = 1_000_000_000_000; // 1  DOLLARS
    pub const AMAS_CENTS: CurrencyBalance = AMAS_UNITS / 100; // 0.01  CENTS
    pub const AMAS_MILLI_UNITS: CurrencyBalance = AMAS_UNITS / 1_000; // 0.001
    pub const AMAS_MILLI_CENTS: CurrencyBalance = AMAS_CENTS / 1_000; // 0.00001

    // pub const MILLICENTS: Balance = CENTS / 1_000; // 0.00001
    // pub const GRAND: Balance = CENTS * 100_000; // 1000

    // pub const fn deposit(items: u32, bytes: u32) -> Balance {
    // 	// 1/10 of Westend testnet
    // 	(items as Balance * 100 * CENTS + (bytes as Balance) * 5 * MILLICENTS) / 10
    //
    //  // (items as Balance * UNITS + (bytes as Balance) * 5 * MILLICENTS) / 10
    // }

    // TODO:: Check if it is right.
    pub const fn deposit(items: u32, bytes: u32) -> CurrencyBalance {
        items as CurrencyBalance * 15 * AMAS_CENTS + (bytes as CurrencyBalance) * 6 * AMAS_CENTS
    }
}

/// Time.
pub mod time {

    /// An index to a block.
    pub type BlockNumber = u32;
    /// Type used for expressing timestamp.
    pub type Moment = u64;

    /// Since BABE is probabilistic this is the average expected block time that
    /// we are targeting. Blocks will be produced at a minimum duration defined
    /// by `SLOT_DURATION`, but some slots will not be allocated to any
    /// authority and hence no block will be produced. We expect to have this
    /// block time on average following the defined slot duration and the value
    /// of `c` configured for BABE (where `1 - c` represents the probability of
    /// a slot being empty).
    /// This value is only used indirectly to define the unit constants below
    /// that are expressed in blocks. The rest of the code should use
    /// `SLOT_DURATION` instead (like the Timestamp pallet for calculating the
    /// minimum period).
    ///
    /// If using BABE with secondary slots (default) then all of the slots will
    /// always be assigned, in which case `MILLISECS_PER_BLOCK` and
    /// `SLOT_DURATION` should have the same value.
    ///
    /// <https://research.web3.foundation/en/latest/polkadot/block-production/Babe.html#-6.-practical-results>
    pub const MILLISECS_PER_BLOCK: Moment = 12000;
    pub const SECS_PER_BLOCK: Moment = MILLISECS_PER_BLOCK / 1000;

    // NOTE: Currently it is not possible to change the slot duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const SLOT_DURATION: Moment = MILLISECS_PER_BLOCK;

    // 1 in 4 blocks (on average, not counting collisions) will be primary BABE blocks.
    pub const PRIMARY_PROBABILITY: (u64, u64) = (1, 4);

    // NOTE: Currently it is not possible to change the epoch duration after the chain has started.
    //       Attempting to do so will brick block production.
    pub const EPOCH_DURATION_IN_BLOCKS: BlockNumber = 2 * MINUTES;
    pub const EPOCH_DURATION_IN_SLOTS: u64 = {
        const SLOT_FILL_RATE: f64 = MILLISECS_PER_BLOCK as f64 / SLOT_DURATION as f64;

        (EPOCH_DURATION_IN_BLOCKS as f64 * SLOT_FILL_RATE) as u64
    };

    // These time units are defined in number of blocks.
    pub const MINUTES: BlockNumber = 60 / (SECS_PER_BLOCK as BlockNumber);
    pub const HOURS: BlockNumber = MINUTES * 60;
    pub const DAYS: BlockNumber = HOURS * 24;
}

/// Fee-related.
pub mod fee {
    use frame_support::weights::{
        WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
    };
    use frame_support::weights::constants::ExtrinsicBaseWeight;
    // use primitives::v2::Balance;
    use smallvec::smallvec;
    pub use sp_runtime::Perbill;
    use crate::Balance;

    /// The block saturation level. Fees will be updates based on this value.
    pub const TARGET_BLOCK_FULLNESS: Perbill = Perbill::from_percent(25);

    /// Handles converting a weight scalar to a fee value, based on the scale and granularity of the
    /// node's balance type.
    ///
    /// This should typically create a mapping between the following ranges:
    ///   - [0, `MAXIMUM_BLOCK_WEIGHT`]
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
            let p = super::currency::AMAS_CENTS;
            let q = 10 * Balance::from(ExtrinsicBaseWeight::get());
            smallvec![WeightToFeeCoefficient {
				degree: 1,
				negative: false,
				coeff_frac: Perbill::from_rational(p % q, q),
				coeff_integer: p / q,
			}]
        }
    }
}