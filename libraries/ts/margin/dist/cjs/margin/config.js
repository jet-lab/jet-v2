"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getLatestConfig = exports.MARGIN_CONFIG_URL = void 0;
const axios_1 = __importDefault(require("axios"));
exports.MARGIN_CONFIG_URL = "https://storage.googleapis.com/jet-app-config/config.json";
async function getLatestConfig(cluster) {
    let response = await axios_1.default.get(exports.MARGIN_CONFIG_URL);
    return (await response.data)[cluster];
}
exports.getLatestConfig = getLatestConfig;
//# sourceMappingURL=config.js.map