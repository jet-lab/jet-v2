/* tslint:disable */
/* eslint-disable */
/**
* @returns {bigint}
*/
export function MAX_U64(): bigint;
/**
* Converts a buffer from an orderbook side into an array of orders on the book
*
* Params:
*
* `slab_bytes`: a `UInt8Array` from the AccountInfo data
* @param {Uint8Array} slab_bytes
* @returns {Array<any>}
*/
export function get_orders_from_slab(slab_bytes: Uint8Array): Array<any>;
/**
* Given a base quanity and fixed-point 32 price value, calculate the quote
* @param {bigint} base
* @param {bigint} price
* @returns {bigint}
*/
export function base_to_quote(base: bigint, price: bigint): bigint;
/**
* Given a base quanity and fixed-point 32 price value, calculate the quote
* @param {bigint} quote
* @param {bigint} price
* @returns {bigint}
*/
export function quote_to_base(quote: bigint, price: bigint): bigint;
/**
* Given a fixed-point 32 value, convert to decimal representation
* @param {bigint} fp
* @returns {bigint}
*/
export function fixed_point_to_decimal(fp: bigint): bigint;
/**
* Given an interest rate and bond duration, calculates a price
*
* NOTE: price is returned in fixed point 32 format
* @param {number} _interest_rate
* @param {bigint} _duration
* @returns {bigint}
*/
export function rate_to_price(_interest_rate: number, _duration: bigint): bigint;
/**
* Given a price and bond duration, calculates an interest rate
*
* NOTE: price is expected to be in fixed point 32 format
* @param {bigint} _price
* @param {bigint} _duration
* @returns {bigint}
*/
export function price_to_rate(_price: bigint, _duration: bigint): bigint;
/**
* Converts a fixed point 32 price to an f64 for UI display
* @param {bigint} _price
* @returns {number}
*/
export function ui_price(_price: bigint): number;
/**
* @param {bigint} amount
* @param {bigint} interest_rate
* @returns {OrderAmount}
*/
export function build_order_amount_deprecated(amount: bigint, interest_rate: bigint): OrderAmount;
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
export function calculate_implied_price(base: bigint, quote: bigint): bigint;
/**
*/
export class Order {
  free(): void;
/**
* Total bond ticket worth of the order
*/
  base_size: bigint;
/**
* Fixed point 32 representation of the price
*/
  limit_price: bigint;
/**
* The orderId as found on the orderbook
* a u128, used for cancel order instructions
*/
  order_id: Uint8Array;
/**
* order tag used to track pdas related to this order
* 16 byte hash derived
*/
  order_tag: Uint8Array;
/**
* Pukbey of the signer allowed to make changes to this order
*/
  owner: Uint8Array;
/**
* Total underlying token worth of the order
*/
  quote_size: bigint;
}
/**
* Represents a 3-tuple of order parameters, returned when calculating order parameters from a given
* amount and interest rate
*/
export class OrderAmount {
  free(): void;
/**
* max base quantity for an order
*/
  base: bigint;
/**
* fixed-point 32 limit price value
*/
  price: bigint;
/**
* max quote quantity for an order
*/
  quote: bigint;
}
