use std::ops::{Add, BitAnd, BitOr, Sub};
use std::simd::{mask64x8, u64x8};

/// Vector of eight u128 values.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct U128x8 {
    hi: u64x8,
    // low 60-bits used
    lo: u64x8,
}

const MASK: u64 = (1 << 60) - 1;

impl U128x8 {
    pub const ZERO: Self = Self::splat(0);
    pub const MAX: Self = Self::splat(u128::MAX);

    #[inline]
    pub fn is_zero(self) -> mask64x8 {
        (self.hi | self.lo).lanes_eq(u64x8::splat(0))
    }

    #[inline]
    pub const fn splat(x: u128) -> Self {
        Self {
            hi: u64x8::splat((x >> 60) as u64),
            lo: u64x8::splat((x as u64) & MASK),
        }
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
        self.hi[index] = (value >> 60) as _;
        self.lo[index] = (value as u64) & MASK;
    }

    #[inline]
    pub fn mul10_add(self, b: u64) -> Self {
        let lo = self.lo * u64x8::splat(10) + u64x8::splat(b);
        let hi = self.hi * u64x8::splat(10) + (lo >> u64x8::splat(60));
        Self {
            hi,
            lo: lo & u64x8::splat(MASK),
        }
    }
}

impl Add for U128x8 {
    type Output = U128x8;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let lo = self.lo + rhs.lo;
        let hi = self.hi + rhs.hi + (lo >> u64x8::splat(60));
        Self {
            hi,
            lo: lo & u64x8::splat(MASK),
        }
    }
}

impl Sub for U128x8 {
    type Output = U128x8;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let lo = self.lo - rhs.lo;
        let hi = self.hi - rhs.hi - (lo >> u64x8::splat(63));
        Self {
            hi,
            lo: lo & u64x8::splat(MASK),
        }
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
