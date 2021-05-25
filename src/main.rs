extern crate jack;
extern crate json;
extern crate xdg;

use anyhow::*;
use jack::{Client, Control};
use json::{object, JsonValue};
use regex::*;
use std::borrow::Borrow;
use std::fs;
use std::path::Path;
use std::sync::{mpsc, Mutex};

fn create_config(path: &Path) -> Result<JsonValue> {
    let default_config = object! {
        mappings: {},
    };

    fs::write(path, default_config.dump())?;

    Ok(default_config)
}

fn read_config(path: &Path) -> Result<JsonValue> {
    let config_file = fs::read_to_string(path)?;
    let json_value = json::parse(config_file.as_ref())?;

    Ok(json_value)
}

fn mappings_to_table(config: &mut JsonValue) -> Result<MappingConfiguration> {
    let mut map_table = MappingConfiguration::default();
    for (key, value) in config["connect"].entries() {
        if !value.is_string() {
            return Err(anyhow!(
                "error in mapping configuration: the value of '{}' is not a string",
                key
            ));
        }
        let from_regex = Regex::new(key)?;
        let to_regex = Regex::new(value.as_str().unwrap())?;
        map_table.connect.push((from_regex, to_regex));
    }

    for (key, value) in config["disconnect"].entries() {
        if !value.is_string() {
            return Err(anyhow!(
                "error in mapping configuration: the value of '{}' is not a string",
                key
            ));
        }
        let from_regex = Regex::new(key)?;
        let to_regex = Regex::new(value.as_str().unwrap())?;
        map_table.disconnect.push((from_regex, to_regex));
    }
    Ok(map_table)
}

#[derive(Default)]
struct MappingConfiguration {
    connect: Vec<(Regex, Regex)>,
    disconnect: Vec<(Regex, Regex)>,
}

fn main() -> Result<()> {
    let xdg_dirs = xdg::BaseDirectories::with_prefix("jack-autoconnect").unwrap();

    let config_path = xdg_dirs
        .place_config_file("config.json")
        .expect("cannot create configuration directory");

    if !config_path.is_file() {
        create_config(&config_path).map_err(|err| {
            anyhow!(
                "failed to create config file '{}': {}",
                config_path.to_string_lossy(),
                err
            )
        })?;
    }

    let mut config = match read_config(config_path.borrow()) {
        Ok(config) => config,
        Err(_) => create_config(config_path.borrow()).unwrap(),
    };

    let map_table =
        mappings_to_table(&mut config).map_err(|err| anyhow!("failed to parse config: {}", err))?;

    let (client, _status) =
        jack::Client::new("rust-autoconnect", jack::ClientOptions::NO_START_SERVER)?;

    let (tx, rx) = mpsc::channel::<JackCommand>();

    let process =
        jack::ClosureProcessHandler::new(move |_: &jack::Client, _: &jack::ProcessScope| {
            Control::Continue
        });

    enforce_connect_rules(&client, &map_table, &tx);

    let notifications = Notifications::new(Mutex::new(tx), Mutex::new(map_table));
    let active_client = client.activate_async(notifications, process).unwrap();

    for cmd in rx {
        match cmd {
            JackCommand::CONNECT(data) => {
                println!("connecting '{}' to '{}'", data.from, data.to);
                let _ = active_client
                    .as_client()
                    .connect_ports_by_name(&data.from, &data.to);
            }
            JackCommand::DISCONNECT(data) => {
                println!("disconnecting '{}' to '{}'", data.from, data.to);
                let _ = active_client
                    .as_client()
                    .disconnect_ports_by_name(&data.from, &data.to);
            }
        }
    }
    Ok(())
}

struct PortPair {
    from: String,
    to: String,
}

enum JackCommand {
    CONNECT(PortPair),
    DISCONNECT(PortPair),
}

type CommandSender = mpsc::Sender<JackCommand>;

struct Notifications {
    tx: Mutex<CommandSender>,
    mapping_config: Mutex<MappingConfiguration>,
}

impl Notifications {
    pub fn new(tx: Mutex<CommandSender>, mapping_config: Mutex<MappingConfiguration>) -> Self {
        Notifications { tx, mapping_config }
    }
}

fn enforce_connect_rules(
    client: &Client,
    mapping_config: &MappingConfiguration,
    tx: &CommandSender,
) {
    let all_port_names = client.ports(None, None, jack::PortFlags::empty());

    for from_port_name in &all_port_names {
        for (from, to) in &mapping_config.connect {
            if !from.is_match(from_port_name) {
                continue;
            }

            for to_port_name in &all_port_names {
                if !to.is_match(to_port_name) {
                    continue;
                }

                let _ = tx.send(JackCommand::CONNECT(PortPair {
                    from: from_port_name.clone(),
                    to: to_port_name.clone(),
                }));
            }
        }

        for (from, to) in &mapping_config.disconnect {
            if !from.is_match(from_port_name) {
                continue;
            }

            for to_port_name in &all_port_names {
                if !to.is_match(to_port_name) {
                    continue;
                }

                let _ = tx.send(JackCommand::DISCONNECT(PortPair {
                    from: from_port_name.clone(),
                    to: to_port_name.clone(),
                }));
            }
        }
    }
}

impl jack::NotificationHandler for Notifications {
    fn port_registration(&mut self, client: &Client, _port_id: u32, is_registered: bool) {
        if !is_registered {
            return;
        }

        enforce_connect_rules(
            client,
            &self.mapping_config.lock().unwrap(),
            &self.tx.lock().unwrap(),
        );
    }
}
