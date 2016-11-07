extern crate futures;
#[macro_use]
extern crate tokio_core;
extern crate tokio_serial;
extern crate dotenv;

use self::dotenv::dotenv;
use std::env;
use std::io::{self, Read};
use std::str;

use futures::{Future, Poll};
use tokio_core::reactor::Core;
use tokio_serial::SerialPort;
use tokio_serial::{PortSettings, BaudRate, CharSize, Parity, StopBits, FlowControl};

struct Reader {
    port: SerialPort,
}

impl Future for Reader {
    type Item = ();
    type Error = io::Error;

    fn poll(&mut self) -> Poll<(), io::Error> {
        let mut buf = [0; 1024];
        loop {
            // TODO:
            // serial-rs TTYPort::read() calls poll() internally which results in timeout error
            // when there is no more bytes to read.
            // I think such implelemtation should be changed for better integration with tokio.
            // let n = try_nb!(self.port.read(&mut buf));
            let n = match self.port.read(&mut buf) {
                Ok(t) => t,
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    continue
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(futures::Async::NotReady)
                }
                Err(e) => return Err(e.into()),
            };
            if let Ok(s) = str::from_utf8(&buf[..n]) {
                print!("{}", s);
            }
        }
    }
}

fn main() {
    dotenv().ok();

    let port_name = env::var("SERIAL_PORT")
        .expect("Environment variable SERIAL_PORT must be specified");

    let mut app = Core::new().unwrap();

    let serial_port = SerialPort::open_with_settings(port_name.as_str(),
                                                     &PortSettings {
                                                         baud_rate: BaudRate::Baud115200,
                                                         char_size: CharSize::Bits8,
                                                         parity: Parity::ParityNone,
                                                         stop_bits: StopBits::Stop1,
                                                         flow_control: FlowControl::FlowNone,
                                                     },
                                                     &app.handle())
        .unwrap();

    app.run(Reader {
        port: serial_port
    }).unwrap();
}
