use std::ops::{Add, BitAnd, BitOr, Shl, Sub};
use std::simd::{mask64x8, u64x8};

/// Vector of eight u192 values.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct U192x8 {
    hi: u64x8,
    mi: u64x8,
    lo: u64x8,
}

impl U192x8 {
    pub const ZERO: Self = Self::splat([0; 3]);
    pub const MAX: Self = Self::splat([u64::MAX; 3]);

    #[inline]
    pub const fn from_array(x: [u128; 8]) -> Self {
        let [x0, x1, x2, x3, x4, x5, x6, x7] = x;
        const fn mi(x: u128) -> u64 {
            (x >> 64) as u64
        }
        const fn lo(x: u128) -> u64 {
            x as u64
        }
        Self {
            hi: u64x8::from_array([0; 8]),
            mi: u64x8::from_array([
                mi(x0),
                mi(x1),
                mi(x2),
                mi(x3),
                mi(x4),
                mi(x5),
                mi(x6),
                mi(x7),
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
    pub const fn splat(x: [u64; 3]) -> Self {
        Self {
            hi: u64x8::splat(x[2]),
            mi: u64x8::splat(x[1]),
            lo: u64x8::splat(x[0]),
        }
    }

    #[inline]
    pub fn lanes_gt(self, other: Self) -> mask64x8 {
        let hi_eq = self.hi.lanes_eq(other.hi);
        let mi_eq = self.mi.lanes_eq(other.mi);
        let hi_gt = self.hi.lanes_gt(other.hi);
        let mi_gt = self.mi.lanes_gt(other.mi);
        let lo_gt = self.lo.lanes_gt(other.lo);
        hi_eq.select_mask(mi_eq.select_mask(lo_gt, mi_gt), hi_gt)
    }

    #[inline]
    pub fn sub_on_ge(self, other: Self) -> Self {
        let c = self - other;
        let underflow = other.lanes_gt(self);
        Self {
            hi: underflow.select(self.hi, c.hi),
            mi: underflow.select(self.mi, c.mi),
            lo: underflow.select(self.lo, c.lo),
        }
    }
}

impl Add for U192x8 {
    type Output = U192x8;

    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        let lo = self.lo + rhs.lo;
        let lo_carry = lo.lanes_lt(rhs.lo);
        let mi = self.mi + rhs.mi - to_u64x8(lo_carry);
        let mi_carry = mi.lanes_lt(rhs.mi) | (self.mi.lanes_eq(u64x8::splat(u64::MAX)) & lo_carry);
        let hi = self.hi + rhs.hi - to_u64x8(mi_carry);
        Self { hi, mi, lo }
    }
}

impl Add<u8> for U192x8 {
    type Output = U192x8;

    #[inline]
    fn add(self, rhs: u8) -> Self::Output {
        self + U192x8::splat([rhs as _, 0, 0])
    }
}

impl Sub for U192x8 {
    type Output = U192x8;

    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        let lo = self.lo - rhs.lo;
        let lo_carry = self.lo.lanes_lt(rhs.lo);
        let mi = self.mi - rhs.mi + to_u64x8(lo_carry);
        let mi_carry = self.mi.lanes_lt(rhs.mi) | (self.mi.lanes_eq(rhs.mi) & lo_carry);
        let hi = self.hi - rhs.hi + to_u64x8(mi_carry);
        Self { hi, mi, lo }
    }
}

fn to_u64x8(mask: mask64x8) -> u64x8 {
    unsafe { std::mem::transmute(mask.to_int()) }
}

impl Shl<u8> for U192x8 {
    type Output = U192x8;

    #[inline]
    fn shl(self, rhs: u8) -> Self::Output {
        let lbit = u64x8::splat(rhs as _);
        let rbit = u64x8::splat(64 - rhs as u64);
        let lo = self.lo << lbit;
        let mi = (self.mi << lbit) | (self.lo >> rbit);
        let hi = (self.hi << lbit) | (self.mi >> rbit);
        Self { hi, mi, lo }
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
