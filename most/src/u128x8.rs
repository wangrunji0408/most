use std::ops::{BitAnd, BitOr};
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
    pub fn sub_on_ge(self, other: Self) -> Self {
        let mut c_lo = self.lo - other.lo;
        let mut c_hi = self.hi - other.hi;
        let hi_eq = c_hi.lanes_eq(u64x8::default());
        let hi_ge = (c_hi & u64x8::splat(1 << 63)).lanes_eq(u64x8::default());
        let lo_ge = (c_lo >> u64x8::splat(63)).lanes_eq(u64x8::default());
        let ge = hi_eq.select_mask(lo_ge, hi_ge);
        c_hi = c_hi - (c_lo >> u64x8::splat(63));
        c_lo = c_lo & u64x8::splat(MASK);
        Self {
            hi: ge.select(c_hi, self.hi),
            lo: ge.select(c_lo, self.lo),
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

    #[inline]
    pub fn rem10(mut self, m: u128) -> Self {
        self = self.sub_on_ge(U128x8::splat(m * 4));
        self = self.sub_on_ge(U128x8::splat(m * 4));
        self = self.sub_on_ge(U128x8::splat(m * 2));
        self = self.sub_on_ge(U128x8::splat(m * 1));
        self
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
