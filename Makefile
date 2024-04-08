.PHONY: readme clean crates/syntax/README.md crates/core/README.md crates/context-processing/README.md crates/expansion/README.md crates/compaction/README.md crates/serialization/README.md crates/testing/README.md crates/tests/README.md crates/cli/README.md README.md

readme: crates/syntax/README.md crates/core/README.md crates/context-processing/README.md crates/expansion/README.md crates/compaction/README.md crates/serialization/README.md crates/testing/README.md crates/tests/README.md crates/cli/README.md README.md

crates/syntax/README.md: crates/syntax/src/lib.rs
	make -C crates/syntax readme

crates/core/README.md: crates/core/src/lib.rs
	make -C crates/core readme

crates/context-processing/README.md: crates/context-processing/src/lib.rs
	make -C crates/context-processing readme

crates/expansion/README.md: crates/expansion/src/lib.rs
	make -C crates/expansion readme

crates/compaction/README.md: crates/compaction/src/lib.rs
	make -C crates/compaction readme

crates/serialization/README.md: crates/serialization/src/lib.rs
	make -C crates/serialization readme

crates/cli/README.md: crates/cli/src/main.rs
	make -C crates/cli/json-ld readme

crates/testing/README.md: crates/testing/src/lib.rs
	make -C crates/testing readme

README.md: src/lib.rs
	cargo rdme

clean:
	rm README.md