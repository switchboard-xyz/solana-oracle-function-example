use crate::*;
use bytemuck::{Pod, Zeroable};
use switchboard_solana::error::SwitchboardError;
use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use rust_decimal::Decimal;
use std::cell::Ref;
// #![allow(unaligned_references)]
use core::cmp::Ordering;
use rust_decimal::prelude::*;
use std::convert::{From, TryInto};

// #[derive(Default, Eq, PartialEq, Copy, Clone, AnchorSerialize, AnchorDeserialize)]
// pub struct BorshDecimal {
//     pub mantissa: i128,
//     pub scale: u32,
// }
// impl From<SwitchboardDecimal> for BorshDecimal {
//     fn from(s: SwitchboardDecimal) -> Self {
//         Self {
//             mantissa: s.mantissa,
//             scale: s.scale,
//         }
//     }
// }
// impl From<BorshDecimal> for SwitchboardDecimal {
//     fn from(val: BorshDecimal) -> Self {
//         SwitchboardDecimal {
//             mantissa: val.mantissa,
//             scale: val.scale,
//         }
//     }
// }

#[zero_copy(unsafe)]
#[repr(packed)]
#[derive(Default, Debug, Eq, PartialEq, AnchorDeserialize)]
pub struct SwitchboardDecimal {
    /// The part of a floating-point number that represents the significant digits of that number, and that is multiplied by the base, 10, raised to the power of scale to give the actual value of the number.
    pub mantissa: i128,
    /// The number of decimal places to move to the left to yield the actual value.
    pub scale: u32,
}

impl SwitchboardDecimal {
    pub fn new(mantissa: i128, scale: u32) -> SwitchboardDecimal {
        Self { mantissa, scale }
    }
    pub fn from_rust_decimal(d: Decimal) -> SwitchboardDecimal {
        Self::new(d.mantissa(), d.scale())
    }
    pub fn from_f64(v: f64) -> SwitchboardDecimal {
        let dec = Decimal::from_f64(v).unwrap();
        Self::from_rust_decimal(dec)
    }
    pub fn scale_to(&self, new_scale: u32) -> i128 {
        match { self.scale }.cmp(&new_scale) {
            std::cmp::Ordering::Greater => self
                .mantissa
                .checked_div(10_i128.pow(self.scale - new_scale))
                .unwrap(),
            std::cmp::Ordering::Less => self
                .mantissa
                .checked_mul(10_i128.pow(new_scale - self.scale))
                .unwrap(),
            std::cmp::Ordering::Equal => self.mantissa,
        }
    }
    pub fn new_with_scale(&self, new_scale: u32) -> Self {
        let mantissa = self.scale_to(new_scale);
        SwitchboardDecimal {
            mantissa,
            scale: new_scale,
        }
    }
}
impl From<Decimal> for SwitchboardDecimal {
    fn from(val: Decimal) -> Self {
        SwitchboardDecimal::new(val.mantissa(), val.scale())
    }
}
impl TryInto<Decimal> for &SwitchboardDecimal {
    type Error = anchor_lang::error::Error;
    fn try_into(self) -> anchor_lang::Result<Decimal> {
        Decimal::try_from_i128_with_scale(self.mantissa, self.scale)
            .map_err(|_| error!(SwitchboardError::DecimalConversionError))
    }
}

impl TryInto<Decimal> for SwitchboardDecimal {
    type Error = anchor_lang::error::Error;
    fn try_into(self) -> anchor_lang::Result<Decimal> {
        Decimal::try_from_i128_with_scale(self.mantissa, self.scale)
            .map_err(|_| error!(SwitchboardError::DecimalConversionError))
    }
}

impl Ord for SwitchboardDecimal {
    fn cmp(&self, other: &Self) -> Ordering {
        let s: Decimal = self.try_into().unwrap();
        let other: Decimal = other.try_into().unwrap();
        s.cmp(&other)
    }
}

impl PartialOrd for SwitchboardDecimal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
    fn lt(&self, other: &Self) -> bool {
        let s: Decimal = self.try_into().unwrap();
        let other: Decimal = other.try_into().unwrap();
        s < other
    }
    fn le(&self, other: &Self) -> bool {
        let s: Decimal = self.try_into().unwrap();
        let other: Decimal = other.try_into().unwrap();
        s <= other
    }
    fn gt(&self, other: &Self) -> bool {
        let s: Decimal = self.try_into().unwrap();
        let other: Decimal = other.try_into().unwrap();
        s > other
    }
    fn ge(&self, other: &Self) -> bool {
        let s: Decimal = self.try_into().unwrap();
        let other: Decimal = other.try_into().unwrap();
        s >= other
    }
}

