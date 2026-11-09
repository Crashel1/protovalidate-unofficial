SHELL := bash
.SHELLFLAGS := -eu -o pipefail -c
PYTHON ?= python3.8
.PHONY: build test format
build:
	uv tool run --from maturin maturin build --release -i $(PYTHON)
test:
	PYTHONPATH=gen $(PYTHON) -m pytest
format:
	cargo fmt
	uv tool run ruff format protovalidate_unofficial test
