use switchboard_utils::Decimal;
use ethers::types::U256;
use switchboard_utils::ToPrimitive;
use std::cmp::Ordering;
pub struct Math {
}


impl Math {
   pub fn median(mut numbers: Vec<Decimal>) -> Option<ethers::types::U256> {

        if numbers.is_empty() {
            return None; // Cannot find the median of an empty list
        }

        // Sort the numbers using `partial_cmp` because `Decimal` doesn't implement `Ord`
        numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

        let mid = numbers.len() / 2; // Find the middle index
        Some(if numbers.len() % 2 == 0 {
            // If even number of elements, average the middle two
            let decimal = (numbers[mid - 1] + numbers[mid]) / Decimal::from(2);
            let i256 = (ethers::types::U256::from(decimal.to_u128().unwrap()));
            i256
        } else {
            // If odd, return the middle element
            let decimal = numbers[mid];
            let i256 = (ethers::types::U256::from(decimal.to_u128().unwrap()));
            i256
        })
    }

    pub fn u256_to_f64(value: U256) -> f64 {
        // This function assumes that the value can fit within an f64
        // It's a simple way to convert, and more sophisticated methods may be needed for larger values
        value.as_u128() as f64
    }

    pub fn get_percentage_of_total(part: U256, total: U256) -> f64 {
        if total.is_zero() {
            panic!("Total must not be zero");
        }

        // Convert both U256 values to f64
        let part = Self::u256_to_f64(part);
        let total = Self::u256_to_f64(total);

        // Calculate the percentage
        (part / total) * 100.0
    }
}