use anyhow::{Error, Result};

use std::fs;
use std::fs::read_to_string;
use std::path::Path;

pub(crate) fn read_program(path: impl AsRef<Path>) -> Result<String> {
    read_to_string(path).map_err(Error::new)
}

pub(crate) fn write_bytecode(path: impl AsRef<Path>, code: &[u8]) -> Result<()> {
    fs::write(path, code).map(drop).map_err(Error::new)
}