impl From<SwitchboardDecimal> for bool {
    fn from(s: SwitchboardDecimal) -> Self {
        let dec: Decimal = (&s).try_into().unwrap();
        dec.round().mantissa() != 0
    }
}

impl TryInto<u64> for SwitchboardDecimal {
    type Error = anchor_lang::error::Error;
    fn try_into(self) -> anchor_lang::Result<u64> {
        let dec: Decimal = (&self).try_into().unwrap();
        dec.to_u64()
            .ok_or(error!(SwitchboardError::IntegerOverflowError))
    }
}

impl TryInto<i64> for SwitchboardDecimal {
    type Error = anchor_lang::error::Error;
    fn try_into(self) -> anchor_lang::Result<i64> {
        let dec: Decimal = (&self).try_into().unwrap();
        dec.to_i64()
            .ok_or(error!(SwitchboardError::IntegerOverflowError))
    }
}

impl TryInto<f64> for SwitchboardDecimal {
    type Error = anchor_lang::error::Error;
    fn try_into(self) -> anchor_lang::Result<f64> {
        let dec: Decimal = (&self).try_into().unwrap();
        dec.to_f64()
            .ok_or(error!(SwitchboardError::IntegerOverflowError))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switchboard_decimal_into_rust_decimal() {
        let swb_decimal = &SwitchboardDecimal {
            mantissa: 12345,
            scale: 2,
        };
        let decimal: Decimal = swb_decimal.try_into().unwrap();
        assert_eq!(decimal.mantissa(), 12345);
        assert_eq!(decimal.scale(), 2);
    }

    #[test]
    fn empty_switchboard_decimal_is_false() {
        let swb_decimal = SwitchboardDecimal {
            mantissa: 0,
            scale: 0,
        };
        let b: bool = swb_decimal.into();
        assert!(!b);
        let swb_decimal_neg = SwitchboardDecimal {
            mantissa: -0,
            scale: 0,
        };
        let b: bool = swb_decimal_neg.into();
        assert!(!b);
    }

    #[test]
    fn switchboard_decimal_to_u64() {
        // 1234.5678
        let swb_decimal = SwitchboardDecimal {
            mantissa: 12345678,
            scale: 4,
        };
        let b: u64 = swb_decimal.try_into().unwrap();
        assert_eq!(b, 1234);
    }

    #[test]
    fn switchboard_decimal_to_f64() {
        // 1234.5678
        let swb_decimal = SwitchboardDecimal {
            mantissa: 12345678,
            scale: 4,
        };
        let b: f64 = swb_decimal.try_into().unwrap();
        assert_eq!(b, 1234.5678);

        let swb_f64 = SwitchboardDecimal::from_f64(1234.5678);
        assert_eq!(swb_decimal, swb_f64);
    }
}

#[account(zero_copy(unsafe))]
pub struct MyProgramState {
    pub bump: u8,
    pub authority: Pubkey,
    pub switchboard_function: Pubkey,
}

#[repr(packed)]
#[zero_copy(unsafe)]
pub struct OracleData {
    pub oracle_timestamp: i64,
    pub ondo_price: u64,
    pub traded_price: u64,
}

#[derive(Copy, Clone, Default, AnchorSerialize, AnchorDeserialize)]
pub struct OracleDataBorsh {
    pub oracle_timestamp: i64,
    pub ondo_price: u64,
    pub traded_price: u64,
}
impl From<OracleDataBorsh> for OracleData {
    fn from(value: OracleDataBorsh) -> Self {
        Self {
            oracle_timestamp: value.oracle_timestamp,
            ondo_price: value.ondo_price,
            traded_price: value.traded_price,
        }
    }
}

#[derive(Copy, Clone, Default, AnchorSerialize, AnchorDeserialize)]
pub struct OracleDataWithTradingSymbol {
    pub symbol: TradingSymbol,
    pub data: OracleDataBorsh,
}

impl OracleData {
    pub fn get_fair_price(&self) -> anchor_lang::Result<f64> {
        // Check the price was updated in the last 10 seconds

        // Do some logic here based on the twap

        let price: f64 = SwitchboardDecimal {
            mantissa: self.traded_price as i128,
            scale: 18,
        }
        .try_into()?;

        Ok(price)
    }
}

#[repr(packed)]
#[account(zero_copy(unsafe))]
pub struct MyOracleState {
    pub bump: u8,
    pub usdy_usd: OracleData,
    // can always re-allocate to add more
    // pub reserved: [u8; 2400],
}

impl MyOracleState {
    pub fn save_rows(&mut self, rows: &[OracleDataWithTradingSymbol]) -> anchor_lang::Result<()> {
        for row in rows.iter() {
            match row.symbol {
                TradingSymbol::Usdy_usdc => {
                    self.usdy_usd = row.data.into();
                    msg!("ondo_price: ${}", {self.usdy_usd.ondo_price});
                    msg!("traded_price: ${}", {self.usdy_usd.traded_price});
                }
                _ => {
                    msg!("no trading symbol found for {:?}", row.symbol);
                    // TODO: emit an event so we can detect and fix
                }
            }
        }

        Ok(())
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, AnchorSerialize, AnchorDeserialize)]
pub enum TradingSymbol {
    #[default]
    Unknown = 0,
    Usdy_usdc = 1
}

unsafe impl Pod for TradingSymbol {}
unsafe impl Zeroable for TradingSymbol {}

impl From<TradingSymbol> for u8 {
    fn from(value: TradingSymbol) -> Self {
        match value {
            TradingSymbol::Usdy_usdc => 1,
            _ => 0,
        }
    }
}
impl From<u8> for TradingSymbol {
    fn from(value: u8) -> Self {
        match value {
            1 => TradingSymbol::Usdy_usdc,
            _ => TradingSymbol::Unknown,
        }
    }
}

#[zero_copy(unsafe)]
#[repr(packed)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct Hash {
    /// The bytes used to derive the hash.
    pub data: [u8; 32],
}

