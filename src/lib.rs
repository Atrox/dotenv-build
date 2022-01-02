//! # Overview
//! This crate allows you to load .env files in your compilation step.
//! It is built to be used in your **[build.rs](https://doc.rust-lang.org/cargo/reference/build-scripts.html)** file.
//!
//! # Usage
//!
//! 1. Ensure you have build scripts enabled via the `build` configuration in your `Cargo.toml`
//! 1. Add `dotenv-build` as a build dependency
//! 1. Create a `build.rs` file that uses `dotenv-build` to generate `cargo:` instructions.
//! 1. Use the [`env!`](std::env!) or [`option_env!`](std::option_env!) macro in your code
//!
//! ### Cargo.toml
//! ```toml
//! [package]
//! #..
//! build = "build.rs"
//!
//! [dependencies]
//! #..
//!
//! [build-dependencies]
//! dotenv-build = "0.1"
//! ```
//!
//! ### build.rs
//! ```
//! // in build.rs
//! fn main() {
//!     dotenv_build::output(dotenv_build::Config::default()).unwrap();
//! }
//! ```
//!
//! ### Use in code
//! ```ignore
//! println!("Your environment variable in .env: {}", env!("TEST_VARIABLE"));
//! ```
//!
//! [build scripts]: https://doc.rust-lang.org/cargo/reference/build-scripts.html
//! [cargo:rustc-env]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-env
//! [cargo:rerun-if-changed]: https://doc.rust-lang.org/cargo/reference/build-scripts.html#rerun-if-changed
//!
//! ### Configuration
//!
//! Read more about the available options here: [`Config`]
//! ```
//! let config = dotenv_build::Config {
//!     filename: std::path::Path::new(".env.other"),
//!     recursive_search: false,
//!     fail_if_missing_dotenv: false,
//!     ..Default::default()
//! };
//!
//! dotenv_build::output(config).unwrap();
//! ```
//!
//! ## Multiple files
//! Use [`output_multiple`] for this:
//!
//! ```
//! use std::path::Path;
//!
//! use dotenv_build::Config;
//!
//! let configs = vec![
//!     // load .env.base
//!     Config {
//!         filename: Path::new(".env.base"),
//!         // fail_if_missing_dotenv: true,
//!         ..Default::default()
//!     },
//!     // load .env.staging
//!     Config {
//!         filename: Path::new(".env.staging"),
//!         ..Default::default()
//!     },
//!     // load .env
//!     Config::default(),
//! ];
//!
//! dotenv_build::output_multiple(configs).unwrap();
//! ```

mod errors;
mod find;
mod iter;
mod parse;

use std::io;
use std::io::Write;
use std::path::Path;

use crate::errors::*;

/// Config for [`output`]
#[derive(Debug)]
pub struct Config<'a> {
    /// The filename that is getting read for the environment variables. Defaults to `.env`
    pub filename: &'a Path,
    /// This specifies if we should search for the file recursively upwards in the file tree.
    /// Defaults to `true`.
    pub recursive_search: bool,
    /// This specifies if we should return an error if we don't find the file. Defaults to `false`.
    pub fail_if_missing_dotenv: bool,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Config {
            filename: Path::new(".env"),
            recursive_search: true,
            fail_if_missing_dotenv: false,
        }
    }
}

/// Outputs the necessary [build.rs](https://doc.rust-lang.org/cargo/reference/build-scripts.html) instructions.
///
/// ## Example
///
/// ```
/// dotenv_build::output(dotenv_build::Config::default()).unwrap();
/// ```
///
/// _.env_:
/// ```text
/// RUST_LOG=debug
/// RUST_BACKTRACE=1
///
/// ## comment
/// TEST="hello world!"
/// ANOTHER_ONE=test
/// ```
///
/// _output_:
/// ```text
/// cargo:rustc-env=RUST_LOG=debug
/// cargo:rustc-env=RUST_BACKTRACE=1
/// cargo:rustc-env=TEST=hello world!
/// cargo:rustc-env=ANOTHER_ONE=test
/// cargo:rerun-if-changed=$PATH_TO_YOUR_FILE/.env
/// ```
pub fn output(config: Config) -> Result<()> {
    output_write_to(config, &mut io::stdout())
}

/// Same as [`output`] but to read multiple files
///
/// ## Example
///
/// ```
/// use std::path::Path;
///
/// use dotenv_build::Config;
///
/// let configs = vec![
///     // load .env.base
///     Config {
///         filename: Path::new(".env.base"),
///         // fail_if_missing_dotenv: true,
///         ..Default::default()
///     },
///     // load .env.staging
///     Config {
///         filename: Path::new(".env.staging"),
///         ..Default::default()
///     },
///     // load .env
///     Config::default(),
/// ];
///
/// dotenv_build::output_multiple(configs).unwrap();
/// ```
pub fn output_multiple(configs: Vec<Config>) -> Result<()> {
    for config in configs {
        output_write_to(config, &mut io::stdout())?;
    }

    Ok(())
}

fn output_write_to<T>(config: Config, stdout: &mut T) -> Result<()>
where
    T: Write,
{
    let (path, lines) = match find::find(&config) {
        Ok(res) => res,
        Err(err) if err.not_found() => {
            return if config.fail_if_missing_dotenv {
                eprintln!(
                    "[dotenv-build] {:?} file not found, err: {}",
                    config.filename, err
                );
                Err(err)
            } else {
                Ok(())
            };
        }
        Err(err) => return Err(err),
    };

    for line in lines {
        let (key, value) = match line {
            Ok(l) => l,
            Err(err) => {
                eprintln!("[dotenv-build] {}", err);
                return Err(err);
            }
        };

        writeln!(stdout, "cargo:rustc-env={}={}", key, value)?;
    }

    writeln!(stdout, "cargo:rerun-if-changed={}", path.to_str().unwrap())?;
    Ok(())
}
