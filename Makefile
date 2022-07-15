.DEFAULT_GOAL := build

.PHONY: package
package:
	rm -rf package/oci-pilot
	rm -rf launcher/target
	(cd launcher &&	cargo vendor)
	mkdir package/oci-pilot
	cp -a launcher package/oci-pilot/
	tar -C package -czf package/oci-pilot.tar.gz oci-pilot
	rm -rf launcher/vendor
	rm -rf package/oci-pilot

.PHONY:build
build:
	cd launcher && cargo build -v --release

clean:
	cd launcher && cargo -v clean

test:
	cd launcher && cargo -v build
	cd launcher && cargo -v test
