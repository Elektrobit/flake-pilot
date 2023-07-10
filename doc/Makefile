man:
	for man in $(shell find -name "*.rst" | sed -e "s@.rst@@"); do \
		rst2man $$man.rst $$man.8 ;\
	done

clean:
	rm -f *.8
