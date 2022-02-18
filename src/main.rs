#![feature(core_intrinsics)]
#![feature(portable_simd)]
#![feature(stdsimd)]

use most::{U128x8, U192};
use std::collections::VecDeque;
use std::intrinsics::unlikely;
use std::io::{IoSlice, Read, Write};
use std::net::TcpStream;
use std::process::exit;
use std::simd::u64x8;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

// N  = 256
// M1 = 20220217214410 = 2 * 5 * 431 * 46589 * 100699
// M2 = 104648257118348370704723119
// M3 = 125000000000000140750000000000052207500000000006359661
//    = 500000000000000147 * 500000000000000207 * 500000000000000209
// M4 = a hidden but fixed integer, whose prime factors include and only include 3, 7 and 11
const N: usize = 256;
const M1: u64 = 20220217214410;
const M1_1: u32 = 431 * 46589;
const M1_2: u32 = 2 * 5 * 100699;
const M2: u128 = 104648257118348370704723119;
const M3: U192 = U192([0x32b716db666f0a6d, 0x4286a9e7b0336f0c, 0x14e1878814c9d]);
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

#[tokio::main]
async fn main() {
    let (tcp_tx, tcp_rx) = async_channel::bounded::<TcpStream>(8);
    tokio::spawn(async move {
        if NO_SEND {
            std::mem::forget(tcp_tx);
            return;
        }
        loop {
            async fn connect() -> std::io::Result<TcpStream> {
                let stream = tokio::net::TcpStream::connect("172.1.1.119:10002").await?;
                let stream = stream.into_std()?;
                stream.set_nonblocking(false)?;
                stream.set_nodelay(true)?;
                Ok(stream)
            }
            match connect().await {
                Ok(s) => tcp_tx.send(s).await.unwrap(),
                Err(e) => eprintln!("{}", e),
            }
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });
    let (tx, _rx) = broadcast::channel(8);
    tokio::spawn(task1(tx.subscribe(), tcp_rx.clone()));
    tokio::spawn(task2(tx.subscribe(), tcp_rx.clone()));
    tokio::spawn(task3(tx.subscribe(), tcp_rx.clone()));
    tokio::spawn(task4(tx.subscribe(), tcp_rx.clone()));

    let mut get_tcp = TcpStream::connect(IN_IP).unwrap();
    get_tcp
        .write(format!("GET HTTP/1.1\r\nHost: {IN_IP}\r\n\r\n").as_bytes())
        .unwrap();
    const OK_HEADER: &str = "HTTP/1.1 200 OK\r\nServer: Most\r\nContent-type: text/plain\r\n\r\n";
    let mut buf = [0; 1024];
    let len = get_tcp.read(&mut buf[..OK_HEADER.len()]).unwrap();
    assert_eq!(&buf[..len], OK_HEADER.as_bytes());

    loop {
        let mut buf = vec![0; 1024];
        let len = get_tcp.read(&mut buf).unwrap();
        let t0 = Instant::now();
        buf.truncate(len);
        let bytes = Arc::<[u8]>::from(buf);

        tx.send((t0, bytes)).unwrap();
    }
}

async fn task1(
    mut rx: broadcast::Receiver<(Instant, Arc<[u8]>)>,
    tcp_rx: async_channel::Receiver<TcpStream>,
) {
    use std::simd::u32x16;
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f1 = [(u32x16::default(), u32x16::default()); N / 16];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Ok((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

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

            f1[pos / 16].0[pos % 16] = 0;
            f1[pos / 16].1[pos % 16] = 0;
            let mut zpos = 0;
            for (i, (f1, f2)) in f1.iter_mut().enumerate() {
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

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let len = if i < pos { pos - i } else { N - (i - pos) };
                if i >= deque.len() || deque[deque.len() - len] == b'0' {
                    continue;
                }
                send(&tcp_rx, len, &deque).await;
                stat.add(1, t0);
            }
        }
    }
}

async fn task2(
    mut rx: broadcast::Receiver<(Instant, Arc<[u8]>)>,
    tcp_rx: async_channel::Receiver<TcpStream>,
) {
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f2 = [U128x8::ZERO; N / 8];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Ok((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            #[inline]
            fn rem_u128x8_m2(mut x: U128x8) -> U128x8 {
                const MX8: U128x8 = U128x8::splat(M2 * 8);
                const MX4: U128x8 = U128x8::splat(M2 * 4);
                const MX2: U128x8 = U128x8::splat(M2 * 2);
                const MX1: U128x8 = U128x8::splat(M2 * 1);
                x = x.sub_on_ge(MX8);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX2);
                x = x.sub_on_ge(MX1);
                x
            }

            f2[pos / 8].set(pos % 8, 0);
            let mut zpos = 0;
            for (i, f) in f2.iter_mut().enumerate() {
                let ff = rem_u128x8_m2(f.mul10_add(x as _));
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

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let len = if i < pos { pos - i } else { N - (i - pos) };
                if i >= deque.len() || deque[deque.len() - len] == b'0' {
                    continue;
                }
                send(&tcp_rx, len, &deque).await;
                stat.add(2, t0);
            }
        }
    }
}

