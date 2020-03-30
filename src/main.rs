extern crate jack;

use jack::{Client, Control};
use std::collections::HashMap;
use std::io;
use std::sync::{mpsc, Mutex};

fn main() {
    let mut map_table = HashMap::new();
    map_table.insert(
        String::from("deadbeef:deadbeef_1"),
        String::from("Non-Mixer/music:in-1"),
    );
    map_table.insert(
        String::from("deadbeef:deadbeef_2"),
        String::from("Non-Mixer/music:in-2"),
    );

    let (client, _status) =
        jack::Client::new("rust-autoconnect", jack::ClientOptions::NO_START_SERVER).unwrap();

    let (tx, rx) = mpsc::channel::<Data>();
    let mtx = Mutex::new(tx);

    let process =
        jack::ClosureProcessHandler::new(move |_: &jack::Client, _: &jack::ProcessScope| {
            Control::Continue
        });
    let notifications = Notifications::new(mtx, Mutex::new(map_table));
    let active_client = client.activate_async(notifications, process).unwrap();

    for data in rx {
        active_client
            .as_client()
            .connect_ports_by_name(data.from.as_str(), data.to.as_str());
    }
}

struct Data {
    from: String,
    to: String,
}

struct Notifications {
    ch: Mutex<std::sync::mpsc::Sender<Data>>,
    map_table: Mutex<HashMap<String, String>>,
}

impl Notifications {
    pub fn new(ch: Mutex<mpsc::Sender<Data>>, map_table: Mutex<HashMap<String, String>>) -> Self {
        Notifications { ch, map_table }
    }
}

impl jack::NotificationHandler for Notifications {
    fn port_registration(&mut self, client: &Client, _port_id: u32, is_registered: bool) {
        if !is_registered {
            return;
        }

        let strings = client.ports(None, None, jack::PortFlags::empty());

        for string in strings {
            for (key, value) in self.map_table.lock().unwrap().iter() {
                if string == *key {
                    self.ch.get_mut().unwrap().send(Data {
                        from: key.clone(),
                        to: value.clone(),
                    });
                }
            }
        }
    }
}
