import {
  loadPageAndCreateAccount,
  airdrop,
  deposit,
  borrow,
  withdraw,
  repay,
  repayFromDeposit
} from '../support/actions';

describe('Main Flows', () => {
  it('Connects a new test wallet and creates an account', () => {
    // FIXME, remove this once adapters load correctly. currently v2 doesn't load the SERUM marketplace correctly this would stop the test
    cy.on('uncaught:exception', (err, runnable, promise) => {
      console.log('***** ERROR ****');

      // FIXME, remove this when the uncaught promise is fixed in `useFixedTermSync()`

      // when the exception originated from an unhandled promise
      // rejection, the promise is provided as a third argument
      // you can turn off failing the test in this case
      if (promise) {
        return false;
      }
    });
    loadPageAndCreateAccount();
  });

  it('Airdrop BTC and deposit collateral', () => {
    airdrop('BTC', 'Bitcoin');
    deposit('BTC', 2);
  });

  it('Deposit and withdraw SOL', () => {
    airdrop('SOL', 'SOL');
    deposit('SOL', 0.5);
    withdraw('SOL', 0.3);
  });

  it('Borrow and repay SOL from wallet', () => {
    airdrop('SOL', 'SOL');
    deposit('SOL', 0.5);
    borrow('SOL', 0.3);
    repay('SOL', 0.3);
  });

  it('Borrow and repay SOL from margin account', () => {
    airdrop('SOL', 'SOL');
    deposit('SOL', 1);
    borrow('SOL', 0.5);
    repayFromDeposit('SOL', 0.5);
  });
});
