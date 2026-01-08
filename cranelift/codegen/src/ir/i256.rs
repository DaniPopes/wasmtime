//! 256-bit signed integer type for Cranelift IR.
//!
//! This module provides an `I256` type that wraps `[u64; 4]` in little-endian
//! order (least significant limb first). It implements the standard library-like
//! APIs needed for constant folding and optimizer support in Cranelift.

use core::cmp::Ordering;
use core::fmt::{self, Debug, Display, Formatter};
use core::ops::{
    Add, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not, Shl, Shr, Sub,
};

#[cfg(feature = "enable-serde")]
use serde_derive::{Deserialize, Serialize};

/// A 256-bit signed integer type stored as four 64-bit limbs in little-endian order.
///
/// The limbs are stored with the least significant limb first:
/// `limbs[0]` is bits 0-63, `limbs[1]` is bits 64-127, etc.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct I256([u64; 4]);

impl I256 {
    /// The additive identity (zero).
    pub const ZERO: Self = Self([0, 0, 0, 0]);

    /// The value 1.
    pub const ONE: Self = Self([1, 0, 0, 0]);

    /// The maximum value (all bits set except sign bit).
    pub const MAX: Self = Self([u64::MAX, u64::MAX, u64::MAX, i64::MAX as u64]);

    /// The minimum value (only sign bit set).
    pub const MIN: Self = Self([0, 0, 0, 0x8000_0000_0000_0000]);

    /// All bits set to one.
    pub const ALL_ONES: Self = Self([u64::MAX, u64::MAX, u64::MAX, u64::MAX]);

    /// Create a new `I256` from four limbs in little-endian order.
    #[inline]
    pub const fn from_limbs(limbs: [u64; 4]) -> Self {
        Self(limbs)
    }

