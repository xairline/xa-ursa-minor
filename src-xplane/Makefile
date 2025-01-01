mkfile_path := $(abspath $(lastword $(MAKEFILE_LIST)))
current_dir := $(notdir $(patsubst %/,%,$(dir $(mkfile_path))))

dev:
	cargo build
	mkdir -p ~/X-Plane\ 12/Resources/plugins/xa-ursa-minor
	cp target/debug/libxa_ursa_minor.dylib ~/X-Plane\ 12/Resources/plugins/xa-ursa-minor/mac.xpl