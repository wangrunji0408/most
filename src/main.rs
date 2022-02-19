#![feature(core_intrinsics)]
#![feature(portable_simd)]
#![feature(stdsimd)]

use most::U128x8;
use std::collections::VecDeque;
use std::intrinsics::unlikely;
use std::io::{IoSlice, Read, Write};
use std::net::TcpStream;
use std::simd::{u32x16, u64x8};
use std::time::{Duration, Instant};

// N  = 256
// M1 = 20220217214410 = 2 * 5 * 431 * 46589 * 100699
// M2 = 104648257118348370704723119
// M3 = 125000000000000140750000000000052207500000000006359661
//    = 500000000000000147 * 500000000000000207 * 500000000000000209
// M4 = a hidden but fixed integer, whose prime factors include and only include 3, 7 and 11
const N: usize = 256;
const M1_1: u32 = 431 * 46589;
const M1_2: u32 = 2 * 5 * 100699;
const M2: u128 = 104648257118348370704723119;
const M3_1: u64 = 500000000000000147;
const M3_2: u64 = 500000000000000207;
const M3_3: u64 = 500000000000000209;
const M4_3: u128 = 717897987691852588770249;
const M4_7: u128 = 22539340290692258087863249;
const M4_11: u128 = 672749994932560009201;
// M4: 3^50 * 7^30 * 11^20

// const IN_IP: &str = "47.95.111.217:10001";  // public
// const IN_IP: &str = "172.1.1.119:10001"; // inner
// const OUT_IP: &str = "172.1.1.119:10002"; // inner
const IN_IP: &str = "127.0.0.1:10001"; // mock
const OUT_IP: &str = "127.0.0.1:10002"; // mock

fn main() {
    env_logger::init();

    let mut send_tcp = TcpStream::connect(OUT_IP).unwrap();
    send_tcp.set_nonblocking(true).unwrap();
    send_tcp.set_nodelay(true).unwrap();

    let mut get_tcp = TcpStream::connect(IN_IP).unwrap();
    get_tcp
        .write(format!("GET HTTP/1.1\r\nHost: {IN_IP}\r\n\r\n").as_bytes())
        .unwrap();
    const OK_HEADER: &str = "HTTP/1.1 200 OK\r\nServer: Most\r\nContent-type: text/plain\r\n\r\n";
    let mut buf = [0; 1024];
    let len = get_tcp.read(&mut buf[..OK_HEADER.len()]).unwrap();
    assert_eq!(&buf[..len], OK_HEADER.as_bytes());

    let mut task1 = Task::<M1Data>::new(1);
    let mut task2 = Task::<M2Data>::new(2);
    let mut task3 = Task::<M3Data>::new(3);
    let mut task4 = Task::<M4Data>::new(4);

    let mut buf = [0; 1024];
    let mut bytes = &buf[..0];
    let mut t0 = Instant::now();
    loop {
        if bytes.is_empty() {
            let len = get_tcp.read(&mut buf).unwrap();
            t0 = Instant::now();
            bytes = &buf[..len];
        }
        if let Some(idx) = task1.append(bytes, t0, &mut send_tcp) {
            bytes = &bytes[idx..];
            task1.clear();
            task2.clear();
            task3.clear();
            task4.clear();
            continue;
        }
        if let Some(idx) = task3.append(bytes, t0, &mut send_tcp) {
            bytes = &bytes[idx..];
            task1.clear();
            task2.clear();
            task3.clear();
            task4.clear();
            continue;
        }
        if let Some(idx) = task2.append(bytes, t0, &mut send_tcp) {
            bytes = &bytes[idx..];
            task1.clear();
            task2.clear();
            task3.clear();
            task4.clear();
            continue;
        }
        if let Some(idx) = task4.append(bytes, t0, &mut send_tcp) {
            bytes = &bytes[idx..];
            task1.clear();
            task2.clear();
            task3.clear();
            task4.clear();
            continue;
        }
        bytes = &buf[..0];
    }
}

trait Data: Default {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize;
}

#[derive(Default)]
struct M1Data {
    f: [(u32x16, u32x16); N / 16],
}

