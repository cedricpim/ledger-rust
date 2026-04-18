# Ledger

The main idea behind this project is to create a ledger that can generate some
useful reports and balances and can be used from the command line tool. For
someone used to work in the terminal, this allows you to use the ledger through
your preferred editor, so that adding new lines is faster (than through any other UI).

The storage type is a simple CSV (that can be encrypted) with a couple of
default fields. A couple of other commands were implemented so that you can
integrate the `ledger` with other command line tools.

One extra feature is the usage of a networth file. Personally, I wanted to see
how I was doing and what was my current progress, so in a separate file I can
track my current assets (including the current value of investments). This is
available through a single command, so it can be set to automatically run daily
and get the values to generate a, hopefully, nice and motivating graph.

### Installation

1) Download the most recent release from https://github.com/cedricpim/ledger-rust/releases

2) Unarchive it.

4) Run `ledger configure` (a default configuration will be installed on `~/.config/ledger/config`)

5) Run `ledger create` (create ledger file) and `ledger create -n` (create networth file)

6) Run `ledger --help`

### Configuration

This is an example of a configuration file:

```
encryption: SOME_PASS
files:
  ledger: ~/.config/ledger/ledger.csv
  networth: ~/.config/ledger/networth.csv
exchange_key: API_KEY
transfer: Transfer
ignored_accounts: ['Vacation', 'Personal']
investments: Investment
currency: EUR
```

#### Encryption

This is optional, but if you want to use it, you just need to define a value for
the key `encryption:` and the system will encrypt the files with your data with
the encryption scheme ChaCha20Poly1305 (disclaimer: from what I read, this
seemed like a good choice but I claim no expertises in this domain). Due to how
the encryption is implemented, it is important that the password has 32
characters.

#### Files

There are two values (ledger and networth) and both are mandatory (even though
only ledger is essential for using the tool). Ledger points to the file that
will contain all the income and expenses and it it the file that is primarily
used. Networth points to the file that contains entries for the calculated
networth from `ledger networth --save` command.

#### Exchange Key

This is mandatory, even though maybe it didn't need to be. The idea is to
support multiple currencies (one per account) and for that, you can create an
account with [open exchange rates](https://openexchangerates.org/) for free,
generate the API key and add it as `exchange_key`. Then, `ledger` will be able
to convert entries with alternative currencies and it will be able to
consolidate all accounts with different currencies into a single one, the
default one.

#### Transfer

Mandatory field (but can be empty) that defines the category that is meant to be
recognized as a transfer. The idea of a transfer is special type of transaction,
from one account to the other. That means that, in the ledger file, there will
be two entries (an expense and an income) from one account to the other. These
type of lines could result in incorrect reports that would show a much higher
value of income and of expenses. To avoid that, the value defined for `transfer`
can be assigned to the line category and reports will ignore such values in the
calculations.

#### Ignored Accounts

Mandatory field (but can be an empty array) that defines the list of accounts
that don't count on calculations (either for reports or for networth). An use
case can be an account for hobbies where the transactions in that account
shouldn't be taken in consideration in reports nor should it enter the
calculation of networth.

#### Investments

Mandatory field (but can be empty) that defines which category of a line is
considered an investment. The lines with the same category as the one defined
for this field will be considered an investment and when running the command to
calculate the networth, the system will fetch the current valuation for each of
these lines.

#### Currency

Mandatory field and cannot be empty. It the defines default currency to which
all values should be converted when generating a report (mostly useful when
handling multiple accounts with different currencies). The format for the value
is the currency code for ISO 4217.

### Usage

`ledger --help` will provide with most of the information needed.

`ledger configure` will generate a default configuration file.

`ledger create` and `ledger create -n` can be used to create (and encrypt) the
files that will store the financial data.

#### Networth

`ledger networth` will calculate the current balances for each account and
consider that as "Cash" section. Furthermore, it will go through all entries
matching the value defined in configuration for `investment` and it will take
the description as the ISIN and the quatity column as the number of shares and
use [justETF](https://www.justetf.com/uk/) to calculate the current valuation
and display such value.

When append `--save`, the whole value (cash + investments valuation) will be
summed and added as a new entry (with the current date) to the networth file (as
part of that entry, it will also be calculated the total amount invested in the
current date as well as the total value of investments only).

### Development

#### Release

To ensure that the system is compatible to most popular Linux distributions, the
default compilation target is `x86_64-unknown-linux-musl`. For that to run
(`make release`), you need to ensure that `musl-gcc` is installed.
