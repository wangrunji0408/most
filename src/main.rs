use futures::stream::StreamExt;
use num_bigint::BigUint;
use std::collections::VecDeque;
use std::time::Instant;

const N: usize = 512;

#[tokio::main]
async fn main() {
    let mut stream = reqwest::get("http://47.95.111.217:10001")
        .await
        .unwrap()
        .bytes_stream();

    // $ factor 20220209192254
    // 20220209192254: 2 23 122509 3588061
    // $ factor 104648257118348370704723099
    // 104648257118348370704723099: 104648257118348370704723099
    // $ factor 125000000000000064750000000000009507500000000000294357
    // factor: ‘125000000000000064750000000000009507500000000000294357’ is too large
    let m1 = Box::leak(Box::new("20220209192254".parse::<BigUint>().unwrap()));
    let m2 = Box::leak(Box::new(
        "104648257118348370704723099".parse::<BigUint>().unwrap(),
    ));
    let m3 = Box::leak(Box::new(
        "125000000000000064750000000000009507500000000000294357"
            .parse::<BigUint>()
            .unwrap(),
    ));
    let zero = "0".parse::<BigUint>().unwrap();
    let ms: [&_; 3] = [m1, m2, m3];

    let mut deque = VecDeque::with_capacity(N);
    // rem[k][i] = s[-i:] % m_k
    let mut rem = vec![vec![BigUint::new(vec![]); N + 1]; 3];
    while let Some(item) = stream.next().await {
        let t0 = Instant::now();
        for b in item.unwrap() {
            while deque.len() >= N {
                deque.pop_front();
            }
            deque.push_back(b);
            let s = deque_to_vec(&deque);
            let len = s.len();

            // update rem matrix
            let x = b - b'0';
            let mut tasks = vec![];
            for (mut rem, &m) in rem.drain(..).zip(ms.iter()) {
                tasks.push(tokio::spawn(async move {
                    for i in (1..=len).rev() {
                        // rem[i] = (&rem[i - 1] * 10u8 + x) % m;
                        rem[i] = &rem[i - 1] * 10u8 + x;
                        while rem[i] >= *m {
                            rem[i] -= m;
                        }
                    }
                    rem
                }));
            }
            for t in tasks {
                rem.push(t.await.unwrap());
            }

            // test rem == 0
            for len in 1..=s.len() {
                let n = &s[s.len() - len..];
                if n[0] == b'0' {
                    continue;
                }
                if let Some(j) = rem.iter().position(|r| r[len] == zero) {
                    send(n).await;
                    println!("{:?}", t0.elapsed(),);
                }
            }
        }
    }
}

fn deque_to_vec(d: &VecDeque<u8>) -> Vec<u8> {
    let (s1, s2) = d.as_slices();
    let mut s = Vec::with_capacity(d.len());
    s.extend_from_slice(s1);
    s.extend_from_slice(s2);
    s
}

async fn send(body: &[u8]) {
    reqwest::Client::new()
        .post("http://47.95.111.217:10002/submit?user=omicron&passwd=y8J6IGKr")
        .body(body.to_vec())
        .send()
        .await
        .unwrap();
}
