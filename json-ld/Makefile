.PHONY: readme clean

readme: README.md

README.md: src/lib.rs
	cargo rdme
	sh .insert-toc.sh