impl Data for M1Data {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize {
        #[inline]
        fn rem_u32x16(x: u32x16, m: u32) -> u32x16 {
            use std::arch::x86_64::_mm512_min_epu32;
            use std::mem::transmute;
            unsafe {
                let mut x = transmute(x);
                x = _mm512_min_epu32(x, transmute(u32x16::from(x) - u32x16::splat(m * 4)));
                x = _mm512_min_epu32(x, transmute(u32x16::from(x) - u32x16::splat(m * 4)));
                x = _mm512_min_epu32(x, transmute(u32x16::from(x) - u32x16::splat(m * 2)));
                x = _mm512_min_epu32(x, transmute(u32x16::from(x) - u32x16::splat(m * 1)));
                u32x16::from(x)
            }
        }

        self.f[len % N / 16].0[len % 16] = 0;
        self.f[len % N / 16].1[len % 16] = 0;
        let mut zpos = 0;
        for (i, (f1, f2)) in self.f.iter_mut().enumerate() {
            let ff1 = rem_u32x16(*f1 * u32x16::splat(10) + u32x16::splat(x as _), M1_1);
            let ff2 = rem_u32x16(*f2 * u32x16::splat(10) + u32x16::splat(x as _), M1_2);
            (*f1, *f2) = (ff1, ff2);
            let zeros = (ff1 | ff2).lanes_eq(u32x16::default());
            if unlikely(zeros.any()) {
                for j in 0..16 {
                    if zeros.test(j) && i * 16 + j <= len {
                        zbuf[zpos] = (i * 16 + j) as u16;
                        zpos += 1;
                    }
                }
            }
            if i * 16 > len {
                break;
            }
        }
        zpos
    }
}

#[derive(Default)]
struct M2Data {
    f: [U128x8; N / 8],
}

impl Data for M2Data {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize {
        self.f[len % N / 8].set(len % 8, 0);
        let mut zpos = 0;
        for (i, f) in self.f.iter_mut().enumerate() {
            let ff = f.mul10_add(x as _).rem10(M2);
            *f = ff;
            let zeros = ff.is_zero();
            if unlikely(zeros.any()) {
                for j in 0..8 {
                    if zeros.test(j) && i * 8 + j <= len {
                        zbuf[zpos] = (i * 8 + j) as u16;
                        zpos += 1;
                    }
                }
            }
            if i * 8 > len {
                break;
            }
        }
        zpos
    }
}

#[derive(Default)]
struct M3Data {
    f: [(u64x8, u64x8, u64x8); N / 8],
}

impl Data for M3Data {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize {
        #[inline]
        fn rem_u64x8(x: u64x8, m: u64) -> u64x8 {
            use std::arch::x86_64::_mm512_min_epu64;
            use std::mem::transmute;
            unsafe {
                let mut x = transmute(x);
                x = _mm512_min_epu64(x, transmute(u64x8::from(x) - u64x8::splat(m * 8)));
                x = _mm512_min_epu64(x, transmute(u64x8::from(x) - u64x8::splat(m * 4)));
                x = _mm512_min_epu64(x, transmute(u64x8::from(x) - u64x8::splat(m * 2)));
                x = _mm512_min_epu64(x, transmute(u64x8::from(x) - u64x8::splat(m * 1)));
                u64x8::from(x)
            }
        }

        self.f[len % N / 8].0[len % 8] = 0;
        self.f[len % N / 8].1[len % 8] = 0;
        self.f[len % N / 8].2[len % 8] = 0;
        let mut zpos = 0;
        for (i, (f1, f2, f3)) in self.f.iter_mut().enumerate() {
            let ff1 = rem_u64x8(*f1 * u64x8::splat(10) + u64x8::splat(x as _), M3_1);
            let ff2 = rem_u64x8(*f2 * u64x8::splat(10) + u64x8::splat(x as _), M3_2);
            let ff3 = rem_u64x8(*f3 * u64x8::splat(10) + u64x8::splat(x as _), M3_3);
            (*f1, *f2, *f3) = (ff1, ff2, ff3);
            let zeros = (ff1 | ff2 | ff3).lanes_eq(u64x8::default());
            if unlikely(zeros.any()) {
                for j in 0..8 {
                    if zeros.test(j) && i * 8 + j <= len {
                        zbuf[zpos] = (i * 8 + j) as u16;
                        zpos += 1;
                    }
                }
            }
            if i * 8 > len {
                break;
            }
        }
        zpos
    }
}

#[derive(Default)]
struct M4Data {
    f: [(U128x8, U128x8, U128x8); N / 8],
}

