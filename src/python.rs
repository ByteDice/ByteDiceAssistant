use crate::db::env_vars::AssistantEnv;
use crate::db::terminal_args::Args;
use crate::messages::send_dm_min;
use crate::{errln, lang, rs_println};

use std::fs;
use std::ffi::CString;
use std::env;

use pyo3::prelude::*;
use pyo3::types::PyList;


pub async fn start(args: Args, lang_name: String, env_vars: AssistantEnv) -> PyResult<()> { 
  rs_println!("Running Python program...");

  let args_str = serde_json::to_string(&args).expect("Error serializing args to JSON");

  let slash = if cfg!(windows) { "\\" } else if cfg!(unix) { "/" } else { "" };
  if slash.is_empty() { errln!("Man what kinda OS do you have? Neither unix or windows, what the hell!? I can't process this anymore, you're too weird!"); }

  let path = format!("{0}{1}src{1}python", env!("CARGO_MANIFEST_DIR"), slash);

  let code = get_code(&format!("{}{}main.py", path, slash));
  let py_args = args_str.replace(":true", ":True").replace(":false", ":False");
  let app_path: CString;
  
  app_path = CString::new(format!(
    "args = {}\nlang_name = \"{}\"\n{}",
    py_args,
    lang_name,
    code
  )).unwrap();
  
  let mut traceback: String = String::new();
  let mut is_error = false;

  Python::initialize();

  let _ = Python::attach(|py| -> Result<(), PyErr> {
    let syspath = py.import("sys")?.getattr("path")?.cast_into::<PyList>()?;
    syspath.insert(0, path)?;
    let empty = CString::new("").unwrap();

    let py_result = PyModule::from_code(py, &app_path, &empty, &empty);

    if let Err(ref e) = py_result {
      traceback = py.import("traceback")?
        .call_method1("format_exception", (e.get_type(py), e.value(py), e.traceback(py)))?
        .extract::<Vec<String>>()?
        .join("");
      is_error = true;
    }

    return Ok(());
  });

  if is_error {
    send_dm_min(
      lang!("dc_msg_dm_python_err", traceback),
      env_vars.token.clone(),
      env_vars.bot_owners.clone()
    ).await;
    errln!("pyO3: {}", traceback);
  }

  return Ok(());
}


fn get_code(path: &str) -> String {
  let file = fs::read_to_string(path);
  if file.is_err() { errln!("Failed to read Python file.\nPath: {}", path); }

  return file.unwrap().to_string();
}
