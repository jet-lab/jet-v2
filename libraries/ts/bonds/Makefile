build-cjs:
	wasm-pack build --target nodejs ./wasm-utils --out-dir ../dist/cjs/wasm-utils;
build-esm:
	wasm-pack build --target bundler ./wasm-utils --out-dir ../dist/esm/wasm-utils;
	cp -r ./dist/esm/wasm-utils/* ./src/wasm-utils
build-wasm:
	make build-cjs;
	make build-esm;