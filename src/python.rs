use crate::messages::send_dm;
use crate::{errln, rs_println, Args};

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
  
  pyo3::prepare_freethreaded_python();

  let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
    let syspath = py.import("sys")?.getattr("path")?.downcast_into::<PyList>()?;
    syspath.insert(0, path)?;
    let empty = CString::new("").unwrap();

    let app: Py<PyAny> = PyModule::from_code(py, &app_path, &empty, &empty)?.into();

    return Ok(app);
  });

  if from_python.is_err() {
    let own_env = std::env::var("ASSISTANT_OWNERS").unwrap_or("0".to_string());
    let own_vec_str: Vec<String> = own_env.split(",").map(String::from).collect();
    let own_vec_u64: Vec<u64> = own_vec_str
      .iter()
      .map(|s| s.parse::<u64>().expect("Failed to parse ASSISTANT_OWNERS. Invalid syntax."))
      .collect();

    send_dm(format!("Unknown internal Python Error: {:?}", from_python), args, own_vec_u64).await;
    errln!("pyO3: {:?}", from_python);
  }
  return Ok(());
}


fn get_code(path: &str) -> String {
  return fs::read_to_string(path)
    .unwrap_or_else(|_| errln!("Failed to read Python file.\nPath: {}", path))
    .to_string();
}
