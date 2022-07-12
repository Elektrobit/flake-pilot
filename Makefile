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
