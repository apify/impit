# This is used by the Github Actions to run the static analysis.

.PHONY: clean install-dev lint type-check format check-code

clean:
	rm -rf .mypy_cache .pytest_cache .ruff_cache build dist htmlcov .coverage

install-dev:
	uv sync --all-extras

lint:
	uv run ruff format --check
	uv run ruff check

type-check:
	uv run mypy

unit-tests:
	uv run pytest --numprocesses=auto --verbose --cov=python/impit test

format:
	uv run ruff check --fix
	uv run ruff format

check-code: lint type-check unit-tests
