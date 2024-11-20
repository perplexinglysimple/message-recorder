# message-recorder
A simple message recorder for zmq

## Necassary installs
Cargo and rustup are needed to build the code

protoc is needed to autogen protos in the protos folder to usable rust code

## How to run unit tests and show code coverage

``` bash
cargo install grcov
rustup component add llvm-tools-preview
export RUSTFLAGS="-Cinstrument-coverage"
cargo build
export LLVM_PROFILE_FILE="message_recorder-%p-%m.profraw"
cargo test
grcov . -s . --binary-path ./target/debug/ -o target/coverage --keep-only 'src/*' --output-types html,cobertura
```

## How to run full system tests

This requires 2 terminals. This should be a docker compose thing but I am getting lazy

In one terminal
```bash
cargo run
```

In second terminal
```bash
cd test
python3.<9, 10, 11...> -m venv .venv
source .venv/bin/activate
python main.py
```

This will start publishing data. Depending on main.py and the config that data should start writing data to a file.

To view the data in the file use the read_compressed_data.py script
```bash
cd test
source .venv/bin/activate
python main.py
```
