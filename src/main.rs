//! An TCP server meant as a backend for persistent DCS missions
//!
//!     cargo run [addr]

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;

use std::fmt;
use std::env;
use std::net::SocketAddr;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;
use std::iter;
use std::io::{Error, ErrorKind, BufReader};

use futures::Future;
use futures::stream::{self, Stream};
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;
use tokio_io::io;
use tokio_io::AsyncRead;

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

    // create TCP listener
    let socket = TcpListener::bind(&addr, &handle).unwrap();
    println!("Listening on {}", addr);

    // single-threaded server for now -> just use Rc and RefCell to store map of
    // all connections
    let connections = Rc::new(RefCell::new(HashMap::new()));

    // convert TCP listener to stream of incoming connections
    let done = socket.incoming().for_each(move |(socket, addr)| {
        // this closure represents an accepted client (with socket being the client
        // connection and addr the remote address of the client)

        //
        println!("New Connection: {}", addr);
        let (reader, writer) = socket.split();

        // create a channel for our stream
        let (tx, rx) = futures::sync::mpsc::unbounded();
        connections.borrow_mut().insert(addr, tx);

        let connections_inner = connections.clone();
        let reader = BufReader::new(reader);

        // create an infinite iterator to read lines from the socket
        let iter = stream::iter_ok::<_, Error>(iter::repeat(()));
        let socket_reader = iter.fold(reader, move |reader, _| {
            // read until newline
            let line = io::read_until(reader, b'\n', Vec::new());
            let line = line.and_then(|(reader, vec)| {
                if vec.len() == 0 {
                    Err(Error::new(ErrorKind::BrokenPipe, "broken pipe"))
                } else {
                    Ok((reader, vec))
                }
            });

            // convert bytes into string
            let line = line.map(|(reader, vec)| (reader, String::from_utf8(vec)));

            let connections = connections_inner.clone();
            line.map(move |(reader, message)| {
                let mut conns = connections.borrow_mut();
                if let Ok(mut msg) = message {
                    msg.pop(); // remove trailing newline

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

                                        let iter = conns.iter_mut()
                                                        // .filter(|&(&k, _)| k != addr)
                                                        .map(|(_, v)| v);
                                        for tx in iter {
                                            tx.unbounded_send("cyaaa".to_string()).unwrap();
                                        }
                                    },
                                    _ => {}
                                }

                            }
                            _ => {
                                println!("received invalid operation: {}", parts[0]);
                            }
                        }
                    }

                    // let iter = conns.iter_mut()
                    //                 // .filter(|&(&k, _)| k != addr)
                    //                 .map(|(_, v)| v);
                    // for tx in iter {
                    //     tx.send(format!("{}: {}", addr, msg)).unwrap();
                    // }
                } else {
                    let tx = conns.get_mut(&addr).unwrap();
                    tx.unbounded_send("invalid UTF-8".to_string()).unwrap();
                }
                reader
            })
        });

        // writer part
        let socket_writer = rx.fold(writer, |writer, msg| {
            let amt = io::write_all(writer, msg.into_bytes());
            let amt = amt.map(|(writer, _)| writer);
            amt.map_err(|_| ())
        });

        // combine reader and writer to wait for either half to be done to tear down the other
        let connections = connections.clone();
        let socket_reader = socket_reader.map_err(|_| ());
        let connection = socket_reader.map(|_| ()).select(socket_writer.map(|_| ()));
        handle.spawn(connection.then(move |_| {
            connections.borrow_mut().remove(&addr);
            println!("Connection {} closed", addr);
            Ok(())
        }));

        Ok(())
    });

    // run server ...
    core.run(done).unwrap();
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
