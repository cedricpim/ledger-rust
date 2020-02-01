use custom_error::custom_error;

custom_error! { pub CliError
    Flag { source: docopt::Error }                                = @{ source },
    Csv { source: csv::Error }                                    = @{ source },
    Io { source: std::io::Error }                                 = @{ source },
    Yaml { source: serde_yaml::Error }                            = @{ source },
    Xdg { source: xdg::BaseDirectoriesError }                     = @{ source },
    Value { source: std::num::ParseIntError }                     = @{ source },
    DateFormat { source: chrono::format::ParseError }             = @{ source },
    JustETF { source: crate::service::justetf::Error }            = @{ source },
    NumberFormat { source: std::num::ParseFloatError }            = @{ source },
    MoneyFormat { source: steel_cent::formatting::ParseError }    = @{ source },
    Exchange { source: crate::service::openexchangerates::Error } = @{ source },

    IncorrectPath { filename: String }      = "An error occurred while determining the path to {filename}",
    ExistingConfiguration                   = "Configuration file already exists, use --force to overwrite it",
    ExistingFile  { filepath: String }      = "File {filepath} already exists, use --force to overwrite it",
    InvalidCommand { command: String }      = "ledger expects commands in lowercase. Did you mean '{command}'?",
    ExchangeInternetRequired                = "An internet connection is required for operations involving exchange rates",
    IncorrectCurrencyCode {code: String }   = "The currency code '{code}' does not exist",
    MissingExchangeRate {code: String }     = "There is no exchange currency for '{code}'",
    UndefinedEditor                         = "EDITOR variable is not set",
    CryptoPushFailed                        = "init_push failed",
    CryptoPullFailed                        = "init_pull failed",
    EncryptionFailed                        = "Encrypting file failed",
    DecryptionFailed                        = "Decrypting file failed",
    NotEncrypted                            = "File not big enough to have been encrypted",
    CryptoIncorrectPassword                 = "Incorrect password",
    CryptoDerivingKeyFailed                 = "Deriving key failed",
    Other { msg: &'static str }             = @{ msg }
}
