use std::ops::{Add, BitAnd, BitOr, Shl, Sub, SubAssign};
use std::simd::{mask64x8, u64x8};

/// Vector of eight u128 values.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct U128x8 {
    hi: u64x8,
    lo: u64x8,
}

impl U128x8 {
    pub const ZERO: Self = Self::from_array([0; 8]);
    pub const MAX: Self = Self::from_array([u128::MAX; 8]);

    #[inline]
    pub const fn from_array(x: [u128; 8]) -> Self {
        let [x0, x1, x2, x3, x4, x5, x6, x7] = x;
        const fn hi(x: u128) -> u64 {
            (x >> 64) as u64
        }
        const fn lo(x: u128) -> u64 {
            x as u64
        }
        Self {
            hi: u64x8::from_array([
                hi(x0),
                hi(x1),
                hi(x2),
                hi(x3),
                hi(x4),
                hi(x5),
                hi(x6),
                hi(x7),
            ]),
            lo: u64x8::from_array([
                lo(x0),
                lo(x1),
                lo(x2),
                lo(x3),
                lo(x4),
                lo(x5),
                lo(x6),
                lo(x7),
            ]),
        }
    }

    #[inline]
    pub fn is_zero(self) -> mask64x8 {
        (self.hi | self.lo).lanes_eq(u64x8::splat(0))
    }

    #[inline]
    pub const fn splat(x: u128) -> Self {
        Self {
            hi: u64x8::splat((x >> 64) as _),
            lo: u64x8::splat(x as _),
        }
    }

    #[inline]
    pub fn lanes_eq(self, other: Self) -> mask64x8 {
        let hi_eq = self.hi.lanes_eq(other.hi);
        let lo_eq = self.lo.lanes_eq(other.lo);
        hi_eq & lo_eq
    }

    #[inline]
    pub fn lanes_gt(self, other: Self) -> mask64x8 {
        let hi_eq = self.hi.lanes_eq(other.hi);
        let hi_gt = self.hi.lanes_gt(other.hi);
        let lo_gt = self.lo.lanes_gt(other.lo);
        hi_eq.select_mask(lo_gt, hi_gt)
    }

    #[inline]
    pub fn sub_on_ge(self, other: Self) -> Self {
        let underflow = other.lanes_gt(self);
        if underflow.all() {
            return self;
        }
        let c = self - other;
        Self {
            hi: underflow.select(self.hi, c.hi),
            lo: underflow.select(self.lo, c.lo),
        }
    }

    #[inline]
    pub fn set(&mut self, index: usize, value: u128) {
        self.hi[index] = (value >> 64) as _;
        self.lo[index] = value as _;
    }
}

impl Add for U128x8 {
    type Output = U128x8;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let lo = self.lo + rhs.lo;
        let carry: u64x8 = unsafe { std::mem::transmute(lo.lanes_lt(rhs.lo).to_int()) };
        let hi = self.hi + rhs.hi - carry;
        Self { hi, lo }
    }
}

impl Add<u8> for U128x8 {
    type Output = U128x8;

    #[inline]
    fn add(self, rhs: u8) -> Self::Output {
        self + U128x8::splat(rhs as _)
    }
}

impl Sub for U128x8 {
    type Output = U128x8;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let lo = self.lo - rhs.lo;
        let carry: u64x8 = unsafe { std::mem::transmute(rhs.lo.lanes_gt(self.lo).to_int()) };
        let hi = self.hi - rhs.hi + carry;
        Self { hi, lo }
    }
}

impl SubAssign for U128x8 {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Shl<u8> for U128x8 {
    type Output = U128x8;

    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        let lo = self.lo << u64x8::splat(rhs as _);
        let hi = self.hi << u64x8::splat(rhs as _);
        let hi = hi | (self.lo >> u64x8::splat(64 - rhs as u64));
        Self { hi, lo }
    }
}

impl BitAnd for U128x8 {
    type Output = U128x8;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        let hi = self.hi & rhs.hi;
        let lo = self.lo & rhs.lo;
        Self { hi, lo }
    }
}

impl BitOr for U128x8 {
    type Output = U128x8;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        let hi = self.hi | rhs.hi;
        let lo = self.lo | rhs.lo;
        Self { hi, lo }
    }
}

#[test]
fn add_u8() {
    assert_eq!(
        U128x8::from_array([0, 1, 2, 3, 4, 5, 6, u128::MAX]) + 1,
        U128x8::from_array([1, 2, 3, 4, 5, 6, 7, 0])
    );
}

#[test]
fn sub() {
    assert_eq!(
        U128x8::from_array([u128::MAX, 1, 0, 0, 0, 0, 0, 0])
            - U128x8::from_array([0, 2, 0, 0, 0, 0, 0, 0]),
        U128x8::from_array([u128::MAX, u128::MAX, 0, 0, 0, 0, 0, 0])
    );
}
