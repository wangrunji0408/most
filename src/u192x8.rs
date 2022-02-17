use std::ops::{Add, BitAnd, BitOr, Sub};
use std::simd::{mask64x8, u64x8};

/// Vector of eight u192 values.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct U192x8 {
    hi: u64x8,
    // low 60-bits used
    mi: u64x8,
    // low 60-bits used
    lo: u64x8,
}

const MASK: u64 = (1 << 60) - 1;

impl U192x8 {
    pub const ZERO: Self = Self::splat([0; 3]);
    pub const MAX: Self = Self::splat([u64::MAX; 3]);

    #[inline]
    pub const fn splat(x: [u64; 3]) -> Self {
        let [a0, a1, a2] = x;
        let a2 = (a2 << 8) | (a1 >> (64 - 8));
        let a1 = (a1 << 4) | (a0 >> (64 - 4));
        Self {
            hi: u64x8::splat(a2 & MASK),
            mi: u64x8::splat(a1 & MASK),
            lo: u64x8::splat(a0 & MASK),
        }
    }

    #[inline]
    pub fn lanes_gt(self, other: Self) -> mask64x8 {
        let hi_eq = self.hi.lanes_eq(other.hi);
        let hi_gt = self.hi.lanes_gt(other.hi);
        if !hi_eq.any() {
            return hi_gt;
        }
        let mi_eq = self.mi.lanes_eq(other.mi);
        let mi_gt = self.mi.lanes_gt(other.mi);
        let lo_gt = self.lo.lanes_gt(other.lo);
        hi_eq.select_mask(mi_eq.select_mask(lo_gt, mi_gt), hi_gt)
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
            mi: underflow.select(self.mi, c.mi),
            lo: underflow.select(self.lo, c.lo),
        }
    }

    #[inline]
    pub fn mul10_add(self, b: u64) -> Self {
        let lo = self.lo * u64x8::splat(10) + u64x8::splat(b);
        let mi = self.mi * u64x8::splat(10) + (lo >> u64x8::splat(60));
        let hi = self.hi * u64x8::splat(10) + (mi >> u64x8::splat(60));
        Self {
            hi: hi & u64x8::splat(MASK),
            mi: mi & u64x8::splat(MASK),
            lo: lo & u64x8::splat(MASK),
        }
    }
}

impl Add for U192x8 {
    type Output = U192x8;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let lo = self.lo + rhs.lo;
        let mi = self.mi + rhs.mi + (lo >> u64x8::splat(60));
        let hi = self.hi + rhs.hi + (mi >> u64x8::splat(60));
        Self {
            hi: hi & u64x8::splat(MASK),
            mi: mi & u64x8::splat(MASK),
            lo: lo & u64x8::splat(MASK),
        }
    }
}

impl Sub for U192x8 {
    type Output = U192x8;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let lo = self.lo - rhs.lo;
        let mi = self.mi - rhs.mi - (lo >> u64x8::splat(63));
        let hi = self.hi - rhs.hi - (mi >> u64x8::splat(63));
        Self {
            hi: hi & u64x8::splat(MASK),
            mi: mi & u64x8::splat(MASK),
            lo: lo & u64x8::splat(MASK),
        }
    }
}

impl BitAnd for U192x8 {
    type Output = U192x8;

    #[inline]
    fn bitand(self, rhs: Self) -> Self::Output {
        let hi = self.hi & rhs.hi;
        let mi = self.mi & rhs.mi;
        let lo = self.lo & rhs.lo;
        Self { hi, mi, lo }
    }
}

impl BitOr for U192x8 {
    type Output = U192x8;

    #[inline]
    fn bitor(self, rhs: Self) -> Self::Output {
        let hi = self.hi | rhs.hi;
        let mi = self.mi | rhs.mi;
        let lo = self.lo | rhs.lo;
        Self { hi, mi, lo }
    }
}
