//! 256-bit signed integer type for Cranelift IR.
//!
//! This module provides an `I256` type that wraps `alloy_primitives::I256`.

use core::cmp::Ordering;
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::{
    Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr, Sub,
};

#[cfg(feature = "enable-serde")]
use serde_derive::{Deserialize, Serialize};

/// A 256-bit signed integer type wrapping `alloy_primitives::I256`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct I256(alloy_primitives::I256);

impl I256 {
    /// The additive identity (zero).
    pub const ZERO: Self = Self(alloy_primitives::I256::ZERO);

    /// The value 1.
    pub const ONE: Self = Self(alloy_primitives::I256::ONE);

    /// The maximum value (all bits set except sign bit).
    pub const MAX: Self = Self(alloy_primitives::I256::MAX);

    /// The minimum value (only sign bit set).
    pub const MIN: Self = Self(alloy_primitives::I256::MIN);

    /// All bits set to one (-1 in two's complement).
    pub const ALL_ONES: Self = Self(alloy_primitives::I256::MINUS_ONE);

    /// Create a new `I256` from four limbs in little-endian order.
    #[inline]
    pub const fn from_limbs(limbs: [u64; 4]) -> Self {
        Self(alloy_primitives::I256::from_raw(
            alloy_primitives::U256::from_limbs(limbs),
        ))
    }

    /// Get the underlying limbs in little-endian order.
    #[inline]
    pub const fn limbs(&self) -> [u64; 4] {
        *self.0.into_raw().as_limbs()
    }

    /// Create a new `I256` from a byte array in native-endian order.
    #[inline]
    pub fn from_ne_bytes(bytes: [u8; 32]) -> Self {
        if cfg!(target_endian = "little") {
            Self::from_le_bytes(bytes)
        } else {
            Self::from_be_bytes(bytes)
        }
    }

    /// Create an `I256` from a byte array in little-endian order.
    #[inline]
    pub fn from_le_bytes(bytes: [u8; 32]) -> Self {
        Self(alloy_primitives::I256::from_le_bytes(bytes))
    }

