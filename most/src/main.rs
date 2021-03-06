#![feature(core_intrinsics)]
#![feature(portable_simd)]
#![feature(stdsimd)]

use most::*;
use std::intrinsics::unlikely;
use std::io::{IoSlice, Read, Write};
use std::net::TcpStream;
use std::simd::{u32x16, u64x8};
use std::time::{Duration, Instant};

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
        if let Some(idx) = task4.append(bytes, t0, &mut send_tcp) {
            bytes = &bytes[idx..];
            task1.clear();
            task2.clear();
            task3.clear();
            task4.clear();
            continue;
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
        bytes = &buf[..0];
    }
}

trait Data: Default {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize;
    fn check(&mut self, digits: impl Iterator<Item = u8>) -> bool;
}

#[derive(Default)]
struct M1Data {
    f: [u32x16; N / 16],
}

impl Data for M1Data {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize {
        self.f[len % N / 16][len % 16] = 0;
        let mut zpos = 0;
        for (i, f1) in self.f.iter_mut().enumerate() {
            let ff1 = rem_u32x16(*f1 * u32x16::splat(10) + u32x16::splat(x as _), M1_1);
            *f1 = ff1;
            let zeros = ff1.lanes_eq(u32x16::default());
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

    fn check(&mut self, digits: impl Iterator<Item = u8>) -> bool {
        let mut f = 0;
        for x in digits {
            f = (f * 10 + x as u32) % M1_2;
        }
        f == 0
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

    fn check(&mut self, _digits: impl Iterator<Item = u8>) -> bool {
        true
    }
}

#[derive(Default)]
struct M3Data {
    f: [u64x8; N / 8],
}

impl Data for M3Data {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize {
        self.f[len % N / 8][len % 8] = 0;
        let mut zpos = 0;
        for (i, f1) in self.f.iter_mut().enumerate() {
            let ff1 = rem_u64x8(*f1 * u64x8::splat(10) + u64x8::splat(x as _), M3_1);
            *f1 = ff1;
            let zeros = ff1.lanes_eq(u64x8::default());
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

    fn check(&mut self, _digits: impl Iterator<Item = u8>) -> bool {
        true
        // consume < 1us, no false positive observed, disable it
        // let mut f2 = 0;
        // let mut f3 = 0;
        // for x in digits {
        //     f2 = (f2 * 10 + x as u64) % M3_2;
        //     f3 = (f3 * 10 + x as u64) % M3_3;
        // }
        // f2 == 0 && f3 == 0
    }
}

#[derive(Default)]
struct M4Data {
    f: [u32x16; N / 16],
}

impl Data for M4Data {
    fn push(&mut self, x: u8, len: usize, zbuf: &mut [u16]) -> usize {
        self.f[len % N / 16][len % 16] = 0;
        let mut zpos = 0;
        for (i, f1) in self.f.iter_mut().enumerate() {
            let ff1 = rem_u32x16(*f1 * u32x16::splat(10) + u32x16::splat(x as _), M4_TEST);
            *f1 = ff1;
            let zeros = ff1.lanes_eq(u32x16::default());
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

    fn check(&mut self, digits: impl Iterator<Item = u8>) -> bool {
        // consume < 2.5us
        let mut f1 = 0;
        let mut f2 = 0;
        let mut f3 = 0;
        for x in digits {
            f1 = rem_u128(f1 * 10 + x as u128, M4_3);
            f2 = rem_u128(f2 * 10 + x as u128, M4_7);
            f3 = rem_u128(f3 * 10 + x as u128, M4_11);
        }
        f1 == 0 && f2 == 0 && f3 == 0
    }
}

struct Task<T: Data> {
    stat: Stat,
    deque: [u8; N],
    len: usize,
    f: T,
    k: u8,
}

impl<T: Data> Task<T> {
    fn new(k: u8) -> Self {
        Task {
            stat: Stat::new(),
            deque: [0; N],
            f: T::default(),
            len: 0,
            k,
        }
    }

    fn clear(&mut self) {
        self.len = 0;
        // no need to clear `f`
    }

    /// If found return the end index.
    fn append(&mut self, bytes: &[u8], t0: Instant, tcp: &mut TcpStream) -> Option<usize> {
        let mut zbuf = [unsafe { std::mem::MaybeUninit::uninit().assume_init() }; N];
        let mut iter = bytes.iter().enumerate();
        while let Some((idx, &b)) = iter.next() {
            self.deque[self.len % N] = b;
            let x = b - b'0';

            let zpos = self.f.push(x, self.len, &mut zbuf);

            self.len += 1;

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let pos = self.len % N;
                let len = if i < pos { pos - i } else { N - (i - pos) };
                if i >= self.len || self.deque[i] == b'0' {
                    continue;
                }
                if !self
                    .f
                    .check((0..len).map(|j| self.deque[(i + j) % N] - b'0'))
                {
                    log::debug!("M{} false positive", self.k);
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
                send(tcp, i, len, zeros, &self.deque);
                self.stat.add(self.k, len, zeros, t0);
                return Some(idx + zeros);
            }
        }
        None
    }
}

fn send(tcp: &mut TcpStream, i0: usize, len: usize, zeros: usize, deque: &[u8; N]) {
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
            IoSlice::new(&deque[i0..(i0 + len).min(N)]),
            IoSlice::new(&deque[..(i0 + len).max(N) - N]),
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
