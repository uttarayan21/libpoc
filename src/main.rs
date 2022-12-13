use libpoc::{image, Result};

pub fn main() -> Result<()> {
    for path in std::env::args().skip(1) {
        std::fs::write(
            std::path::PathBuf::from(&path).with_extension("jpg"),
            image(&path)?,
        )?;
    }
    Ok(())
}
