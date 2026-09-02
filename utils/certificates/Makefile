.PHONY: check
check:
	uv run python3 check-certificates.py
	ruff check
	ruff format --check
	ty check check-certificates.py
