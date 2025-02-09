use std::{fs, io::Write};
use std::path::Path;

use serde_json::{self, Value, json};

use crate::Data;
use crate::websocket::send_cmd_json;


static DATA_PATH_DC: &str = "./data/discord_data.json";
static PRESET_PATH_DC: &str = "./data/discord_data_preset.json";
static DATA_PATH_RE: &str = "./data/reddit_data.json";
static PRESET_PATH_RE: &str = "./data/reddit_data_preset.json";


pub fn read_dc_data(data: &Data) {
  if !Path::new(DATA_PATH_DC).exists() {
    generate_dc_data();
  }

  let str_data = fs::read_to_string(DATA_PATH_DC).unwrap();
  let json_data = serde_json::from_str(&str_data).unwrap();
  let mut dc_data = data.discord_data.lock().unwrap();
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


pub fn write_dc_data(data: &Data) {
  if !Path::new(DATA_PATH_DC).exists() {
    generate_dc_data();
  }

  let mut file = fs::OpenOptions::new()
    .write(true)
    .truncate(true)
    .open(DATA_PATH_DC)
    .unwrap();

  let json_str = serde_json::to_string_pretty(&data.discord_data).unwrap();

  file.write_all(json_str.as_bytes()).unwrap();
}


pub fn read_re_data(data: &Data) {
  if !Path::new(DATA_PATH_RE).exists() {
    generate_re_data();
  }

  let str_data = fs::read_to_string(DATA_PATH_RE).unwrap();
  let json_data = serde_json::from_str(&str_data).unwrap();
  let mut re_data = data.reddit_data.lock().unwrap();
  *re_data = json_data;
}


fn generate_re_data() {
  let preset_str = fs::read_to_string(PRESET_PATH_RE).unwrap();
  let mut preset_json: Value = serde_json::from_str(&preset_str).unwrap();

  if let Some(bk_week) = preset_json["bk_weekly_art_posts"].as_object_mut() {
    bk_week.remove("EXAMPLE VALUE");
    bk_week.remove("EXAMPLE VALUE DELETED");
  }

  let json_str = serde_json::to_string_pretty(&preset_json).unwrap();

  let mut file = fs::File::create(DATA_PATH_RE).unwrap();
  file.write_all(json_str.as_bytes()).unwrap();
}


pub async fn update_re_data(data: &Data) {
  send_cmd_json("update_data_file", json!([])).await;
  read_re_data(data);
}


pub async fn write_re_data() {
  send_cmd_json("update_data_file", json!([])).await;
}