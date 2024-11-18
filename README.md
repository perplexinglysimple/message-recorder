# message-recorder
A simple message recorder for zmq

## Necassary installs

### Cargo and rustup are needed to build the code
### protoc is needed to autogen protos in the protos folder to usable rust code

## How to run tests and show code coverage

``` bash
cargo install grcov
rustup component add llvm-tools-preview
export RUSTFLAGS="-Cinstrument-coverage"
cargo build
export LLVM_PROFILE_FILE="message_recorder-%p-%m.profraw"
cargo test
grcov . -s . --binary-path ./target/debug/ -o target/coverage --keep-only 'src/*' --output-types html,cobertura
```