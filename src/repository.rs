use std::fs::File;

use crate::{config, crypto, entry, transaction, CliResult};

pub struct Resource {
    pub filepath: String,
    headers: Vec<&'static str>,
    tempfile: tempfile::NamedTempFile,
    pass: Option<String>,
}

impl Resource {
    pub fn new(config: config::Config, networth: bool) -> CliResult<Resource> {
        Ok(Resource {
            pass: config.pass(),
            filepath: config.filepath(networth),
            tempfile: tempfile::NamedTempFile::new()?,
            headers: if networth {
                entry::Entry::headers()
            } else {
                transaction::Transaction::headers()
            },
        })
    }

    pub fn create(&self) -> CliResult<()> {
        let mut wtr = csv::WriterBuilder::new().from_writer(&self.tempfile);
        wtr.write_record(&self.headers)?;
        wtr.flush()?;

        self.close()?;

        Ok(())
    }

    pub fn apply<F>(&self, action: F) -> CliResult<()>
    where
        F: FnOnce(&tempfile::NamedTempFile) -> CliResult<()>,
    {
        self.open()?;

        action(&self.tempfile)?;

        self.close()?;

        Ok(())
    }

    fn open(&self) -> CliResult<()> {
        match &self.pass {
            Some(pass) => {
                let mut out_file = self.tempfile.reopen()?;
                let mut in_file = File::open(&self.filepath)?;
                crypto::decrypt(&mut in_file, &mut out_file, &pass)?;
            }
            None => {
                std::fs::copy(&self.filepath, self.tempfile.path())?;
            }
        };

        Ok(())
    }

    fn close(&self) -> CliResult<()> {
        match &self.pass {
            Some(pass) => {
                let mut in_file = self.tempfile.reopen()?;
                let mut out_file = File::create(&self.filepath)?;
                crypto::encrypt(&mut in_file, &mut out_file, &pass)?;
            }
            None => {
                std::fs::copy(self.tempfile.path(), &self.filepath)?;
            }
        };

        Ok(())
    }
}
