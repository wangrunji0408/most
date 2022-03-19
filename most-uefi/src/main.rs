#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![deny(unused_must_use)]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate log;

use core::fmt::Write;
use core::ops::Deref;

use most2::*;
use uefi::table::boot::*;
use uefi::{
    prelude::*,
    proto::net::{tcp4, Ipv4Address},
    Error, Event, Result,
};

#[entry]
fn efi_main(image: uefi::Handle, mut st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&mut st).expect("failed to initialize utilities");
    let bs = st.boot_services();
    // disable watchdog
    bs.set_watchdog_timer(0, 0x10000, None)
        .expect("failed to disable watchdog");

    log::set_max_level(log::LevelFilter::Debug);
    let config = Config {
        local_addr: Ipv4Address::from(192, 168, 2, 3),
        remote_addr: Ipv4Address::from(192, 168, 2, 1),
        input_port: 10001,
        output_port: 10002,
    };
    info!("{:#?}", config);
    let mut uefi = Uefi::open(bs, image, &config);
    while let Err(e) = work(&mut uefi) {
        error!("{:?}", e);
        uefi.reset();
    }
    Status::SUCCESS
}

fn work(uefi: &mut Uefi) -> Result<(), &'static str> {
    uefi.connect()
        .map_err(|e| Error::new(e.status(), "failed to connect"))?;

    const OK_HEADER: &str = "HTTP/1.1 200 OK\r\nServer: Most\r\nContent-type: text/plain\r\n\r\n";
    let mut buf = [0; 1024];
    let len = uefi
        .get_input(&mut buf[..OK_HEADER.len()])
        .map_err(|e| Error::new(e.status(), "failed to get input header"))?;
    assert_eq!(&buf[..len], OK_HEADER.as_bytes());

    let mut m1 = M1Data::default();
    let mut m2 = M2Data::default();
    let mut m3 = M3Data::default();
    let mut m4 = M4Data::default();
    let mut prev = vec![];
    loop {
        let len = uefi
            .get_input(&mut buf)
            .map_err(|e| Error::new(e.status(), "failed to get input data"))?;
        // let msg = core::str::from_utf8(&buf[..len]).unwrap();
        // debug!("get: {:?}", msg);

        for i in 0..len {
            let x = buf[i] - b'0';
            let mut send = |k, len| {
                let mut zeros = 0;
                let mut i = i;
                while i + 1 < len && buf[i + 1] == b'0' {
                    zeros += 1;
                    i += 1;
                }
                send(uefi, len, zeros, &prev, &buf[..=i])?;
                info!("M{k} {len:3}+{zeros}");
                Ok(())
            };
            if let Some(len) = m1.push(x) {
                send(1, len)?;
            }
            if let Some(len) = m2.push(x) {
                send(2, len)?;
            }
            if let Some(len) = m3.push(x) {
                send(3, len)?;
            }
            if let Some(len) = m4.push(x) {
                send(4, len)?;
            }
        }
        // update prev
        prev.extend_from_slice(&buf[..len]);
        if prev.len() > N {
            prev.drain(..prev.len() - N);
        }
        // prepare for next
        m1.prepare();
        m2.prepare();
        m3.prepare();
        m4.prepare();
    }
}

fn send(
    uefi: &mut Uefi,
    len: usize,
    zeros: usize,
    prev: &[u8],
    buf: &[u8],
) -> Result<(), &'static str> {
    const HEADER: &str = "POST /submit HTTP/1.1\r\nHost: 59.110.124.141:10002\r\nUser-Agent: Go-http-client/1.1\r\nContent-Type: application/x-www-form-urlencoded\r\nContent-Length: ";
    let mut output = StaticString::new();
    for i in 0..=zeros {
        write!(&mut output, "{HEADER}{}\r\n\r\n", len + i).unwrap();
        output.extend_from_slice(&prev[(prev.len() + buf.len() - zeros - len).min(prev.len())..]);
        output.extend_from_slice(&buf[(buf.len() - zeros).max(len) - len..buf.len() - (zeros - i)]);
    }
    uefi.send_output(&output)
        .map_err(|e| Error::new(e.status(), "failed to send output"))
}

struct StaticString {
    buf: [u8; 0x1000],
    len: usize,
}

impl StaticString {
    fn new() -> Self {
        StaticString {
            buf: unsafe { core::mem::MaybeUninit::uninit().assume_init() },
            len: 0,
        }
    }
    fn extend_from_slice(&mut self, s: &[u8]) {
        self.buf[self.len..self.len + s.len()].copy_from_slice(s);
        self.len += s.len();
    }
}

