use std::env;
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::process;

fn main() {
    let ref args: Vec<String> = env::args().collect();
    let exit_code;
    if !running_as_rustc_proxy() {
        turn_on_rustc_proxy();
        exit_code = run_cargo(args);
    } else {
        turn_off_rustc_proxy();
        exit_code = run_rustc(args);
    }

    let exit_code = match exit_code {
        Ok(c) => c,
        Err(e) => {
            println!("error: {}", e.description());
            1
        }
    };

    process::exit(exit_code);
}

fn run_cargo(args: &[String]) -> Result<i32, Error>  {
    let this_exe = try!(env::current_exe());
    env::set_var("RUSTC", this_exe);
    panic!()
}

fn run_rustc(args: &[String]) -> Result<i32, Error> {
    panic!()
}

fn running_as_rustc_proxy() -> bool {
    env::var("CARGO_BAKE_PROXY").is_ok()
}

fn turn_on_rustc_proxy() {
    env::set_var("CARGO_BAKE_PROXY", "1");
}

fn turn_off_rustc_proxy() {
    env::remove_var("CARGO_BAKE_PROXY");
}

#[derive(Debug)]
enum Error {
    StdError(Box<StdError + Send>)
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::StdError(ref e) => e.description(),
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::StdError(ref e) => Some(&**e),
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result<(), fmt::Error> {
        f.write_str(self.description())
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::StdError(Box::new(e))
    }
}
