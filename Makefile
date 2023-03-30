.DEFAULT_GOAL := build

PREFIX ?= /usr
BINDIR ?= ${PREFIX}/bin
SHAREDIR ?= ${PREFIX}/share/podman-pilot
FLAKEDIR ?= ${PREFIX}/share/flakes
TEMPLATEDIR ?= /etc/flakes

.PHONY: package
package: clean vendor sourcetar
	rm -rf package/build
	mkdir -p package/build
	gzip package/flake-pilot.tar
	mv package/flake-pilot.tar.gz package/build
	cp package/flake-pilot.spec package/build
	cp package/cargo_config package/build
	cp package/flake-pilot-rpmlintrc package/build
	# update changelog using reference file
	helper/update_changelog.py --since package/flake-pilot.changes.ref > \
		package/build/flake-pilot.changes
	helper/update_changelog.py --file package/flake-pilot.changes.ref >> \
		package/build/flake-pilot.changes
	@echo "Find package data at package/build"

vendor:
	(cd podman-pilot && cargo vendor)
	(cd flake-ctl && cargo vendor)

sourcetar:
	rm -rf package/flake-pilot
	mkdir package/flake-pilot
	cp Makefile package/flake-pilot
	cp -a podman-pilot package/flake-pilot/
	cp -a flake-ctl package/flake-pilot/
	cp -a doc package/flake-pilot/
	tar -C package -cf package/flake-pilot.tar flake-pilot
	rm -rf package/flake-pilot

.PHONY:build
build: man
	cd podman-pilot && cargo build -v --release && upx --best --lzma target/release/podman-pilot
	cd flake-ctl && cargo build -v --release && upx --best --lzma target/release/flake-ctl

clean:
	cd podman-pilot && cargo -v clean
	cd flake-ctl && cargo -v clean
	rm -rf podman-pilot/vendor
	rm -rf flake-ctl/vendor
	${MAKE} -C doc clean

test:
	cd podman-pilot && cargo -v build
	cd podman-pilot && cargo -v test

install:
	install -d -m 755 $(DESTDIR)$(BINDIR)
	install -d -m 755 $(DESTDIR)$(SHAREDIR)
	install -d -m 755 $(DESTDIR)$(TEMPLATEDIR)
	install -d -m 755 $(DESTDIR)$(FLAKEDIR)
	install -d -m 755 ${DESTDIR}usr/share/man/man8
	install -m 755 podman-pilot/target/release/podman-pilot \
		$(DESTDIR)$(BINDIR)/podman-pilot
	install -m 755 flake-ctl/target/release/flake-ctl \
		$(DESTDIR)$(BINDIR)/flake-ctl
	install -m 755 flake-ctl/debbuild/oci-deb \
		$(DESTDIR)$(BINDIR)/oci-deb
	install -m 644 flake-ctl/debbuild/container.spec.in \
		$(DESTDIR)$(SHAREDIR)/container.spec.in
	install -m 644 flake-ctl/template/container-flake.yaml \
		$(DESTDIR)$(TEMPLATEDIR)/container-flake.yaml
	install -m 644 doc/*.8 ${DESTDIR}usr/share/man/man8

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/flake-ctl
	rm -f $(DESTDIR)$(BINDIR)/podman-pilot
	rm -rf $(DESTDIR)$(FLAKEDIR) $(DESTDIR)$(SHAREDIR) $(DESTDIR)$(TEMPLATEDIR)

man:
	${MAKE} -C doc man
