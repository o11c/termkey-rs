RUSTC = rustc
RUSTCFLAGS = -g --opt-level=2

PKGCONFIG_TERMKEY = $$(pkg-config --libs termkey) $$(pkg-config --libs-only-L termkey | sed 's:-L:-Wl,-rpath=:')

all: demo-sync demo-async test
demo-%: examples/demo-%.rs termkey
	${RUSTC} ${RUSTCFLAGS} $< -L .

termkey: src/lib.rs
	${RUSTC} ${RUSTCFLAGS} $< -C link-args="${PKGCONFIG_TERMKEY}"

test: src/test.rs termkey
	${RUSTC} ${RUSTCFLAGS} $< -L . --cfg=test -o run-tests
	./run-tests
