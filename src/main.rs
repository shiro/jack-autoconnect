extern crate jack;
extern crate json;
extern crate xdg;

use jack::{Client, Control};
use json::{object, JsonValue};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::{mpsc, Mutex};

fn create_config(path: &Path) -> Result<JsonValue, Box<dyn Error>> {
    let default_config = object! {
        mappings: {},
    };

    fs::write(path, default_config.dump()).unwrap_or_else(|_| {
        panic!(
            "cannot initialize configuration file at '{}'",
            path.display()
        )
    });

    Ok(default_config)
}

fn read_config(path: &Path) -> Result<JsonValue, Box<dyn Error>> {
    let config_file = fs::read_to_string(path)?;

    let obj = match json::parse(config_file.as_ref()) {
        Ok(obj) => obj,
        Err(err) => return Err(Box::new(err)),
    };

    Ok(obj)
}

fn mappings_to_table(config: &mut JsonValue) -> HashMap<String, String> {
    let mut map_table = HashMap::new();
    for (a, b) in config["mappings"].entries() {
        if !b.is_string() {
            panic!(
                "error in mapping configuration: the value of '{}' is not a string",
                a
            );
        }

        map_table.insert(String::from(a), b.to_string());
    }
    map_table
}

fn main() {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("jack-autoconnect").unwrap();

    let config_path = xdg_dirs
        .place_config_file("config.json")
        .expect("cannot create configuration directory");

    let mut config = match read_config(config_path.borrow()) {
        Ok(config) => config,
        Err(_) => create_config(config_path.borrow()).unwrap(),
    };

    let map_table = mappings_to_table(&mut config);

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
        let _ = active_client
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
                    if let Ok(tx) = self.ch.get_mut() {
                        let _ = tx.send(Data {
                            from: key.clone(),
                            to: value.clone(),
                        });
                    }
                }
            }
        }
    }
}
