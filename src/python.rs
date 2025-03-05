use crate::{rs_println, errln};

use std::fs;
use std::ffi::CString;
use std::env;

use pyo3::prelude::*;
use pyo3::types::PyList;


pub fn start(args: String) -> PyResult<()> { 
  rs_println!("Running Python program...");

  let slash = if cfg!(windows) { "\\" } else if cfg!(unix) { "/" } else { "" };
  if slash.is_empty() { errln!("Man what kinda OS do you have? Neither unix or windows, what the hell!? I can't process this anymore, you're too weird!"); }

  let path = format!("{0}{1}src{1}python", env!("CARGO_MANIFEST_DIR"), slash);

  let code = get_code(&format!("{}{}main.py", path, slash));
  let py_args = args.replace(":true", ":True").replace(":false", ":False");
  let app_path = CString::new(format!("args = {}\n{}", py_args, code)).unwrap();
  
  pyo3::prepare_freethreaded_python();

  let from_python = Python::with_gil(|py| -> PyResult<Py<PyAny>> {
    let syspath = py.import("sys")?.getattr("path")?.downcast_into::<PyList>()?;
    syspath.insert(0, path)?;
    let empty = CString::new("").unwrap();

    let app: Py<PyAny> = PyModule::from_code(py, &app_path, &empty, &empty)?.into();

    return Ok(app);
  });

  if from_python.is_err() { errln!("pyO3: {:?}", from_python); }
  return Ok(());
}


fn get_code(path: &str) -> String {
  return fs::read_to_string(path)
    .unwrap_or_else(|_| panic!("Failed to read Python file.\nPath: {}", path))
    .to_string();
}
