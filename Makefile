CRATES := rxui_rx rxui_signals
SOURCE_DIRS := $(foreach crate,$(CRATES),$(shell find . -type d -wholename "*$(crate)/src"))
SOURCE_DIR_PATTERNS := $(shell echo $(SOURCE_DIRS) | sed -e 's/ /,/g' | sed -e 's/\.\///g')

all:
	cargo build

test:
	cargo test
	cargo clippy --all-targets --all-features -- -D warnings
	cargo fmt -- --check

coverage:
	@echo "Cleaning coverage..."
	@rm -rf target/cov
	@mkdir -p target/cov
	@echo "Cleaning tests..."
	@for crate in $(CRATES); do \
		find target/debug -maxdepth 1 -executable -name "$$crate-*" | xargs -r rm; \
	done
	@echo "Building tests..."
	@cargo test --no-run
	@export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$(shell rustc --print sysroot)/lib && \
		for crate in $(CRATES); do \
			test=$$(find target/debug/deps -maxdepth 1 -executable -name "$$crate-*"); \
			echo "Collecting coverage for" $$test "...";\
			kcov target/cov/$$crate \
				--include-pattern=$(SOURCE_DIR_PATTERNS) \
				--exclude-region='#[cfg(test)]:#[cfg(testkcovstopmarker)]' \
				$$test --quiet; \
		done
	@echo "Merging and publishing coverage..."
	@kcov --coveralls-id=$$TRAVIS_JOB_ID --merge target/cov $(foreach crate,$(CRATES),target/cov/$(crate))

clean:
	cargo clean
