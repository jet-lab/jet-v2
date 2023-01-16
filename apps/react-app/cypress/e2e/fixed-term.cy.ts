import { airdrop, borrow, deposit, loadPageAndCreateAccount } from '../support/actions';

describe('Fixed Term Market', () => {
  it("can get data from the API endpoint", () => {
    cy.visit("http://localhost:3002/health")
    cy.contains('database')
  })
  // it('creates a market maker account', () => {
  //   loadPageAndCreateAccount();
  // });

  // it('funds the market maker account', () => {
  //   airdrop('SOL', 'SOL');
  //   airdrop('USDC', 'USDC');
  //   airdrop('BTC', 'BTC');
  //   airdrop('USDT', 'USDT');
  //   deposit('SOL', 1);
  //   deposit('BTC', 1);
  //   deposit('USDT', 1);
  //   deposit('USDC', 50000);
  // });

  // it('can create one fixed rate lend order', () => {
  //   cy.wait(1000);
  //   const lendLink = cy.contains('.nav-link', 'Lend');
  //   lendLink.click();
  //   const amountInput = cy.get('.fixed-term .offer-loan .input-amount input');
  //   const interestInput = cy.get('.fixed-term .offer-loan .input-rate input');
  //   amountInput.click().type(`1000`);
  //   interestInput.click().type(`10`);
  //   const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
  //   submitButton.click();
  //   cy.contains('Your lend offer for 1000 USDC at 10% was created successfully');
  // });

  // it('can create a second fixed rate lend order', () => {
  //   const amountInput = cy.get('.fixed-term .offer-loan .input-amount input');
  //   const interestInput = cy.get('.fixed-term .offer-loan .input-rate input');
  //   amountInput.focus().clear();
  //   amountInput.click().type(`2000`);
  //   interestInput.focus().clear();
  //   interestInput.click().type(`10`);
  //   const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
  //   submitButton.click();
  //   cy.contains('Your lend offer for 2000 USDC at 10% was created successfully');
  // });

  // it('can create one fixed rate borrow order', () => {
  //   cy.wait(1000);
  //   const borrowLink = cy.contains('.nav-link', 'Borrow');
  //   borrowLink.click();
  //   const amountInput = cy.get('.fixed-term .request-loan .input-amount input');
  //   const interestInput = cy.get('.fixed-term .request-loan .input-rate input');
  //   amountInput.click().type(`1000`);
  //   interestInput.click().type(`5`);
  //   const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
  //   submitButton.click();
  //   cy.contains('Your borrow offer for 1000 USDC at 5% was created successfully');
  // });

  // it('can create a second fixed rate borrow order', () => {
  //   const amountInput = cy.get('.fixed-term .request-loan .input-amount input');
  //   const interestInput = cy.get('.fixed-term .request-loan .input-rate input');
  //   amountInput.focus().clear();
  //   amountInput.click().type(`2000`);
  //   interestInput.focus().clear();
  //   interestInput.click().type(`5`);
  //   const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
  //   submitButton.click();
  //   cy.contains('Your borrow offer for 2000 USDC at 5% was created successfully');
  // });

  // it('creates a market taker account', () => {
  //   loadPageAndCreateAccount();
  // });

  // it('funds the market taker account', () => {
  //   airdrop('SOL', 'SOL');
  //   airdrop('USDC', 'USDC');
  //   airdrop('BTC', 'BTC');
  //   airdrop('USDT', 'USDT');
  //   deposit('SOL', 1);
  //   deposit('BTC', 1);
  //   deposit('USDT', 1);
  //   deposit('USDC', 50000);
  // });

  // it('issues a lend now order', () => {
  //   cy.wait(1000);
  //   const lendLink = cy.contains('.nav-link', 'Lend');
  //   lendLink.click();
  //   const lendNow = cy.contains('lend now');
  //   lendNow.click();
  //   const amountInput = cy.get('.fixed-term .lend-now .input-amount input').should('not.be.disabled');
  //   amountInput.click()
  //   cy.wait(5000)
  //   amountInput.type(`1000`);
  //   const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
  //   submitButton.click();
  //   cy.contains('Your lend order for 1000 USDC was filled successfully');
  // });

  // it('issues a borrow now order', () => {
  //   cy.wait(1000);
  //   const borrowLink = cy.contains('.nav-link', 'Borrow');
  //   borrowLink.click();
  //   const borrowNowTab = cy.contains('borrow now');
  //   borrowNowTab.click();
  //   const amountInput = cy.get('.fixed-term .borrow-now .input-amount input').should('not.be.disabled');
  //   amountInput.click()
  //   cy.wait(5000)
  //   amountInput.type(`100`);
  //   const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
  //   submitButton.click();
  //   cy.contains('Your borrow order for 100 USDC was filled successfully');
  // });

  // it('can perform a borrow on a pool after a position on a fixed market could have gone stale', () => {
  //   cy.wait(31000); // current stale time is 30 seconds
  //   const poolsLink = cy.contains('.nav-link', 'Pools');
  //   poolsLink.click();
  //   borrow('USDC', 10);
  // });
});
