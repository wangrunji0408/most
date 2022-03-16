use log::*;
use most2::*;
use std::io::{IoSlice, Read, Write};
use std::net::TcpStream;
use std::time::{Duration, Instant};

const IN_IP: &str = "59.110.124.141:10001";
const OUT_IP: &str = "59.110.124.141:10002";

fn main() {
    env_logger::init();

    let mut send_tcp = TcpStream::connect(OUT_IP).unwrap();
    send_tcp.set_nonblocking(true).unwrap();
    send_tcp.set_nodelay(true).unwrap();

    let mut get_tcp = TcpStream::connect(IN_IP).unwrap();
    get_tcp
        .write(format!("GET HTTP/1.1\r\nHost: {IN_IP}\r\n\r\n").as_bytes())
        .unwrap();
    const OK_HEADER: &str = "HTTP/1.1 200 OK\r\nServer: Most\r\nContent-type: text/plain\r\n\r\n";
    let mut buf = [0; 1024];
    let len = get_tcp.read(&mut buf[..OK_HEADER.len()]).unwrap();
    assert_eq!(&buf[..len], OK_HEADER.as_bytes());

    let mut m1 = M1Data::default();
    let mut buf = [0; 1024];
    let mut prev = vec![];
    let mut stat = Stat::new();
    loop {
        let len = get_tcp.read(&mut buf).unwrap();
        let t0 = Instant::now();
        for i in 0..len {
            let x = buf[i] - b'0';
            if let Some(len) = m1.push(x) {
                let mut zeros = 0;
                let mut i = i;
                while i + 1 < len && buf[i + 1] == b'0' {
                    zeros += 1;
                    i += 1;
                }
                send(&mut send_tcp, len, zeros, &prev, &buf[..=i]);
                stat.add(1, len, zeros, t0);
            }
        }
        // update prev
        prev.extend_from_slice(&buf[..len]);
        if prev.len() > N {
            prev.drain(..prev.len() - N);
        }
    }
}

fn send(tcp: &mut TcpStream, len: usize, zeros: usize, prev: &[u8], buf: &[u8]) {
    const HEADER: &str = "POST /submit HTTP/1.1\r\nHost: 59.110.124.141:10002\r\nUser-Agent: Go-http-client/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: ";
    let mut len_strs = vec![];
    for i in 0..=zeros {
        len_strs.push(format!("{}\r\n\r\n", len + i));
    }
    let mut iov = vec![];
    for i in 0..=zeros {
        iov.extend([
            IoSlice::new(HEADER.as_bytes()),
            IoSlice::new(len_strs[i].as_bytes()),
            IoSlice::new(&prev[(prev.len() + buf.len() - zeros - len).min(prev.len())..]),
            IoSlice::new(&buf[(buf.len() - zeros).max(len) - len..buf.len() - i]),
        ]);
        // let mut s = vec![];
        // s.extend_from_slice(&prev[(prev.len() + buf.len() - zeros - len).min(prev.len())..]);
        // s.extend_from_slice(&buf[(buf.len() - zeros).max(len) - len..buf.len() - i]);
        // info!("{}", String::from_utf8(s).unwrap());
    }
    match tcp.write_vectored(&iov) {
        Ok(_) => {}
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
            log::warn!("TCP send would block, ignore");
        }
        Err(e) => panic!("{}", e),
    }
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

    fn add(&mut self, k: u8, len: usize, zeros: usize, t0: Instant) {
        // statistics
        let latency = t0.elapsed();
        self.dsum += latency;
        self.count += 1;
        let avg = self.dsum / self.count;
        let nps = self.count as f32 / self.t00.elapsed().as_secs_f32();
        log::info!("M{k} {len:3}+{zeros}  lat: {latency:>9?}  avg: {avg:>9?}  nps: {nps:.3?}");
    }
}
