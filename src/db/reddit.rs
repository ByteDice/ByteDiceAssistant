use std::{fs, io::Write, path::Path};

use serde_json::Value;

use crate::{rs_println, rs_warnln, websocket::send_cmd_json};


static DATA_PATH:    &str = "./data/db/re_data.json";
static PRESET_PATH:  &str = "./data/defaults/re_data_preset.json";

pub static POSTS_KEY: &str = "posts";


pub async fn read_data(wipe: bool) -> Value {
  if !Path::new(DATA_PATH).exists() || wipe {
    rs_println!(
      "{} creating new from preset...",
      if !wipe { "reddit_data.json not found," } else { "[WIPE] (reddit_data.json)" }
    );
    generate_data();
  }

  let str_data = fs::read_to_string(DATA_PATH).unwrap();
  let json_data: Value = serde_json::from_str(&str_data).unwrap();
  return json_data;
}


fn generate_data() {
  let preset_str = fs::read_to_string(PRESET_PATH).unwrap();
  let mut preset_json: Value = serde_json::from_str(&preset_str).unwrap();

  if let Some(bk_week) = preset_json[POSTS_KEY].as_object_mut() {
    bk_week.remove("EXAMPLE URL");
  }
  else {
    rs_warnln!("Couldn't find key \"{}\" in the Reddit data file ({})!", POSTS_KEY, DATA_PATH);
  }

  let json_str = serde_json::to_string_pretty(&preset_json).unwrap();

  let mut file = fs::File::create(DATA_PATH).unwrap();
  file.write_all(json_str.as_bytes()).unwrap();
}


pub async fn update_data() {
  send_cmd_json("update_data_file", None, true).await;
  read_data(false).await;
}


pub async fn write_data() {
  send_cmd_json("update_data_file", None, true).await;
}