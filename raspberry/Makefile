.PHONY: build build-release deploy clean
build:
	cargo build
build-release:
	cargo install cross
	~/.cargo/bin/cross build --release --target armv7-unknown-linux-musleabihf
deploy: build-release
	scp target/armv7-unknown-linux-musleabihf/release/arduino_i2c eee@eee.local:~/
clean:
	cargo clean
