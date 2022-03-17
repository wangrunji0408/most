#![no_std]

#[macro_use]
extern crate alloc;

use log::*;

// N  = 256
// M1 = 20220311122858
//    = 2 7 11 887 6143 24097
// M2 = 104648257118348370704723401
//    = prime
// M3 = 125000000000000173750000000000080443500000000012405393
//    = 500000000000000221 * 500000000000000231 * 500000000000000243
// M4 = a hidden but fixed integer, whose prime factors include and only include 2, 3 and 7
//    = 2^75 * 3^50 * 7^25
//
// 10^(-1) = 1011015556143              mod M1/2
//         = 44885482                   mod M1_1
//         = 94183431406513533634251061 mod M2
//         = 450000000000000199         mod M3_1
//         = 450000000000000208         mod M3_2
//         = 150000000000000073         mod M3_3
//         = 38742049                   mod 3^16
pub const N: usize = 256;
const M1: u64 = 20220311122858;
const M1_1: u32 = 7 * 887 * 24097;
const M1_2: u32 = 2 * 11 * 6143;
const M2: u128 = 104648257118348370704723401;
const M3_1: u64 = 500000000000000221;
const M3_2: u64 = 500000000000000231;
const M3_3: u64 = 500000000000000243;
const M1_R: u64 = 44885482;
const M2_R: u128 = 94183431406513533634251061;
const M3_R: u64 = 450000000000000199;
const M4_1: u32 = 43046721; // 3^16
const M4_R: u64 = 38742049;
const HASH_SIZE: usize = 1 << 16;
const PRE_LEN: usize = 400;

pub trait Data: Default {
    fn push(&mut self, x: u8) -> Option<usize>;
    // fn check(&mut self, digits: impl Iterator<Item = u8>) -> bool;
    fn prepare(&mut self);
}

trait AsUsize {
    fn as_usize(self) -> usize;
}
impl AsUsize for u32 {
    fn as_usize(self) -> usize {
        self as usize
    }
}
impl AsUsize for u64 {
    fn as_usize(self) -> usize {
        self as usize
    }
}
impl AsUsize for u128 {
    fn as_usize(self) -> usize {
        self as usize
    }
}

struct WindowArray<T> {
    i: usize,
    t: [T; N],
    tset_i: [u8; HASH_SIZE],
    tset_v: [T; HASH_SIZE],
}

impl<T: Default + Copy> Default for WindowArray<T> {
    fn default() -> Self {
        Self {
            i: 0,
            t: [T::default(); N],
            tset_i: [0; HASH_SIZE],
            tset_v: [T::default(); HASH_SIZE],
        }
    }
}

impl<T: Default + Eq + Copy + AsUsize> WindowArray<T> {
    /// Push an element and return the distance to the nearest element equals to it.
    fn push(&mut self, x: T) -> Option<usize> {
        self.i += 1;
        let x0 = self.t[self.i % N];
        self.t[self.i % N] = x;
        // update hash table
        if self.i > N {
            self.tset_i[x0.as_usize() % HASH_SIZE] = 0;
            self.tset_v[x0.as_usize() % HASH_SIZE] = T::default();
        }
        let hashi = &mut self.tset_i[x.as_usize() % HASH_SIZE];
        let hashv = &mut self.tset_v[x.as_usize() % HASH_SIZE];
        let len = if *hashv == x {
            Some((self.i - *hashi as usize) % N)
        } else {
            None
        };
        *hashi = (self.i % N) as u8;
        *hashv = x;
        len
    }

    /// Get the last element.
    fn last(&self) -> T {
        self.t[self.i % N]
    }
}

pub struct M1Data {
    i: usize,
    window: WindowArray<u32>,
    pre_start: usize,
    rtable: [[u32; 10]; PRE_LEN],
}

impl Default for M1Data {
    fn default() -> Self {
        let mut s = Self {
            i: 0,
            window: Default::default(),
            pre_start: 0,
            rtable: [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]; PRE_LEN],
        };
        s.prepare();
        s
    }
}

impl Data for M1Data {
    fn push(&mut self, x: u8) -> Option<usize> {
        self.i += 1;
        let t0 = self.window.last();
        let mut t1 = t0 + self.rtable[self.i - self.pre_start][x as usize];
        if t1 >= M1_1 {
            t1 -= M1_1;
        }
        // trace!("{} t[{}] = {}", x, self.i, t1);
        let len = self.window.push(t1);
        match len {
            Some(l) if l >= 14 && x != 0 && x % 2 == 0 => Some(l),
            _ => None,
        }
    }

    fn prepare(&mut self) {
        let mut rs = self.rtable[self.i - self.pre_start];
        self.pre_start = self.i;
        for rr in &mut self.rtable {
            *rr = rs;
            rs = rs.map(|x| (x as u64 * M1_R % M1_1 as u64) as u32);
        }
    }
}

impl M1Data {
    /// For bench only.
    pub fn prepare_nop(&mut self) {
        self.pre_start = self.i;
    }
}

pub struct M2Data {
    i: usize,
    window: WindowArray<u128>,
    pre_start: usize,
    rtable: [[u128; 10]; PRE_LEN],
}

impl Default for M2Data {
    fn default() -> Self {
        let mut s = Self {
            i: 0,
            window: Default::default(),
            pre_start: 0,
            rtable: [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]; PRE_LEN],
        };
        s.prepare();
        s
    }
}

