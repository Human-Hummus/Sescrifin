clean:
	rm -r target/*

install:build
	cp ./sescrifin /bin/sescrifin
	@echo "If you're unable to run Sescrifin, you may need to add /bin to your PATH. To do this, run \"export PATH=\$$PATH:/bin\""
build:
	cargo build --release
	cp target/release/sescrifin ./sescrifin

docs:
	@echo "If this fails, you'll need to install Flaarc(https://github.com/Human-Hummus/Flaarc)"
	flaarc -i readme.flaarc -f markdown -o README.md
	flaarc -i readme.flaarc -f text -o src/help.txt