    /// Create an `I256` from a byte array in big-endian order.
    #[inline]
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self(alloy_primitives::I256::from_be_bytes(bytes))
    }

    /// Convert to a byte array in little-endian order.
    #[inline]
    pub fn to_le_bytes(self) -> [u8; 32] {
        self.0.to_le_bytes()
    }

    /// Convert to a byte array in big-endian order.
    #[inline]
    pub fn to_be_bytes(self) -> [u8; 32] {
        self.0.to_be_bytes()
    }

    /// Returns `true` if the value is zero.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns `true` if the sign bit is set (negative in two's complement).
    #[inline]
    pub const fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    /// Wrapping addition.
    #[inline]
    pub fn wrapping_add(self, rhs: Self) -> Self {
        Self(self.0.wrapping_add(rhs.0))
    }

    /// Wrapping subtraction.
    #[inline]
    pub fn wrapping_sub(self, rhs: Self) -> Self {
        Self(self.0.wrapping_sub(rhs.0))
    }

    /// Wrapping negation.
    #[inline]
    pub fn wrapping_neg(self) -> Self {
        Self(self.0.wrapping_neg())
    }

    /// Wrapping multiplication.
    #[inline]
    pub fn wrapping_mul(self, rhs: Self) -> Self {
        Self(self.0.wrapping_mul(rhs.0))
    }

    /// Bitwise AND.
    #[inline]
    pub fn bitand(self, rhs: Self) -> Self {
        Self(self.0 & rhs.0)
    }

    /// Bitwise OR.
    #[inline]
    pub fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }

    /// Bitwise XOR.
    #[inline]
    pub fn bitxor(self, rhs: Self) -> Self {
        Self(self.0 ^ rhs.0)
    }

    /// Bitwise NOT.
    #[inline]
    pub fn not(self) -> Self {
        Self(!self.0)
    }

    /// Left shift by `n` bits (wrapping, n is masked to 0..255).
    #[inline]
    pub fn wrapping_shl(self, n: u32) -> Self {
        Self(self.0.wrapping_shl(n as usize))
    }

    /// Logical right shift by `n` bits (wrapping, n is masked to 0..255).
    #[inline]
    pub fn wrapping_ushr(self, n: u32) -> Self {
        Self(alloy_primitives::I256::from_raw(
            self.0.into_raw().wrapping_shr(n as usize),
        ))
    }

    /// Arithmetic right shift by `n` bits (wrapping, n is masked to 0..255).
    #[inline]
    pub fn wrapping_sshr(self, n: u32) -> Self {
        Self(self.0.asr(n as usize))
    }

    /// Count leading zeros.
    #[inline]
    pub fn leading_zeros(&self) -> u32 {
        self.0.leading_zeros() as u32
    }

    /// Count trailing zeros.
    #[inline]
    pub fn trailing_zeros(&self) -> u32 {
        self.0.trailing_zeros() as u32
    }

    /// Count the number of ones.
    #[inline]
    pub fn count_ones(&self) -> u32 {
        self.0.count_ones() as u32
    }

    /// Reverse the byte order.
    #[inline]
    pub fn swap_bytes(self) -> Self {
        Self::from_be_bytes(self.to_le_bytes())
    }

    /// Signed comparison.
    #[inline]
    pub fn cmp_signed(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }

    /// Unsigned comparison.
    #[inline]
    pub fn cmp_unsigned(&self, other: &Self) -> Ordering {
        self.0.into_raw().cmp(&other.0.into_raw())
    }

    /// Parse an I256 from a string with the given radix.
    pub fn from_str_radix(s: &str, radix: u32) -> Result<Self, core::num::ParseIntError> {
        let invalid_radix = || u8::from_str_radix("0", 69).unwrap_err();
        let empty = || u8::from_str_radix("", 10).unwrap_err();
        let overflow = || u8::from_str_radix("256", 10).unwrap_err();
        let invalid_digit = || u8::from_str_radix("a", 10).unwrap_err();

        if !(2..=36).contains(&radix) {
            return Err(invalid_radix());
        }

        let bytes = s.as_bytes();
        if bytes.is_empty() {
            return Err(empty());
        }

        let (negative, digits) = match bytes[0] {
            b'-' => (true, &bytes[1..]),
            b'+' => (false, &bytes[1..]),
            _ => (false, bytes),
        };

        if digits.is_empty() {
            return Err(empty());
        }

        let radix_val = Self::from(radix);
        let mut result = Self::ZERO;

        for &byte in digits {
            let digit = match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'z' => byte - b'a' + 10,
                b'A'..=b'Z' => byte - b'A' + 10,
                _ => return Err(invalid_digit()),
            };

            if digit as u32 >= radix {
                return Err(invalid_digit());
            }

            let digit_val = Self::from(digit as u64);

            let (new_result, overflow_mul) = result.overflowing_mul(radix_val);
            if overflow_mul {
                return Err(overflow());
            }
            result = new_result;

            if negative {
                let (new_result, overflow_sub) = result.overflowing_sub(digit_val);
                if overflow_sub {
                    return Err(overflow());
                }
                result = new_result;
            } else {
                let (new_result, overflow_add) = result.overflowing_add(digit_val);
                if overflow_add {
                    return Err(overflow());
                }
                result = new_result;
            }
        }

        Ok(result)
    }

    /// Overflowing addition.
    #[inline]
    fn overflowing_add(self, rhs: Self) -> (Self, bool) {
        let (result, overflow) = self.0.overflowing_add(rhs.0);
        (Self(result), overflow)
    }

    /// Overflowing subtraction.
    #[inline]
    fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
        let (result, overflow) = self.0.overflowing_sub(rhs.0);
        (Self(result), overflow)
    }

    /// Overflowing multiplication.
    #[inline]
    fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
        let (result, overflow) = self.0.overflowing_mul(rhs.0);
        (Self(result), overflow)
    }
}

impl Debug for I256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let limbs = self.limbs();
        write!(
            f,
            "I256([{:#018x}, {:#018x}, {:#018x}, {:#018x}])",
            limbs[0], limbs[1], limbs[2], limbs[3]
        )
    }
}

impl Display for I256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for &limb in self.limbs().iter().rev() {
            write!(f, "{limb:016x}")?;
        }
        Ok(())
    }
}

impl PartialOrd for I256 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_signed(other))
    }
}

impl Ord for I256 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_signed(other)
    }
}

impl From<i64> for I256 {
    #[inline]
    fn from(v: i64) -> Self {
        Self(alloy_primitives::I256::try_from(v).unwrap())
    }
}

impl From<u64> for I256 {
    #[inline]
    fn from(v: u64) -> Self {
        Self(alloy_primitives::I256::try_from(v).unwrap())
    }
}

impl From<i128> for I256 {
    #[inline]
    fn from(v: i128) -> Self {
        Self(alloy_primitives::I256::try_from(v).unwrap())
    }
}

impl From<u128> for I256 {
    #[inline]
    fn from(v: u128) -> Self {
        Self(alloy_primitives::I256::try_from(v).unwrap())
    }
}

impl From<u32> for I256 {
    #[inline]
    fn from(v: u32) -> Self {
        Self(alloy_primitives::I256::try_from(v).unwrap())
    }
}

