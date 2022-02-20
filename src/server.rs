const IN_IP: &str = "172.24.179.9:10001"; // mock
const OUT_IP: &str = "172.24.179.9:10002"; // mock

use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    time::{Instant, Duration},
};

fn main() {
    let put_listener = TcpListener::bind(IN_IP).unwrap();
    let get_listener = TcpListener::bind(OUT_IP).unwrap();
    let data = std::iter::repeat('1').take(200).collect::<String>();
    let mut buf = [0; 1024];
    loop {
        let (mut put, src) = put_listener.accept().unwrap();
        let (mut get, _) = get_listener.accept().unwrap();
        println!("accept: {}", src);
        const OK_HEADER: &str =
            "HTTP/1.1 200 OK\r\nServer: Most\r\nContent-type: text/plain\r\n\r\n";
        const M1: &str = "104648257118348370704723119";

        put.write(OK_HEADER.as_bytes()).unwrap();
        let mut dsum = Duration::default();
        let mut count = 0;
        for i in 0.. {
            if i % 3 == 1 {
                let mut s = String::new();
                for _ in 0..200 {
                    s.push('0');
                }
                s += M1;
                let t0 = Instant::now();
                if let Err(e) = put.write(s.as_bytes()) {
                    break;
                }
                let len = get.read(&mut buf).unwrap();
                let d = t0.elapsed();
                count += 1;
                dsum += d;
                println!("{:>10?} avg: {:>10?}", d, dsum / count);
            } else {
                if let Err(e) = put.write(data.as_bytes()) {
                    break;
                }
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}
