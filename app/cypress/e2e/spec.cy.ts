import { loadPageAndCreateAccount, airdrop, deposit, borrow, withdraw, repay } from '../support/actions';

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
    deposit('BTC', 1);
  });

  it('Deposit and withdraw SOL', () => {
    deposit('SOL', 0.5);
    withdraw('SOL', 0.3);
  });

  it('Deposit SOL to under fees buffer in wallet', () => {
    deposit('SOL', 0.9);
    cy.get(`.USDC-pools-table-row`, { timeout: 30000 }).click();
    cy.get(`.account-snapshot-footer button`).contains('Deposit', { timeout: 1000 }).click();
    cy.get('.ant-modal-content input.ant-input', { timeout: 30000 }).should('be.disabled');
    cy.contains(`You don't have enough SOL in your wallet to cover transaction fees.`, { timeout: 10000 });

    cy.get('button.ant-modal-close', { timeout: 1000 }).click();

    cy.get(`.account-snapshot-footer button`).contains('Withdraw', { timeout: 1000 }).click();
    cy.get('.ant-modal-content input.ant-input', { timeout: 30000 }).should('be.disabled');
    cy.contains(`You don't have enough SOL in your wallet to cover transaction fees.`, { timeout: 10000 });

    cy.get('button.ant-modal-close', { timeout: 1000 }).click();

    cy.get(`.account-snapshot-footer button`).contains('Borrow', { timeout: 1000 }).click();
    cy.get('.ant-modal-content input.ant-input', { timeout: 30000 }).should('be.disabled');
    cy.contains(`You don't have enough SOL in your wallet to cover transaction fees.`, { timeout: 10000 });

    cy.get('button.ant-modal-close', { timeout: 1000 }).click();

    cy.get(`.account-snapshot-footer button`).contains('Repay', { timeout: 1000 }).click();
    cy.get('.ant-modal-content input.ant-input', { timeout: 30000 }).should('be.disabled');
    cy.contains(`You don't have enough SOL in your wallet to cover transaction fees.`, { timeout: 10000 });

    cy.get('button.ant-modal-close', { timeout: 1000 }).click();

    airdrop('SOL', 'SOL');
  });

  it('Borrow and repay SOL from wallet', () => {
    airdrop('SOL', 'SOL');
    deposit('SOL', 0.5);
    borrow('SOL', 0.3);
    repay('SOL', 0.3, false);
  });

  it('Borrow and repay SOL from existing margin account', () => {
    airdrop('SOL', 'SOL');
    deposit('SOL', 1);
    borrow('SOL', 0.5, true);
    repay('SOL', 0.5, true);
  });
});
