use most::U192;
use std::collections::VecDeque;
use std::io::{IoSlice, Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

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
    let (tx1, rx1) = mpsc::channel(8);
    tokio::spawn(task1(rx1));
    let (tx2, rx2) = mpsc::channel(8);
    tokio::spawn(task2(rx2));
    let (tx3, rx3) = mpsc::channel(8);
    tokio::spawn(task3(rx3));
    let (tx4, rx4) = mpsc::channel(8);
    tokio::spawn(task4(rx4));

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

        tx1.send((t0, bytes.clone())).await.unwrap();
        tx2.send((t0, bytes.clone())).await.unwrap();
        tx3.send((t0, bytes.clone())).await.unwrap();
        tx4.send((t0, bytes)).await.unwrap();
    }
}

fn tcps() -> mpsc::Receiver<TcpStream> {
    let (tx, rx) = mpsc::channel::<TcpStream>(2);
    tokio::spawn(async move {
        loop {
            let stream = TcpStream::connect("47.95.111.217:10002").unwrap();
            tx.send(stream).await.unwrap();
        }
    });
    rx
}

async fn task1(mut rx: mpsc::Receiver<(Instant, Arc<[u8]>)>) {
    let mut tcp_rx = tcps();
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f1 = [0u64; N];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Some((t0, bytes)) = rx.recv().await {
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

async fn task2(mut rx: mpsc::Receiver<(Instant, Arc<[u8]>)>) {
    let mut tcp_rx = tcps();
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f2 = [0u128; N];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Some((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            f2[pos] = 0;
            let mut zpos = 0;
            for (i, f) in f2.iter_mut().enumerate() {
                let ff = rem_u128(*f * 10 + x as u128, M2);
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
                stat.add(2, t0);
            }
        }
    }
}

async fn task3(mut rx: mpsc::Receiver<(Instant, Arc<[u8]>)>) {
    let mut tcp_rx = tcps();
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
    while let Some((t0, bytes)) = rx.recv().await {
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

async fn task4(mut rx: mpsc::Receiver<(Instant, Arc<[u8]>)>) {
    let mut tcp_rx = tcps();
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f4 = [(0u128, 0u128, 0u128); N];
    let mut pos = 0;
    let mut zbuf = [0u16; N];
    while let Some((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            f4[pos] = (0, 0, 0);
            let mut zpos = 0;
            for (i, (f2, f3, f7)) in f4.iter_mut().enumerate() {
                let ff2 = (*f2 * 10 + x as u128) & ((1 << 75) - 1);
                let ff3 = rem_u128(*f3 * 10 + x as u128, M4_3);
                let ff7 = rem_u128(*f7 * 10 + x as u128, M4_7);
                (*f2, *f3, *f7) = (ff2, ff3, ff7);
                if (ff2, ff3, ff7) == (0, 0, 0) {
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
    // tcp.write_vectored(&iov).unwrap();
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
