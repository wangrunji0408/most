#![no_std]
#![no_main]
#![feature(abi_efiapi)]
#![deny(unused_must_use)]

#[macro_use]
extern crate alloc;

#[macro_use]
extern crate log;

use uefi::table::boot::*;
use uefi::{
    prelude::*,
    proto::net::{tcp4, udp4, Ipv4Address},
    Event, Guid, Result,
};

#[entry]
fn efi_main(image: uefi::Handle, mut st: SystemTable<Boot>) -> Status {
    // Initialize utilities (logging, memory allocation...)
    uefi_services::init(&mut st).expect("failed to initialize utilities");
    let bs = st.boot_services();

    log::set_max_level(log::LevelFilter::Debug);
    let config = Config {
        local_addr: Ipv4Address::from(192, 168, 2, 3),
        remote_addr: Ipv4Address::from(192, 168, 2, 1),
        input_port: 10001,
        output_port: 10002,
    };
    let mut uefi = Uefi::open(bs, &config);

    const OK_HEADER: &str = "HTTP/1.1 200 OK\r\nServer: Most\r\nContent-type: text/plain\r\n\r\n";
    let mut buf = [0; 1024];
    let len = uefi.get_input(&mut buf[..OK_HEADER.len()]).unwrap();
    assert_eq!(&buf[..len], OK_HEADER.as_bytes());

    loop {
        let len = uefi.get_input(&mut buf).unwrap();
        let msg = core::str::from_utf8(&buf[..len]).unwrap();
        info!("get: {:?}", msg);
    }

    panic!("end");
    Status::SUCCESS
}

#[derive(Debug)]
struct Config {
    local_addr: Ipv4Address,
    remote_addr: Ipv4Address,
    input_port: u16,
    output_port: u16,
}

struct Uefi<'a> {
    bs: &'a BootServices,
    event: Event,
    input: &'a mut tcp4::Tcp4,
    // output: &'a mut tcp4::Tcp4,
}

impl<'a> Uefi<'a> {
    fn open(bs: &'a BootServices, config: &Config) -> Self {
        info!("opening UEFI services: {:#?}", config);
        Uefi {
            bs,
            input: open_tcp(bs, config, config.input_port),
            // output: open_tcp(bs, config, config.output_port),
            event: unsafe { bs.create_event(EventType::empty(), Tpl::APPLICATION, None, None) }
                .expect("failed to create event"),
        }
    }

    fn get_input(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut token = tcp4::ReceiveToken::new(unsafe { self.event.unsafe_clone() }, buf);
        // debug!("wait for input");
        self.input.receive(&mut token)?;
        while token.status() == Status::NOT_READY {
            let _ = self.input.poll();
        }
        // let msg = core::str::from_utf8(token.as_ref()).unwrap();
        // debug!("get: {:?}", msg);
        Ok(token.len())
    }
}

fn open_tcp<'a>(bs: &'a BootServices, config: &Config, port: u16) -> &'a mut tcp4::Tcp4 {
    let tcp = bs
        .locate_protocol::<tcp4::Tcp4>()
        .expect("failed to get Tcp4 protocol");
    let tcp = unsafe { &mut *tcp.get() };

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
    .expect("failed to config TCP");

    let event = unsafe { bs.create_event(EventType::empty(), Tpl::APPLICATION, None, None) }
        .expect("failed to create event");
    let mut token = tcp4::ConnectionToken::new(event);
    tcp.connect(&mut token).expect("failed to connect");
    while token.status() == Status::NOT_READY {
        let _ = tcp.poll();
    }

    tcp
}