async fn task3(
    mut rx: broadcast::Receiver<(Instant, Arc<[u8]>)>,
    tcp_rx: async_channel::Receiver<TcpStream>,
) {
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f3 = [(u64x8::default(), u64x8::default(), u64x8::default()); N / 8];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Ok((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

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

            f3[pos / 8].0[pos % 8] = 0;
            f3[pos / 8].1[pos % 8] = 0;
            f3[pos / 8].2[pos % 8] = 0;
            let mut zpos = 0;
            for (i, (f1, f2, f3)) in f3.iter_mut().enumerate() {
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

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let len = if i < pos { pos - i } else { N - (i - pos) };
                if i >= deque.len() || deque[deque.len() - len] == b'0' {
                    continue;
                }
                send(&tcp_rx, len, &deque).await;
                stat.add(3, t0);
            }
        }
    }
}

async fn task4(
    mut rx: broadcast::Receiver<(Instant, Arc<[u8]>)>,
    tcp_rx: async_channel::Receiver<TcpStream>,
) {
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f4 = [(U128x8::ZERO, U128x8::ZERO, U128x8::ZERO); N / 8];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Ok((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            #[inline]
            fn rem_u128x8_m4_3(mut x: U128x8) -> U128x8 {
                const MX8: U128x8 = U128x8::splat(M4_3 * 8);
                const MX4: U128x8 = U128x8::splat(M4_3 * 4);
                const MX2: U128x8 = U128x8::splat(M4_3 * 2);
                const MX1: U128x8 = U128x8::splat(M4_3 * 1);
                x = x.sub_on_ge(MX8);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX2);
                x = x.sub_on_ge(MX1);
                x
            }
            #[inline]
            fn rem_u128x8_m4_7(mut x: U128x8) -> U128x8 {
                const MX8: U128x8 = U128x8::splat(M4_7 * 8);
                const MX4: U128x8 = U128x8::splat(M4_7 * 4);
                const MX2: U128x8 = U128x8::splat(M4_7 * 2);
                const MX1: U128x8 = U128x8::splat(M4_7 * 1);
                x = x.sub_on_ge(MX8);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX2);
                x = x.sub_on_ge(MX1);
                x
            }
            #[inline]
            fn rem_u128x8_m4_11(mut x: U128x8) -> U128x8 {
                const MX8: U128x8 = U128x8::splat(M4_11 * 8);
                const MX4: U128x8 = U128x8::splat(M4_11 * 4);
                const MX2: U128x8 = U128x8::splat(M4_11 * 2);
                const MX1: U128x8 = U128x8::splat(M4_11 * 1);
                x = x.sub_on_ge(MX8);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX2);
                x = x.sub_on_ge(MX1);
                x
            }

            f4[pos / 8].0.set(pos % 8, 0);
            f4[pos / 8].1.set(pos % 8, 0);
            f4[pos / 8].2.set(pos % 8, 0);
            let mut zpos = 0;
            for (i, (f3, f7, f11)) in f4.iter_mut().enumerate() {
                let ff3 = rem_u128x8_m4_3(f3.mul10_add(x as _));
                let ff7 = rem_u128x8_m4_7(f7.mul10_add(x as _));
                let ff11 = rem_u128x8_m4_11(f11.mul10_add(x as _));
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

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let len = if i < pos { pos - i } else { N - (i - pos) };
                if i >= deque.len() || deque[deque.len() - len] == b'0' {
                    continue;
                }
                send(&tcp_rx, len, &deque).await;
                stat.add(4, t0);
            }
        }
    }
}

async fn send(tcp_rx: &async_channel::Receiver<TcpStream>, len: usize, deque: &VecDeque<u8>) {
    if NO_SEND {
        return;
    }
    let mut tcp = tcp_rx.recv().await.map_err(|_| exit(-1)).unwrap();
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

    fn add(&mut self, k: u8, t0: Instant) {
        // statistics
        let latency = t0.elapsed();
        self.dsum += latency;
        self.count += 1;
        let avg = self.dsum / self.count;
        let nps = self.count as f32 / self.t00.elapsed().as_secs_f32();
        println!("M{k} lat: {latency:?}\tavg: {avg:?}\tnps: {nps:.3?}");
    }
}

#[inline]
fn rem_u128(x: u128, m: u128) -> u128 {
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
fn rem_u192_m3(x: U192) -> U192 {
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
