"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.getTimestamp = void 0;
const anchor_1 = require("@project-serum/anchor");
function getTimestamp() {
    return new anchor_1.BN(Math.floor(Date.now() / 1000));
}
exports.getTimestamp = getTimestamp;
//# sourceMappingURL=timestamp.js.map