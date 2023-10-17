.DEFAULT_GOAL := build

PREFIX ?= /usr
BINDIR ?= ${PREFIX}/bin
SBINDIR ?= ${PREFIX}/sbin
SHAREDIR ?= ${PREFIX}/share/podman-pilot
FLAKEDIR ?= ${PREFIX}/share/flakes
TEMPLATEDIR ?= /etc/flakes

ARCH = $(shell uname -m)

.PHONY: package
package: clean vendor sourcetar
	rm -rf package/build
	mkdir -p package/build
	gzip package/flake-pilot.tar
	mv package/flake-pilot.tar.gz package/build
	cp package/flake-pilot.spec package/build
	cp package/cargo_config package/build
	cp package/gcc_fix_static.sh package/build
	cp package/flake-pilot-rpmlintrc package/build
	# update changelog using reference file
	helper/update_changelog.py --since package/flake-pilot.changes.ref > \
		package/build/flake-pilot.changes
	helper/update_changelog.py --file package/flake-pilot.changes.ref >> \
		package/build/flake-pilot.changes
	@echo "Find package data at package/build"

vendor:
	cargo vendor

sourcetar:
	rm -rf package/flake-pilot
	mkdir package/flake-pilot
	cp Makefile package/flake-pilot
	cp -a podman-pilot package/flake-pilot/
	cp -a flake-ctl package/flake-pilot/
	cp -a firecracker-pilot package/flake-pilot/
	cp -a doc package/flake-pilot/
	cp -a utils package/flake-pilot/
	cp -a vendor package/flake-pilot
	cp Cargo.toml package/flake-pilot

	# Delete any target directories that may be present
	find package/flake-pilot -type d -wholename "*/target" -prune -exec rm -rf {} \;

	# Delete large chunk windows and wasm dependencies
	# Use filtered vendoring in the future
	# https://github.com/rust-lang/cargo/issues/7058
	find package/flake-pilot -type d -wholename "*/vendor/winapi*" -prune -exec \
		rm -rf {}/src \; -exec mkdir -p {}/src \; -exec touch {}/src/lib.rs \; -exec rm -rf {}/lib \;
	find package/flake-pilot -type d -wholename "*/vendor/windows*" -prune -exec \
		rm -rf {}/src \; -exec mkdir -p {}/src \;  -exec touch {}/src/lib.rs \; -exec rm -rf {}/lib \;

	rm -rf package/flake-pilot/vendor/web-sys/src/*
	rm -rf package/flake-pilot/vendor/web-sys/webidls
	touch package/flake-pilot/vendor/web-sys/src/lib.rs

	tar -C package -cf package/flake-pilot.tar flake-pilot
	rm -rf package/flake-pilot

.PHONY:build
build: man
	cargo build -v --release
	cd firecracker-pilot/guestvm-tools/sci && RUSTFLAGS='-C target-feature=+crt-static' cargo build -v --release --target $(ARCH)-unknown-linux-gnu

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
	$(shell find . -name Cargo.lock | xargs rm -f)
	$(shell find . -type d -name vendor | xargs rm -rf)

test:
	cargo nextest run

install:
	install -d -m 755 $(DESTDIR)$(BINDIR)
	install -d -m 755 $(DESTDIR)$(SBINDIR)
	install -d -m 755 $(DESTDIR)$(SHAREDIR)
	install -d -m 755 $(DESTDIR)$(TEMPLATEDIR)
	install -d -m 755 $(DESTDIR)$(FLAKEDIR)
	install -d -m 755 ${DESTDIR}/usr/share/man/man8
	install -m 755 target/release/podman-pilot \
		$(DESTDIR)$(BINDIR)/podman-pilot
	install -m 755 target/release/firecracker-pilot \
		$(DESTDIR)$(BINDIR)/firecracker-pilot
	install -m 755 target/release/firecracker-service \
		$(DESTDIR)$(BINDIR)/firecracker-service
	install -m 755 target/$(ARCH)-unknown-linux-gnu/release/sci \
		$(DESTDIR)$(SBINDIR)/sci
	install -m 755 target/release/flake-ctl \
		$(DESTDIR)$(BINDIR)/flake-ctl
	install -m 755 target/release/flake-ctl-podman \
		$(DESTDIR)$(BINDIR)/flake-ctl-podman
	install -m 755 target/release/flake-ctl-firecracker \
		$(DESTDIR)$(BINDIR)/flake-ctl-firecracker
	install -m 755 flake-ctl/flake-ctl-podman/debbuild/oci-deb \
		$(DESTDIR)$(BINDIR)/oci-deb
	install -m 644 flake-ctl/flake-ctl-podman/debbuild/container.spec.in \
		$(DESTDIR)$(SHAREDIR)/container.spec.in
	install -m 644 firecracker-pilot/template/firecracker.json \
		$(DESTDIR)$(TEMPLATEDIR)/firecracker.json
	install -m 644 doc/*.8 ${DESTDIR}/usr/share/man/man8
	install -m 755 utils/* $(DESTDIR)$(SBINDIR)

	# dpkg

	install -d -m 755 $(FLAKEDIR)/package/dpkg
	install -m 644 flake-ctl/flake-ctl-build-dpkg/templates/* $(FLAKEDIR)/package/dpkg
	install -m 755 target/release/flake-ctl-build-dpkg $(DESTDIR)$(BINDIR)/flake-ctl-build-dpkg

	# rpmbuild
	install -d -m 755 $(FLAKEDIR)/package/rpmbuild
	install -m 644 flake-ctl/flake-ctl-build-rpmbuild/templates/* $(FLAKEDIR)/package/rpmbuild
	install -m 755 target/release/flake-ctl-build-rpmbuild $(DESTDIR)$(BINDIR)/flake-ctl-build-rpmbuild

uninstall:
	rm -f $(DESTDIR)$(BINDIR)/flake-ctl
	rm -f $(DESTDIR)$(BINDIR)/flake-ctl-podman
	rm -f $(DESTDIR)$(BINDIR)/flake-ctl-firecracker
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
