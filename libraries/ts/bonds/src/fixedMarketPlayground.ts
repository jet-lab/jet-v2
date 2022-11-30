// //Assume jet margin account exists
// //Assume already have collateral
// //Borrow money and take it (no limit lend order)

// let walletKey = ""
// let marginAccountKey = ""
// let marketKey = ""
// let program = FixedMarketProgram
// let marginAccount = MarginAccount.load(marginAccountKey)

// let fixedMarket= FixedMarket.load(program, marketKey)
// let orderbook = fixedMarket.fetchOrderbook()

// let fixedUser = FixedUser.loadWithMarginAccount(/fixedMarket/, marginAccount)

// let autoroll_rate = "19.99"

// function requestBorrow(idOfUser, amount, rate, autorollRate){
//     //call placeOrder
// }

// function borrowNow(idOfUser, amount, autorollRate){
//     //call placeOrder
// }

// function offerLoan(idOfUser, amount, rate, autorollRate){
//    //call placeOrder
// }

// function lendNow(idOfUser, amount, autorollRate){
//     //call placeOrder

// }

// function placeOrder(idOfUser, side: OrderSide, price, size, type: OrderType, autorollPrice: number, autoStake: boolean){

// }

// enum OrderSide {
//     "bid",
//     "ask"
// }

// enum OrderType {
//     "postOnly",
//     "IOC",
// }

// fixedMarket.borrowNow(marginAccount.address, 192.00, 10.00)

// //Abstraction atop the orderbook, there's functionality to borrow and lend

// //WASM helpers

// toPrice(rate, tenor)

// toRate(price, tenor)
