use futures::stream::StreamExt;
use num_bigint::BigUint;

const N: usize = 512;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut stream = reqwest::get("http://47.95.111.217:10001")
        .await
        .unwrap()
        .bytes_stream();

    let m1: BigUint = "20220209192254".parse().unwrap();
    let m2: BigUint = "104648257118348370704723099".parse().unwrap();
    let m3: BigUint = "125000000000000064750000000000009507500000000000294357"
        .parse()
        .unwrap();
    let zero: BigUint = "0".parse().unwrap();

    let mut s = vec![];
    while let Some(item) = stream.next().await {
        for b in item.unwrap() {
            s.push(b);
            while s.len() > N {
                s.remove(0);
            }
            for i in 0..s.len() {
                if s[i] == b'0' {
                    continue;
                }
                let n = BigUint::parse_bytes(&s[i..], 10).unwrap();
                if &n % &m1 == zero {
                    println!("{}: {}", m1, n);
                    send(&s[i..]).await;
                } else if &n % &m2 == zero {
                    println!("{}: {}", m2, n);
                    send(&s[i..]).await;
                } else if &n % &m3 == zero {
                    println!("{}: {}", m3, n);
                    send(&s[i..]).await;
                }
            }
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
