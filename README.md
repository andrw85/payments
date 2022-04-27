# Payments engine

Payments is a toy payment engine that reads a series of transactions from a CSV, updates client accounts, handles disputes and chargebacks, and then outputs the state of clients accounts as a CSV.


## Quick start on Linux/Mac

Run the following script which builds the docker image that contains the rust toolchain and the developent environment:

```
scripts/run.sh
```

It leaves you logged in the docker container after compiling and running all unit tests in the project.



### Running App with different log level

By using the `RUST_LOG` environment variable you can set the log level. You can choose between several log levels althould for the current implementation `debug` and `info` are the most useful. If no variable is specified it uses `info` level by default.

Running in debug mode:

```
$ RUST_LOG=debug  cargo run --  sample-frezing-account.csv
[2022-04-27T12:24:17Z DEBUG payments] Account before: 0,0,0,false, tx: Deposit(2, 1, 3.0)
[2022-04-27T12:24:17Z DEBUG payments] Account after: 3,0,3,false
[2022-04-27T12:24:17Z DEBUG payments] Account before: 3,0,3,false, tx: Withdrawal(2, 2, 3.0)
[2022-04-27T12:24:17Z DEBUG payments] Account after: 0,0,0,false
[2022-04-27T12:24:17Z DEBUG payments] Account before: 0,0,0,false, tx: Dispute(2, 2, 0.0)
[2022-04-27T12:24:17Z DEBUG payments] Account after: 0,3,3,false
[2022-04-27T12:24:17Z DEBUG payments] Account before: 0,3,3,false, tx: Chargeback(2, 2, 0.0)
[2022-04-27T12:24:17Z DEBUG payments] Account after: 0,0,0,true
client, available, held, total, locked
2,0,0,0,true
```


## Project structure:

The project is organized in the following way:

```
├── Cargo.toml # rust workspace
├── Dockerfile
├── LICENSE
├── payments # contains the logic for the payment application
│   ├── Cargo.toml # payments app
│   ├── README.md
│   └── src
├── payments-core # framework/library that provides a set of tools for building the payment application
│   ├── Cargo.toml # crate providing a library
│   └── src
├── README.md
├── sample.csv # sample csv file
├── sample-frezing-account.csv # another sample csv file
└── scripts
    └── run.sh # script for starting a docker container for running the app
```



## Build docs without dependencies:

To build the documentation and open it in the browser run the following command:

```
cargo doc --open --no-deps
```
