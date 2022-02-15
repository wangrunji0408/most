use most::U256;
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
const M3: U256 = U256([0x32b9c8672a627dd5, 0x959989af0854b90, 0x14e1878814c9d, 0x0]);
const M4_MASK: U256 = U256([u64::MAX, u64::MAX, u64::MAX >> 14, 0]);
// M4: 2^178 * 3^0 * 7^0

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
    let (tx, rx) = mpsc::channel::<TcpStream>(4);
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
    let mut f1: Vec<u64> = vec![0; N];
    let mut pos = 0;
    while let Some((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            f1[pos] = 0;
            for f in &mut f1 {
                *f = (*f * 10 + x as u64) % M1;
            }

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for i in 0..deque.len() {
                if f1[i] == 0 {
                    let len = if i < pos { pos - i } else { N - (i - pos) };
                    if deque[deque.len() - len] == b'0' {
                        continue;
                    }
                    let tcp = tcp_rx.recv().await.unwrap();
                    send(tcp, len, &deque).await;
                    stat.add(1, t0);
                }
            }
        }
    }
}

async fn task2(mut rx: mpsc::Receiver<(Instant, Arc<[u8]>)>) {
    let mut tcp_rx = tcps();
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f2: Vec<u128> = vec![0; N];
    let mut pos = 0;
    while let Some((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            f2[pos] = 0;
            for f in &mut f2 {
                *f = rem_m2(*f * 10 + x as u128);
            }

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for i in 0..deque.len() {
                if f2[i] == 0 {
                    let len = if i < pos { pos - i } else { N - (i - pos) };
                    if deque[deque.len() - len] == b'0' {
                        continue;
                    }
                    let tcp = tcp_rx.recv().await.unwrap();
                    send(tcp, len, &deque).await;
                    stat.add(2, t0);
                }
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
    let mut m3s = vec![U256::ZERO];
    for i in 1..10 {
        m3s.push(m3s[i - 1] + M3);
    }

    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f3: Vec<U256> = vec![U256::ZERO; N];
    let mut pos = 0;
    while let Some((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            f3[pos] = U256::ZERO;
            for f in &mut f3 {
                let ff = (*f << 1) + (*f << 3) + x;
                let idx = m3s.partition_point(|m| &ff >= m);
                *f = ff - m3s[idx - 1];
            }

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for i in 0..deque.len() {
                if f3[i].is_zero() {
                    let len = if i < pos { pos - i } else { N - (i - pos) };
                    if deque[deque.len() - len] == b'0' {
                        continue;
                    }
                    let tcp = tcp_rx.recv().await.unwrap();
                    send(tcp, len, &deque).await;
                    stat.add(3, t0);
                }
            }
        }
    }
}

async fn task4(mut rx: mpsc::Receiver<(Instant, Arc<[u8]>)>) {
    let mut tcp_rx = tcps();
    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut f4: Vec<U256> = vec![U256::ZERO; N];
    let mut pos = 0;
    while let Some((t0, bytes)) = rx.recv().await {
        for &b in bytes.iter() {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            let x = b - b'0';

            f4[pos] = U256::ZERO;
            for f in &mut f4 {
                *f = ((*f << 1) + (*f << 3) + x) & M4_MASK;
            }

            pos = if pos == N - 1 { 0 } else { pos + 1 };

            for i in 0..deque.len() {
                if f4[i].is_zero() {
                    let len = if i < pos { pos - i } else { N - (i - pos) };
                    if deque[deque.len() - len] == b'0' {
                        continue;
                    }
                    let tcp = tcp_rx.recv().await.unwrap();
                    send(tcp, len, &deque).await;
                    stat.add(4, t0);
                }
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

fn rem_m2(x: u128) -> u128 {
    if x >= M2 * 5 {
        if x >= M2 * 7 {
            if x >= M2 * 9 {
                x - M2 * 9
            } else if x >= M2 * 8 {
                x - M2 * 8
            } else {
                x - M2 * 7
            }
        } else {
            if x >= M2 * 6 {
                x - M2 * 6
            } else {
                x - M2 * 5
            }
        }
    } else {
        if x >= M2 * 2 {
            if x >= M2 * 4 {
                x - M2 * 4
            } else if x >= M2 * 3 {
                x - M2 * 3
            } else {
                x - M2 * 2
            }
        } else {
            if x >= M2 * 1 {
                x - M2 * 1
            } else {
                x
            }
        }
    }
}
