use std::error::Error;

mod editor;
mod frontend;
mod log;

fn main() -> Result<(), Box<dyn Error>> {
    frontend::run()?;
    Ok(())
}
