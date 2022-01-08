use sodiumoxide::crypto::pwhash;
use sodiumoxide::crypto::secretstream;

use std::fs::File;
use std::io::{Read, Write};

use crate::error::CliError;
use crate::CliResult;

const CHUNK_SIZE: usize = 4096;
const SIGNATURE: [u8; 4] = [0xC1, 0x0A, 0x4B, 0xED];

pub fn encrypt(in_file: &mut File, out_file: &mut File, password: &str) -> CliResult<()> {
    let mut buf = [0; CHUNK_SIZE];
    let mut bytes_left = in_file.metadata()?.len();

    out_file.write_all(&SIGNATURE)?;

    let salt = pwhash::gen_salt();
    out_file.write_all(&salt.0)?;

    let key = key(password, &salt)?;
    let (mut stream, header) =
        secretstream::Stream::init_push(&key).map_err(|_| CliError::CryptoPushFailed)?;
    out_file.write_all(&header.0)?;

    loop {
        match (*in_file).read(&mut buf) {
            Ok(num_read) if num_read > 0 => {
                bytes_left -= num_read as u64;
                let tag = match bytes_left {
                    0 => secretstream::Tag::Final,
                    _ => secretstream::Tag::Message,
                };
                out_file.write_all(
                    &stream
                        .push(&buf[..num_read], None, tag)
                        .map_err(|_| CliError::EncryptionFailed)?,
                )?;
            }
            Err(e) => return Err(CliError::from(e)),
            _ => break,
        }
    }

    Ok(())
}

pub fn decrypt(in_file: &mut File, out_file: &mut File, password: &str) -> CliResult<()> {
    if in_file.metadata()?.len()
        <= (pwhash::SALTBYTES + secretstream::HEADERBYTES + SIGNATURE.len()) as u64
    {
        return Err(CliError::NotEncrypted);
    }

    let mut salt = [0u8; pwhash::SALTBYTES];
    let mut signature = [0u8; 4];

    in_file.read_exact(&mut signature)?;
    if signature == SIGNATURE {
        // if the signature is present, read into all of salt
        in_file.read_exact(&mut salt)?;
    } else {
        // or take the bytes from signature and read the rest from file
        salt[..4].copy_from_slice(&signature);
        in_file.read_exact(&mut salt[4..])?;
    }
    let salt = pwhash::Salt(salt);

    let mut header = [0u8; secretstream::HEADERBYTES];
    in_file.read_exact(&mut header)?;
    let header = secretstream::Header(header);

    let key = key(password, &salt)?;

    let mut buffer = [0u8; CHUNK_SIZE + secretstream::ABYTES];
    let mut stream =
        secretstream::Stream::init_pull(&header, &key).map_err(|_| CliError::CryptoPullFailed)?;

    while stream.is_not_finalized() {
        match in_file.read(&mut buffer) {
            Ok(num_read) if num_read > 0 => {
                let (decrypted, _tag) = stream
                    .pull(&buffer[..num_read], None)
                    .map_err(|_| CliError::CryptoIncorrectPassword)?;
                out_file.write_all(&decrypted)?;
            }
            Err(_) => return Err(CliError::CryptoIncorrectPassword),
            _ => return Err(CliError::DecryptionFailed),
        }
    }
    Ok(())
}

fn key(password: &str, salt: &pwhash::Salt) -> CliResult<secretstream::Key> {
    let mut key = [0u8; secretstream::KEYBYTES];

    match pwhash::derive_key(
        &mut key,
        password.as_bytes(),
        salt,
        pwhash::OPSLIMIT_INTERACTIVE,
        pwhash::MEMLIMIT_INTERACTIVE,
    ) {
        Ok(_) => Ok(secretstream::Key(key)),
        Err(_) => Err(CliError::CryptoDerivingKeyFailed),
    }
}
