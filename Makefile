.PHONY: clean
clean:
	cargo clean

.PHONY: build/linux
build/linux:
	mkdir -p persistent-loadout/lin_x64
	cargo build --target x86_64-unknown-linux-gnu --release
	cp target/x86_64-unknown-linux-gnu/release/libpersistent_loadout.so persistent-loadout/lin_x64/persistent-loadout.xpl

.PHONY: build/mac
build/mac:
	mkdir -p persistent-loadout/mac_x64
	cargo build --target aarch64-apple-darwin --release
	cargo build --target x86_64-apple-darwin --release
	lipo -create -output persistent-loadout/mac_x64/persistent-loadout.xpl target/aarch64-apple-darwin/release/libpersistent_loadout.dylib target/x86_64-apple-darwin/release/libpersistent_loadout.dylib

.PHONY: build/windows
build/windows:
	mkdir -p persistent-loadout/win_x64
	cross build --target x86_64-pc-windows-gnu --release
	cp target/x86_64-pc-windows-gnu/release/persistent_loadout.dll persistent-loadout/win_x64/persistent-loadout.xpl