impl Data for M2Data {
    fn push(&mut self, x: u8) -> Option<usize> {
        self.i += 1;
        let t0 = self.window.last();
        let mut t1 = t0 + self.rtable[self.i - self.pre_start][x as usize];
        if t1 >= M2 {
            t1 -= M2;
        }
        // trace!("{} t[{}] = {}", x, self.i, t1);
        let len = self.window.push(t1);
        match len {
            Some(l) if l >= 27 && x != 0 => Some(l),
            _ => None,
        }
    }

    fn prepare(&mut self) {
        let mut rs = self.rtable[self.i - self.pre_start];
        self.pre_start = self.i;
        for rr in &mut self.rtable {
            *rr = rs;
            rs = rs.map(|mut x| {
                let c1 = x * u128_mod_u128(M2_R as u128 & 0xFFFF_FFFF, M2);
                x = u128_mod_u128(x << 32, M2);
                let c2 = x * u128_mod_u128((M2_R >> 32) as u128 & 0xFFFF_FFFF, M2);
                x = u128_mod_u128(x << 32, M2);
                let c3 = x * u128_mod_u128((M2_R >> 64) as u128 & 0xFFFF_FFFF, M2);
                u128_mod_u128(c1 + c2 + c3, M2)
            });
        }
    }
}

impl M2Data {
    /// For bench only.
    pub fn prepare_nop(&mut self) {
        self.pre_start = self.i;
    }
}

pub struct M3Data {
    i: usize,
    window: WindowArray<u64>,
    pre_start: usize,
    rtable: [[u64; 10]; PRE_LEN],
}

impl Default for M3Data {
    fn default() -> Self {
        let mut s = Self {
            i: 0,
            window: Default::default(),
            pre_start: 0,
            rtable: [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]; PRE_LEN],
        };
        s.prepare();
        s
    }
}

impl Data for M3Data {
    fn push(&mut self, x: u8) -> Option<usize> {
        self.i += 1;
        let t0 = self.window.last();
        let mut t1 = t0 + self.rtable[self.i - self.pre_start][x as usize];
        if t1 >= M3_1 {
            t1 -= M3_1;
        }
        // trace!("{} t[{}] = {}", x, self.i, t1);
        let len = self.window.push(t1);
        match len {
            Some(l) if l >= 54 && x != 0 => Some(l),
            _ => None,
        }
    }

    fn prepare(&mut self) {
        let mut rs = self.rtable[self.i - self.pre_start];
        self.pre_start = self.i;
        for rr in &mut self.rtable {
            *rr = rs;
            rs = rs.map(|x| u128_mod_u128(x as u128 * M3_R as u128, M3_1 as u128) as u64);
        }
    }
}

#[cfg(target_os = "uefi")]
#[inline]
fn u128_mod_u128(n: u128, d: u128) -> u128 {
    // WARN: u128 % u128 produce wrong result on UEFI
    extern "C" {
        #[allow(improper_ctypes)]
        fn __umodti3(n: u128, d: u128) -> u128;
    }
    unsafe { __umodti3(n, d) }
}
#[cfg(not(target_os = "uefi"))]
#[inline]
fn u128_mod_u128(n: u128, d: u128) -> u128 {
    n % d
}

impl M3Data {
    /// For bench only.
    pub fn prepare_nop(&mut self) {
        self.pre_start = self.i;
    }
}

pub struct M4Data {
    i: usize,
    window: WindowArray<u32>,
    pre_start: usize,
    rtable: [[u32; 10]; PRE_LEN],
}

impl Default for M4Data {
    fn default() -> Self {
        let mut s = Self {
            i: 0,
            window: Default::default(),
            pre_start: 0,
            rtable: [[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]; PRE_LEN],
        };
        s.prepare();
        s
    }
}

impl Data for M4Data {
    fn push(&mut self, x: u8) -> Option<usize> {
        self.i += 1;
        let t0 = self.window.last();
        let mut t1 = t0 + self.rtable[self.i - self.pre_start][x as usize];
        if t1 >= M4_1 {
            t1 -= M4_1;
        }
        // trace!("{} t[{}] = {}", x, self.i, t1);
        let len = self.window.push(t1);
        match len {
            Some(l) if l >= 68 && x != 0 => Some(l),
            _ => None,
        }
    }

    fn prepare(&mut self) {
        let mut rs = self.rtable[self.i - self.pre_start];
        self.pre_start = self.i;
        for rr in &mut self.rtable {
            *rr = rs;
            rs = rs.map(|x| (x as u64 * M4_R % M4_1 as u64) as u32);
        }
    }
}

impl M4Data {
    /// For bench only.
    pub fn prepare_nop(&mut self) {
        self.pre_start = self.i;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    #[test]
    fn m1() {
        env_logger::init();
        let mut state = M1Data::default();
        let m1str = (M1 * 7).to_string();
        for b in m1str[..m1str.len() - 1].bytes() {
            assert_eq!(state.push(b - b'0'), None);
        }
        assert_eq!(
            state.push(m1str.bytes().last().unwrap() - b'0'),
            Some(m1str.len())
        );
    }

    #[test]
    fn m2() {
        env_logger::init();
        let mut state = M2Data::default();
        let m2str = "357608048612464930233730043816992249527720913075773951798";
        for b in m2str[..m2str.len() - 1].bytes() {
            assert_eq!(state.push(b - b'0'), None);
        }
        assert_eq!(
            state.push(m2str.bytes().last().unwrap() - b'0'),
            Some(m2str.len())
        );
    }
}
