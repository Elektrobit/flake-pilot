.PHONY: package
package:
	rm -rf package/pilot
	rm -rf launcher/target
	(cd launcher &&	cargo vendor)
	mkdir package/pilot
	cp -a launcher package/pilot/
	tar -C package -czf package/pilot.tar.gz pilot
	rm -rf launcher/vendor
	rm -rf package/pilot
