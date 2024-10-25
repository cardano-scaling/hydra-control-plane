use std::fmt;

pub mod game;
pub mod hydra;
pub mod node;
pub mod tx_builder;

pub fn format_hex<T: AsRef<[u8]>>(data: T, f: &mut fmt::Formatter) -> fmt::Result {
    for b in data.as_ref() {
        write!(f, "{:02X}", b)?;
    }
    Ok(())
}
