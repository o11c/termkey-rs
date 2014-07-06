RUSTC = rustc
RUSTCFLAGS = -g --opt-level=2

comma=,
PKGCONFIG_TERMKEY_LIBS := $(shell pkg-config --libs termkey)
ifeq '' '${PKGCONFIG_TERMKEY_LIBS}'
$(error Unable to get libs from pkg-config)
endif
PKGCONFIG_TERMKEY_RPATH := $(subst -L,-Wl${comma}-rpath=,$(shell pkg-config --libs-only-L termkey))
PKGCONFIG_TERMKEY := $(strip ${PKGCONFIG_TERMKEY_LIBS} ${PKGCONFIG_TERMKEY_RPATH})

default: termkey
all: demo termkey
demo: $(patsubst examples/demo-%.rs,demo-%,$(wildcard examples/demo-*.rs))
demo-%: examples/demo-%.rs termkey.stamp
	${RUSTC} ${RUSTCFLAGS} $< -L .

source.stamp: $(filter-out src/test.rs,$(wildcard src/*.rs))
	@touch $@

termkey.stamp: src/lib.rs source.stamp
	echo '#[link_args = "'${PKGCONFIG_TERMKEY}'"] extern {}' > src/generated_link.rs
	${RUSTC} ${RUSTCFLAGS} $< -C extra_filename=-rust
	@touch $@
termkey: termkey.stamp
.SUFFIXES:

test.stamp: src/test.rs termkey.stamp
	${RUSTC} ${RUSTCFLAGS} $< -L . --cfg=test -o run-tests
	@touch $@
test: test.stamp
	./run-tests

clean:
	rm -f libtermkey*.so run-tests demo-async demo-sync *.stamp
