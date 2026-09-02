VIRTUAL_PYTHON3 ?= uv run python3

METADATA_VERSION ?= v6
OUT_DIR ?= metadata/$(METADATA_VERSION)

INPUTS := generate-mds.py nitrokey.svg get-info.json

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

$(OUT_DIR)/metadata-%-$(METADATA_VERSION).json: $(INPUTS)
	$(VIRTUAL_PYTHON3) generate-mds.py $* > "$@"

$(OUT_DIR)/metadata-%-$(METADATA_VERSION).test.json: $(INPUTS)
	$(VIRTUAL_PYTHON3) generate-mds.py $* --test > "$@"

get-info.json: get-info.py
	$(VIRTUAL_PYTHON3) get-info.py > "$@"
