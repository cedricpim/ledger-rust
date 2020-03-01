# Ledger

A command line ledger.

### Instructions

1) Download the most recent release from https://github.com/cedricpim/ledger-rust/releases

2) Unarchive it.

4) Run `ledger configure` (a default configuration will be installed on `~/.config/ledger/config`)

5) Run `ledger create` (create ledger file) and `ledger create -n` (create networth file)

6) Run `ledger --help`

### Release

To ensure that the system is compatible to most popular Linux distributions, the
default compilation target is `x86_64-unknown-linux-musl`. For that to run
(`make release`), you need to ensure that `musl-gcc` is installed.
