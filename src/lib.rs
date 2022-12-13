#[macro_use]
extern crate log;

mod error;
mod thumbnail;
use std::path::Path;

pub use error::Result;

pub fn image(path: impl AsRef<Path>) -> Result<Vec<u8>> {
    thumbnail::extract_images(path, 2)
}
