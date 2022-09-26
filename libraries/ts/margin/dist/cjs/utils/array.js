"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.chunks = void 0;
function chunks(chunkSize, array) {
    let chunks = [];
    for (let i = 0; i < array.length; i += chunkSize) {
        chunks.push(array.slice(i, i + chunkSize));
    }
    return chunks;
}
exports.chunks = chunks;
//# sourceMappingURL=array.js.map