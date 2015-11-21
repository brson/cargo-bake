#[derive(Clone, Copy)]
enum BakeMode {
    Fast, Normal, Slow, Glacial
}

enum DebugMode {
    Off, On
}

fn bake_mode_args(mode: BakeMode) -> Vec<String> {
    let basic_args = match mode {
        BakeMode::Fast => vec![
            "-Copt-level=0",
            ],
        BakeMode::Normal => vec![
            "-Copt-level=1",
            "-Cinline-threshold=25",
            "-Cno-vectorize-loops",
            ],
        BakeMode::Slow => vec![
            "-Copt-level=3",
            "-Cinline-threshold=275",
            ],
        BakeMode::Glacial => vec![
            "-Copt-level=3",
            "-Cinline-threshold=275",
            "-C -lto"
            ]
    };

    let par_args = vec![format!("-Ccodegen-units={}", codegen_units())];

    let gold_args = if have_gold() {
        vec!["-Clink-args=-fuse-ld=gold"]
    } else { vec![] };

    let common_args = vec![
            "-Zno-verify",
        ];

    vec![].into_iter()
        .chain(basic_args.into_iter().map(str::to_owned))
        .chain(par_args.into_iter())
        .chain(gold_args.into_iter().map(str::to_owned))
        .chain(common_args.into_iter().map(str::to_owned))
        .collect()
}

fn debug_mode_args(mode: DebugMode) -> Vec<String> {
    let args = match mode {
        DebugMode::Off => vec![
            "-Cdebuginfo=0",
            ],
        DebugMode::On => vec![
            "-Cdebuginfo=2",
            ],
    };

    args.into_iter().map(str::to_owned).collect()
}

fn cargo_args_for_bake_mode(mode: BakeMode) -> Vec<String> {
    let args = match mode {
        BakeMode::Slow => vec![],
        _ => vec!["--release"]
    };

    args.into_iter().map(str::to_owned).collect()
}

fn codegen_units() -> usize {
    let cpus = num_cpus::get();
    if cpus > 4 { 4 } else { cpus }
}

#[macro_use]
extern crate log;
#[macro_use]
extern crate env_logger;
extern crate num_cpus;
extern crate stopwatch;

use std::env::{self, VarError};
use std::error::Error as StdError;
use std::fmt::{self, Display, Formatter};
use std::io;
use std::fs;
use std::process::{self, Command};
use stopwatch::Stopwatch;

fn main() {
    env_logger::init().unwrap();

    let ref args: Vec<String> = env::args().collect();

    let exit_code;
    if args.len() > 1 && args[1] == "--compare" {
        exit_code = do_comparison();
    } else if !running_as_rustc_proxy() {
        turn_on_rustc_proxy();
        let cargo_name = get_cargo_name();
        exit_code = run_cargo(cargo_name, args);
    } else {
        let rustc_name = get_rustc_name();
        exit_code = run_rustc(rustc_name, args);
    }

    let exit_code = match exit_code {
        Ok(c) => c,
        Err(e) => {
            println!("error: {:?}", e);
            1
        }
    };

    process::exit(exit_code);
}

fn do_comparison() -> Result<i32, Error> {
    let cargo_name = get_cargo_name();

    println!("fetching");
    let mut child = try!(get_command(cargo_name.clone())
                     .arg("fetch")
                     .spawn());
    let exit_status = try!(child.wait());
    let exit_code = exit_status.code().unwrap_or(1);
    if exit_code != 0 {
        return Ok(exit_code)
    }

    println!("cleaning");
    let mut child = try!(get_command(cargo_name.clone())
                         .arg("clean")
                         .spawn());
    let exit_status = try!(child.wait());
    let exit_code = exit_status.code().unwrap_or(1);
    if exit_code != 0 {
        return Ok(exit_code)
    }

    println!("testing 'cargo build --release'");
    let sw1 = Stopwatch::start_new();
    let mut child = try!(get_command(cargo_name.clone())
                         .arg("build")
                         .arg("--release")
                         .spawn());
    let exit_status = try!(child.wait());
    let exit_code = exit_status.code().unwrap_or(1);
    if exit_code != 0 {
        return Ok(exit_code)
    }
    let elapsed1 = sw1.elapsed_ms();

    println!("cleaning");
    let mut child = try!(get_command(cargo_name.clone())
                         .arg("clean")
                         .spawn());
    let exit_status = try!(child.wait());
    let exit_code = exit_status.code().unwrap_or(1);
    if exit_code != 0 {
        return Ok(exit_code)
    }

    println!("testing 'cargo bake'");
    let sw2 = Stopwatch::start_new();
    turn_on_rustc_proxy();
    let exit_code = try!(run_cargo(cargo_name, &[String::new()]));
    let elapsed2 = sw2.elapsed_ms();

    println!("cargo build --release: {}", elapsed1);
    println!("cargo bake: {}", elapsed2);

    Ok(exit_code)
}

fn running_as_rustc_proxy() -> bool {
    env::var("CARGO_BAKE_PROXY").is_ok()
}

fn turn_on_rustc_proxy() {
    env::set_var("CARGO_BAKE_PROXY", "1");

    // Save any existing "RUSTC" env vars
    if let Ok(v) = env::var("RUSTC") {
        env::set_var("CARGO_BAKE_RUSTC", v);
    }

    // Tell cargo to invoke *this* program as rustc
    let this_exe = env::current_exe().unwrap();
    env::set_var("RUSTC", this_exe);
}

fn get_cargo_name() -> String {
    if let Ok(v) = env::var("CARGO") {
        v
    } else {
        "cargo".to_owned()
    }
}

