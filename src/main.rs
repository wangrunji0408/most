use futures::future::try_join_all;
use hyper::{body::HttpBody, Uri};
use num_bigint::BigUint;
use std::collections::VecDeque;
use std::io::IoSlice;
use std::time::Instant;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

const N: usize = 512;

#[tokio::main]
async fn main() {
    let mut body = hyper::Client::new()
        .get(Uri::from_static("http://47.95.111.217:10001"))
        .await
        .unwrap();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<(Instant, Instant, Vec<u8>)>(8);
    // responsor
    tokio::spawn(async move {
        loop {
            let mut stream = TcpStream::connect("47.95.111.217:10002").await.unwrap();
            let (t0, t1, body) = rx.recv().await.unwrap();

            const HEADER: &str = "POST /submit?user=omicron&passwd=y8J6IGKr HTTP/1.1\r\nHost: 47.95.111.217:10002\r\nUser-Agent: Go-http-client/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\n";
            let content_length = format!("Content-Length: {}\r\n\r\n", body.len());
            let iov = [
                IoSlice::new(HEADER.as_bytes()),
                IoSlice::new(content_length.as_bytes()),
                IoSlice::new(&body),
            ];
            stream.write_vectored(&iov).await.unwrap();
            println!("{:?} {:?}", t0.elapsed(), t1 - t0);
        }
    });

    // $ factor 20220209192254
    // 20220209192254: 2 23 122509 3588061
    // $ factor 104648257118348370704723099
    // 104648257118348370704723099: 104648257118348370704723099
    // $ factor 125000000000000064750000000000009507500000000000294357
    // factor: ‘125000000000000064750000000000009507500000000000294357’ is too large
    let m1 = "20220209192254".parse::<BigUint>().unwrap();
    let m2 = "104648257118348370704723099".parse::<BigUint>().unwrap();
    let m3 = "125000000000000064750000000000009507500000000000294357"
        .parse::<BigUint>()
        .unwrap();
    let ms = &*Box::leak(Box::new([
        [&m1 * 1u8, &m1 * 2u8, &m1 * 4u8],
        [&m2 * 1u8, &m2 * 2u8, &m2 * 4u8],
        [&m3 * 1u8, &m3 * 2u8, &m3 * 4u8],
    ]));

    let mut deque = VecDeque::new();
    let mut rem: Vec<[BigUint; 3]> = vec![];
    // rem[i][k] = deque[i..] % m[k]
    while let Some(item) = body.data().await {
        let t0 = Instant::now();
        let bytes = item.unwrap();
        let tail_len = deque.len();
        deque.extend(bytes.clone());

        let mut tasks = vec![];
        for i in 0..deque.len() {
            let mut f = rem.get(i).cloned().unwrap_or_default();
            let deque: &VecDeque<u8> = unsafe { std::mem::transmute(&deque) };
            let tx = tx.clone();
            tasks.push(tokio::spawn(async move {
                if deque[i] == b'0' {
                    return Default::default();
                }
                let zero = BigUint::default();
                for j in tail_len.max(i)..deque.len() {
                    let len = j + 1 - i;
                    if len > N {
                        return f;
                    }
                    let x = deque[j] - b'0';
                    for (f, m) in f.iter_mut().zip(ms) {
                        *f = &*f * 10u8 + x;
                        while &*f >= &m[2] {
                            *f -= &m[2];
                        }
                        if &*f >= &m[1] {
                            *f -= &m[1];
                        }
                        if &*f >= &m[0] {
                            *f -= &m[0];
                        }
                    }
                    if let Some(k) = f.iter().position(|f| f == &zero) {
                        let t1 = Instant::now();
                        let n: Vec<u8> = deque.range(i..=j).cloned().collect();
                        tx.send((t0, t1, n)).await.unwrap();
                        // println!(
                        //     "{:?}: {}: {}",
                        //     t0.elapsed(),
                        //     ms[k],
                        //     std::str::from_utf8(&n).unwrap()
                        // );
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
