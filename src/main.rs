extern crate jack;
extern crate jack_sys;

use std::io;
use std::sync::{mpsc, Mutex};
use jack::{Control, Client};
use jack_sys::jack_get_ports;

fn main() {
    let (client, _status) =
        jack::Client::new("rust-autoconnect", jack::ClientOptions::NO_START_SERVER).unwrap();


    let (tx, rx) = mpsc::channel::<Data>();
    let mtx = Mutex::new(tx);

    let process = jack::ClosureProcessHandler::new(move |_: &jack::Client, _: &jack::ProcessScope| Control::Continue);
    let notifications = Notifications::new(mtx);
    let active_client = client.activate_async(notifications, process).unwrap();


    for data in rx {
        active_client.as_client().connect_ports_by_name(data.from.as_str(), data.to.as_str());
    }
}

struct Data {
    from: String,
    to: String,
}

struct Notifications {
    ch: Mutex<std::sync::mpsc::Sender<Data>>
}

impl Notifications {
    pub fn new(ch: Mutex<mpsc::Sender<Data>>) -> Self {
        Notifications { ch }
    }
}

impl jack::NotificationHandler for Notifications {
    fn port_registration(&mut self, client: &Client, _port_id: u32, is_registered: bool) {
        if !is_registered { return; }

        let strings = client.ports(None, None, jack::PortFlags::empty());

        for string in strings {
            match string.as_str() {
                "deadbeef:deadbeef_1" => {
                    self.ch.get_mut().unwrap().send(Data { from: string, to: "Non-Mixer/music:in-1".to_string() });
                }
                "deadbeef:deadbeef_2" => {
                    self.ch.get_mut().unwrap().send(Data { from: string, to: "Non-Mixer/music:in-2".to_string() });
                }
                _ => ()
            }
        }
    }
}
