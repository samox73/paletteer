IMAGE := alps-lake.jpg
OUTPUT := alps-lake-everforest-dark-medium.png

.PHONY: run install test readme-images benchmark clean help

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

benchmark:
	cargo run --release --example benchmark
	cargo run --release -- --theme everforest-dark-medium -o benchmark.png
	loupe benchmark.png benchmark-everforest-dark-medium.png >/dev/null 2>&1 &

clean:
	rm -f $(OUTPUT) benchmark.png benchmark-everforest-dark-medium.png

help:
	@printf '%s\n' 'make run           recolor alps-lake.jpg and open it with loupe' 'make install       install paletteer with cargo' 'make test          run tests' 'make readme-images safely regenerate README previews' 'make benchmark     recolor the gradient chart and open both with loupe' 'make clean         remove the generated preview'
