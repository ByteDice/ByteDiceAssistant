use std::{fs, io::Write};
use std::path::Path;

use serde_json::{self, Value, json};
use tokio::sync::Mutex;

use crate::{errln, rs_println, Data, Error, CFG_DATA_RE, LANG};
use crate::websocket::send_cmd_json;


static DATA_PATH_DC:    &str = "./data/dc_data.json";
static PRESET_PATH_DC:  &str = "./data/dc_data_preset.json";
static DATA_PATH_RE:    &str = "./data/re_data.json";
static PRESET_PATH_RE:  &str = "./data/re_data_preset.json";
static DATA_PATH_CFG:   &str = "./data/cfg.json";
static PRESET_PATH_CFG: &str = "./data/cfg_default.json";
static DATA_PATH_LANG:  &str = "./data/lang/";

pub static DC_POSTS_CHANNEL_KEY: &str = "re_posts_channel";


pub async fn read_dc_data(data: &Data, wipe: bool) {
  if !Path::new(DATA_PATH_DC).exists() || wipe {
    rs_println!(
      "{} creating new from preset...",
      if !wipe { "discord_data.json not found," } else { "[WIPE] (discord_data.json)" }
    );
    generate_dc_data();
  }

  let str_data = fs::read_to_string(DATA_PATH_DC).unwrap();
  let json_data = serde_json::from_str(&str_data).unwrap();
  let mut dc_data = data.discord_data.lock().await;
  *dc_data = json_data;
}


fn generate_dc_data() {
  let preset_str = fs::read_to_string(PRESET_PATH_DC).unwrap();
  let mut preset_json: Value = serde_json::from_str(&preset_str).unwrap();

  if let Some(servers) = preset_json["servers"].as_object_mut() {
    servers.remove("SERVER ID");
  }

  let json_str = serde_json::to_string_pretty(&preset_json).unwrap();

  let mut file = fs::File::create(DATA_PATH_DC).unwrap();
  file.write_all(json_str.as_bytes()).unwrap();
}


pub async fn write_dc_data(data: &Data) {
  if !Path::new(DATA_PATH_DC).exists() {
    generate_dc_data();
  }

  let mut file = fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(DATA_PATH_DC)
    .unwrap();


  let mut dc_data_lock = data.discord_data.lock().await;
  let dc_data = dc_data_lock.as_mut().unwrap(); 
  let json_str = serde_json::to_string_pretty(dc_data).unwrap();
  
  file.write_all(json_str.as_bytes()).unwrap();
}


pub async fn read_re_data(data: &Data, wipe: bool) {
  if !Path::new(DATA_PATH_RE).exists() || wipe {
    rs_println!(
      "{} creating new from preset...",
      if !wipe { "reddit_data.json not found," } else { "[WIPE] (reddit_data.json)" }
    );
    generate_re_data();
  }

  let str_data = fs::read_to_string(DATA_PATH_RE).unwrap();
  let json_data = serde_json::from_str(&str_data).unwrap();
  let mut re_data = data.reddit_data.lock().await;
  *re_data = json_data;
}


fn generate_re_data() {
  let preset_str = fs::read_to_string(PRESET_PATH_RE).unwrap();
  let mut preset_json: Value = serde_json::from_str(&preset_str).unwrap();

  if let Some(bk_week) = preset_json[CFG_DATA_RE].as_object_mut() {
    bk_week.remove("EXAMPLE VALUE");
    bk_week.remove("EXAMPLE VALUE DELETED");
  }

  let json_str = serde_json::to_string_pretty(&preset_json).unwrap();

  let mut file = fs::File::create(DATA_PATH_RE).unwrap();
  file.write_all(json_str.as_bytes()).unwrap();
}


pub async fn update_re_data(data: &Data) {
  send_cmd_json("update_data_file", None).await;
  read_re_data(data, false).await;
}


pub async fn write_re_data() {
  send_cmd_json("update_data_file", None).await;
}


pub async fn read_cfg_data(data: &Data, wipe: bool) {
  if !Path::new(DATA_PATH_CFG).exists() || wipe {
    rs_println!(
      "{} creating new from preset...",
      if !wipe { "cfg.json not found," } else { "[WIPE] (cfg.json)" }
    );
    generate_cfg_data();
  }

  let str_data = fs::read_to_string(DATA_PATH_CFG).unwrap();
  let json_data = serde_json::from_str(&str_data).unwrap();
  let mut cfg_data = data.cfg.lock().await;
  *cfg_data = json_data;

  send_cmd_json("update_cfg", Some(json!([str_data]))).await;
}


fn generate_cfg_data() {
  let preset_str = fs::read_to_string(PRESET_PATH_CFG).unwrap();
  let preset_json: Value = serde_json::from_str(&preset_str).unwrap();

  let json_str = serde_json::to_string_pretty(&preset_json).unwrap();

  let mut file = fs::File::create(DATA_PATH_CFG).unwrap();
  file.write_all(json_str.as_bytes()).unwrap();
}


pub async fn dc_add_server(data: &Data, server_id: u64) -> Result<(), ()> {
  let mut dc_data_lock = data.discord_data.lock().await;
  let dc_data = dc_data_lock.as_mut().unwrap(); 

  if dc_data.get("servers").is_none() { return Err(()); }

  let servers = dc_data["servers"].as_object_mut().unwrap();

  if !servers.contains_key(&server_id.to_string()) {
    servers.insert(server_id.to_string(), json!({ DC_POSTS_CHANNEL_KEY: 0 }));
  }

  return Ok(());
}


pub async fn dc_bind_bk(data: &Data, server_id: u64, channel_id: u64) -> Result<(), ()> {
  let mut dc_data_lock = data.discord_data.lock().await;
  let dc_data = dc_data_lock.as_mut().unwrap(); 

  if dc_data.get("servers").is_none() { return Err(()); }

  let servers = dc_data["servers"].as_object_mut().unwrap();

  if !servers.contains_key(&server_id.to_string()) {
    return Err(());
  }

  let server = servers[&server_id.to_string()].as_object_mut().unwrap();

  server.insert(DC_POSTS_CHANNEL_KEY.to_string(), channel_id.into());

  return Ok(());
}


pub async fn dc_contains_server(data: &Data, server_id: u64) -> bool {
  let dc_data_lock = data.discord_data.lock().await;
  let dc_data = dc_data_lock.as_ref().unwrap();

  if dc_data.get("servers").is_none() { return false; }

  let mut clone = dc_data.clone();
  let servers = clone["servers"].as_object_mut().unwrap();

  return servers.contains_key(&server_id.to_string())
}


pub async fn get_mutex_data(data: &Mutex<Option<Value>>) -> Result<Value, Error> {
  let data_lock = data.lock().await;
  return match data_lock.as_ref() {
    Some(data) => Ok(data.clone()),
    None => Err("Cannot get mutex data: The data is corrupted!".into()),
  };
}


pub fn load_lang_data(lang: String) {
  let full_path = format!("{}{}.json", DATA_PATH_LANG, lang);

  if !Path::new(&full_path).exists() {
    errln!(
      "File for language \"{0}\" ({0}.json) not found!\n    Hint: You can download official language files at https://github.com/ByteDice/ByteDiceAssistant in the data/langs/... folder",
      lang
    );
  }

  let str_data = fs::read_to_string(full_path).unwrap();
  let json_data = serde_json::from_str(&str_data).unwrap();

  unsafe {
    LANG = json_data;
  };
}