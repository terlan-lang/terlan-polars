TERLC ?= ../terlan/target/debug/terlc
PYTORCH ?= ../terlan-pytorch
NDARRAY ?= ../terlan-ndarray
LIBTORCH ?=
IRIS := $(CURDIR)/examples/iris_dataset_audit/data/iris.csv
POLARS_HELPER := $(CURDIR)/native/target/debug/terlan-polars-native-boundary
NDARRAY_HELPER := $(abspath $(NDARRAY))/native/rust/target/debug/native-boundary-helper
PYTORCH_HELPER := $(abspath $(PYTORCH))/_build/rust-target-cpu/debug/native-boundary-helper

.PHONY: package-check scripts-check parity-check strict-docs-check visualization-feature-check terlan-grouped-binding-check terlan-function-reference-check release-check polars-pytorch-interop-check polars-ndarray-interop-check polars-ndarray-pytorch-interop-check polars-immutable-ml-interop-check

package-check: terlan-grouped-binding-check terlan-function-reference-check scripts-check
	@test -x "$(TERLC)" || { echo "missing terlc: $(TERLC)" >&2; exit 1; }
	cargo test --manifest-path native/Cargo.toml --offline --features real-polars
	cargo build --manifest-path native/Cargo.toml --offline --features real-polars --bin terlan-polars-native-boundary
	"$(TERLC)" check src
	TERLAN_POLARS_NATIVE_BOUNDARY_HELPER_PATH="$(POLARS_HELPER)" \
		"$(TERLC)" test test

scripts-check:
	"$(TERLC)" run script check-sources -- "$(abspath $(TERLC))" "$(CURDIR)" scripts

terlan-grouped-binding-check:
	"$(TERLC)" run script check-sources -- "$(abspath $(TERLC))" "$(CURDIR)" TL0009

terlan-function-reference-check:
	"$(TERLC)" run script check-sources -- "$(abspath $(TERLC))" "$(CURDIR)" TL0010

parity-check:
	cargo run --manifest-path native/Cargo.toml --offline --features real-polars --bin polars_api_parity -- --require-complete

strict-docs-check:
	RUSTDOCFLAGS='--document-private-items -D missing_docs' \
		cargo doc --offline --no-deps --manifest-path native/Cargo.toml --features real-polars

visualization-feature-check:
	cargo test --offline --manifest-path native/Cargo.toml \
		--features real-polars,plotly-html,plot-png visualization::tests

release-check: package-check parity-check strict-docs-check visualization-feature-check
	"$(TERLC)" fmt --check src
	"$(TERLC)" fmt --check test

polars-pytorch-interop-check:
	@test -x "$(TERLC)" || { echo "missing terlc: $(TERLC)" >&2; exit 1; }
	@test -n "$(LIBTORCH)" || { echo "LIBTORCH must point at a LibTorch distribution" >&2; exit 1; }
	cargo build --manifest-path native/Cargo.toml --offline --features real-polars --bin terlan-polars-native-boundary
	$(MAKE) -C "$(PYTORCH)" generate TERLC="$(abspath $(TERLC))"
	CARGO_TARGET_DIR="$(abspath $(PYTORCH))/_build/rust-target-cpu" \
	LIBTORCH="$(abspath $(LIBTORCH))" cargo build --manifest-path "$(PYTORCH)/generated/native/rust/Cargo.toml" --offline --bin native-boundary-helper
	TERLAN_POLARS_NATIVE_BOUNDARY_HELPER_PATH="$(POLARS_HELPER)" \
	TERLAN_PYTORCH_NATIVE_BOUNDARY_HELPER_PATH="$(PYTORCH_HELPER)" \
		"$(TERLC)" run script check-pytorch-interop -- \
			"$(abspath $(TERLC))" "$(CURDIR)" "$(abspath $(PYTORCH))" "$(IRIS)"

polars-ndarray-pytorch-interop-check:
	@test -x "$(TERLC)" || { echo "missing terlc: $(TERLC)" >&2; exit 1; }
	@test -f "$(NDARRAY)/terlan.toml" || { echo "missing ndarray package: $(NDARRAY)" >&2; exit 1; }
	@test -n "$(LIBTORCH)" || { echo "LIBTORCH must point at a LibTorch distribution" >&2; exit 1; }
	cargo build --manifest-path native/Cargo.toml --offline --features real-polars --bin terlan-polars-native-boundary
	cargo build --manifest-path "$(NDARRAY)/native/rust/Cargo.toml" --offline --bin native-boundary-helper
	$(MAKE) -C "$(PYTORCH)" generate TERLC="$(abspath $(TERLC))"
	CARGO_TARGET_DIR="$(abspath $(PYTORCH))/_build/rust-target-cpu" \
	LIBTORCH="$(abspath $(LIBTORCH))" cargo build --manifest-path "$(PYTORCH)/generated/native/rust/Cargo.toml" --offline --bin native-boundary-helper
	TERLAN_POLARS_NATIVE_BOUNDARY_HELPER_PATH="$(POLARS_HELPER)" \
	TERLAN_NDARRAY_NATIVE_BOUNDARY_HELPER_PATH="$(NDARRAY_HELPER)" \
	TERLAN_PYTORCH_NATIVE_BOUNDARY_HELPER_PATH="$(PYTORCH_HELPER)" \
		"$(TERLC)" run script check-ndarray-pytorch-interop -- \
			"$(abspath $(TERLC))" "$(CURDIR)" "$(abspath $(NDARRAY))" "$(abspath $(PYTORCH))" "$(IRIS)"

# Backward-compatible name used by the numerical-convergence roadmap.
polars-ndarray-interop-check: polars-ndarray-pytorch-interop-check

polars-immutable-ml-interop-check:
	@test -x "$(TERLC)" || { echo "missing terlc: $(TERLC)" >&2; exit 1; }
	@test -f "$(NDARRAY)/terlan.toml" || { echo "missing ndarray package: $(NDARRAY)" >&2; exit 1; }
	@test -f "$(PYTORCH)/terlan.toml" || { echo "missing pytorch package: $(PYTORCH)" >&2; exit 1; }
	@test -n "$(LIBTORCH)" || { echo "LIBTORCH must point at a LibTorch distribution" >&2; exit 1; }
	"$(TERLC)" run script check-immutable-ml-interop -- \
		"$(abspath $(TERLC))" "$(CURDIR)" "$(abspath $(NDARRAY))" \
		"$(abspath $(PYTORCH))" "$(abspath $(LIBTORCH))" "$(IRIS)"
