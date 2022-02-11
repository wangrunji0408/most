use futures::stream::StreamExt;
use num_bigint::BigUint;

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
    let m1: &_ = Box::leak(Box::new("20220209192254".parse::<BigUint>().unwrap()));
    let m2: &_ = Box::leak(Box::new(
        "104648257118348370704723099".parse::<BigUint>().unwrap(),
    ));
    let m3: &_ = Box::leak(Box::new(
        "125000000000000064750000000000009507500000000000294357"
            .parse::<BigUint>()
            .unwrap(),
    ));

    let mut s = vec![];
    while let Some(item) = stream.next().await {
        for b in item.unwrap() {
            s.push(b);
            while s.len() > N {
                s.remove(0);
            }
            let s = s.clone();
            tokio::spawn(async move {
                let zero = "0".parse::<BigUint>().unwrap();
                let even = s[s.len() - 1] % 2 == 0;
                for i in 0..s.len() {
                    if s[i] == b'0' {
                        continue;
                    }
                    let n = BigUint::parse_bytes(&s[i..], 10).unwrap();
                    if even && &n % m1 == zero {
                        send(&s[i..]).await;
                        println!("{}: {}", m1, n);
                    } else if &n % m2 == zero {
                        send(&s[i..]).await;
                        println!("{}: {}", m2, n);
                    } else if &n % m3 == zero {
                        send(&s[i..]).await;
                        println!("{}: {}", m3, n);
                    }
                }
            });
        }
    }
}

async fn send(body: &[u8]) {
    reqwest::Client::new()
        .post("http://47.95.111.217:10002/submit?user=omicron&passwd=y8J6IGKr")
        .body(body.to_vec())
        .send()
        .await
        .unwrap();
}
