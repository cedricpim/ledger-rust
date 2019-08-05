use std::fs::File;
use std::io::{Read, Write};

use crate::{config, crypto, CliResult};

pub struct Resource {
    pass: Option<String>,
    filepath: String,
    tempfile: tempfile::NamedTempFile,
}

impl Resource {
    pub fn new(config: config::Config, networth: Option<bool>) -> CliResult<Resource> {
        return Ok(Resource {
            pass: config.pass(),
            filepath: config.filepath(networth)?,
            tempfile: tempfile::NamedTempFile::new()?,
        });
    }

    pub fn apply<F>(&self, action: F) -> CliResult<()>
    where
        F: FnOnce(&tempfile::NamedTempFile) -> CliResult<()>,
    {
        match &self.pass {
            Some(pass) => {
                let mut in_file = File::open(&self.filepath)?;
                let mut out_file = self.tempfile.reopen()?;
                crypto::decrypt(&mut in_file, &mut out_file, &pass)?;

                action(&self.tempfile)?;

                let mut in_file = File::create(&self.filepath)?;
                let mut out_file = self.tempfile.reopen()?;
                crypto::encrypt(&mut out_file, &mut in_file, &pass)?;
            }
            None => {
                let mut out_file = self.tempfile.reopen()?;
                let mut in_file = File::open(&self.filepath)?;
                let mut buf = String::new();
                in_file.read_to_string(&mut buf)?;
                out_file.write_all(buf.as_bytes())?;

                action(&self.tempfile)?;

                let mut in_file = File::create(&self.filepath)?;
                let mut out_file = self.tempfile.reopen()?;
                let mut buf = String::new();
                out_file.read_to_string(&mut buf)?;
                in_file.write_all(buf.as_bytes())?;
            }
        };

        return Ok(());
    }
}
