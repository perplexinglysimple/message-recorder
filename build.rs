extern crate prost_build;

fn main() {
    prost_build::compile_protos(&["src/protos/example.proto"], &["src/"]).unwrap();
}
