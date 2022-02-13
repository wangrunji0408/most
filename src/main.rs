use futures::future::try_join_all;
use hyper::{body::HttpBody, Uri};
use primitive_types::U256;
use std::collections::VecDeque;
use std::io::IoSlice;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

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

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(u8, Instant, Vec<u8>)>(8);
    // responsor
    tokio::spawn(async move {
        let t00 = Instant::now();
        let mut dsum = Duration::default();
        let mut count = 0;
        loop {
            let mut stream = TcpStream::connect("47.95.111.217:10002").await.unwrap();
            let (k, t0, body) = rx.recv().await.unwrap();

            const HEADER: &str = "POST /submit?user=omicron&passwd=y8J6IGKr HTTP/1.1\r\nHost: 47.95.111.217:10002\r\nUser-Agent: Go-http-client/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\n";
            let content_length = format!("Content-Length: {}\r\n\r\n", body.len());
            let iov = [
                IoSlice::new(HEADER.as_bytes()),
                IoSlice::new(content_length.as_bytes()),
                IoSlice::new(&body),
            ];
            stream.write_vectored(&iov).await.unwrap();

            // statistics
            let latency = t0.elapsed();
            dsum += latency;
            count += 1;
            let avg = dsum / count;
            let nps = count as f32 / t00.elapsed().as_secs_f32();
            println!("M{k} lat: {latency:?}\tavg: {avg:?}\tnps: {nps:.3?}");
        }
    });

    assert_eq!(
        M3.to_string(),
        "125000000000000064750000000000009507500000000000294357"
    );
    let m3s = &*Vec::leak((0u8..10).map(|i| M3 * i).collect());

    let mut deque = VecDeque::new();
    let mut rem: Vec<(u64, u128, U256, U256)> = vec![];
    // rem[i][k] = deque[i..] % m[k]
    while let Some(item) = body.data().await {
        let t0 = Instant::now();
        let bytes = item.unwrap();
        let tail_len = deque.len();
        deque.extend(bytes.clone());

        let mut tasks = vec![];
        for i in 0..deque.len() {
            let mut f = match rem.get_mut(i) {
                Some(f) => std::mem::take(f),
                None => Default::default(),
            };
            let deque: &VecDeque<u8> = unsafe { std::mem::transmute(&deque) };
            let tx = tx.clone();
            tasks.push(tokio::spawn(async move {
                if deque[i] == b'0' {
                    return Default::default();
                }
                for j in tail_len.max(i)..deque.len() {
                    let len = j + 1 - i;
                    if len > N {
                        return f;
                    }
                    let x = deque[j] - b'0';

                    f.0 = (f.0 * 10 + x as u64) % M1;
                    f.1 = rem_m2(f.1 * 10 + x as u128);
                    // f.2 = (f.2 * 10u8 + x) % m3;
                    f.2 = f.2 * 10 + x;
                    let idx = m3s.partition_point(|m| &f.2 >= m);
                    if idx > 0 {
                        f.2 -= m3s[idx - 1];
                    }
                    const M4_MASK: U256 = U256([u64::MAX, u64::MAX, u64::MAX >> 14, 0]);
                    f.3 = (f.3 * 10 + x) & M4_MASK;

                    let k = match () {
                        _ if f.0 == 0 => 1,
                        _ if f.1 == 0 => 2,
                        _ if f.2.is_zero() => 3,
                        _ if f.3.is_zero() => 4,
                        _ => 0,
                    };
                    if k != 0 {
                        let n: Vec<u8> = deque.range(i..=j).cloned().collect();
                        tx.send((k, t0, n)).await.unwrap();
                        tokio::task::yield_now().await;
                    }
                }
                f
            }));
        }
        let mut new_rem = try_join_all(tasks).await.unwrap();
        if deque.len() >= N {
            rem = new_rem.split_off(deque.len() - (N - 1));
        } else {
            rem = new_rem;
        }

        while deque.len() >= N {
            deque.pop_front();
        }
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
