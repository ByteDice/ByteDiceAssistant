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