build-cjs:
<<<<<<< HEAD
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
=======
	wasm-pack build --target nodejs ./wasm-utils --out-dir ../dist/cjs/wasm-utils;
build-esm:
	wasm-pack build --target bundler ./wasm-utils --out-dir ../dist/esm/wasm-utils;
	cp -r ./dist/esm/wasm-utils ./src/wasm-utils
build-wasm:
	make build-cjs;
	make build-esm;
>>>>>>> 0c72a305 (build wasm separately for cjs and esm)
