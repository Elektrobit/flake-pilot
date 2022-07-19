.DEFAULT_GOAL := build

PREFIX ?= /usr
BINDIR ?= ${PREFIX}/bin

.PHONY: debian
debian: clean vendor sourcetar
	cp package/debbuild/cargo_config .
	tar --append --file=package/oci-pilot.tar cargo_config
	rm -f cargo_config
	gzip package/oci-pilot.tar
	mv package/oci-pilot.tar.gz package/debian
	@echo "Find package data for debian at package/debian"

.PHONY: debbuild
debbuild: clean vendor sourcetar
	gzip package/oci-pilot.tar
	mv package/oci-pilot.tar.gz package/debbuild
	@echo "Find package data for debbuild at package/debbuild"

vendor:
	(cd oci-pilot && cargo vendor)
	(cd oci-register && cargo vendor)

sourcetar:
	rm -rf package/oci-pilot
	mkdir package/oci-pilot
	cp Makefile package/oci-pilot
	cp -a oci-pilot package/oci-pilot/
	cp -a oci-register package/oci-pilot/
	tar -C package -cf package/oci-pilot.tar oci-pilot
	rm -rf package/oci-pilot

.PHONY:build
build:
	cd oci-pilot && cargo build -v --release
	cd oci-register && cargo build -v --release

clean:
	cd oci-pilot && cargo -v clean
	cd oci-register && cargo -v clean
	rm -rf oci-pilot/vendor
	rm -rf oci-register/vendor

test:
	cd oci-pilot && cargo -v build
	cd oci-pilot && cargo -v test

install:
	install -d -m 755 $(DESTDIR)$(BINDIR)
	install -m 755 oci-pilot/target/release/oci-pilot $(DESTDIR)$(BINDIR)/oci-pilot
	install -m 755 oci-register/target/release/oci-register $(DESTDIR)$(BINDIR)/oci-register

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/oci-*
