use std::error::Error;

mod frontend;
mod keys;
mod log;

fn main() -> Result<(), Box<dyn Error>> {
    frontend::run()?;
    Ok(())
}
