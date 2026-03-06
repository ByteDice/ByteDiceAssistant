use std::{fs, path::PathBuf};

use serde_json::Value;
use dynfmt::{Format, SimpleCurlyFormat};

use crate::{Error, rs_warnln};


#[derive(Debug)]
enum LangErrorType {
  Fallback,
  ShortIndex,
  KeyNotFound
}


pub struct Lang {
  data: Value
}


impl From<Value> for Lang {
  fn from(value: Value) -> Self
    { return Lang { data: value }; }
}


impl Lang {
  pub fn from_file(filepath: PathBuf) -> Result<Self, Error> {
    if !filepath.exists() { return Err(Error::from("LANG filepath not found!")); }
    
    let file_contents = fs::read_to_string(filepath).unwrap();
    let json = serde_json::from_str(&file_contents).unwrap();

    return Ok(Lang { data: json });
  }


  pub fn get(&self, path: &str, args: &[String]) -> String {
    let path_arr: Vec<&str> = path.split(".").collect();
    return self.get_from_arr(path_arr, args);
  }


  pub fn get_from_arr(&self, path: Vec<&str>, args: &[String]) -> String {
    let str_path = path.join(".");
    let mut search: &Value = &self.data;
    
    for i in &path {
      let Some(r) = search.get(i)
        else { rs_warnln!("LANG warning ({:?})! ({})", LangErrorType::KeyNotFound, str_path); return str_path; };

      let str_r = r.as_str();

      if let Some(string) = str_r
        { return Lang::format_str(string, args, str_path); }
      else { search = r; }
    }

    rs_warnln!("LANG warning ({:?})! ({})", LangErrorType::ShortIndex, str_path);
    return str_path;
  }


  fn format_str(string: &str, args: &[String], fallback: String) -> String {
    let cow = SimpleCurlyFormat.format(string, args);
    
    if let Ok(ok) = cow
      { return ok.to_string(); }
    else {
      rs_warnln!("LANG warning ({:?})! ({})", LangErrorType::Fallback, fallback);
      return fallback;
    }
  }
}