#[zero_copy(unsafe)]
#[repr(packed)]
#[derive(Default, Debug, PartialEq, Eq)]
pub struct AggregatorRound {
    /// Maintains the number of successful responses received from nodes.
    /// Nodes can submit one successful response per round.
    pub num_success: u32,
    /// Number of error responses.
    pub num_error: u32,
    /// Whether an update request round has ended.
    pub is_closed: bool,
    /// Maintains the `solana_program::clock::Slot` that the round was opened at.
    pub round_open_slot: u64,
    /// Maintains the `solana_program::clock::UnixTimestamp;` the round was opened at.
    pub round_open_timestamp: i64,
    /// Maintains the current median of all successful round responses.
    pub result: SwitchboardDecimal,
    /// Standard deviation of the accepted results in the round.
    pub std_deviation: SwitchboardDecimal,
    /// Maintains the minimum node response this round.
    pub min_response: SwitchboardDecimal,
    /// Maintains the maximum node response this round.
    pub max_response: SwitchboardDecimal,
    /// Pubkeys of the oracles fulfilling this round.
    pub oracle_pubkeys_data: [Pubkey; 16],
    /// Represents all successful node responses this round. `NaN` if empty.
    pub medians_data: [SwitchboardDecimal; 16],
    /// Current rewards/slashes oracles have received this round.
    pub current_payout: [i64; 16],
    /// Keep track of which responses are fulfilled here.
    pub medians_fulfilled: [bool; 16],
    /// Keeps track of which errors are fulfilled here.
    pub errors_fulfilled: [bool; 16],
}

#[derive(Copy, Clone, Debug, AnchorSerialize, AnchorDeserialize, Eq, PartialEq)]
#[repr(u8)]
pub enum AggregatorResolutionMode {
    ModeRoundResolution = 0,
    ModeSlidingResolution = 1,
}
#[account(zero_copy(unsafe))]
#[repr(packed)]
pub struct SlidingResultAccountData {
    pub data: [SlidingWindowElement; 16],
    pub bump: u8,
    pub _ebuf: [u8; 512],
}
#[zero_copy(unsafe)]
#[derive(Default)]
#[repr(packed)]
pub struct SlidingWindowElement {
    pub oracle_key: Pubkey,
    pub value: SwitchboardDecimal,
    pub slot: u64,
    pub timestamp: i64,
}

