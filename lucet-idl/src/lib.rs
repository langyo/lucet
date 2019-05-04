#![deny(bare_trait_objects)]

#[macro_use]
extern crate failure;

mod backend;
mod c;
mod cache;
mod config;
mod error;
mod generator;
mod lexer;
mod package;
mod parser;
mod pretty_writer;
mod rust;
mod target;
mod types;

pub use crate::backend::{Backend, BackendConfig};
pub use crate::config::Config;
pub use crate::error::IDLError;
pub use crate::target::Target;

use crate::cache::Cache;
use crate::package::Package;
use crate::parser::Parser;
use crate::pretty_writer::PrettyWriter;
use std::io::Write;

pub fn run<W: Write>(config: &Config, input: &str, output: W) -> Result<W, IDLError> {
    let mut parser = Parser::new(&input);
    let decls = parser.match_decls()?;

    let pkg = Package::from_declarations(&decls)?;
    let deps = pkg
        .ordered_dependencies()
        .map_err(|_| IDLError::InternalError("Unable to resolve dependencies"))?;

    let mut cache = Cache::default();
    let mut generator = config.generator();

    let mut pretty_writer = PrettyWriter::new(output);
    generator.gen_prelude(&mut pretty_writer)?;
    for id in deps {
        generator.gen_for_id(&pkg, &mut cache, &mut pretty_writer, id)?;
    }
    Ok(pretty_writer
        .into_inner()
        .expect("outermost pretty_writer can unwrap"))
}
