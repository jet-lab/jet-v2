import { loadPageAndFundSol, airdrop, deposit, borrow, withdraw, repay, createAccount } from '../support/actions';

describe('Main Flows', () => {
  it('Connects a new test wallet and creates an account', () => {
    loadPageAndFundSol();
    createAccount();
  });

  it('Airdrop USDC and deposit collateral', () => {
    airdrop('USDC', 'USDC');
    deposit('USDC', 10);
  });

  it('Deposit and withdraw SOL', () => {
    deposit('SOL', 10);
    withdraw('SOL', 0.3);
  });

  it('Borrow and repay SOL from existing margin account', () => {
    deposit('SOL', 1);
    borrow('SOL', 0.3, true);
    repay('SOL', 0.3);
  });
});

describe('Error Flows', () => {
  it('Connects a new test wallet and creates an account', () => {
    loadPageAndFundSol();
    createAccount();
  });

  it('All deposits should be disabled, because SOL in wallet is under fees buffer amount', () => {
    // There are multiple transactions issued to create a margin account.
    // By the time this test runs, the wallet's SOL balance is still before
    // these transactions, so the amount deposited is incorrect, causing a failure.
    // The delay addresses this issue.
    cy.wait(5000);
    cy.get('.SOL-pools-table-row').click();
    cy.contains('button', 'Deposit').should('not.be.disabled').click();
    cy.contains('Max').click();
    cy.contains('.ant-modal-body button', 'Deposit').should('not.be.disabled').click();
    cy.contains('deposit successful');

    cy.contains('button', 'Deposit').click();
    cy.get('.ant-modal-content input.ant-input').should('be.disabled');
    cy.contains('Please make sure you have a buffer of at least');
    cy.get('button.ant-modal-close').click();
  });
});