impl Write for StaticString {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if self.len + s.len() > self.buf.len() {
            return Err(core::fmt::Error);
        }
        self.extend_from_slice(s.as_bytes());
        Ok(())
    }
}

impl Deref for StaticString {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

#[derive(Debug)]
struct Config {
    local_addr: Ipv4Address,
    remote_addr: Ipv4Address,
    input_port: u16,
    output_port: u16,
}

struct Uefi<'a> {
    config: &'a Config,
    event: Event,
    input: ScopedProtocol<'a, tcp4::Tcp4>,
    output: ScopedProtocol<'a, tcp4::Tcp4>,
}

impl<'a> Uefi<'a> {
    fn open(bs: &'a BootServices, image: uefi::Handle, config: &'a Config) -> Self {
        Uefi {
            config,
            input: open_tcp(bs, image),
            output: open_tcp(bs, image),
            event: unsafe { bs.create_event(EventType::empty(), Tpl::APPLICATION, None, None) }
                .expect("failed to create event"),
        }
    }

    fn event(&self) -> Event {
        unsafe { self.event.unsafe_clone() }
    }

    fn input<'b>(&mut self) -> &'b mut tcp4::Tcp4 {
        unsafe { &mut *self.input.interface.get() }
    }

    fn output<'b>(&mut self) -> &'b mut tcp4::Tcp4 {
        unsafe { &mut *self.output.interface.get() }
    }

    fn connect(&mut self) -> Result {
        let mut token = tcp4::ConnectionToken::new(self.event());
        configure_tcp(self.input(), &self.config, self.config.input_port)?;
        self.input().connect(&mut token)?;
        busy_poll(self.input(), &token)?;

        let mut token = tcp4::ConnectionToken::new(self.event());
        configure_tcp(self.output(), &self.config, self.config.output_port)?;
        self.output().connect(&mut token)?;
        busy_poll(self.output(), &token)?;
        Ok(())
    }

    fn reset(&mut self) {
        let _ = self.input().reset();
        let _ = self.output().reset();
    }

    fn get_input(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut token = tcp4::ReceiveToken::new(self.event(), buf);
        // debug!("wait for input");
        self.input().receive(&mut token)?;
        busy_poll(self.input(), &token)?;
        // let msg = core::str::from_utf8(token.as_ref()).unwrap();
        // debug!("get: {:?}", msg);
        Ok(token.len())
    }

    fn send_output(&mut self, buf: &[u8]) -> Result {
        let mut token = tcp4::TransmitToken::new(self.event(), buf);
        self.output().transmit(&mut token)?;
        busy_poll(self.output(), &token)?;
        Ok(())
    }
}

fn busy_poll(tcp: &mut tcp4::Tcp4, token: &tcp4::CompletionToken) -> Result {
    let mut i = 0;
    while token.status() == Status::NOT_READY {
        let _ = tcp.poll();
        i += 1;
        if i == 10000000 {
            return Err(Error::from(Status::TIMEOUT));
        }
    }
    if token.status() != Status::SUCCESS {
        return Err(Error::from(token.status()));
    }
    Ok(())
}

fn open_tcp<'a>(bs: &'a BootServices, image: uefi::Handle) -> ScopedProtocol<'a, tcp4::Tcp4> {
    let tcp4sb = bs
        .locate_protocol::<tcp4::Tcp4ServiceBinding>()
        .expect("failed to get Tcp4ServiceBinding protocol");
    let tcp4sb = unsafe { &mut *tcp4sb.get() };
    let handle = tcp4sb.create_child().expect("failed to create child");
    let tcp_protocol = bs
        .open_protocol::<tcp4::Tcp4>(
            OpenProtocolParams {
                handle,
                agent: image,
                controller: None,
            },
            OpenProtocolAttributes::GetProtocol,
        )
        .expect("failed to open net protocol");
    tcp_protocol
}

fn configure_tcp(tcp: &mut tcp4::Tcp4, config: &Config, port: u16) -> Result {
    tcp.configure(&tcp4::ConfigData {
        type_of_service: 0,
        time_to_live: 64,
        access_point: tcp4::AccessPoint {
            use_default_address: false,
            station_addr: config.local_addr,
            subnet_mask: Ipv4Address::from(255, 255, 255, 0),
            station_port: 0,
            remote_addr: config.remote_addr,
            remote_port: port,
            active_flag: true,
        },
        control_options: tcp4::Options {
            ..Default::default()
        },
    })
}
