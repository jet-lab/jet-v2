import { loadPageAndCreateAccount, airdrop, deposit, borrow, withdraw, repay } from '../support/actions';

describe('Main Flows', () => {
  it('Connects a new test wallet and creates an account', () => {
    // FIXME, remove this once adapters load correctly. currently v2 doesn't load the SERUM marketplace correctly this would stop the test
    cy.on('uncaught:exception', (err, runnable) => {
      console.log('***** ERROR ****');
    });
    loadPageAndCreateAccount();
  });

  it('Airdrop BTC and deposit collateral', () => {
    airdrop('BTC', 'Bitcoin');
    deposit('BTC', 2);
    //FIXME: Delays needed waiting for tx to complete
    cy.wait(5000);
  });

  it('Airdrop, deposit and withdraw USDC', () => {
    airdrop('USDC', 'USDC');
    deposit('USDC', 30);
    //FIXME: Delays needed waiting for tx to complete
    cy.wait(10000);
    withdraw('USDC', 29.5);
  });

  it('Borrow and repay SOL', () => {
    borrow('SOL', 3);
    //FIXME: Delays needed waiting for tx to complete
    cy.wait(10000);
    repay('SOL', 2.98);
  });
});
