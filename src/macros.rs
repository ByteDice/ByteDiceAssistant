#[macro_export]
macro_rules! rs_println {
  ($($arg:tt)*) => {
    println!("{}RS - {}{}",
      "\x1b[31m",
      format!($($arg)*),
      "\x1b[0m"
    );
  };
}


#[macro_export]
macro_rules! rs_errln {
  ($($arg:tt)*) => {
    println!("{}ERROR{} RS - {}{}",
      "\x1b[41m",
      "\x1b[0m\x1b[31m",
      format!($($arg)*),
      "\x1b[0m"
    );
    process::exit(1);
  };
}


#[macro_export]
macro_rules! errln {
  ($($arg:tt)*) => {
    println!("{}ERROR{} - {}",
      "\x1b[41m",
      "\x1b[0m",
      format!($($arg)*)
    );
    std::process::exit(1);
  };
}


#[macro_export]
macro_rules! lang {
  ($key:expr) => {
    {
      use crate::{LANG, errln};
      let value = unsafe {
        LANG
          .as_ref()
          .expect("LANG must be initialized before use")
          .get($key)
      };

      if value.is_none() { errln!("Key not found in LANG JSON: \"{}\"", $key); }

      value.unwrap().as_str().expect("LANG JSON value is not a string!").to_string()
    }
  };
  ($key:expr, $($arg:expr),*) => {{
    use crate::{LANG, errln};
    use formatx::formatx;

    let value = unsafe {
      LANG
        .as_ref()
        .expect("LANG must be initialized before use")
        .get($key)
    };

    if value.is_none() { errln!("Key not found in LANG JSON: \"{}\"", $key); }

    let format_str = value.unwrap().as_str().expect("LANG JSON value is not a string!");
    formatx!(format_str, $($arg),*).unwrap()
  }};
}