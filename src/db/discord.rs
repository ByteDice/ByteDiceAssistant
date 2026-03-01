use std::{fs, io::Write, path::Path};

use serde_json::{Value, json};

use crate::{Data, db::keys, rs_println};

static DATA_PATH:    &str = "./data/db/dc_data.json";
static PRESET_PATH:  &str = "./data/defaults/dc_data_preset.json";


pub async fn read_data(wipe: bool) -> Value {
  if !Path::new(DATA_PATH).exists() || wipe {
    rs_println!(
      "{} creating new from preset...",
      if !wipe { "discord_data.json not found," } else { "[WIPE] (discord_data.json)" }
    );
    generate_data();
  }

  let str_data = fs::read_to_string(DATA_PATH).unwrap();
  let json_data = serde_json::from_str(&str_data).unwrap();
  return json_data;
}


fn generate_data() {
  let preset_str = fs::read_to_string(PRESET_PATH).unwrap();
  let mut preset_json: Value = serde_json::from_str(&preset_str).unwrap();

  if let Some(servers) = preset_json["servers"].as_object_mut() {
    servers.remove("SERVER ID");
  }

  let json_str = serde_json::to_string_pretty(&preset_json).unwrap();

  let mut file = fs::File::create(DATA_PATH).unwrap();
  file.write_all(json_str.as_bytes()).unwrap();
}


pub async fn write_data(data: &Data) {
  if !Path::new(DATA_PATH).exists() {
    generate_data();
  }

  let mut file = fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(DATA_PATH)
    .unwrap();


  let dc_data = data.discord_data.lock().await;
  let json_str = serde_json::to_string_pretty(&dc_data.clone()).unwrap();
  
  file.write_all(json_str.as_bytes()).unwrap();
}


pub async fn add_server(data: &Data, server_id: u64) -> Result<(), ()> {
  let mut dc_data = data.discord_data.lock().await;

  if dc_data.get("servers").is_none() { return Err(()); }

  let servers = dc_data["servers"].as_object_mut().unwrap();

  if !servers.contains_key(&server_id.to_string()) {
    servers.insert(server_id.to_string(), json!({}));
  }

  return Ok(());
}


pub async fn bind_bk(data: &Data, server_id: u64, channel_id: u64) -> Result<(), ()> {
  let mut dc_data = data.discord_data.lock().await;

  if dc_data.get("servers").is_none() { return Err(()); }

  let servers = dc_data["servers"].as_object_mut().unwrap();

  if !servers.contains_key(&server_id.to_string()) {
    return Err(());
  }

  let server = servers[&server_id.to_string()].as_object_mut().unwrap();

  server.insert(keys::DC_POSTS_CHANNEL_KEY.to_string(), channel_id.into());

  return Ok(());
}


pub async fn bind_wwrps(data: &Data, server_id: u64, channel_id: u64) -> Result<(), ()> {
  let mut dc_data = data.discord_data.lock().await;

  if dc_data.get("servers").is_none() { return Err(()); }

  let servers = dc_data["servers"].as_object_mut().unwrap();

  if !servers.contains_key(&server_id.to_string())
    { return Err(()); }

  let server = servers[&server_id.to_string()].as_object_mut().unwrap();

  server.insert(keys::DC_WWRPS_CHANNEL_KEY.to_string(), channel_id.into());

  return Ok(());
}


pub async fn contains_server(data: &Data, server_id: u64) -> bool {
  let dc_data = data.discord_data.lock().await;

  if dc_data.get("servers").is_none() { return false; }

  let mut clone = dc_data.clone();
  let servers = clone["servers"].as_object_mut().unwrap();

  return servers.contains_key(&server_id.to_string())
}