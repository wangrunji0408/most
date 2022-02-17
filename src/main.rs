#![feature(core_intrinsics)]
#![feature(portable_simd)]

use most::{U128x8, U192};
use std::collections::VecDeque;
use std::intrinsics::unlikely;
use std::io::{IoSlice, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::broadcast;

const N: usize = 512;

// $ factor 20220209192254
// 20220209192254: 2 23 122509 3588061
// $ factor 104648257118348370704723099
// 104648257118348370704723099: 104648257118348370704723099
// $ factor 125000000000000064750000000000009507500000000000294357
// factor: ‘125000000000000064750000000000009507500000000000294357’ is too large
const M1: u64 = 20220209192254;
const M2: u128 = 104648257118348370704723099;
const M3: U192 = U192([0x32b9c8672a627dd5, 0x959989af0854b90, 0x14e1878814c9d]);
const M4_3: u128 = 717897987691852588770249;
const M4_7: u128 = 1341068619663964900807;
// M4: 2^75 * 3^50 * 7^25

#[tokio::main]
async fn main() {
    let (tcp_tx, tcp_rx) = async_channel::bounded::<TcpStream>(4);
    tokio::spawn(async move {
        loop {
            let stream = TcpStream::connect("47.95.111.217:10002").unwrap();
            tcp_tx.send(stream).await.unwrap();
        }
    });
    let (tx, _rx) = broadcast::channel(8);
    tokio::spawn(task1(tx.subscribe(), tcp_rx.clone()));
    tokio::spawn(task2(tx.subscribe(), tcp_rx.clone()));
    tokio::spawn(task3(tx.subscribe(), tcp_rx.clone()));
    tokio::spawn(task4(tx.subscribe(), tcp_rx.clone()));

    let mut get_tcp = TcpStream::connect("47.95.111.217:10001").unwrap();
    get_tcp
        .write(b"GET HTTP/1.1\r\nHost: 47.95.111.217:10001\r\n\r\n")
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
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f1 = [0u64; N];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Ok((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            f1[pos] = 0;
            let mut zpos = 0;
            for (i, f) in f1.iter_mut().enumerate() {
                let ff = (*f * 10 + x as u64) % M1;
                *f = ff;
                if ff == 0 {
                    zbuf[zpos] = i as u16;
                    zpos += 1;
                }
            }

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let len = if i < pos { pos - i } else { N - (i - pos) };
                if i >= deque.len() || deque[deque.len() - len] == b'0' {
                    continue;
                }
                let tcp = tcp_rx.recv().await.unwrap();
                send(tcp, len, &deque).await;
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
                const MX4: U128x8 = U128x8::splat(M2 * 4);
                const MX2: U128x8 = U128x8::splat(M2 * 2);
                const MX1: U128x8 = U128x8::splat(M2 * 1);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX2);
                x = x.sub_on_ge(MX1);
                x
            }

            f2[pos / 8].set(pos % 8, 0);
            let mut zpos = 0;
            for (i, f) in f2.iter_mut().enumerate() {
                let ff = rem_u128x8_m2((*f << 1) + (*f << 3) + x);
                *f = ff;
                let zeros = ff.lanes_eq(U128x8::ZERO);
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
                let tcp = tcp_rx.recv().await.unwrap();
                send(tcp, len, &deque).await;
                stat.add(2, t0);
            }
        }
    }
}

async fn task3(
    mut rx: broadcast::Receiver<(Instant, Arc<[u8]>)>,
    tcp_rx: async_channel::Receiver<TcpStream>,
) {
    // assert_eq!(
    //     M3.to_string(),
    //     "125000000000000064750000000000009507500000000000294357"
    // );
    let mut m3s = vec![U192::ZERO];
    for i in 1..10 {
        m3s.push(m3s[i - 1] + M3);
    }

    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f3 = [U192::ZERO; N];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Ok((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            f3[pos] = U192::ZERO;
            let mut zpos = 0;
            for (i, f) in f3.iter_mut().enumerate() {
                let ff = rem_u192_m3((*f << 1) + (*f << 3) + x);
                *f = ff;
                if ff.is_zero() {
                    zbuf[zpos] = i as u16;
                    zpos += 1;
                }
            }

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for &i in &zbuf[0..zpos] {
                let i = i as usize;
                let len = if i < pos { pos - i } else { N - (i - pos) };
                if i >= deque.len() || deque[deque.len() - len] == b'0' {
                    continue;
                }
                let tcp = tcp_rx.recv().await.unwrap();
                send(tcp, len, &deque).await;
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
                const MX4: U128x8 = U128x8::splat(M4_3 * 4);
                const MX2: U128x8 = U128x8::splat(M4_3 * 2);
                const MX1: U128x8 = U128x8::splat(M4_3 * 1);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX2);
                x = x.sub_on_ge(MX1);
                x
            }
            #[inline]
            fn rem_u128x8_m4_7(mut x: U128x8) -> U128x8 {
                const MX4: U128x8 = U128x8::splat(M4_7 * 4);
                const MX2: U128x8 = U128x8::splat(M4_7 * 2);
                const MX1: U128x8 = U128x8::splat(M4_7 * 1);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX4);
                x = x.sub_on_ge(MX2);
                x = x.sub_on_ge(MX1);
                x
            }

            f4[pos / 8].0.set(pos % 8, 0);
            f4[pos / 8].1.set(pos % 8, 0);
            f4[pos / 8].2.set(pos % 8, 0);
            let mut zpos = 0;
            for (i, (f2, f3, f7)) in f4.iter_mut().enumerate() {
                let ff2 = ((*f2 << 1) + (*f2 << 3) + x) & U128x8::splat((1 << 75) - 1);
                let ff3 = rem_u128x8_m4_3((*f3 << 1) + (*f3 << 3) + x);
                let ff7 = rem_u128x8_m4_7((*f7 << 1) + (*f7 << 3) + x);
                (*f2, *f3, *f7) = (ff2, ff3, ff7);
                let zeros = (ff2 | ff3 | ff7).lanes_eq(U128x8::ZERO);
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
                let tcp = tcp_rx.recv().await.unwrap();
                send(tcp, len, &deque).await;
                stat.add(4, t0);
            }
        }
    }
}

async fn send(mut tcp: TcpStream, len: usize, deque: &VecDeque<u8>) {
    let (mut n0, mut n1) = deque.as_slices();
    if n1.len() >= len {
        n0 = &[];
        n1 = &n1[n1.len() - len..];
    } else {
        n0 = &n0[deque.len() - len..];
    }
    const HEADER: &str = "POST /submit?user=omicron&passwd=y8J6IGKr HTTP/1.1\r\nHost: 47.95.111.217:10002\r\nUser-Agent: Go-http-client/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\n";
    let content_length = format!("Content-Length: {}\r\n\r\n", len);
    let iov = [
        IoSlice::new(HEADER.as_bytes()),
        IoSlice::new(content_length.as_bytes()),
        IoSlice::new(n0),
        IoSlice::new(n1),
    ];
    tcp.write_vectored(&iov).unwrap();
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
