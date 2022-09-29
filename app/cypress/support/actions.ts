export const loadPageAndCreateAccount = () => {
  const url = Cypress.config().baseUrl;

  cy.visit(url);

  cy.contains('Connect Wallet').click();
  cy.contains('E2E').click();

  cy.contains('I understand and accept the risks').click();
  cy.contains('Enter Mainnet').click();

  cy.contains('Connected');
  cy.get('.nav-section .settings-btn').click();
  cy.contains('Devnet').click();
  cy.contains('Save Preferences').click();

  cy.contains('All Assets').click();
  airdrop('SOL', 'Solana');
  cy.contains('Create an account', { timeout: 5000 }).click();
  cy.contains('Create Account').click();
  cy.contains('Account created', { timeout: 10000 });
  cy.contains('ACCOUNT 1', { timeout: 10000 });
};

export const airdrop = (symbol: string, asset: string) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.contains('.ant-typography.pool-detail-header', `${asset}`);
  cy.contains('Airdrop').click();
  cy.contains('Airdrop successful', { timeout: 10000 });
  cy.contains(`${symbol} was successfully processed`, { timeout: 10000 });
};

export const deposit = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer button`).contains('Deposit', { timeout: 1000 }).click();
  const input = cy.get('.ant-modal-content input.ant-input', { timeout: 30000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').contains('Deposit').click();
  cy.contains('deposit successful', { timeout: 10000 });
  cy.wait(5000);
};

export const borrow = (symbol: string, amount: number, resetMaxState?: boolean) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer button`).contains('Borrow', { timeout: 1000 }).click();
  if (resetMaxState) {
    // Reset max trade values to simulate borrowing on existing account
    cy.get('[data-testid="reset-max-trade"]').click();
  }
  const input = cy.get('.ant-modal-content input.ant-input', { timeout: 10000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').contains('Borrow').click();
  cy.contains('borrow successful', { timeout: 10000 });
  cy.wait(5000);
};

export const withdraw = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer button`).contains('Withdraw', { timeout: 1000 }).click();
  const input = cy.get('.ant-modal-content input.ant-input', { timeout: 30000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').contains('Withdraw').click();
  cy.contains('withdraw successful', { timeout: 10000 });
  cy.wait(5000);
};

export const repay = (symbol: string, amount: number, fromDeposit: boolean) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer button`).contains('Repay', { timeout: 1000 }).click();
  const input = cy.get('.ant-modal-content input.ant-input', { timeout: 30000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  const isRepayFromWallet = cy.get('button.ant-switch').should('have.class', 'ant-switch-checked');
  if (fromDeposit && isRepayFromWallet) {
    isRepayFromWallet.click();
  }
  cy.contains('Repay From Wallet')
    .siblings()
    .should(fromDeposit ? 'not.have.class' : 'have.class', 'ant-switch-checked');
  cy.get('.ant-modal-body button.ant-btn').contains('Repay').click();
  cy.contains(`${symbol} was successfully processed.`, { timeout: 10000 });
  cy.wait(5000);
};

export const swap = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.navbar.nav-link`, { timeout: 10000 });
  cy.contains('Swaps', { timeout: 10000 }).click();
  const input = cy.get('.order-entry input.ant-input:first-of-type', { timeout: 10000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.order-entry-footer button.ant-btn').contains('Swap').click();
  cy.contains('swap successful', { timeout: 10000 });
};
