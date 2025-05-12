
clean:
	rm -r test/output

BIN = ./target/release/pump

$(BIN): $(shell find src/ -name "*.rs")
	cargo build --release

ALL_TESTS = $(shell find test -name "*.sh" | sed 's/sh\>/suc/g' | sed 's/\<test/test\/output/g')

test: $(ALL_TESTS)

test/output:
	mkdir test/output

test/output/%.suc: test/%.sh $(BIN) test/output
	bash "$<" $(BIN) && touch $@
