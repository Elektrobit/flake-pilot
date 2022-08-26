man:
	for man in $(shell find -name "*.rst" | sed -e "s@.rst@@"); do \
		pandoc --standalone --to man $$man.rst -o $$man.8 ;\
	done

clean:
	rm -f *.8
