.DEFAULT_GOAL := build

PREFIX ?= /usr
BINDIR ?= ${PREFIX}/bin
SBINDIR ?= ${PREFIX}/sbin
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
	(cd firecracker-pilot && cargo vendor)
	(cd flake-ctl && cargo vendor)
	(cd firecracker-pilot/firecracker-service/service && cargo vendor)
	(cd firecracker-pilot/firecracker-service/service-communication && cargo vendor)
	(cd firecracker-pilot/guestvm-tools/sci && cargo vendor)

sourcetar:
	rm -rf package/flake-pilot
	mkdir package/flake-pilot
	cp Makefile package/flake-pilot
	cp -a podman-pilot package/flake-pilot/
	cp -a flake-ctl package/flake-pilot/
	cp -a firecracker-pilot package/flake-pilot/
	cp -a doc package/flake-pilot/
	cp -a utils package/flake-pilot/
	tar -C package -cf package/flake-pilot.tar flake-pilot
	rm -rf package/flake-pilot

.PHONY:build
build: man
	cd podman-pilot && cargo build -v --release && upx --best --lzma target/release/podman-pilot
	cd flake-ctl && cargo build -v --release && upx --best --lzma target/release/flake-ctl
	cd firecracker-pilot/firecracker-service/service && cargo build -v --release && upx --best --lzma target/release/firecracker-service
	cd firecracker-pilot/guestvm-tools/sci && cargo build -v --release
	cd firecracker-pilot && cargo build -v --release && upx --best --lzma target/release/firecracker-pilot

clean:
	cd podman-pilot && cargo -v clean
	cd firecracker-pilot && cargo -v clean
	cd flake-ctl && cargo -v clean
	cd firecracker-pilot/firecracker-service/service && cargo -v clean
	cd firecracker-pilot/guestvm-tools/sci && cargo -v clean
	rm -rf podman-pilot/vendor
	rm -rf flake-ctl/vendor
	rm -rf firecracker-pilot/firecracker-service/service/vendor
	rm -rf firecracker-pilot/firecracker-service/service-communication/vendor
	rm -rf firecracker-pilot/guestvm-tools/sci/vendor
	${MAKE} -C doc clean

test:
	cd podman-pilot && cargo -v build
	cd podman-pilot && cargo -v test

install:
	install -d -m 755 $(DESTDIR)$(BINDIR)
	install -d -m 755 $(DESTDIR)$(SBINDIR)
	install -d -m 755 $(DESTDIR)$(SHAREDIR)
	install -d -m 755 $(DESTDIR)$(TEMPLATEDIR)
	install -d -m 755 $(DESTDIR)$(FLAKEDIR)
	install -d -m 755 ${DESTDIR}usr/share/man/man8
	install -m 755 podman-pilot/target/release/podman-pilot \
		$(DESTDIR)$(BINDIR)/podman-pilot
	install -m 755 firecracker-pilot/target/release/firecracker-pilot \
		$(DESTDIR)$(BINDIR)/firecracker-pilot
	install -m 755 firecracker-pilot/firecracker-service/service/target/release/firecracker-service \
		$(DESTDIR)$(BINDIR)/firecracker-service
	install -m 755 firecracker-pilot/guestvm-tools/sci/target/release/sci \
		$(DESTDIR)$(SBINDIR)/sci
	install -m 755 flake-ctl/target/release/flake-ctl \
		$(DESTDIR)$(BINDIR)/flake-ctl
	install -m 755 flake-ctl/debbuild/oci-deb \
		$(DESTDIR)$(BINDIR)/oci-deb
	install -m 644 flake-ctl/debbuild/container.spec.in \
		$(DESTDIR)$(SHAREDIR)/container.spec.in
	install -m 644 flake-ctl/template/container-flake.yaml \
		$(DESTDIR)$(TEMPLATEDIR)/container-flake.yaml
	install -m 644 flake-ctl/template/firecracker-flake.yaml \
		$(DESTDIR)$(TEMPLATEDIR)/firecracker-flake.yaml
	install -m 644 firecracker-pilot/template/firecracker.json \
		$(DESTDIR)$(TEMPLATEDIR)/firecracker.json
	install -m 644 doc/*.8 ${DESTDIR}usr/share/man/man8
	install -m 755 utils/* $(DESTDIR)$(SBINDIR)

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/flake-ctl
	rm -f $(DESTDIR)$(BINDIR)/podman-pilot
	rm -f $(DESTDIR)$(BINDIR)/firecracker-pilot
	rm -f $(DESTDIR)$(BINDIR)/firecracker-service
	rm -rf $(DESTDIR)$(FLAKEDIR) $(DESTDIR)$(SHAREDIR) $(DESTDIR)$(TEMPLATEDIR)

man:
	${MAKE} -C doc man

cargo:
	for path in $(shell find . -name Cargo.toml ! -path "*/vendor/*");do \
		pushd `dirname $$path`; cargo build || exit 1; popd;\
	done
