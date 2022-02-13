use futures::future::try_join_all;
use hyper::{body::HttpBody, Uri};
use primitive_types::U256;
use std::collections::VecDeque;
use std::io::IoSlice;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
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
// M4: 2^178 * 3^0 * 7^0

#[tokio::main]
async fn main() {
    let mut body = hyper::Client::new()
        .get(Uri::from_static("http://47.95.111.217:10001"))
        .await
        .unwrap();

    let (tx, mut rx) = mpsc::channel::<TcpStream>(8);
    tokio::spawn(async move {
        loop {
            let stream = TcpStream::connect("47.95.111.217:10002").await.unwrap();
            tx.send(stream).await.unwrap();
        }
    });

    assert_eq!(
        M3.to_string(),
        "125000000000000064750000000000009507500000000000294357"
    );
    let m3s = &*Vec::leak((0u8..10).map(|i| M3 * i).collect());

    let mut stat = Stat::new();
    let mut deque = VecDeque::with_capacity(N);
    let mut rem: Vec<u64> = vec![0; N];
    let mut valid: Vec<bool> = vec![false; N];
    let mut pos = 0;
    while let Some(item) = body.data().await {
        let t0 = Instant::now();
        let bytes = item.unwrap();

        for b in bytes {
            if deque.len() == N {
                deque.pop_front();
            }
            deque.push_back(b);

            rem[pos] = 0;
            let x = b - b'0';
            for f in &mut rem {
                *f = (*f * 10 + x as u64) % M1;
            }
            valid[pos] = x != 0;
            pos += 1;
            if pos == N {
                pos = 0;
            }

            for i in 0..deque.len() {
                if valid[i] && rem[i] == 0 {
                    let len = if i < pos { pos - i } else { N - (i - pos) };
                    let n: Vec<u8> = deque.range(deque.len() - len..).cloned().collect();
                    let tcp = rx.recv().await.unwrap();
                    send(tcp, &n).await;
                    stat.add(0, t0);
                }
            }
        }
    }
}

async fn send(mut tcp: TcpStream, body: &[u8]) {
    const HEADER: &str = "POST /submit?user=omicron&passwd=y8J6IGKr HTTP/1.1\r\nHost: 47.95.111.217:10002\r\nUser-Agent: Go-http-client/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\n";
    let content_length = format!("Content-Length: {}\r\n\r\n", body.len());
    let iov = [
        IoSlice::new(HEADER.as_bytes()),
        IoSlice::new(content_length.as_bytes()),
        IoSlice::new(body),
    ];
    tcp.write_vectored(&iov).await.unwrap();
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
