use message::{Message, MessageType};
use std::{io, marker, net, sync, thread, time};

use tuple::*;

pub mod message;
pub mod parser;
pub mod tuple;
pub mod utils;

type MessageSender = sync::mpsc::Sender<Message>;
type MessageRecv = sync::mpsc::Receiver<Message>;
type ValueSender = sync::mpsc::Sender<Tuple<Value>>;
type ValueRecv = sync::mpsc::Receiver<Tuple<Value>>;
type ArcMutex<T> = sync::Arc<sync::Mutex<T>>;
type LocalTuples = ArcMutex<Vec<Tuple<Value>>>;

pub struct Linda<U> {
    rx: ValueRecv,
    tx: MessageSender,
    output_stream: ArcMutex<U>,
    local_tuples: LocalTuples,
    ip: net::SocketAddr,
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
    println!("Sending: {msg:?}");
    match output.lock() {
        Ok(mut guard) => msg.send(&mut *guard),
        Err(e) => Err(format!("Failed to open output! {e}")),
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
    mut input: T,
    output: ArcMutex<U>,
    local_tuples: LocalTuples,
    tx: ValueSender,
    rx: MessageRecv,
    ip: net::SocketAddr,
) {
    let mut request = None;
    loop {
        let msg = match Message::recv(&mut input) {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("{e}");
                break;
            }
        };
        println!("Received: {msg:?}");
        match rx.try_recv() {
            Ok(msg) => match msg.tuple {
                MessageType::Request(req) => request = Some(req),
                _ => eprintln!("Wrong message type received! Skipping."),
            },
            Err(sync::mpsc::TryRecvError::Disconnected) => {
                eprintln!("Disconnected from main thread!");
                break;
            }
            Err(sync::mpsc::TryRecvError::Empty) => (),
        }
        println!("Request: {request:?}");
        match msg.tuple {
            MessageType::Value(val) if msg.ip == ip => add_tuple(&local_tuples, val),
            MessageType::Value(val) if matches!(request, Some(ref req) if satisfies(&req, &val)) => {
                if let Err(e) = tx.send(val) {
                    eprintln!("{e}");
                }
                request = None;
            }
            MessageType::Value(_) => {
                if let Err(e) = send(&output, msg) {
                    eprintln!("{e}");
                }
            }
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

impl<U: 'static + io::Write + marker::Send> Linda<U> {
    pub fn new<T: 'static + io::Read + marker::Send>(
        input_stream: T,
        output_stream: U,
        ip: net::SocketAddr,
    ) -> Linda<U> {
        let (tx_to_worker, rx_to_worker) = sync::mpsc::channel();
        let (tx_from_worker, rx_from_worker) = sync::mpsc::channel();

        let output_stream = sync::Arc::new(sync::Mutex::new(output_stream));
        let os_clone = output_stream.clone();

        let tuples = sync::Arc::new(sync::Mutex::new(vec![]));
        let tuples_clone = tuples.clone();
        thread::spawn(move || {
            worker(
                input_stream,
                os_clone,
                tuples_clone,
                tx_from_worker,
                rx_to_worker,
                ip.clone(),
            )
        });

        Linda {
            rx: rx_from_worker,
            tx: tx_to_worker,
            output_stream,
            local_tuples: tuples,
            ip,
        }
    }

    pub fn out(&self, tuple: Tuple<Value>) -> Result<(), String> {
        let msg = Message::value(tuple, self.ip.clone());
        send(&self.output_stream, msg)
    }

    pub fn input(
        &self,
        tuple: Tuple<Request>,
        timeout: time::Duration,
    ) -> Result<Tuple<Value>, String> {
        if let Ok(tuple) = self.inp(&tuple) {
            return Ok(tuple);
        }

        let msg = Message::request(tuple, self.ip.clone());
        if let Err(e) = self.tx.send(msg.clone()) {
            return Err(e.to_string());
        }

        send(&self.output_stream, msg)?;

        match self.rx.recv_timeout(timeout) {
            Ok(tuple) => Ok(tuple),
            Err(e) => Err(e.to_string()),
        }
    }

    pub fn read(
        &self,
        tuple: Tuple<Request>,
        timeout: time::Duration,
    ) -> Result<Tuple<Value>, String> {
        let tuple = self.input(tuple, timeout)?;
        add_tuple(&self.local_tuples, tuple.clone());

        Ok(tuple)
    }

    pub fn inp(&self, tuple: &Tuple<Request>) -> Result<Tuple<Value>, String> {
        match find_tuple(&self.local_tuples, &tuple) {
            Some(tuple) => Ok(tuple),
            None => Err(String::from("No tuple")),
        }
    }

    pub fn rdp(&self, tuple: &Tuple<Request>) -> Result<Tuple<Value>, String> {
        let tuple = self.inp(tuple)?;
        add_tuple(&self.local_tuples, tuple.clone());
        Ok(tuple)
    }
}
