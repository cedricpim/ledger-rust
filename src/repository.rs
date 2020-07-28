use lockfile::Lockfile;

use std::fs::File;
use std::fs::OpenOptions;

use crate::entity::line::{Line, Liner};
use crate::error::CliError;
use crate::{config, crypto, CliResult};

pub struct Resource {
    pub kind: Line,
    pub filepath: String,
    pub tempfile: tempfile::NamedTempFile,
    pass: Option<String>,
}

impl Resource {
    pub fn new(config: &config::Config, networth: bool) -> CliResult<Resource> {
        Ok(Resource {
            pass: config.pass(),
            filepath: config.filepath(networth),
            tempfile: tempfile::Builder::new().suffix(".csv").tempfile()?,
            kind: Line::default(networth),
        })
    }

    pub fn create(&self) -> CliResult<()> {
        let mut wtr = csv::WriterBuilder::new().from_writer(&self.tempfile);

        wtr.write_record(self.kind.headers())?;

        wtr.flush()?;

        self.close()?;

        Ok(())
    }

    pub fn line<F>(&self, action: &mut F) -> CliResult<()>
    where
        F: FnMut(&mut Line) -> CliResult<()>,
    {
        self.apply(|file| {
            let mut rdr = csv::Reader::from_reader(file);

            match self.kind {
                Line::Entry(_) => {
                    for result in rdr.deserialize() {
                        action(&mut Line::Entry(result?))?;
                    }
                }
                Line::Transaction(_) => {
                    for result in rdr.deserialize() {
                        action(&mut Line::Transaction(result?))?;
                    }
                }
            };

            Ok(())
        })?;

        Ok(())
    }

    pub fn apply<F>(&self, action: F) -> CliResult<()>
    where
        F: FnOnce(&tempfile::NamedTempFile) -> CliResult<()>,
    {
        let lock = self.lock()?;

        self.open()?;

        action(&self.tempfile)?;

        self.close()?;

        lock.release()?;

        Ok(())
    }

    pub fn book(&self, lines: &[Line]) -> CliResult<()> {
        self.apply(|file| {
            let afile = OpenOptions::new().append(true).open(file.path())?;
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(afile);

            for line in lines {
                line.write(&mut wtr)?;
                wtr.flush()?;
            }

            Ok(())
        })
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

    fn lock(&self) -> CliResult<Lockfile> {
        Lockfile::create(format!("{}.lock", self.filepath)).map_err(|_| CliError::LockNotAcquired {
            filepath: self.filepath.to_string(),
        })
    }
}
