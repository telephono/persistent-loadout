.PHONY: clean
clean:
	cargo clean

.PHONY: build/linux
build/linux:
	mkdir -p target/persistent-loadout/lin_x64
	cross build --target x86_64-unknown-linux-gnu --release
	cp target/x86_64-unknown-linux-gnu/release/libpersistent_loadout.so target/persistent-loadout/lin_x64/persistent-loadout.xpl

.PHONY: build/windows
build/windows:
	mkdir -p target/persistent-loadout/win_x64
	cross build --target x86_64-pc-windows-gnu --release
	cp target/x86_64-pc-windows-gnu/release/persistent_loadout.dll target/persistent-loadout/win_x64/persistent-loadout.xpl


.PHONY: build/all
build/all: build/windows build/linux
