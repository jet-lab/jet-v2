build-cjs:
	rm -rf ./src/wasm-utils &&\
	wasm-pack build --target nodejs ./wasm-utils --out-dir ../src/wasm-utils &&\
	tsc -p tsconfig-cjs.json &&\
	mkdir -p ./dist/cjs && cp -r ./src/wasm-utils ./dist/cjs;

build-esm:
	rm -rf ./src/wasm-utils &&\
	wasm-pack build --target bundler ./wasm-utils --out-dir ../src/wasm-utils &&\
	tsc &&\
	mkdir -p ./dist/esm && cp -r ./src/wasm-utils ./dist/esm;

build:
	make build-cjs && \
	make build-esm;

build-js-only:
	tsc -p tsconfig-cjs.json && tsc;

watch:
	tsc -p tsconfig-cjs.json --watch && tsc --watch &\
	cargo watch --why --watch-when-idle -w 'libraries/ts/bonds/wasm-utils' -s 'make -C libraries/ts/bonds build';
