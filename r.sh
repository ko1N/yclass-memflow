cargo build -r
sudo setcap 'CAP_SYS_PTRACE=ep' ./target/release/yclass-memflow
cd ./target/release && RUST_BACKTRACE=FULL RUST_SETPTRACE=1 ./yclass-memflow -c qemu -vv
