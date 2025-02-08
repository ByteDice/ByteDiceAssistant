use crate::rs_println;

use std::fs;
use std::ffi::CString;
use std::env;

use pyo3::prelude::*;
use pyo3::types::PyList;


pub fn start(args: String) -> PyResult<()> { 
  rs_println!("Running Python program...");

  let path = concat!(env!("CARGO_MANIFEST_DIR"), "\\src\\python");

  let code = get_code(&(path.to_owned() + "\\main.py"));
  let py_args = args.replace(":true", ":True").replace(":false", ":False");
  let app_path = CString::new(format!("args = {}\n{}", py_args, code)).unwrap();
  
  pyo3::prepare_freethreaded_python();

  let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
    let syspath = py.import("sys")?.getattr("path")?.downcast_into::<PyList>()?;
    syspath.insert(0, path)?;
    let empty = CString::new("").unwrap();

    let app: Py<PyAny> = PyModule::from_code(py, &app_path, &empty, &empty)?
      .getattr("run")?
      .into();

    return app.call0(py);
  });

  if from_python.is_err() { println!("py: {:?}", from_python); }
  return Ok(());
}


fn get_code(path: &str) -> String {
  return fs::read_to_string(path)
    .expect("Failed to read Python file.")
    .to_string();
}