impl From<[u8; 32]> for I256 {
    #[inline]
    fn from(bytes: [u8; 32]) -> Self {
        Self::from_le_bytes(bytes)
    }
}

impl From<I256> for [u8; 32] {
    #[inline]
    fn from(val: I256) -> [u8; 32] {
        val.to_le_bytes()
    }
}

impl From<[u64; 4]> for I256 {
    #[inline]
    fn from(limbs: [u64; 4]) -> Self {
        Self::from_limbs(limbs)
    }
}

impl From<I256> for [u64; 4] {
    #[inline]
    fn from(val: I256) -> [u64; 4] {
        val.limbs()
    }
}

impl core::str::FromStr for I256 {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str_radix(s, 10)
    }
}

impl Add for I256 {
    type Output = Self;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        self.wrapping_add(rhs)
    }
}

impl Sub for I256 {
    type Output = Self;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        self.wrapping_sub(rhs)
    }
}

impl BitAnd for I256 {
    type Output = Self;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        self.bitand(rhs)
    }
}

impl BitAndAssign for I256 {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = self.bitand(rhs);
    }
}

impl BitOr for I256 {
    type Output = Self;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        self.bitor(rhs)
    }
}

impl BitOrAssign for I256 {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = self.bitor(rhs);
    }
}

impl BitXor for I256 {
    type Output = Self;

    #[inline]
    fn bitxor(self, rhs: Self) -> Self::Output {
        self.bitxor(rhs)
    }
}

impl BitXorAssign for I256 {
    #[inline]
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = self.bitxor(rhs);
    }
}

impl Not for I256 {
    type Output = Self;

    #[inline]
    fn not(self) -> Self::Output {
        self.not()
    }
}

impl Shl<u32> for I256 {
    type Output = Self;

    #[inline]
    fn shl(self, rhs: u32) -> Self::Output {
        self.wrapping_shl(rhs)
    }
}

impl Shr<u32> for I256 {
    type Output = Self;

