use std::str::FromStr;

use crate::{config,CliResult,crypto};

pub fn ledger() -> CliResult<()> {
    let config = config::load()?;

    match config.pass() {
        Some(pass) => {
            let filepath = config.filepath(false)?;

            let mut in_file = std::fs::File::open(filepath)?;

            let mut out_file = std::fs::File::create("x2")?;

            crypto::encrypt(&mut in_file, &mut out_file, &pass)?;

            let path2 = std::path::PathBuf::from_str("x2").unwrap();
            let mut in_file2 = std::fs::File::open(path2)?;
            let mut out_file2 = std::fs::File::create("x3")?;

            crypto::decrypt(&mut in_file2, &mut out_file2, &pass)?;
        },
        None => {

        }
    }

    // Check if it has encryption if enabled. If not, just return file (or tempfile?)
    return Ok(());
}

// pub fn networth() {
//
// }
