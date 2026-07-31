IMAGE := alps-lake.jpg
OUTPUT := alps-lake-everforest-dark-medium.png

.PHONY: run install test readme-images clean help

run: $(OUTPUT)
	loupe $(OUTPUT) >/dev/null 2>&1 &

$(OUTPUT): $(IMAGE)
	cargo run --release -- --theme everforest-dark-medium $(IMAGE)

install:
	cargo install --path .

test:
	cargo test

readme-images:
	./scripts/regenerate-readme-images.sh

clean:
	rm -f $(OUTPUT)

help:
	@printf '%s\n' 'make run           recolor alps-lake.jpg and open it with loupe' 'make install       install paletteer with cargo' 'make test          run tests' 'make readme-images safely regenerate README previews' 'make clean         remove the generated preview'
