export const loadPageAndCreateAccount = () => {
  cy.visit('http://localhost:3000/');

  cy.contains('Connect Wallet').click();
  cy.contains('E2E').click();

  cy.contains('I understand and accept the risks').click();
  cy.contains('Enter Mainnet').click();
  cy.get('.walkthrough-modal-close').click();

  cy.contains('Wallet Connected');
  cy.get('.nav-section .anticon-setting ').click();
  cy.contains('Devnet').click();
  cy.contains('Save Preferences').click();

  cy.contains('Pools').click();
  airdrop('SOL', 'SOL');
  cy.contains('Create an account', { timeout: 5000 }).click();
  cy.get('.ant-modal-content input.ant-input').type('My new test account');
  cy.contains('Create Account').click();
  cy.contains('Account created.', { timeout: 10000 });
};

export const airdrop = (symbol: string, asset: string) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.contains('.ant-typography.pool-detail-header', `${asset}`, { timeout: 10000 });
  cy.contains('Airdrop', { timeout: 30000 });
  cy.wait(1000);
  cy.contains('Airdrop').click();
  cy.contains('Airdrop successful', { timeout: 10000 });
  cy.contains('Airdrop successful', { timeout: 10000 });
  cy.contains(`${symbol} was successfully processed`, { timeout: 10000 });
};

export const deposit = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer.view-element-item`, { timeout: 10000 });
  cy.contains('Deposit', { timeout: 10000 }).click();
  const input = cy.get('.ant-modal-content input.ant-input', { timeout: 10000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').contains('Deposit').click();
  cy.contains('deposit successful', { timeout: 10000 });
};

export const borrow = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer.view-element-item`, { timeout: 10000 });
  cy.contains('Borrow', { timeout: 10000 }).click();
  const input = cy.get('.ant-modal-content input.ant-input', { timeout: 10000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').contains('Borrow').click();
  cy.contains('borrow successful', { timeout: 10000 });
};

export const withdraw = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer.view-element-item`, { timeout: 10000 });
  cy.contains('Withdraw', { timeout: 10000 }).click();
  const input = cy.get('.ant-modal-content input.ant-input', { timeout: 10000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').contains('Withdraw').click();
  cy.contains('withdraw successful', { timeout: 10000 });
};

export const assertWithdrawnAndRepay = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer.view-element-item`, { timeout: 10000 });
  cy.contains('Repay', { timeout: 10000 }).click();
  const walletBalance = cy.get('.ant-modal-content div.wallet-balance div.ant-typography-secondary', {
    timeout: 10000
  });
  walletBalance.should('be.at.least', amount);
  repay(symbol, amount);
};

export const repay = (symbol: string, amount: number) => {
  cy.get(`.${symbol}-pools-table-row`, { timeout: 30000 }).click();
  cy.get(`.account-snapshot-footer.view-element-item`, { timeout: 10000 });
  cy.contains('Repay', { timeout: 10000 }).click();
  const input = cy.get('.ant-modal-content input.ant-input', { timeout: 10000 }).should('not.be.disabled');
  input.click().type(`${amount}`);
  cy.get('.ant-modal-body button.ant-btn').contains('Repay').click();
  cy.contains('repay successful', { timeout: 10000 });
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
