use std::{fs, io::Write, path::Path};

use serde_json::{Value, json};

use crate::{Data, rs_println, websocket::send_cmd_json};


static DATA_PATH:   &str = "./cfg/cfg.toml";
static PRESET_PATH: &str = "./data/defaults/cfg_default.toml";


pub async fn read_data(data: &Data, wipe: bool) -> Option<Value> {
  if !Path::new(DATA_PATH).exists() || wipe {
    rs_println!(
      "{} creating new from preset...",
      if !wipe { "cfg.toml not found," } else { "[WIPE] (cfg.toml)" }
    );
    generate_data();
  }

  let str_data = fs::read_to_string(DATA_PATH).unwrap();
  let json_data: toml::Value = str_data.parse().unwrap();
  let mut cfg_data = data.cfg.lock().await;
  *cfg_data = Some(json_data.clone());

  let r = send_cmd_json(
    "update_cfg",
    Some(json!([toml::to_string(&json_data).unwrap()])), 
    true
  ).await;

  return r;
}


fn generate_data() {
  let preset_str = fs::read_to_string(PRESET_PATH).unwrap();

  let mut file = fs::File::create(DATA_PATH).unwrap();
  file.write_all(preset_str.as_bytes()).unwrap();
}
