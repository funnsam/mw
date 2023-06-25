EXEC_SUFFIX=
# ifeq $(WINDOWS_NT)
# endif

build:
	cargo -C ../ -Z unstable-options build -r -j8
	../target/release/mkwebsite$(EXEC_SUFFIX)
