build-cjs:
	rm -rf ./src/wasm-utils &&\
	wasm-pack build --target nodejs ./wasm-utils --out-dir ../src/wasm-utils &&\
	tsc -p tsconfig-cjs.json &&\
	cp -r ./src/wasm-utils ./dist/cjs;

build-esm:
	rm -rf ./src/wasm-utils &&\
	wasm-pack build --target bundler ./wasm-utils --out-dir ../src/wasm-utils &&\
	tsc &&\
	cp -r ./src/wasm-utils ./dist/esm;

build:
	make build-esm;
	make build-cjs;