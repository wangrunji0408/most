use futures::future::try_join_all;
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
    let ms: [&_; 3] = [m1, m2, m3];

    let mut deque = VecDeque::new();
    let mut rem: Vec<[BigUint; 3]> = vec![];
    // rem[i][k] = deque[i..] % m[k]
    while let Some(item) = stream.next().await {
        let t0 = Instant::now();
        let bytes = item.unwrap();
        let tail_len = deque.len();
        deque.extend(bytes.clone());

        let mut tasks = vec![];
        for i in 0..deque.len() {
            let mut f = rem.get(i).cloned().unwrap_or_default();
            let deque: &VecDeque<u8> = unsafe { std::mem::transmute(&deque) };
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
                        while &*f >= m {
                            *f -= m;
                        }
                    }
                    if let Some(k) = f.iter().position(|f| f == &zero) {
                        let n: Vec<u8> = deque.range(i..=j).cloned().collect();
                        send(n).await;
                        println!("{:?}", t0.elapsed());
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

async fn send(body: Vec<u8>) {
    reqwest::Client::new()
        .post("http://47.95.111.217:10002/submit?user=omicron&passwd=y8J6IGKr")
        .body(body)
        .send()
        .await
        .unwrap();
}
