.PHONY: readme clean

readme: README.md

README.md: src/lib.rs
	cargo rdme
	@ awk -i inplace -v r="$(cat .toc.md)" '{gsub(/<!-- toc -->/,r)}1' README.md