    /// Get the underlying limbs in little-endian order.
    #[inline]
    pub const fn limbs(&self) -> [u64; 4] {
        self.0
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
        Self([
            u64::from_le_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
            u64::from_le_bytes([
                bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15],
            ]),
            u64::from_le_bytes([
                bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22],
                bytes[23],
            ]),
            u64::from_le_bytes([
                bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30],
                bytes[31],
            ]),
        ])
    }

    /// Create an `I256` from a byte array in big-endian order.
    #[inline]
    pub fn from_be_bytes(bytes: [u8; 32]) -> Self {
        Self([
            u64::from_be_bytes([
                bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30],
                bytes[31],
            ]),
            u64::from_be_bytes([
                bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22],
                bytes[23],
            ]),
            u64::from_be_bytes([
                bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14],
                bytes[15],
            ]),
            u64::from_be_bytes([
                bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            ]),
        ])
    }

    /// Convert to a byte array in little-endian order.
    #[inline]
    pub fn to_le_bytes(self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&self.0[0].to_le_bytes());
        bytes[8..16].copy_from_slice(&self.0[1].to_le_bytes());
        bytes[16..24].copy_from_slice(&self.0[2].to_le_bytes());
        bytes[24..32].copy_from_slice(&self.0[3].to_le_bytes());
        bytes
    }

    /// Convert to a byte array in big-endian order.
    #[inline]
    pub fn to_be_bytes(self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes[0..8].copy_from_slice(&self.0[3].to_be_bytes());
        bytes[8..16].copy_from_slice(&self.0[2].to_be_bytes());
        bytes[16..24].copy_from_slice(&self.0[1].to_be_bytes());
        bytes[24..32].copy_from_slice(&self.0[0].to_be_bytes());
        bytes
    }

    /// Returns `true` if the value is zero.
    #[inline]
    pub const fn is_zero(&self) -> bool {
        self.0[0] == 0 && self.0[1] == 0 && self.0[2] == 0 && self.0[3] == 0
    }

    /// Returns `true` if the sign bit is set (negative in two's complement).
    #[inline]
    pub const fn is_negative(&self) -> bool {
        (self.0[3] as i64) < 0
    }

    /// Wrapping addition.
    #[inline]
    pub fn wrapping_add(self, rhs: Self) -> Self {
        let (r0, c0) = self.0[0].overflowing_add(rhs.0[0]);
        let (r1, c1) = self.0[1].carrying_add(rhs.0[1], c0);
        let (r2, c2) = self.0[2].carrying_add(rhs.0[2], c1);
        let (r3, _) = self.0[3].carrying_add(rhs.0[3], c2);
        Self([r0, r1, r2, r3])
    }

    /// Wrapping subtraction.
    #[inline]
    pub fn wrapping_sub(self, rhs: Self) -> Self {
        let (r0, b0) = self.0[0].overflowing_sub(rhs.0[0]);
        let (r1, b1) = self.0[1].borrowing_sub(rhs.0[1], b0);
        let (r2, b2) = self.0[2].borrowing_sub(rhs.0[2], b1);
        let (r3, _) = self.0[3].borrowing_sub(rhs.0[3], b2);
        Self([r0, r1, r2, r3])
    }

    /// Wrapping negation.
    #[inline]
    pub fn wrapping_neg(self) -> Self {
        Self::ZERO.wrapping_sub(self)
    }

    /// Wrapping multiplication.
    #[inline]
    pub fn wrapping_mul(self, rhs: Self) -> Self {
        let mut result = [0u64; 4];

        for i in 0..4 {
            let mut carry = 0u64;
            for j in 0..(4 - i) {
                let (lo, hi) = carrying_mul(self.0[i], rhs.0[j], carry);
                let (sum, c) = result[i + j].overflowing_add(lo);
                result[i + j] = sum;
                carry = hi + c as u64;
            }
        }

        Self(result)
    }

    /// Bitwise AND.
    #[inline]
    pub const fn bitand(self, rhs: Self) -> Self {
        Self([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
            self.0[3] & rhs.0[3],
        ])
    }

    /// Bitwise OR.
    #[inline]
    pub const fn bitor(self, rhs: Self) -> Self {
        Self([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
        ])
    }

    /// Bitwise XOR.
    #[inline]
    pub const fn bitxor(self, rhs: Self) -> Self {
        Self([
            self.0[0] ^ rhs.0[0],
            self.0[1] ^ rhs.0[1],
            self.0[2] ^ rhs.0[2],
            self.0[3] ^ rhs.0[3],
        ])
    }

    /// Bitwise NOT.
    #[inline]
    pub const fn not(self) -> Self {
        Self([!self.0[0], !self.0[1], !self.0[2], !self.0[3]])
    }

    /// Left shift by `n` bits (wrapping, n is masked to 0..255).
    #[inline]
    pub fn wrapping_shl(self, n: u32) -> Self {
        let n = n & 255;
        if n == 0 {
            return self;
        }
        if n >= 256 {
            return Self::ZERO;
        }

        let limb_shift = (n / 64) as usize;
        let bit_shift = n % 64;

        let mut result = [0u64; 4];
        if bit_shift == 0 {
            for i in limb_shift..4 {
                result[i] = self.0[i - limb_shift];
            }
        } else {
            for i in limb_shift..4 {
                result[i] = self.0[i - limb_shift] << bit_shift;
                if i > limb_shift {
                    result[i] |= self.0[i - limb_shift - 1] >> (64 - bit_shift);
                }
            }
        }
        Self(result)
    }

    /// Logical right shift by `n` bits (wrapping, n is masked to 0..255).
    #[inline]
    pub fn wrapping_ushr(self, n: u32) -> Self {
        let n = n & 255;
        if n == 0 {
            return self;
        }
        if n >= 256 {
            return Self::ZERO;
        }

        let limb_shift = (n / 64) as usize;
        let bit_shift = n % 64;

        let mut result = [0u64; 4];
        if bit_shift == 0 {
            for i in 0..(4 - limb_shift) {
                result[i] = self.0[i + limb_shift];
            }
        } else {
            for i in 0..(4 - limb_shift) {
                result[i] = self.0[i + limb_shift] >> bit_shift;
                if i + limb_shift + 1 < 4 {
                    result[i] |= self.0[i + limb_shift + 1] << (64 - bit_shift);
                }
            }
        }
        Self(result)
    }

    /// Arithmetic right shift by `n` bits (wrapping, n is masked to 0..255).
    #[inline]
    pub fn wrapping_sshr(self, n: u32) -> Self {
        let n = n & 255;
        if n == 0 {
            return self;
        }

        let sign_extension = if self.is_negative() {
            Self::ALL_ONES
        } else {
            Self::ZERO
        };

        if n >= 256 {
            return sign_extension;
        }

        let limb_shift = (n / 64) as usize;
        let bit_shift = n % 64;

        let mut result = [0u64; 4];
        if bit_shift == 0 {
            for i in 0..(4 - limb_shift) {
                result[i] = self.0[i + limb_shift];
            }
            for i in (4 - limb_shift)..4 {
                result[i] = sign_extension.0[i];
            }
        } else {
            for i in 0..(4 - limb_shift) {
                result[i] = self.0[i + limb_shift] >> bit_shift;
                if i + limb_shift + 1 < 4 {
                    result[i] |= self.0[i + limb_shift + 1] << (64 - bit_shift);
                } else {
                    result[i] |= sign_extension.0[3] << (64 - bit_shift);
                }
            }
            for i in (4 - limb_shift)..4 {
                result[i] = sign_extension.0[i];
            }
        }
        Self(result)
    }

    /// Count leading zeros.
    #[inline]
    pub fn leading_zeros(&self) -> u32 {
        if self.0[3] != 0 {
            self.0[3].leading_zeros()
        } else if self.0[2] != 0 {
            64 + self.0[2].leading_zeros()
        } else if self.0[1] != 0 {
            128 + self.0[1].leading_zeros()
        } else {
            192 + self.0[0].leading_zeros()
        }
    }

    /// Count trailing zeros.
    #[inline]
    pub fn trailing_zeros(&self) -> u32 {
        if self.0[0] != 0 {
            self.0[0].trailing_zeros()
        } else if self.0[1] != 0 {
            64 + self.0[1].trailing_zeros()
        } else if self.0[2] != 0 {
            128 + self.0[2].trailing_zeros()
        } else {
            192 + self.0[3].trailing_zeros()
        }
    }

    /// Count the number of ones.
    #[inline]
    pub fn count_ones(&self) -> u32 {
        self.0[0].count_ones()
            + self.0[1].count_ones()
            + self.0[2].count_ones()
            + self.0[3].count_ones()
    }

    /// Reverse the byte order.
    #[inline]
    pub fn swap_bytes(self) -> Self {
        Self([
            self.0[3].swap_bytes(),
            self.0[2].swap_bytes(),
            self.0[1].swap_bytes(),
            self.0[0].swap_bytes(),
        ])
    }

    /// Signed comparison.
    #[inline]
    pub fn cmp_signed(&self, other: &Self) -> Ordering {
        let self_neg = self.is_negative();
        let other_neg = other.is_negative();

        match (self_neg, other_neg) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => self.cmp_unsigned(other),
        }
    }

    /// Unsigned comparison.
    #[inline]
    pub fn cmp_unsigned(&self, other: &Self) -> Ordering {
        match self.0[3].cmp(&other.0[3]) {
            Ordering::Equal => match self.0[2].cmp(&other.0[2]) {
                Ordering::Equal => match self.0[1].cmp(&other.0[1]) {
                    Ordering::Equal => self.0[0].cmp(&other.0[0]),
                    ord => ord,
                },
                ord => ord,
            },
            ord => ord,
        }
    }
}

