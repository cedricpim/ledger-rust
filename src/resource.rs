use lockfile::Lockfile;
use tempfile::NamedTempFile;

use std::fs::File;
use std::fs::OpenOptions;

use crate::entity::line::{Line, Liner};
use crate::entity::{entry, transaction};
use crate::error::CliError;
use crate::{config, crypto, CliResult, Mode};

pub struct Resource {
    pub filepath: String,
    pub tempfile: NamedTempFile,
    pass: Option<String>,
    mode: Mode,
    _lock: Lockfile,
}

impl Resource {
    pub fn new(config: &config::Config, mode: Mode) -> CliResult<Resource> {
        let filepath = config.filepath(mode);

        Ok(Resource {
            pass: config.pass(),
            filepath: filepath.to_string(),
            tempfile: tempfile::Builder::new().suffix(".csv").tempfile()?,
            mode,
            _lock: Lockfile::create(format!("{}.lock", filepath))
                .map_err(|_| CliError::LockNotAcquired { filepath })?,
        })
    }

    pub fn headers(&self) -> Vec<&str> {
        match self.mode {
            Mode::Ledger => transaction::FIELDS.to_vec(),
            Mode::Networth => entry::FIELDS.to_vec(),
        }
    }

    pub fn create(&self) -> CliResult<()> {
        let mut wtr = csv::WriterBuilder::new().from_writer(&self.tempfile);

        wtr.write_record(self.headers())?;

        wtr.flush()?;

        self.close(&self.tempfile)?;

        Ok(())
    }

    pub fn create_with(&self, lines: Vec<Line>) -> CliResult<()> {
        let nfile = tempfile::Builder::new().suffix(".csv").tempfile()?;

        let mut wtr = csv::WriterBuilder::new().from_path(nfile.path())?;

        for line in lines {
            line.write(&mut wtr)?;
        }

        wtr.flush()?;

        self.close(&nfile)?;

        Ok(())
    }

    pub fn book(&mut self, lines: &[Line]) -> CliResult<()> {
        self.apply(|file| {
            let afile = OpenOptions::new().append(true).open(file.path())?;
            let mut wtr = csv::WriterBuilder::new()
                .has_headers(false)
                .from_writer(afile);

            for line in lines {
                line.write(&mut wtr)?;
            }

            wtr.flush()?;

            Ok(())
        })
    }

    pub fn apply<F>(&mut self, action: F) -> CliResult<()>
    where
        F: FnOnce(&NamedTempFile) -> CliResult<()>,
    {
        self.open()?;

        action(&self.tempfile)?;

        self.close(&self.tempfile)?;

        Ok(())
    }

    pub fn rewrite<F>(&mut self, action: &mut F) -> CliResult<()>
    where
        F: FnMut(&mut Line) -> CliResult<Vec<Line>>,
    {
        let accumulator = tempfile::Builder::new().suffix(".csv").tempfile()?;

        let mut wtr = csv::WriterBuilder::new().from_path(accumulator.path())?;

        self.line(&mut |record| {
            for line in action(record)? {
                line.write(&mut wtr)?;
            }

            wtr.flush()?;

            Ok(())
        })?;

        self.close(&accumulator)?;

        Ok(())
    }

    pub fn line<F>(&mut self, action: &mut F) -> CliResult<()>
    where
        F: FnMut(&mut Line) -> CliResult<()>,
    {
        let mode = self.mode;

        self.apply(|file| {
            let mut rdr = csv::Reader::from_reader(file);

            match mode {
                Mode::Ledger => {
                    for result in rdr.deserialize() {
                        action(&mut Line::Transaction(result?))?;
                    }
                }
                Mode::Networth => {
                    for result in rdr.deserialize() {
                        action(&mut Line::Entry(result?))?;
                    }
                }
            };

            Ok(())
        })?;

        Ok(())
    }

    fn open(&mut self) -> CliResult<()> {
        match self.pass.take() {
            Some(pass) => {
                let mut in_file = File::open(&self.filepath)?;
                let mut out_file = self.tempfile.reopen()?;

                match crypto::decrypt(&mut in_file, &mut out_file, &pass) {
                    Ok(_) => {
                        self.pass = Some(pass);
                        Ok(())
                    }
                    Err(CliError::CryptoIncorrectPassword) => {
                        std::fs::copy(&self.filepath, self.tempfile.path())?;
                        Ok(())
                    }
                    Err(err) => Err(err),
                }?;
            }
            None => {
                std::fs::copy(&self.filepath, self.tempfile.path())?;
            }
        };

        Ok(())
    }

    fn close(&self, tempfile: &NamedTempFile) -> CliResult<()> {
        match &self.pass {
            Some(pass) => {
                let mut in_file = tempfile.reopen()?;
                let mut out_file = File::create(&self.filepath)?;
                crypto::encrypt(&mut in_file, &mut out_file, pass)?;
            }
            None => {
                std::fs::copy(tempfile.path(), &self.filepath)?;
            }
        };

        Ok(())
    }
}
