#![no_std]

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
const HASH_SIZE: usize = 1 << 12;

pub trait Data: Default {
    fn push(&mut self, x: u8) -> Option<usize>;
    // fn check(&mut self, digits: impl Iterator<Item = u8>) -> bool;
    // fn prepare(&mut self);
}

pub struct M1Data {
    r: u32,
    t: [u32; N],
    x: [u8; N],
    i: usize,
    tset: [u8; HASH_SIZE],
    // rtable: [[u64; 9]; N],
}

impl Default for M1Data {
    fn default() -> Self {
        Self {
            r: 1,
            t: [0; N],
            x: [0; N],
            i: 0,
            tset: [0; HASH_SIZE],
        }
    }
}

impl Data for M1Data {
    fn push(&mut self, x: u8) -> Option<usize> {
        let t0 = self.t[self.i % N];
        self.i += 1;
        self.r = (self.r as u64 * M1_R % M1_1 as u64) as u32;
        let t1 = ((t0 as u64 + x as u64 * self.r as u64) % M1_1 as u64) as u32;
        let tn = self.t[self.i % N];
        self.t[self.i % N] = t1;
        if self.i > N {
            self.tset[tn as usize % HASH_SIZE] = 0;
        }
        let hashv = &mut self.tset[t1 as usize % HASH_SIZE];
        let len = if *hashv != 0 {
            Some((self.i - *hashv as usize) % N)
        } else {
            None
        };
        *hashv = (self.i % N) as u8;
        len
    }
    // fn check(&mut self, digits: impl Iterator<Item = u8>) -> bool {

    // }
}
