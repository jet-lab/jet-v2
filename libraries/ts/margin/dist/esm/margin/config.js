import axios from "axios";
export const MARGIN_CONFIG_URL = "https://storage.googleapis.com/jet-app-config/config.json";
export async function getLatestConfig(cluster) {
    let response = await axios.get(MARGIN_CONFIG_URL);
    return (await response.data)[cluster];
}
//# sourceMappingURL=config.js.map