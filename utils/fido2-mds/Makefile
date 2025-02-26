PYTHON3 ?= python3
VIRTUAL_ENV ?= .venv
VIRTUAL_PYTHON3 ?= $(VIRTUAL_ENV)/bin/python3

METADATA_VERSION ?= v5
OUT_DIR ?= metadata/$(METADATA_VERSION)

INPUTS := generate-mds.py nitrokey.svg

MODELS := nk3am nk3xn
EXTS := json test.json
METADATA_FILES := \
    $(foreach ext,$(EXTS), \
        $(foreach model,$(MODELS), \
            $(OUT_DIR)/metadata-$(model)-$(METADATA_VERSION).$(ext) \
        ) \
    )

.PHONY: all
all:
	mkdir -p $(OUT_DIR)
	$(MAKE) $(METADATA_FILES)

.PHONY: clean
clean:
	rm -rf $(VIRTUAL_ENV)

$(OUT_DIR)/metadata-%-$(METADATA_VERSION).json: $(INPUTS)
	$(VIRTUAL_PYTHON3) generate-mds.py $* > "$@"

$(OUT_DIR)/metadata-%-$(METADATA_VERSION).test.json: $(INPUTS)
	$(VIRTUAL_PYTHON3) generate-mds.py $* --test > "$@"

# dependency & venv management

.PHONY: pip-compile
pip-compile:
	$(VIRTUAL_PYTHON3) -m piptools compile requirements.in -o requirements.txt

.PHONY: pip-venv
pip-venv:
	$(PYTHON3) -m venv $(VIRTUAL_ENV)
	$(VIRTUAL_PYTHON3) -m pip install pip-tools
	$(VIRTUAL_PYTHON3) -m piptools sync requirements.txt

.PHONY: uv-compile
uv-compile:
	uv pip compile requirements.in -o requirements.txt

.PHONY: uv-venv
uv-venv:
	uv venv
	uv pip sync requirements.txt
