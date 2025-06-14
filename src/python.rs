use crate::messages::send_dm;
use crate::{errln, lang, rs_println, Args};

use std::fs;
use std::ffi::CString;
use std::env;

use pyo3::prelude::*;
use pyo3::types::PyList;


pub async fn start(args: Args) -> PyResult<()> { 
  rs_println!("Running Python program...");

  let args_str = serde_json::to_string(&args).expect("Error serializing args to JSON");

  let slash = if cfg!(windows) { "\\" } else if cfg!(unix) { "/" } else { "" };
  if slash.is_empty() { errln!("Man what kinda OS do you have? Neither unix or windows, what the hell!? I can't process this anymore, you're too weird!"); }

  let path = format!("{0}{1}src{1}python", env!("CARGO_MANIFEST_DIR"), slash);

  let code = get_code(&format!("{}{}main.py", path, slash));
  let py_args = args_str.replace(":true", ":True").replace(":false", ":False");
  let app_path = CString::new(format!("args = {}\n{}", py_args, code)).unwrap();
  
  let mut traceback: String = String::new();
  let mut is_error = false;

  pyo3::prepare_freethreaded_python();

  let _ = Python::with_gil(|py| -> Result<(), PyErr> {
    let syspath = py.import("sys")?.getattr("path")?.downcast_into::<PyList>()?;
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
    let own_env = std::env::var("ASSISTANT_OWNERS").unwrap_or("0".to_string());
    let own_vec_str: Vec<String> = own_env.split(",").map(String::from).collect();
    let own_vec_u64: Vec<u64> = own_vec_str
      .iter()
      .map(|s| s.parse::<u64>().expect("Failed to parse ASSISTANT_OWNERS. Invalid syntax."))
      .collect();

    send_dm(
      lang!("dc_msg_dm_python_err", format!("{}", traceback)),
      args, 
      own_vec_u64
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