    #[inline]
    fn shr(self, rhs: u32) -> Self::Output {
        self.wrapping_ushr(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zero_and_one() {
        assert!(I256::ZERO.is_zero());
        assert!(!I256::ONE.is_zero());
        assert_eq!(I256::ZERO.limbs(), [0, 0, 0, 0]);
        assert_eq!(I256::ONE.limbs(), [1, 0, 0, 0]);
    }

    #[test]
    fn test_from_i64() {
        let pos = I256::from(42i64);
        assert_eq!(pos.limbs(), [42, 0, 0, 0]);
        assert!(!pos.is_negative());

        let neg = I256::from(-1i64);
        assert_eq!(neg.limbs(), [u64::MAX, u64::MAX, u64::MAX, u64::MAX]);
        assert!(neg.is_negative());

        let neg2 = I256::from(-42i64);
        assert!(neg2.is_negative());
    }

    #[test]
    fn test_from_u128() {
        let val = I256::from(u128::MAX);
        assert_eq!(val.limbs(), [u64::MAX, u64::MAX, 0, 0]);
        assert!(!val.is_negative());
    }

    #[test]
    fn test_wrapping_add() {
        let a = I256::from(100u64);
        let b = I256::from(200u64);
        assert_eq!(a.wrapping_add(b).limbs()[0], 300);

        let max = I256::ALL_ONES;
        let one = I256::ONE;
        assert_eq!(max.wrapping_add(one), I256::ZERO);
    }

    #[test]
    fn test_wrapping_sub() {
        let a = I256::from(300u64);
        let b = I256::from(100u64);
        assert_eq!(a.wrapping_sub(b).limbs()[0], 200);

        let zero = I256::ZERO;
        let one = I256::ONE;
        assert_eq!(zero.wrapping_sub(one), I256::ALL_ONES);
    }

    #[test]
    fn test_wrapping_neg() {
        let one = I256::ONE;
        let neg_one = one.wrapping_neg();
        assert_eq!(neg_one, I256::ALL_ONES);

        let zero = I256::ZERO;
        assert_eq!(zero.wrapping_neg(), I256::ZERO);
    }

    #[test]
    fn test_wrapping_mul() {
        let a = I256::from(7u64);
        let b = I256::from(6u64);
        assert_eq!(a.wrapping_mul(b).limbs()[0], 42);

        let large = I256::from(u64::MAX);
        let two = I256::from(2u64);
        let result = large.wrapping_mul(two);
        assert_eq!(result.limbs()[0], u64::MAX - 1);
        assert_eq!(result.limbs()[1], 1);
    }

    #[test]
    fn test_bitwise_ops() {
        let a = I256::from(0b1100u64);
        let b = I256::from(0b1010u64);

        assert_eq!(a.bitand(b).limbs()[0], 0b1000);
        assert_eq!(a.bitor(b).limbs()[0], 0b1110);
        assert_eq!(a.bitxor(b).limbs()[0], 0b0110);
        assert_eq!(a.not().limbs()[0], !0b1100u64);
    }

    #[test]
    fn test_shift_left() {
        let one = I256::ONE;

        assert_eq!(one.wrapping_shl(0), one);
        assert_eq!(one.wrapping_shl(1).limbs()[0], 2);
        assert_eq!(one.wrapping_shl(63).limbs()[0], 1 << 63);
        assert_eq!(one.wrapping_shl(64).limbs(), [0, 1, 0, 0]);
        assert_eq!(one.wrapping_shl(128).limbs(), [0, 0, 1, 0]);
        assert_eq!(one.wrapping_shl(192).limbs(), [0, 0, 0, 1]);
        assert_eq!(one.wrapping_shl(255).limbs(), [0, 0, 0, 1 << 63]);
    }

    #[test]
    fn test_shift_right_logical() {
        let high = I256::from_limbs([0, 0, 0, 1]);

        assert_eq!(high.wrapping_ushr(0), high);
        assert_eq!(high.wrapping_ushr(64).limbs(), [0, 0, 1, 0]);
        assert_eq!(high.wrapping_ushr(128).limbs(), [0, 1, 0, 0]);
        assert_eq!(high.wrapping_ushr(192).limbs(), [1, 0, 0, 0]);

        let neg = I256::ALL_ONES;
        let shifted = neg.wrapping_ushr(128);
        assert_eq!(shifted.limbs(), [u64::MAX, u64::MAX, 0, 0]);
    }

    #[test]
    fn test_shift_right_arithmetic() {
        let neg = I256::from(-1i64);
        assert_eq!(neg.wrapping_sshr(1), neg);
        assert_eq!(neg.wrapping_sshr(64), neg);
        assert_eq!(neg.wrapping_sshr(255), neg);

        let min = I256::MIN;
        let shifted = min.wrapping_sshr(1);
        assert_eq!(shifted.limbs(), [0, 0, 0, 0xC000_0000_0000_0000]);
    }

    #[test]
    fn test_leading_trailing_zeros() {
        assert_eq!(I256::ZERO.leading_zeros(), 256);
        assert_eq!(I256::ONE.leading_zeros(), 255);
        assert_eq!(I256::from_limbs([0, 0, 0, 1]).leading_zeros(), 63);

        assert_eq!(I256::ZERO.trailing_zeros(), 256);
        assert_eq!(I256::ONE.trailing_zeros(), 0);
        assert_eq!(I256::from_limbs([0, 1, 0, 0]).trailing_zeros(), 64);
    }

    #[test]
    fn test_byte_conversions() {
        let val = I256::from(0x0102030405060708u64);
        let le_bytes = val.to_le_bytes();
        assert_eq!(le_bytes[0], 0x08);
        assert_eq!(le_bytes[7], 0x01);

        let roundtrip = I256::from_le_bytes(le_bytes);
        assert_eq!(roundtrip, val);

        let be_bytes = val.to_be_bytes();
        assert_eq!(be_bytes[31], 0x08);
        assert_eq!(be_bytes[24], 0x01);

        let roundtrip_be = I256::from_be_bytes(be_bytes);
        assert_eq!(roundtrip_be, val);
    }

    #[test]
    fn test_comparison() {
        let zero = I256::ZERO;
        let one = I256::ONE;
        let neg_one = I256::from(-1i64);

        assert!(zero < one);
        assert!(neg_one < zero);
        assert!(neg_one < one);

        assert_eq!(zero.cmp_unsigned(&one), Ordering::Less);
        assert_eq!(neg_one.cmp_unsigned(&zero), Ordering::Greater);
    }

    #[test]
    fn test_display() {
        let val = I256::from(0xABCDu64);
        let s = alloc::format!("{val}");
        assert!(s.starts_with("0x"));
        assert!(s.ends_with("abcd"));
    }

    #[test]
    fn test_swap_bytes() {
        let val = I256::from_limbs([
            0x0102030405060708,
            0x090a0b0c0d0e0f10,
            0x1112131415161718,
            0x191a1b1c1d1e1f20,
        ]);
        let swapped = val.swap_bytes();
        assert_eq!(swapped.limbs()[0], 0x201f1e1d1c1b1a19);
        assert_eq!(swapped.limbs()[3], 0x0807060504030201);
    }
}
