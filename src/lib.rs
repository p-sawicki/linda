use message::{Message, MessageType};
use std::{io, net, sync};

use tuple::*;

pub mod message;
pub mod parser;
pub mod tuple;
pub mod utils;

type MessageSender = sync::mpsc::Sender<Message>;
type MessageRecv = sync::mpsc::Receiver<Message>;
type ArcMutex<T> = sync::Arc<sync::Mutex<T>>;
type LocalTuples = ArcMutex<Vec<Tuple<Value>>>;

pub struct Handle<T: io::Read, U: io::Write> {
    rx: MessageRecv,
    input_stream: ArcMutex<T>,
    output_stream: ArcMutex<U>,
    local_tuples: LocalTuples,
}

fn satisfies(request: &Tuple<Request>, value: &Tuple<Value>) -> bool {
    if request.len() == value.len() {
        for (req, val) in request.iter().zip(value.iter()) {
            if !req.satisfies(val) {
                return false;
            }
        }
        true
    } else {
        false
    }
}

fn send<T: io::Write>(output: &ArcMutex<T>, msg: Message) -> Result<(), String> {
    match output.lock() {
        Ok(mut guard) => msg.send(&mut *guard),
        Err(e) => Err(format!("Failed to open output! {e}")),
    }
}

fn recv<T: io::Read>(input: &ArcMutex<T>) -> Result<Message, String> {
    match input.lock() {
        Ok(mut guard) => Message::recv(&mut *guard),
        Err(e) => return Err(format!("Failed to open input! {e}")),
    }
}

fn add_tuple(local_tuples: &LocalTuples, tuple: Tuple<Value>) {
    match local_tuples.lock() {
        Ok(mut guard) => guard.push(tuple),
        Err(e) => eprintln!("{e}"),
    }
}

fn find_tuple(local_tuples: &LocalTuples, request: &Tuple<Request>) -> Option<Tuple<Value>> {
    match local_tuples.lock() {
        Ok(mut guard) => {
            let mut i = 0;
            for tuple in guard.iter() {
                if satisfies(request, tuple) {
                    break;
                }
                i += 1;
            }
            if i < guard.len() {
                Some(guard.remove(i))
            } else {
                None
            }
        }
        Err(e) => {
            eprintln!("{e}");
            None
        }
    }
}

fn worker<T: io::Read, U: io::Write>(
    input: ArcMutex<T>,
    output: ArcMutex<U>,
    local_tuples: LocalTuples,
    tx: MessageSender,
    rx: MessageRecv,
    ip: net::SocketAddr,
) {
    loop {
        let msg = match recv(&input) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };
        match msg.tuple {
            MessageType::Value(val) if msg.ip == ip => add_tuple(&local_tuples, val),
            MessageType::Value(ref val) => match rx.try_recv() {
                Ok(request) => match request.tuple {
                    MessageType::Request(req) if satisfies(&req, &val) => {
                        if let Err(e) = tx.send(msg) {
                            eprintln!("{e}");
                        }
                    }
                    MessageType::Request(_) => {
                        if let Err(e) = send(&output, msg) {
                            eprintln!("{e}");
                        }
                    }
                    _ => {
                        eprintln!("Incorrect message type sent to worker!");
                    }
                },
                Err(e) => {
                    eprintln!("{e}");
                }
            },
            MessageType::Request(_) if msg.ip == ip => (),
            MessageType::Request(ref request) => match find_tuple(&local_tuples, request) {
                Some(value) => {
                    if let Err(e) = send(&output, Message::value(value, ip)) {
                        eprintln!("{e}");
                    }
                }
                None => {
                    if let Err(e) = send(&output, msg) {
                        eprintln!("{e}");
                    }
                }
            },
        }
    }
}

pub fn init<T: io::Read, U: io::Write>(input_stream: T, output_stream: U) {}
