.DEFAULT_GOAL := build

PREFIX ?= /usr
BINDIR ?= ${PREFIX}/bin

.PHONY: package
package:
	rm -rf package/oci-pilot
	rm -rf oci-pilot/target
	(cd oci-pilot && cargo vendor)
	mkdir package/oci-pilot
	cp -a oci-pilot package/oci-pilot/
	tar -C package -czf package/oci-pilot.tar.gz oci-pilot
	rm -rf oci-pilot/vendor
	rm -rf package/oci-pilot

.PHONY:build
build:
	cd oci-pilot && cargo build -v --release

clean:
	cd oci-pilot && cargo -v clean

test:
	cd oci-pilot && cargo -v build
	cd oci-pilot && cargo -v test

install:
	install -d -m 755 $(DESTDIR)$(BINDIR)
	install -m 755 oci-pilot/target/release/oci-pilot $(DESTDIR)$(BINDIR)/oci-pilot

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/oci-pilot