// #[zero_copy(unsafe)]
#[account(zero_copy(unsafe))]
#[repr(packed)]
#[derive(Debug, PartialEq)]
pub struct AggregatorAccountData {
    /// Name of the aggregator to store on-chain.
    pub name: [u8; 32],
    /// Metadata of the aggregator to store on-chain.
    pub metadata: [u8; 128],
    /// Reserved.
    pub _reserved1: [u8; 32],
    /// Pubkey of the queue the aggregator belongs to.
    pub queue_pubkey: Pubkey,
    /// CONFIGS
    /// Number of oracles assigned to an update request.
    pub oracle_request_batch_size: u32,
    /// Minimum number of oracle responses required before a round is validated.
    pub min_oracle_results: u32,
    /// Minimum number of job results before an oracle accepts a result.
    pub min_job_results: u32,
    /// Minimum number of seconds required between aggregator rounds.
    pub min_update_delay_seconds: u32,
    /// Unix timestamp for which no feed update will occur before.
    pub start_after: i64,
    /// Change percentage required between a previous round and the current round. If variance percentage is not met, reject new oracle responses.
    pub variance_threshold: SwitchboardDecimal,
    /// Number of seconds for which, even if the variance threshold is not passed, accept new responses from oracles.
    pub force_report_period: i64,
    /// Timestamp when the feed is no longer needed.
    pub expiration: i64,
    //
    /// Counter for the number of consecutive failures before a feed is removed from a queue. If set to 0, failed feeds will remain on the queue.
    pub consecutive_failure_count: u64,
    /// Timestamp when the next update request will be available.
    pub next_allowed_update_time: i64,
    /// Flag for whether an aggregators configuration is locked for editing.
    pub is_locked: bool,
    /// Optional, public key of the crank the aggregator is currently using. Event based feeds do not need a crank.
    pub crank_pubkey: Pubkey,
    /// Latest confirmed update request result that has been accepted as valid.
    pub latest_confirmed_round: AggregatorRound,
    /// Oracle results from the current round of update request that has not been accepted as valid yet.
    pub current_round: AggregatorRound,
    /// List of public keys containing the job definitions for how data is sourced off-chain by oracles.
    pub job_pubkeys_data: [Pubkey; 16],
    /// Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment.
    pub job_hashes: [Hash; 16],
    /// Number of jobs assigned to an oracle.
    pub job_pubkeys_size: u32,
    /// Used to protect against malicious RPC nodes providing incorrect task definitions to oracles before fulfillment.
    pub jobs_checksum: [u8; 32],
    //
    /// The account delegated as the authority for making account changes.
    pub authority: Pubkey,
    /// Optional, public key of a history buffer account storing the last N accepted results and their timestamps.
    pub history_buffer: Pubkey,
    /// The previous confirmed round result.
    pub previous_confirmed_round_result: SwitchboardDecimal,
    /// The slot when the previous confirmed round was opened.
    pub previous_confirmed_round_slot: u64,
    /// 	Whether an aggregator is permitted to join a crank.
    pub disable_crank: bool,
    /// Job weights used for the weighted median of the aggregator's assigned job accounts.
    pub job_weights: [u8; 16],
    /// Unix timestamp when the feed was created.
    pub creation_timestamp: i64,
    /// Use sliding windoe or round based resolution
    /// NOTE: This changes result propogation in latest_round_result
    pub resolution_mode: AggregatorResolutionMode,
    /// Reserved for future info.
    pub _ebuf: [u8; 138],
}