/// Helper: widening multiply with carry.
#[inline]
fn carrying_mul(a: u64, b: u64, carry: u64) -> (u64, u64) {
    let wide = (a as u128) * (b as u128) + (carry as u128);
    (wide as u64, (wide >> 64) as u64)
}

impl Debug for I256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "I256([{:#018x}, {:#018x}, {:#018x}, {:#018x}])",
            self.0[0], self.0[1], self.0[2], self.0[3]
        )
    }
}

impl Display for I256 {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "0x")?;
        for &limb in self.0.iter().rev() {
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
        let sign = if v < 0 { u64::MAX } else { 0 };
        Self([v as u64, sign, sign, sign])
    }
}

impl From<u64> for I256 {
    #[inline]
    fn from(v: u64) -> Self {
        Self([v, 0, 0, 0])
    }
}

impl From<i128> for I256 {
    #[inline]
    fn from(v: i128) -> Self {
        let sign = if v < 0 { u64::MAX } else { 0 };
        Self([v as u64, (v >> 64) as u64, sign, sign])
    }
}

impl From<u128> for I256 {
    #[inline]
    fn from(v: u128) -> Self {
        Self([v as u64, (v >> 64) as u64, 0, 0])
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
        Self(limbs)
    }
}

impl From<I256> for [u64; 4] {
    #[inline]
    fn from(val: I256) -> [u64; 4] {
        val.0
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
