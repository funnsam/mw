EXEC_SUFFIX=
# ifeq $(WINDOWS_NT)
# endif

test: build
	- rm mw -rf
	cp website mw -r
	python3 -m http.server

build:
	cargo -C ../ -Z unstable-options build -r -j8
	../target/release/mkwebsite$(EXEC_SUFFIX)