impl AggregatorAccountData {
    /// Returns the deserialized Switchboard Aggregator account
    ///
    /// # Arguments
    ///
    /// * `switchboard_feed` - A Solana AccountInfo referencing an existing Switchboard Aggregator
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::AggregatorAccountData;
    ///
    /// let data_feed = AggregatorAccountData::new(feed_account_info)?;
    /// ```
    pub fn new<'info>(
        switchboard_feed: &'info AccountInfo<'info>,
    ) -> anchor_lang::Result<Ref<'info, AggregatorAccountData>> {
        let data = switchboard_feed.try_borrow_data()?;
        if data.len() < AggregatorAccountData::discriminator().len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        if disc_bytes != AggregatorAccountData::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(Ref::map(data, |data| {
            bytemuck::from_bytes(&data[8..std::mem::size_of::<AggregatorAccountData>() + 8])
        }))
    }

    /// Returns the deserialized Switchboard Aggregator account
    ///
    /// # Arguments
    ///
    /// * `data` - A Solana AccountInfo's data buffer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::AggregatorAccountData;
    ///
    /// let data_feed = AggregatorAccountData::new(feed_account_info.try_borrow_data()?)?;
    /// ```
    pub fn new_from_bytes(data: &[u8]) -> anchor_lang::Result<&AggregatorAccountData> {
        if data.len() < AggregatorAccountData::discriminator().len() {
            return Err(ErrorCode::AccountDiscriminatorNotFound.into());
        }

        let mut disc_bytes = [0u8; 8];
        disc_bytes.copy_from_slice(&data[..8]);
        if disc_bytes != AggregatorAccountData::discriminator() {
            return Err(ErrorCode::AccountDiscriminatorMismatch.into());
        }

        Ok(bytemuck::from_bytes(
            &data[8..std::mem::size_of::<AggregatorAccountData>() + 8],
        ))
    }

    /// If sufficient oracle responses, returns the latest on-chain result in SwitchboardDecimal format
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::AggregatorAccountData;
    /// use std::convert::TryInto;
    ///
    /// let feed_result = AggregatorAccountData::new(feed_account_info)?.get_result()?;
    /// let decimal: f64 = feed_result.try_into()?;
    /// ```
    pub fn get_result(&self) -> anchor_lang::Result<SwitchboardDecimal> {
        if self.resolution_mode == AggregatorResolutionMode::ModeSlidingResolution {
            return Ok(self.latest_confirmed_round.result);
        }
        let min_oracle_results = self.min_oracle_results;
        let latest_confirmed_round_num_success = self.latest_confirmed_round.num_success;
        if min_oracle_results > latest_confirmed_round_num_success {
            return Err(SwitchboardError::InvalidAggregatorRound.into());
        }
        Ok(self.latest_confirmed_round.result)
    }

    /// Check whether the confidence interval exceeds a given threshold
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::{AggregatorAccountData, SwitchboardDecimal};
    ///
    /// let feed = AggregatorAccountData::new(feed_account_info)?;
    /// feed.check_confidence_interval(SwitchboardDecimal::from_f64(0.80))?;
    /// ```
    pub fn check_confidence_interval(
        &self,
        max_confidence_interval: SwitchboardDecimal,
    ) -> anchor_lang::Result<()> {
        if self.latest_confirmed_round.std_deviation > max_confidence_interval {
            return Err(SwitchboardError::ConfidenceIntervalExceeded.into());
        }
        Ok(())
    }

    /// Check the variance (as a percentage difference from the max delivered
    /// oracle value) from all oracles.
    pub fn check_variace(&self, max_variance: Decimal) -> anchor_lang::Result<()> {
        if max_variance > Decimal::ONE {
            return Err(SwitchboardError::InvalidFunctionInput.into());
        }
        let min: Decimal = self.latest_confirmed_round.min_response.try_into().unwrap();
        let max: Decimal = self.latest_confirmed_round.max_response.try_into().unwrap();

        if min < Decimal::ZERO || max < Decimal::ZERO || min > max {
            return Err(SwitchboardError::AllowedVarianceExceeded.into());
        }
        if min / max > max_variance {
            return Err(SwitchboardError::AllowedVarianceExceeded.into());
        }
        Ok(())
    }

    /// Check whether the feed has been updated in the last max_staleness seconds
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use switchboard_v2::AggregatorAccountData;
    ///
    /// let feed = AggregatorAccountData::new(feed_account_info)?;
    /// feed.check_staleness(clock::Clock::get().unwrap().unix_timestamp, 300)?;
    /// ```
    pub fn check_staleness(
        &self,
        unix_timestamp: i64,
        max_staleness: i64,
    ) -> anchor_lang::Result<()> {
        let staleness = unix_timestamp - self.latest_confirmed_round.round_open_timestamp;
        if staleness > max_staleness {
            msg!("Feed has not been updated in {} seconds!", staleness);
            return Err(SwitchboardError::StaleFeed.into());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    impl<'info> Default for AggregatorAccountData {
        fn default() -> Self {
            unsafe { std::mem::zeroed() }
        }
    }

    fn create_aggregator(lastest_round: AggregatorRound) -> AggregatorAccountData {
        let mut aggregator = AggregatorAccountData::default();
        aggregator.min_update_delay_seconds = 10;
        aggregator.latest_confirmed_round = lastest_round;
        aggregator.min_job_results = 10;
        aggregator.min_oracle_results = 10;
        return aggregator;
    }

    fn create_round(value: f64, num_success: u32, num_error: u32) -> AggregatorRound {
        let mut result = AggregatorRound::default();
        result.num_success = num_success;
        result.num_error = num_error;
        result.result = SwitchboardDecimal::from_f64(value);
        return result;
    }

    #[test]
    fn test_accept_current_on_sucess_count() {
        let lastest_round = create_round(100.0, 30, 0); // num success 30 > 10 min oracle result

        let aggregator = create_aggregator(lastest_round.clone());
        assert_eq!(
            aggregator.get_result().unwrap(),
            lastest_round.result.clone()
        );
    }

    #[test]
    fn test_reject_current_on_sucess_count() {
        let lastest_round = create_round(100.0, 5, 0); // num success 30 < 10 min oracle result
        let aggregator = create_aggregator(lastest_round.clone());

        assert!(
            aggregator.get_result().is_err(),
            "Aggregator is not currently populated with a valid round."
        );
    }

    #[test]
    fn test_no_valid_aggregator_result() {
        let aggregator = create_aggregator(AggregatorRound::default());

        assert!(
            aggregator.get_result().is_err(),
            "Aggregator is not currently populated with a valid round."
        );
    }
}
