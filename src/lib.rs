#![feature(bigint_helper_methods)]
#![feature(portable_simd)]
#![feature(core_intrinsics)]
#![feature(stdsimd)]

mod u128x8;
mod u192;
mod u192x8;

pub use self::u128x8::U128x8;
pub use self::u192::U192;
pub use self::u192x8::U192x8;

// N  = 256
// M1 = 20220217214410
//    = 2 * 5 * 431 * 46589 * 100699
// M2 = 104648257118348370704723119
//    = prime
// M3 = 125000000000000140750000000000052207500000000006359661
//    = 500000000000000147 * 500000000000000207 * 500000000000000209
// M4 = a hidden but fixed integer, whose prime factors include and only include 3, 7 and 11
//    = 3^50 * 7^30 * 11^20
pub const N: usize = 256;
pub const M1: u64 = 20220217214410;
pub const M1_1: u32 = 431 * 46589;
pub const M1_2: u32 = 2 * 5 * 100699;
pub const M2: u128 = 104648257118348370704723119;
pub const M3: U192 = U192([0x32b9c8672a627dd5, 0x959989af0854b90, 0x14e1878814c9d]); // old
pub const M3_1: u64 = 500000000000000147;
pub const M3_2: u64 = 500000000000000207;
pub const M3_3: u64 = 500000000000000209;
pub const M4_3: u128 = 717897987691852588770249;
pub const M4_7: u128 = 22539340290692258087863249;
pub const M4_11: u128 = 672749994932560009201;
pub const M4_TEST: u32 = 43046721; // 3^16

use std::simd::{u32x16, u64x8};

#[inline]
pub fn rem_u32x16(x: u32x16, m: u32) -> u32x16 {
    #[cfg(target_feature = "avx512f")]
    unsafe {
        use std::arch::x86_64::_mm512_min_epu32;
        use std::mem::transmute;
        let mut x = transmute(x);
        x = _mm512_min_epu32(x, transmute(u32x16::from(x) - u32x16::splat(m * 4)));
        x = _mm512_min_epu32(x, transmute(u32x16::from(x) - u32x16::splat(m * 4)));
        x = _mm512_min_epu32(x, transmute(u32x16::from(x) - u32x16::splat(m * 2)));
        x = _mm512_min_epu32(x, transmute(u32x16::from(x) - u32x16::splat(m * 1)));
        u32x16::from(x)
    }
    #[cfg(not(target_feature = "avx512f"))]
    {
        // XXX: use std::cmp::Ord? WTF
        let mut x = x;
        x = x.min(x - u32x16::splat(m * 8));
        x = x.min(x - u32x16::splat(m * 4));
        x = x.min(x - u32x16::splat(m * 2));
        x = x.min(x - u32x16::splat(m * 1));
        x
    }
}

#[inline]
pub fn rem_u64x8(x: u64x8, m: u64) -> u64x8 {
    #[cfg(target_feature = "avx512f")]
    unsafe {
        use std::arch::x86_64::_mm512_min_epu64;
        use std::mem::transmute;
        let mut x = transmute(x);
        x = _mm512_min_epu64(x, transmute(u64x8::from(x) - u64x8::splat(m * 4)));
        x = _mm512_min_epu64(x, transmute(u64x8::from(x) - u64x8::splat(m * 4)));
        x = _mm512_min_epu64(x, transmute(u64x8::from(x) - u64x8::splat(m * 2)));
        x = _mm512_min_epu64(x, transmute(u64x8::from(x) - u64x8::splat(m * 1)));
        u64x8::from(x)
    }
    #[cfg(not(target_feature = "avx512f"))]
    {
        // XXX: use std::cmp::Ord? WTF
        let mut x = x;
        x = x.min(x - u64x8::splat(m * 8));
        x = x.min(x - u64x8::splat(m * 4));
        x = x.min(x - u64x8::splat(m * 2));
        x = x.min(x - u64x8::splat(m * 1));
        x
    }
}

#[inline]
pub fn rem_u192x8_m3(mut x: U192x8) -> U192x8 {
    const MX8: U192x8 = U192x8::splat(M3.mul(8).0);
    const MX4: U192x8 = U192x8::splat(M3.mul(4).0);
    const MX2: U192x8 = U192x8::splat(M3.mul(2).0);
    const MX1: U192x8 = U192x8::splat(M3.0);
    x = x.sub_on_ge(MX8);
    x = x.sub_on_ge(MX4);
    x = x.sub_on_ge(MX2);
    x = x.sub_on_ge(MX1);
    x
}

#[inline]
pub fn rem_u128(x: u128, m: u128) -> u128 {
    if x >= m * 5 {
        if x >= m * 7 {
            if x >= m * 9 {
                x - m * 9
            } else if x >= m * 8 {
                x - m * 8
            } else {
                x - m * 7
            }
        } else {
            if x >= m * 6 {
                x - m * 6
            } else {
                x - m * 5
            }
        }
    } else {
        if x >= m * 2 {
            if x >= m * 4 {
                x - m * 4
            } else if x >= m * 3 {
                x - m * 3
            } else {
                x - m * 2
            }
        } else {
            if x >= m * 1 {
                x - m * 1
            } else {
                x
            }
        }
    }
}

#[inline]
pub fn rem_u192_m3(x: U192) -> U192 {
    const M3S: [U192; 10] = [
        M3.mul(0),
        M3.mul(1),
        M3.mul(2),
        M3.mul(3),
        M3.mul(4),
        M3.mul(5),
        M3.mul(6),
        M3.mul(7),
        M3.mul(8),
        M3.mul(9),
    ];
    if x >= M3S[5] {
        if x >= M3S[7] {
            if x >= M3S[9] {
                x - M3S[9]
            } else if x >= M3S[8] {
                x - M3S[8]
            } else {
                x - M3S[7]
            }
        } else {
            if x >= M3S[6] {
                x - M3S[6]
            } else {
                x - M3S[5]
            }
        }
    } else {
        if x >= M3S[2] {
            if x >= M3S[4] {
                x - M3S[4]
            } else if x >= M3S[3] {
                x - M3S[3]
            } else {
                x - M3S[2]
            }
        } else {
            if x >= M3S[1] {
                x - M3S[1]
            } else {
                x
            }
        }
    }
}
