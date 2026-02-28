use serde_json::Value;
use tokio::sync::Mutex;

use crate::Error;

pub async fn get_json_mutex(data: &Mutex<Option<Value>>) -> Result<Value, Error> {
  let data_lock = data.lock().await;
  return match data_lock.as_ref() {
    Some(data) => Ok(data.clone()),
    None => Err("Cannot get mutex data: The data is corrupted!".into()),
  };
}


pub async fn get_toml_mutex(data: &Mutex<Option<toml::Value>>) -> Result<toml::Value, Error> {
  let data_lock = data.lock().await;
  return match data_lock.as_ref() {
    Some(data) => Ok(data.clone()),
    None => Err("Cannot get mutex data: The data is corrupted!".into()),
  };
}