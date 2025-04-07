# Makefile for the impit-python repository, mostly for the CI.

.PHONY: install-dev lint type-check format check-code

install-dev:
	cd impit-python uv sync --all-extras

lint:
	cd impit-python && uv run ruff format --check
	cd impit-python && uv run ruff check

type-check:
	cd impit-python && uv run mypy

format:
	cd impit-python && uv run ruff check --fix
	cd impit-python && uv run ruff format

check-code: lint type-check unit-tests
