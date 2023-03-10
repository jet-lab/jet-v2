/*
 * Copyright (C) 2022 JET PROTOCOL HOLDINGS, LLC.
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Affero General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU Affero General Public License for more details.
 *
 * You should have received a copy of the GNU Affero General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

export * from "./margin"
export * from "./token"
export * from "./types"
export * from "./utils"
export * from "./fixed-term"
export * from "./wasm"

// FIXME This function is supposed to be execute automatically
// in each thread when loading the wasm module but, as far as
// I can tell, that isn't happening. It's idempotent anyway, so
// just force the issue here.
import { initModule } from "./wasm"
initModule()
