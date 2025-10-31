# Makefile

# Name of the image to build/run
IMAGE=delivery-scheduling

# Directory paths
RUST_DIR=rust
INPUT_DIR=$(PWD)/inputs
OUTPUT_DIR=$(PWD)/output

.PHONY: build run clean rebuild shell

build:
	docker build -t $(IMAGE) $(RUST_DIR)

run:
	docker run --rm \
		-v $(INPUT_DIR):/inputs \
		-v $(OUTPUT_DIR):/output \
		$(IMAGE)

rebuild: clean build run

shell:
	docker run --rm -it \
		-v $(INPUT_DIR):/inputs \
		-v $(OUTPUT_DIR):/output \
		$(IMAGE) /bin/bash

test:
	docker run --rm -it \
		-v $(CURDIR)/$(RUST_DIR):/usr/src \
		-w /usr/src \
		rust:latest \
		cargo test

clean:
	rm -f $(OUTPUT_DIR)/*.csv



