//! An TCP server meant as a backend for persistent DCS missions
//!
//!     cargo run [addr]

extern crate futures;
extern crate tokio_core;

use std::fmt;
use std::io;
use std::env;
use std::net::SocketAddr;

use futures::{Future, Stream, Sink};
use tokio_core::reactor::Core;
use tokio_core::net::{UdpSocket, UdpCodec};

fn main() {
    // use first argument as address or default to 127.0.0.1:8080
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    run(&addr);
}

fn run(addr: &SocketAddr) {
    // create eventloop and aquire handle to it
    let mut core = Core::new().unwrap();
    let handle = core.handle();

    // create UDP listener
    let socket = UdpSocket::bind(&addr, &handle).unwrap();
    println!("Listening on {}", addr);

    let (sink, stream) = socket.framed(LineCodec).split();
    let (tx, rx) = futures::sync::mpsc::unbounded();

    let socket_writer = rx.fold(sink, |sink, (addr, msg)| {
        sink.send((addr, msg))
            .map(|sink| sink)
            .map_err(|_| ())
    });

    handle.spawn(socket_writer.map(|_| ()));

    let srv = stream.for_each(move |(addr, mut msg)| {
        msg.pop(); // remove trailing newline
        let msg = String::from_utf8_lossy(&msg);

        println!("received {}", msg);

        let parts = msg.split(":").collect::<Vec<_>>();

        if parts.len() >= 2 {
            match parts[0] {
                "ev" => {
                    let event = to_event(parts[1]);
                    println!("received event {}", event);

                    match event {
                        Event::Ejection => {
                            println!("sending message ...");

                            tx.unbounded_send((addr, b"cyaaa".to_vec())).unwrap();
                        },
                        _ => {}
                    }
                }
                _ => {
                    println!("received invalid operation: {}", parts[0]);
                }
            }
        }

        Ok(())
    });

    core.run(srv).unwrap();
}

pub struct LineCodec;

impl UdpCodec for LineCodec {
    type In = (SocketAddr, Vec<u8>);
    type Out = (SocketAddr, Vec<u8>);

    fn decode(&mut self, addr: &SocketAddr, buf: &[u8]) -> io::Result<Self::In> {
        Ok((*addr, buf.to_vec()))
    }

    fn encode(&mut self, (addr, buf): Self::Out, into: &mut Vec<u8>) -> SocketAddr {
        into.extend(buf);
        addr
    }
}

#[derive(Debug)]
enum Event {
    Invalid,
    Shot,
    Hit,
    Takeoff,
    Land,
    Crash,
    Ejection,
    Refueling,
    Dead,
    PilotDead,
    BaseCaptured,
    MissionStart,
    MissionEnd,
    TookControl,
    RefuelingStop,
    Birth,
    HumanFailure,
    EngineStartup,
    EngineShutdown,
    PlayerEnterUnit,
    PlayerLeaveUnit,
    PlayerComment,
    ShootingStart,
    ShootingEnd,
    Max,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

fn to_event(r: &str) -> Event {
    match r {
        "0" => Event::Invalid,
        "1" => Event::Shot,
        "2" => Event::Hit,
        "3" => Event::Takeoff,
        "4" => Event::Land,
        "5" => Event::Crash,
        "6" => Event::Ejection,
        "7" => Event::Refueling,
        "8" => Event::Dead,
        "9" => Event::PilotDead,
        "10" => Event::BaseCaptured,
        "11" => Event::MissionStart,
        "12" => Event::MissionEnd,
        "13" => Event::TookControl,
        "14" => Event::RefuelingStop,
        "15" => Event::Birth,
        "16" => Event::HumanFailure,
        "17" => Event::EngineStartup,
        "18" => Event::EngineShutdown,
        "19" => Event::PlayerEnterUnit,
        "20" => Event::PlayerLeaveUnit,
        "21" => Event::PlayerComment,
        "22" => Event::ShootingStart,
        "23" => Event::ShootingEnd,
        "24" => Event::Max,
        _ => {
            println!("Received unknown event {}", r);
            Event::Invalid
        }
    }
}
