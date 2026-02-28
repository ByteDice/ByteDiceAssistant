use std::path::PathBuf;

use serde_json::Value;
use dynfmt::{Format, NoopFormat};

use crate::Error;


pub struct Lang {
  data: Value
}


impl From<Value> for Lang {
  fn from(value: Value) -> Self {
      return Lang { data: value };
  }
}


impl Lang {
  pub fn new() -> Self {
    return Lang { data: Value::Null };
  }


  pub fn from_file(filepath: PathBuf) -> Result<Self, Error> {
    if !filepath.exists() { return Err(Error::from("LANG filepath not found!")); }

    return Ok(Lang { data: Value::Null });
  }


  pub fn get(&self, path: &'static str, args: &[String]) -> String {
    let path_arr: Vec<&str> = path.split(".").collect();
    return self.get_from_arr(path_arr, args);
  }


  pub fn get_from_arr(&self, path: Vec<&str>, args: &[String]) -> String {
    let str_path = path.join(".");
    let mut search: &Value = &self.data;

    for i in &path {
      let r = search.get(i);

      if let Some(some) = r {
        let str_r = search.as_str();

        if let Some(string) = str_r
          { return Lang::format_str(string, args, str_path); }
        else { search = some; }
      }
      else { return str_path; }
    }

    return str_path;
  }


  fn format_str(string: &str, args: &[String], fallback: String) -> String {
    let cow = NoopFormat.format(string, args);
    
    if let Ok(ok) = cow
      { return ok.to_string(); }
    else { return fallback; }
  }
}