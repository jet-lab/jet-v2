let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextDecoder } = require(`util`);

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

function getObject(idx) { return heap[idx]; }

let heap_next = heap.length;

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0 = new Uint8Array();

function getUint8Memory0() {
    if (cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}
/**
* @returns {bigint}
*/
module.exports.MAX_U64 = function() {
    const ret = wasm.MAX_U64();
    return BigInt.asUintN(64, ret);
};

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}
/**
* Converts a buffer from an orderbook side into an array of orders on the book
*
* Params:
*
* `slab_bytes`: a `UInt8Array` from the AccountInfo data
* @param {Uint8Array} slab_bytes
* @returns {Array<any>}
*/
module.exports.get_orders_from_slab = function(slab_bytes) {
    const ptr0 = passArray8ToWasm0(slab_bytes, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.get_orders_from_slab(ptr0, len0);
    return takeObject(ret);
};

/**
* Given a base quanity and fixed-point 32 price value, calculate the quote
* @param {bigint} base
* @param {bigint} price
* @returns {bigint}
*/
module.exports.base_to_quote = function(base, price) {
    const ret = wasm.base_to_quote(base, price);
    return BigInt.asUintN(64, ret);
};

/**
* Given a base quanity and fixed-point 32 price value, calculate the quote
* @param {bigint} quote
* @param {bigint} price
* @returns {bigint}
*/
module.exports.quote_to_base = function(quote, price) {
    const ret = wasm.quote_to_base(quote, price);
    return BigInt.asUintN(64, ret);
};

/**
* Given a fixed-point 32 value, convert to decimal representation
* @param {bigint} fp
* @returns {bigint}
*/
module.exports.fixed_point_to_decimal = function(fp) {
    const ret = wasm.fixed_point_to_decimal(fp);
    return BigInt.asUintN(64, ret);
};

/**
* Given an interest rate and bond duration, calculates a price
*
* NOTE: price is returned in fixed point 32 format
* @param {number} _interest_rate
* @param {bigint} _duration
* @returns {bigint}
*/
module.exports.rate_to_price = function(_interest_rate, _duration) {
    const ret = wasm.rate_to_price(_interest_rate, _duration);
    return BigInt.asUintN(64, ret);
};

/**
* Given a price and bond duration, calculates an interest rate
*
* NOTE: price is expected to be in fixed point 32 format
* @param {bigint} _price
* @param {bigint} _duration
* @returns {bigint}
*/
module.exports.price_to_rate = function(_price, _duration) {
    const ret = wasm.price_to_rate(_price, _duration);
    return BigInt.asUintN(64, ret);
};

/**
* Converts a fixed point 32 price to an f64 for UI display
* @param {bigint} _price
* @returns {number}
*/
module.exports.ui_price = function(_price) {
    const ret = wasm.ui_price(_price);
    return ret;
};

/**
* @param {bigint} amount
* @param {bigint} interest_rate
* @returns {OrderAmount}
*/
module.exports.build_order_amount_deprecated = function(amount, interest_rate) {
    const ret = wasm.build_order_amount_deprecated(amount, interest_rate);
    return OrderAmount.__wrap(ret);
};

/**
* For calculation of an implied limit price given to the bonds orderbook
*
* Base is principal plus interest
*
* Quote is principal
*
* Example usage
* ```ignore
* // 100 token lamports at 10% interest
* let price = calculate_implied_price(110, 100);
* ```
* @param {bigint} base
* @param {bigint} quote
* @returns {bigint}
*/
module.exports.calculate_implied_price = function(base, quote) {
    const ret = wasm.calculate_implied_price(base, quote);
    return BigInt.asUintN(64, ret);
};

/**
*/
class Order {

    static __wrap(ptr) {
        const obj = Object.create(Order.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_order_free(ptr);
    }
    /**
    * Pukbey of the signer allowed to make changes to this order
    * @returns {Uint8Array}
    */
    get owner() {
        const ret = wasm.__wbg_get_order_owner(this.ptr);
        return takeObject(ret);
    }
    /**
    * Pukbey of the signer allowed to make changes to this order
    * @param {Uint8Array} arg0
    */
    set owner(arg0) {
        wasm.__wbg_set_order_owner(this.ptr, addHeapObject(arg0));
    }
    /**
    * order tag used to track pdas related to this order
    * 16 byte hash derived
    * @returns {Uint8Array}
    */
    get order_tag() {
        const ret = wasm.__wbg_get_order_order_tag(this.ptr);
        return takeObject(ret);
    }
    /**
    * order tag used to track pdas related to this order
    * 16 byte hash derived
    * @param {Uint8Array} arg0
    */
    set order_tag(arg0) {
        wasm.__wbg_set_order_order_tag(this.ptr, addHeapObject(arg0));
    }
    /**
    * The orderId as found on the orderbook
    * a u128, used for cancel order instructions
    * @returns {Uint8Array}
    */
    get order_id() {
        const ret = wasm.__wbg_get_order_order_id(this.ptr);
        return takeObject(ret);
    }
    /**
    * The orderId as found on the orderbook
    * a u128, used for cancel order instructions
    * @param {Uint8Array} arg0
    */
    set order_id(arg0) {
        wasm.__wbg_set_order_order_id(this.ptr, addHeapObject(arg0));
    }
    /**
    * Total bond ticket worth of the order
    * @returns {bigint}
    */
    get base_size() {
        const ret = wasm.__wbg_get_order_base_size(this.ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
    * Total bond ticket worth of the order
    * @param {bigint} arg0
    */
    set base_size(arg0) {
        wasm.__wbg_set_order_base_size(this.ptr, arg0);
    }
    /**
    * Total underlying token worth of the order
    * @returns {bigint}
    */
    get quote_size() {
        const ret = wasm.__wbg_get_order_quote_size(this.ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
    * Total underlying token worth of the order
    * @param {bigint} arg0
    */
    set quote_size(arg0) {
        wasm.__wbg_set_order_quote_size(this.ptr, arg0);
    }
    /**
    * Fixed point 32 representation of the price
    * @returns {bigint}
    */
    get limit_price() {
        const ret = wasm.__wbg_get_order_limit_price(this.ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
    * Fixed point 32 representation of the price
    * @param {bigint} arg0
    */
    set limit_price(arg0) {
        wasm.__wbg_set_order_limit_price(this.ptr, arg0);
    }
}
module.exports.Order = Order;
/**
* Represents a 3-tuple of order parameters, returned when calculating order parameters from a given
* amount and interest rate
*/
class OrderAmount {

    static __wrap(ptr) {
        const obj = Object.create(OrderAmount.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_orderamount_free(ptr);
    }
    /**
    * max base quantity for an order
    * @returns {bigint}
    */
    get base() {
        const ret = wasm.__wbg_get_order_base_size(this.ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
    * max base quantity for an order
    * @param {bigint} arg0
    */
    set base(arg0) {
        wasm.__wbg_set_order_base_size(this.ptr, arg0);
    }
    /**
    * max quote quantity for an order
    * @returns {bigint}
    */
    get quote() {
        const ret = wasm.__wbg_get_order_quote_size(this.ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
    * max quote quantity for an order
    * @param {bigint} arg0
    */
    set quote(arg0) {
        wasm.__wbg_set_order_quote_size(this.ptr, arg0);
    }
    /**
    * fixed-point 32 limit price value
    * @returns {bigint}
    */
    get price() {
        const ret = wasm.__wbg_get_order_limit_price(this.ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
    * fixed-point 32 limit price value
    * @param {bigint} arg0
    */
    set price(arg0) {
        wasm.__wbg_set_order_limit_price(this.ptr, arg0);
    }
}
module.exports.OrderAmount = OrderAmount;

module.exports.__wbindgen_object_drop_ref = function(arg0) {
    takeObject(arg0);
};

module.exports.__wbindgen_object_clone_ref = function(arg0) {
    const ret = getObject(arg0);
    return addHeapObject(ret);
};

module.exports.__wbg_order_new = function(arg0) {
    const ret = Order.__wrap(arg0);
    return addHeapObject(ret);
};

module.exports.__wbg_new_1d9a920c6bfc44a8 = function() {
    const ret = new Array();
    return addHeapObject(ret);
};

module.exports.__wbg_push_740e4b286702d964 = function(arg0, arg1) {
    const ret = getObject(arg0).push(getObject(arg1));
    return ret;
};

module.exports.__wbg_buffer_3f3d764d4747d564 = function(arg0) {
    const ret = getObject(arg0).buffer;
    return addHeapObject(ret);
};

module.exports.__wbg_newwithbyteoffsetandlength_d9aa266703cb98be = function(arg0, arg1, arg2) {
    const ret = new Uint8Array(getObject(arg0), arg1 >>> 0, arg2 >>> 0);
    return addHeapObject(ret);
};

module.exports.__wbg_new_8c3f0052272a457a = function(arg0) {
    const ret = new Uint8Array(getObject(arg0));
    return addHeapObject(ret);
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

module.exports.__wbindgen_memory = function() {
    const ret = wasm.memory;
    return addHeapObject(ret);
};

const path = require('path').join(__dirname, 'wasm_utils_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

