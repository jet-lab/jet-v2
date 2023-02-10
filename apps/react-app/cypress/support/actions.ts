import { formatWithCommas } from './utils';

export const connectWallet = () => {
  cy.contains('CONNECT').click();
  cy.contains('E2E').click();
  cy.contains('Connected');
};

export const loadPageAndCreateAccount = (path?: string) => {
  cy.clearLocalStorage();
  const url = path ? path : Cypress.config().baseUrl;

  cy.visit(url);
  cy.get('.nav-section .settings-btn').click();
  cy.contains('Localnet').click();
  cy.contains('Solana Explorer').click();
  cy.contains('Save Preferences').click();
  connectWallet();
  airdrop('SOL', 'SOL');

  cy.contains('Create an account').as('createAccountBtn');
  cy.get('@createAccountBtn').should('be.visible');
  cy.get('@createAccountBtn').should('not.be.disabled');
  cy.get('@createAccountBtn').click();

  cy.contains('New Account');

  cy.contains('Create Account').as('createAccountAction');
  cy.get('@createAccountAction').should('be.visible');
  cy.get('@createAccountAction').should('not.be.disabled');
  cy.get('@createAccountAction').click();
  cy.contains('Account created');
  cy.contains('Account 1');
};

export const airdrop = (symbol: string, asset: string) => {
  cy.get(`.${symbol}-pools-table-row`).click();
  cy.contains('.ant-typography.pool-detail-header', `${asset}`);
  cy.contains('Airdrop').click();
  cy.contains('Airdrop successful');
  cy.contains(`${symbol} was successfully processed`);
};

export const deposit = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`).click();
  cy.contains('button', 'Deposit').should('not.be.disabled').click();
  cy.get('.ant-modal-content input.ant-input').as('input');
  cy.get('@input').should('not.be.disabled');
  cy.get('@input').type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').should('not.be.disabled').contains('Deposit').click();
  cy.contains(`Your deposit of ${formatWithCommas(amount)} ${symbol} was successfully processed.`);
};

export const borrow = (symbol: string, amount: number, resetMaxState?: boolean) => {
  cy.get(`.${symbol}-pools-table-row`).click();
  cy.get(`.account-snapshot-footer button`).contains('Borrow').click();
  if (resetMaxState) {
    // Reset max trade values to simulate borrowing on existing account
    cy.get('[data-testid="reset-max-trade"]').click();
  }
  cy.wait(500);
  cy.get('.ant-modal-content input.ant-input').as('input');
  cy.get('@input').should('not.be.disabled');
  cy.get('@input').click();
  cy.get('@input').type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn.ant-btn-default.ant-btn-block').as('submitButton');
  cy.get('@submitButton').should('not.be.disabled');
  cy.get('@submitButton').click();
  cy.contains('borrow successful');
};

export const withdraw = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`).click();
  cy.get(`.account-snapshot-footer button`).contains('Withdraw').click();
  cy.get('.ant-modal-content input.ant-input').as('input');
  cy.get('@input').should('not.be.disabled');
  cy.get('@input').click();
  cy.get('@input').type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').should('not.be.disabled').contains('Withdraw').click();
  cy.contains('withdraw successful');
};

export const repay = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`).click();
  cy.get(`.account-snapshot-footer button`).contains('Repay').click();
  cy.get('.ant-modal-content input.ant-input').as('input');
  cy.get('@input').should('not.be.disabled');
  cy.get('@input').click();
  cy.get('@input').type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').should('not.be.disabled').contains('Repay').click();
  cy.contains(`${symbol} was successfully processed.`);
};

export const swap = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`).click();
  cy.get(`.navbar.nav-link`);
  cy.contains('Swaps').click();
  const input = cy.get('.order-entry input.ant-input:first-of-type').should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.order-entry-footer button.ant-btn').should('not.be.disabled').contains('Swap').click();
  cy.contains('swap successful');
};