impl Data for M4Data {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize {
        self.f[len % N / 8].0.set(len % 8, 0);
        self.f[len % N / 8].1.set(len % 8, 0);
        self.f[len % N / 8].2.set(len % 8, 0);
        let mut zpos = 0;
        for (i, (f3, f7, f11)) in self.f.iter_mut().enumerate() {
            let ff3 = f3.mul10_add(x as _).rem10(M4_3);
            let ff7 = f7.mul10_add(x as _).rem10(M4_7);
            let ff11 = f11.mul10_add(x as _).rem10(M4_11);
            (*f3, *f7, *f11) = (ff3, ff7, ff11);
            let zeros = (ff3 | ff7 | ff11).is_zero();
            if unlikely(zeros.any()) {
                for j in 0..8 {
                    if zeros.test(j) && i * 8 + j <= len {
                        zbuf[zpos] = (i * 8 + j) as u16;
                        zpos += 1;
                    }
                }
            }
            if i * 8 > len {
                break;
            }
        }
        zpos
    }
}

struct Task<T: Data> {
    stat: Stat,
    deque: VecDeque<u8>,
    len: usize,
    f: T,
    k: u8,
}

impl<T: Data> Task<T> {
    fn new(k: u8) -> Self {
        Task {
            stat: Stat::new(),
            deque: VecDeque::with_capacity(N),
            f: T::default(),
            len: 0,
            k,
        }
    }

    fn clear(&mut self) {
        self.deque.clear();
        self.len = 0;
        // no need to clear `f`
    }

    /// If found return the end index.
    fn append(&mut self, bytes: &[u8], t0: Instant, tcp: &mut TcpStream) -> Option<usize> {
        let mut zbuf = [unsafe { std::mem::MaybeUninit::uninit().assume_init() }; N];
        let mut iter = bytes.iter().enumerate();
        while let Some((idx, &b)) = iter.next() {
            if self.deque.len() == N {
                self.deque.pop_front();
            }
            self.deque.push_back(b);

            let x = b - b'0';

            let zpos = self.f.push(x, self.len, &mut zbuf);

            self.len += 1;

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let pos = self.len % N;
                let len = if i < pos { pos - i } else { N - (i - pos) };
                if i >= self.deque.len() || self.deque[self.deque.len() - len] == b'0' {
                    continue;
                }
                // tailing 0s
                let mut zeros = 0;
                while let Some((_, &b'0')) = iter.next() {
                    if len + zeros == N {
                        break;
                    }
                    zeros += 1;
                }
                send(tcp, len, zeros, &self.deque);
                self.stat.add(self.k, len, zeros, t0);
                return Some(idx + zeros);
            }
        }
        None
    }
}

fn send(tcp: &mut TcpStream, len: usize, zeros: usize, deque: &VecDeque<u8>) {
    let (mut n0, mut n1) = deque.as_slices();
    if n1.len() >= len {
        n0 = &[];
        n1 = &n1[n1.len() - len..];
    } else {
        n0 = &n0[deque.len() - len..];
    }
    const HEADER: &str = "POST /submit?user=omicron&passwd=y8J6IGKr HTTP/1.1\r\nHost: 172.1.1.119:10002\r\nUser-Agent: Go-http-client/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: ";
    let mut len_strs = vec![];
    for i in 0..=zeros {
        len_strs.push(format!("{}\r\n\r\n", len + i));
    }
    let mut iov = vec![];
    for i in 0..=zeros {
        iov.extend([
            IoSlice::new(HEADER.as_bytes()),
            IoSlice::new(len_strs[i].as_bytes()),
            IoSlice::new(n0),
            IoSlice::new(n1),
            IoSlice::new(&b"0000000000"[..i]),
        ]);
    }
    match tcp.write_vectored(&iov) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            log::warn!("TCP send would block, ignore");
        }
        Err(e) => panic!("{}", e),
    }
}

struct Stat {
    t00: Instant,
    dsum: Duration,
    count: u32,
}

impl Stat {
    fn new() -> Self {
        Stat {
            t00: Instant::now(),
            dsum: Duration::default(),
            count: 0,
        }
    }

    fn add(&mut self, k: u8, len: usize, zeros: usize, t0: Instant) {
        // statistics
        let latency = t0.elapsed();
        self.dsum += latency;
        self.count += 1;
        let avg = self.dsum / self.count;
        let nps = self.count as f32 / self.t00.elapsed().as_secs_f32();
        log::info!("M{k} {len:3}+{zeros}  lat: {latency:>9?}  avg: {avg:>9?}  nps: {nps:.3?}");
    }
}
