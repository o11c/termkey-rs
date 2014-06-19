RUSTC = rustc
RUSTCFLAGS = -g --opt-level=2

PKGCONFIG_TERMKEY = $$(pkg-config --libs termkey) $$(pkg-config --libs-only-L termkey | sed 's:-L:-Wl,-rpath=:')

all: demo test
demo: $(patsubst examples/demo-%.rs,demo-%,$(wildcard examples/demo-*.rs))
demo-%: examples/demo-%.rs termkey.stamp
	${RUSTC} ${RUSTCFLAGS} $< -L .

source.stamp: $(filter-out src/test.rs,$(wildcard src/*.rs))
	@touch $@

termkey.stamp: src/lib.rs source.stamp
	${RUSTC} ${RUSTCFLAGS} $< -C link-args="${PKGCONFIG_TERMKEY}"
	@touch $@
termkey: termkey.stamp

test.stamp: src/test.rs termkey.stamp
	${RUSTC} ${RUSTCFLAGS} $< -L . --cfg=test -o run-tests
	@touch $@
test: test.stamp
	./run-tests

clean:
	rm -f libtermkey*.so run-tests demo-async demo-sync *.stamp
