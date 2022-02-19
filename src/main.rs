#![feature(core_intrinsics)]
#![feature(portable_simd)]
#![feature(stdsimd)]

use most::U128x8;
use std::collections::VecDeque;
use std::intrinsics::unlikely;
use std::io::{IoSlice, Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::simd::{u32x16, u64x8};
use std::sync::mpsc;
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
const IN_IP: &str = "127.0.0.1:10001"; // mock
const NO_SEND: bool = true;

fn main() {
    env_logger::init();
    let (tcp_tx, tcp_rx) = mpsc::sync_channel::<TcpStream>(8);
    std::thread::spawn(|| {
        if NO_SEND {
            std::mem::forget(tcp_tx);
            return;
        }
        loop {
            fn connect() -> std::io::Result<TcpStream> {
                let stream = TcpStream::connect("172.1.1.119:10002")?;
                stream.set_nonblocking(false)?;
                stream.set_nodelay(true)?;
                Ok(stream)
            }
            match connect() {
                Ok(s) => tcp_tx.send(s).unwrap(),
                Err(e) => log::error!("{}", e),
            }
            std::thread::sleep(Duration::from_secs(1));
        }
    });

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
        if let Some(idx) = task1.append(bytes, t0, &tcp_rx) {
            bytes = &bytes[idx..];
            task1.clear();
            task2.clear();
            task3.clear();
            task4.clear();
            continue;
        }
        if let Some(idx) = task3.append(bytes, t0, &tcp_rx) {
            bytes = &bytes[idx..];
            task1.clear();
            task2.clear();
            task3.clear();
            task4.clear();
            continue;
        }
        if let Some(idx) = task2.append(bytes, t0, &tcp_rx) {
            bytes = &bytes[idx..];
            task1.clear();
            task2.clear();
            task3.clear();
            task4.clear();
            continue;
        }
        if let Some(idx) = task4.append(bytes, t0, &tcp_rx) {
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
    fn push(&mut self, x: u8, pos: usize, zbuf: &mut [u16]) -> usize;
}

#[derive(Default)]
struct M1Data {
    f: [(u32x16, u32x16); N / 16],
}

impl Data for M1Data {
    fn push(&mut self, x: u8, pos: usize, zbuf: &mut [u16]) -> usize {
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

        self.f[pos / 16].0[pos % 16] = 0;
        self.f[pos / 16].1[pos % 16] = 0;
        let mut zpos = 0;
        for (i, (f1, f2)) in self.f.iter_mut().enumerate() {
            let ff1 = rem_u32x16(*f1 * u32x16::splat(10) + u32x16::splat(x as _), M1_1);
            let ff2 = rem_u32x16(*f2 * u32x16::splat(10) + u32x16::splat(x as _), M1_2);
            (*f1, *f2) = (ff1, ff2);
            let zeros = (ff1 | ff2).lanes_eq(u32x16::default());
            if unlikely(zeros.any()) {
                for j in 0..16 {
                    if zeros.test(j) {
                        zbuf[zpos] = (i * 16 + j) as u16;
                        zpos += 1;
                    }
                }
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
    fn push(&mut self, x: u8, pos: usize, zbuf: &mut [u16]) -> usize {
        self.f[pos / 8].set(pos % 8, 0);
        let mut zpos = 0;
        for (i, f) in self.f.iter_mut().enumerate() {
            let ff = f.mul10_add(x as _).rem10(M2);
            *f = ff;
            let zeros = ff.is_zero();
            if unlikely(zeros.any()) {
                for j in 0..8 {
                    if zeros.test(j) {
                        zbuf[zpos] = (i * 8 + j) as u16;
                        zpos += 1;
                    }
                }
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
    fn push(&mut self, x: u8, pos: usize, zbuf: &mut [u16]) -> usize {
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

        self.f[pos / 8].0[pos % 8] = 0;
        self.f[pos / 8].1[pos % 8] = 0;
        self.f[pos / 8].2[pos % 8] = 0;
        let mut zpos = 0;
        for (i, (f1, f2, f3)) in self.f.iter_mut().enumerate() {
            let ff1 = rem_u64x8(*f1 * u64x8::splat(10) + u64x8::splat(x as _), M3_1);
            let ff2 = rem_u64x8(*f2 * u64x8::splat(10) + u64x8::splat(x as _), M3_2);
            let ff3 = rem_u64x8(*f3 * u64x8::splat(10) + u64x8::splat(x as _), M3_3);
            (*f1, *f2, *f3) = (ff1, ff2, ff3);
            let zeros = (ff1 | ff2 | ff3).lanes_eq(u64x8::default());
            if unlikely(zeros.any()) {
                for j in 0..8 {
                    if zeros.test(j) {
                        zbuf[zpos] = (i * 8 + j) as u16;
                        zpos += 1;
                    }
                }
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
    fn push(&mut self, x: u8, pos: usize, zbuf: &mut [u16]) -> usize {
        self.f[pos / 8].0.set(pos % 8, 0);
        self.f[pos / 8].1.set(pos % 8, 0);
        self.f[pos / 8].2.set(pos % 8, 0);
        let mut zpos = 0;
        for (i, (f3, f7, f11)) in self.f.iter_mut().enumerate() {
            let ff3 = f3.mul10_add(x as _).rem10(M4_3);
            let ff7 = f7.mul10_add(x as _).rem10(M4_7);
            let ff11 = f11.mul10_add(x as _).rem10(M4_11);
            (*f3, *f7, *f11) = (ff3, ff7, ff11);
            let zeros = (ff3 | ff7 | ff11).is_zero();
            if unlikely(zeros.any()) {
                for j in 0..8 {
                    if zeros.test(j) {
                        zbuf[zpos] = (i * 8 + j) as u16;
                        zpos += 1;
                    }
                }
            }
        }
        zpos
    }
}

struct Task<T: Data> {
    stat: Stat,
    deque: VecDeque<u8>,
    pos: usize,
    f: T,
    k: u8,
}

impl<T: Data> Task<T> {
    fn new(k: u8) -> Self {
        Task {
            stat: Stat::new(),
            deque: VecDeque::with_capacity(N),
            f: T::default(),
            pos: 0,
            k,
        }
    }

    fn clear(&mut self) {
        self.deque.clear();
        self.pos = 0;
        // no need to clear `f`
    }

    /// If found return the end index.
    fn append(
        &mut self,
        bytes: &[u8],
        t0: Instant,
        tcp_rx: &mpsc::Receiver<TcpStream>,
    ) -> Option<usize> {
        let mut zbuf = [unsafe { std::mem::MaybeUninit::uninit().assume_init() }; N];
        let mut iter = bytes.iter().enumerate();
        while let Some((mut idx, &b)) = iter.next() {
            if self.deque.len() == N {
                self.deque.pop_front();
            }
            self.deque.push_back(b);

            let x = b - b'0';

            let zpos = self.f.push(x, self.pos, &mut zbuf);

            self.pos = if self.pos == N - 1 { 0 } else { self.pos + 1 };

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let mut len = if i < self.pos {
                    self.pos - i
                } else {
                    N - (i - self.pos)
                };
                if i >= self.deque.len() || self.deque[self.deque.len() - len] == b'0' {
                    continue;
                }
                send(&tcp_rx, len, &self.deque);
                self.stat.add(self.k, len, t0);
                // tailing 0s
                while let Some((_, &b'0')) = iter.next() {
                    len += 1;
                    if len > N {
                        break;
                    }
                    idx += 1;
                    if self.deque.len() == N {
                        self.deque.pop_front();
                    }
                    self.deque.push_back(b'0');

                    send(&tcp_rx, len, &self.deque);
                    self.stat.add(self.k, len, t0);
                }
                return Some(idx);
            }
        }
        None
    }
}

fn send(tcp_rx: &mpsc::Receiver<TcpStream>, len: usize, deque: &VecDeque<u8>) {
    if NO_SEND {
        return;
    }
    let mut tcp = tcp_rx.recv().map_err(|_| exit(-1)).unwrap();
    let (mut n0, mut n1) = deque.as_slices();
    if n1.len() >= len {
        n0 = &[];
        n1 = &n1[n1.len() - len..];
    } else {
        n0 = &n0[deque.len() - len..];
    }
    const HEADER: &str = "POST /submit?user=omicron&passwd=y8J6IGKr HTTP/1.1\r\nHost: 172.1.1.119:10002\r\nUser-Agent: Go-http-client/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\n";
    let content_length = format!("Content-Length: {}\r\n\r\n", len);
    let iov = [
        IoSlice::new(HEADER.as_bytes()),
        IoSlice::new(content_length.as_bytes()),
        IoSlice::new(n0),
        IoSlice::new(n1),
    ];
    tcp.write_vectored(&iov).map_err(|_| exit(-1)).unwrap();
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

    fn add(&mut self, k: u8, len: usize, t0: Instant) {
        // statistics
        let latency = t0.elapsed();
        self.dsum += latency;
        self.count += 1;
        let avg = self.dsum / self.count;
        let nps = self.count as f32 / self.t00.elapsed().as_secs_f32();
        log::info!("M{k} {len:3}  lat: {latency:>9?}  avg: {avg:>9?}  nps: {nps:.3?}");
    }
}
