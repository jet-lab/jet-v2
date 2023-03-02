import { airdrop, borrow, deposit, loadPageAndFundSol, createAccount } from '../support/actions';

describe('Fixed Term Market', () => {
  it('can get data from the API endpoint', () => {
    const res = cy.request('http://localhost:3002/health').as('status');
    cy.get('@status').should((response: any) => {
      expect(response.body).to.have.property('database', 'Ready');
      expect(response).to.have.property('headers');
      expect(response).to.have.property('duration');
    });
  });

  it('loads the page', () => {
    loadPageAndFundSol();
  });

  it('creates a lender account', () => {
    // Account 1 = Lender
    createAccount();
  });

  it('funds the lender account', () => {
    airdrop('SOL', 'SOL');
    airdrop('USDC', 'USDC');
    deposit('SOL', 1);
    deposit('USDC', 50000);
  });

  it('creates a borrower account', () => {
    // Account 2 = Borrower
    createAccount();
  });

  it('funds the borrower account', () => {
    airdrop('SOL', 'SOL');
    airdrop('USDC', 'USDC');
    deposit('SOL', 1);
    deposit('USDC', 50000);
  });

  it('selects the lender account', () => {
    cy.contains('ACCOUNT 1').as('lenderAccount');
    cy.get('@lenderAccount').click();
  });

  it('can create one fixed rate lend order', () => {
    const lendLink = cy.contains('.nav-link', 'Lend');
    lendLink.click();
    const amountInput = cy.get('.fixed-term .offer-loan .input-amount input');
    const interestInput = cy.get('.fixed-term .offer-loan .input-rate input');
    amountInput.click().type(`1000`);
    interestInput.click().type(`10`);
    const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
    submitButton.click();
    cy.contains('Your lend offer for 1000 USDC at 10% was created successfully');
  });

  it('can create a second fixed rate lend order', () => {
    const amountInput = cy.get('.fixed-term .offer-loan .input-amount input');
    const interestInput = cy.get('.fixed-term .offer-loan .input-rate input');
    amountInput.focus().clear();
    amountInput.click().type(`2000`);
    interestInput.focus().clear();
    interestInput.click().type(`10`);
    const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
    submitButton.click();
    cy.contains('Your lend offer for 2000 USDC at 10% was created successfully');
  });

  it('selects the borrower account', () => {
    cy.contains('ACCOUNT 2').as('borrowerAccount');
    cy.get('@borrowerAccount').click();
  });
  it('can create one fixed rate borrow order', () => {
    const borrowLink = cy.contains('.nav-link', 'Borrow');
    borrowLink.click();
    const amountInput = cy.get('.fixed-term .request-loan .input-amount input');
    const interestInput = cy.get('.fixed-term .request-loan .input-rate input');
    amountInput.click().type(`1000`);
    interestInput.click().type(`5`);
    const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
    submitButton.click();
    cy.contains('Your borrow offer for 1000 USDC at 5% was created successfully');
  });

  it('can create a second fixed rate borrow order', () => {
    const amountInput = cy.get('.fixed-term .request-loan .input-amount input');
    const interestInput = cy.get('.fixed-term .request-loan .input-rate input');
    amountInput.focus().clear();
    amountInput.click().type(`2000`);
    interestInput.focus().clear();
    interestInput.click().type(`5`);
    const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
    submitButton.click();
    cy.contains('Your borrow offer for 2000 USDC at 5% was created successfully');
  });

  it('issues a lend now order', () => {
    cy.contains('ACCOUNT 1').as('lenderAccount');
    cy.get('@lenderAccount').click();
    const lendLink = cy.contains('.nav-link', 'Lend');
    lendLink.click();
    const lendNow = cy.contains('lend now');
    lendNow.click();
    const amountInput = cy.get('.fixed-term .lend-now .input-amount input').should('not.be.disabled');
    amountInput.click();
    amountInput.type(`1000`);
    const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
    submitButton.click();
    cy.contains('Your lend order for 1000 USDC was filled successfully');
  });

  it('issues a borrow now order', () => {
    cy.contains('ACCOUNT 2').as('borrowerAccount');
    cy.get('@borrowerAccount').click();
    const borrowLink = cy.contains('.nav-link', 'Borrow');
    borrowLink.click();
    const borrowNowTab = cy.contains('borrow now');
    borrowNowTab.click();
    const amountInput = cy.get('.fixed-term .borrow-now .input-amount input').should('not.be.disabled');
    amountInput.click();
    amountInput.type(`100`);
    const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
    submitButton.click();
    cy.contains('Your borrow order for 100 USDC was filled successfully');
  });

  it('can cancel an outstanding order', () => {
    cy.contains('ACCOUNT 1').as('lenderAccount');
    cy.get('@lenderAccount').click();
    cy.get('.debt-detail tr .anticon-close').first().click();
    cy.contains('Order Cancelled');
  });

  it('can repay and outstanding borrow', () => {
    // Switching accounts back and forth to cause a refresh
    // TODO: Ugly, update when websocket is in
    cy.contains('ACCOUNT 1').as('lenderAccount');
    cy.get('@lenderAccount').click();
    cy.contains('ACCOUNT 2').as('borrowerAccount');
    cy.get('@borrowerAccount').click();
    cy.contains('You owe');
    const repayInput = cy.get('.assets-to-settle input').should('not.be.disabled');
    repayInput.click();
    repayInput.type('110');
    const repayButton = cy.contains('Repay Now');
    repayButton.click();
    cy.contains('Repay Successful');
  });
});
