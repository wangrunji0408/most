#![feature(bigint_helper_methods)]
#![feature(portable_simd)]

mod u128x8;

pub use self::u128x8::U128x8;

// use core::simd::u64x4;
use std::ops::{Add, BitAnd, Shl, Sub, SubAssign};

/// Little-endian 256 bits unsigned integer.
#[derive(Default, Debug, PartialEq, Eq, Copy, Clone)]
#[repr(transparent)]
pub struct U256(pub [u64; 4]);

impl U256 {
    pub const ZERO: Self = Self::new([0; 4]);
    pub const MAX: Self = Self::new([u64::MAX; 4]);

    #[inline]
    pub const fn new(x: [u64; 4]) -> Self {
        U256(x)
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        *self == Self::ZERO
    }

    #[inline]
    pub const fn mul(self, x: u64) -> Self {
        let [a0, a1, a2, a3] = self.0;
        let c0 = a0 as u128 * x as u128;
        let c1 = a1 as u128 * x as u128 + (c0 >> 64);
        let c2 = a2 as u128 * x as u128 + (c1 >> 64);
        let c3 = a3 as u128 * x as u128 + (c2 >> 64);
        U256([c0 as u64, c1 as u64, c2 as u64, c3 as u64])
    }
}

impl Add for U256 {
    type Output = U256;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let [a0, a1, a2, a3] = self.0;
        let [b0, b1, b2, b3] = rhs.0;
        let (c0, carry) = a0.carrying_add(b0, false);
        let (c1, carry) = a1.carrying_add(b1, carry);
        let (c2, carry) = a2.carrying_add(b2, carry);
        let (c3, _) = a3.carrying_add(b3, carry);
        U256::new([c0, c1, c2, c3])
    }
}

impl Add<u8> for U256 {
    type Output = U256;

    #[inline]
    fn add(self, rhs: u8) -> Self::Output {
        self + U256([rhs as u64, 0, 0, 0])
    }
}

impl Sub for U256 {
    type Output = U256;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let [a0, a1, a2, a3] = self.0;
        let [b0, b1, b2, b3] = rhs.0;
        let (c0, carry) = a0.carrying_add(!b0, true);
        let (c1, carry) = a1.carrying_add(!b1, carry);
        let (c2, carry) = a2.carrying_add(!b2, carry);
        let (c3, _) = a3.carrying_add(!b3, carry);
        U256::new([c0, c1, c2, c3])
    }
}

impl SubAssign for U256 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Shl<u8> for U256 {
    type Output = U256;

    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        let [a0, a1, a2, a3] = self.0;
        let c0 = a0 << rhs;
        let c1 = (a1 << rhs) | (a0 >> (64 - rhs));
        let c2 = (a2 << rhs) | (a1 >> (64 - rhs));
        let c3 = (a3 << rhs) | (a2 >> (64 - rhs));
        U256([c0, c1, c2, c3])
    }
}

impl BitAnd for U256 {
    type Output = U256;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        let [a0, a1, a2, a3] = self.0;
        let [b0, b1, b2, b3] = rhs.0;
        U256([a0 & b0, a1 & b1, a2 & b2, a3 & b3])
    }
}

impl PartialOrd for U256 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for U256 {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let [a0, a1, a2, a3] = self.0;
        let [b0, b1, b2, b3] = other.0;
        [a3, a2, a1, a0].cmp(&[b3, b2, b1, b0])
    }
}

#[test]
fn add_u8() {
    assert_eq!(U256::new([u64::MAX, 2, 3, 4]) + 1, U256::new([0, 3, 3, 4]));
}

#[test]
fn sub() {
    assert_eq!(
        U256::new([u64::MAX, 2, 3, 4]) - U256::new([0, 2, 3, 4]),
        U256::new([u64::MAX, 0, 0, 0])
    );
}

#[test]
fn cmp() {
    assert!(
        U256([0x51d8e60e0337297d, 0, 0, 0])
            < U256([
                0x32b9c8672a627dd5,
                0x0959989af0854b90,
                0x00014e1878814c9d,
                0
            ])
    );
}

#[test]
fn shl() {
    assert_eq!(
        U256::new([
            0x01234567_89ABCDEF,
            0x01234567_89ABCDEF,
            0x01234567_89ABCDEF,
            0x01234567_89ABCDEF
        ]) << 24,
        U256::new([
            0x6789ABCD_EF000000,
            0x6789ABCD_EF012345,
            0x6789ABCD_EF012345,
            0x6789ABCD_EF012345
        ])
    );
}
