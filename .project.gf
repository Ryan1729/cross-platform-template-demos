[gdb]
path=./rust-gdb

[commands]
Compile demos=shell cargo b --bin demos --profile debugging
Run demos=file target/debugging/demos;run&
