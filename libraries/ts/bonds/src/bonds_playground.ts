//Assume jet margin account exists
//Assume already have collateral
//Borrow money and take it (no limit lend order)




let walletKey = ""
let marginAccountKey = ""
let marketKey = ""
let program = BondProgram
let marginAccount = MarginAccount.load(marginAccountKey)

let bondMarket= BondMarket.load(program, marketKey)
let orderbook = bondMarket.fetchOrderbook()

let bondUser = BondsUser.loadWithMarginAccount(/bondMarket/, marginAccount)




let autoroll_rate = "19.99"



function requestBorrow(id_of_user, amount, rate, autorollRate){
    //call placeOrder
}

function borrowNow(id_of_user, amount, autorollRate){
    //call placeOrder
}


function offerLoan(id_of_user, amount, rate, autorollRate){
   //call placeOrder
}


function lendNow(id_of_user, amount, autorollRate){
    //call placeOrder

}

function placeOrder(idOfUser, side: OrderSide, price, size, type: OrderType, autorollPrice: number, autoStake: boolean){


}

enum OrderSide {
    "bid",
    "ask"
}

enum OrderType {
    "postOnly",
    "IOC",
}


bondMarket.borrowNow(marginAccount.address, 192.00, 10.00)







//Abstraction atop the orderbook, there's functionality to borrow and lend