fn get_rustc_name() -> String {
    if let Ok(v) = env::var("CARGO_BAKE_RUSTC") {
        v
    } else {
        "rustc".to_owned()
    }
}

fn run_cargo(cargo_name: String, args: &[String]) -> Result<i32, Error>  {
    let args = &args[1..];

    let bake = try!(get_bake_mode_from_args(args));
    let debug = try!(get_debug_mode_from_args(args));
    set_bake_mode(bake);
    set_debug_mode(debug);
    
    // Remove the bake-specific arguments so the rest can be
    // passed to cargo.
    let args = strip_bake_args(args);

    let cargo_args = cargo_args_for_bake_mode(bake);

    info!("cargo args: {:?}", args);

    let mut child = try!(get_command(cargo_name)
                         .arg("build")
                         .args(&cargo_args)
                         .args(&args)
                         .spawn());
    let exit_status = try!(child.wait());
    Ok(exit_status.code().unwrap_or(1))
}

fn run_rustc(rustc_name: String, args: &[String]) -> Result<i32, Error> {
    use std::iter::Extend;

    let args = &args[1..];

    // Remove provided options to rustc that may interfere with ours
    let args = strip_opt_args(args);

    let bake = try!(get_bake_mode());
    let debug = try!(get_debug_mode());

    let bake_args = bake_mode_args(bake);
    let debug_args = debug_mode_args(debug);

    let mut args = args;
    args.extend(bake_args);
    args.extend(debug_args);

    info!("rustc args: {:?}", args);

    let mut child = try!(get_command(rustc_name)
                         .args(&args)
                         .spawn());
    let exit_status = try!(child.wait());
    Ok(exit_status.code().unwrap_or(1))
}

fn get_command(name: String) -> Command {
    // HACK multirust on windows
    let msys = env::var("MSYSTEM").is_ok();
    let multirust = fs::metadata("c:/msys64/usr/local/bin/multirust")
        .map(|m| m.is_file()).unwrap_or(false);
    let rel_path = !name.contains("/") && !name.contains("\\");
    let do_multirust_win_hack = msys && multirust && rel_path;
    debug!("msys: {}, multirust: {}, rel_path: {}, hack: {}",
           msys, multirust, rel_path, do_multirust_win_hack);

    if !do_multirust_win_hack {
        Command::new(name)
    } else {
        let mut c = Command::new("bash");
        c.arg("c:/msys64/usr/local/bin/".to_owned() + &name);
        c
    }
}

fn strip_bake_args(args: &[String]) -> Vec<String> {
    let bake_args = ["--fast", "--slow", "--glacial", "--debug"];

    let args = args.iter();
    let mut bake_args = bake_args.iter();

    args.filter(|a| !bake_args.any(|b| a == b))
        .cloned()
        .collect()
}

fn strip_opt_args(args: &[String]) -> Vec<String> {
    let opt_args = ["-g"];

    let opt_args: Vec<_> = args.iter().filter(|a| {
        opt_args.iter().all(|b| a != b)
    }).cloned().collect();

    let mut opt_args_ = vec![];

    // Get rid of -C opt-level
    let mut i = 0;
    while i < opt_args.len() - 1 {
        if opt_args[i] == "-C" && opt_args[i + 1].contains("opt-level") {
            i += 2;
            continue;
        }
        opt_args_.push(opt_args[i].clone());
        i += 1;
    }
    opt_args_.push(opt_args[opt_args.len() - 1].clone());

    opt_args_
}

fn get_bake_mode_from_args(args: &[String]) -> Result<BakeMode, Error> {
    if args.iter().any(|a| a == "--fast") {
        Ok(BakeMode::Fast)
    } else if args.iter().any(|a| a == "--slow") {
        Ok(BakeMode::Slow)
    } else if args.iter().any(|a| a == "--glacial") {
        Ok(BakeMode::Glacial)
    } else {
        Ok(BakeMode::Normal)
    }
}

fn get_debug_mode_from_args(args: &[String]) -> Result<DebugMode, Error> {
    if args.iter().any(|a| a == "--debug") {
        Ok(DebugMode::On)
    } else {
        Ok(DebugMode::Off)
    }
}

fn set_bake_mode(mode: BakeMode) {
    let s = match mode {
        BakeMode::Fast => "fast",
        BakeMode::Normal => "normal",
        BakeMode::Slow => "slow",
        BakeMode::Glacial => "glacial"
    };

    env::set_var("CARGO_BAKE_MODE", s);
}

fn set_debug_mode(mode: DebugMode) {
    let s = match mode {
        DebugMode::Off => "off",
        DebugMode::On => "on"
    };

    env::set_var("CARGO_BAKE_DEBUG_MODE", s);
}

fn get_bake_mode() -> Result<BakeMode, Error> {
    let s = try!(env::var("CARGO_BAKE_MODE"));

    if s == "fast" {
        Ok(BakeMode::Fast)
    } else if s == "normal" {
        Ok(BakeMode::Normal)
    } else if s == "slow" {
        Ok(BakeMode::Slow)
    } else if s == "glacial" {
        Ok(BakeMode::Glacial)
    } else {
        panic!()
    }
}

fn get_debug_mode() -> Result<DebugMode, Error> {
    let s = try!(env::var("CARGO_BAKE_DEBUG_MODE"));

    if s == "off" {
        Ok(DebugMode::Off)
    } else if s == "on" {
        Ok(DebugMode::On)
    } else {
        panic!()
    }
}

fn have_gold() -> bool {
    fs::metadata("/usr/bin/ld.gold")
        .map(|m| m.is_file()).unwrap_or(false)
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

impl From<env::VarError> for Error {
    fn from(e: env::VarError) -> Error {
        Error::StdError(Box::new(e))
    }
}
