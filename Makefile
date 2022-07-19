.DEFAULT_GOAL := build

PREFIX ?= /usr
BINDIR ?= ${PREFIX}/bin

.PHONY: debian
debian:
	rm -rf ../oci-pilot-0.0.1
	mkdir ../oci-pilot-0.0.1
	cp -a . ../oci-pilot-0.0.1
	cd ../oci-pilot-0.0.1/oci-pilot && cargo vendor
	cd ../oci-pilot-0.0.1/oci-register && cargo vendor
	tar -C ../ -czf ../oci-pilot-0.0.1.tar.gz oci-pilot-0.0.1
	cp -a package/debian ../oci-pilot-0.0.1
	cd ../oci-pilot-0.0.1 && debmake
	cd ../oci-pilot-0.0.1 && debuild

.PHONY: package
package:
	rm -rf package/oci-pilot
	rm -rf oci-pilot/target
	rm -rf oci-register/target
	(cd oci-pilot && cargo vendor)
	(cd oci-register && cargo vendor)
	mkdir package/oci-pilot
	cp -a oci-pilot package/oci-pilot/
	cp -a oci-register package/oci-register/
	tar -C package -czf package/oci-pilot.tar.gz oci-pilot oci-register
	rm -rf oci-pilot/vendor
	rm -rf oci-register/vendor
	rm -rf package/oci-pilot
	rm -rf package/oci-register

.PHONY:build
build:
	cd oci-pilot && cargo build -v --release
	cd oci-register && cargo build -v --release

clean:
	cd oci-pilot && cargo -v clean
	cd oci-register && cargo -v clean

test:
	cd oci-pilot && cargo -v build
	cd oci-pilot && cargo -v test

install:
	install -d -m 755 $(DESTDIR)$(BINDIR)
	install -m 755 oci-pilot/target/release/oci-pilot $(DESTDIR)$(BINDIR)/oci-pilot
	install -m 755 oci-register/target/release/oci-register $(DESTDIR)$(BINDIR)/oci-register

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/oci-*
