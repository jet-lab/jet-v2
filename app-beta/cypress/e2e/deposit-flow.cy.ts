import 'cypress-wait-until';

describe('deposit-flow', () => {
  it('tests deposit-flow', () => {
    cy.visit('http://localhost:3000/');
    cy.get('[data-testid=connect-wallet-btn] > span').click();
    cy.get('[data-testid=connect-wallet-E2E]').click();
    cy.contains('[title=Disconnect]', 'CONNECTED', { timeout: 30000 });
    cy.get('[data-testid=Solana-deposit]').click();
    cy.wait(1000);
    cy.get('[data-testid=airdrop-Solana]').click();
    cy.wait(1000);
    cy.contains('[data-testid=Solana-balance]', '1', { timeout: 30000 });
    cy.get('[data-testid=jet-trade-input]').click();
    cy.get('[data-testid=jet-trade-input]').type('0.5');
    cy.get('[data-testid=jet-trade-button]').click();
    cy.contains('.user-collateral-value', '0.', { timeout: 30000 });
  });
});
