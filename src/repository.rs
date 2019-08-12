use std::fs::File;

use crate::{config, crypto, CliResult};

pub struct Resource {
    pass: Option<String>,
    filepath: String,
    tempfile: tempfile::NamedTempFile,
}

impl Resource {
    pub fn new(config: config::Config, networth: Option<bool>) -> CliResult<Resource> {
        Ok(Resource {
            pass: config.pass(),
            filepath: config.filepath(networth),
            tempfile: tempfile::NamedTempFile::new()?,
        })
    }

    pub fn apply<F>(&self, action: F) -> CliResult<()>
    where
        F: FnOnce(&tempfile::NamedTempFile) -> CliResult<()>,
    {
        match &self.pass {
            Some(pass) => {
                let mut out_file = self.tempfile.reopen()?;
                let mut in_file = File::open(&self.filepath)?;
                crypto::decrypt(&mut in_file, &mut out_file, &pass)?;

                action(&self.tempfile)?;

                let mut in_file = self.tempfile.reopen()?;
                let mut out_file = File::create(&self.filepath)?;
                crypto::encrypt(&mut in_file, &mut out_file, &pass)?;
            }
            None => {
                std::fs::copy(&self.filepath, self.tempfile.path())?;

                action(&self.tempfile)?;

                std::fs::copy(self.tempfile.path(), &self.filepath)?;
            }
        };

        Ok(())
    }
}
