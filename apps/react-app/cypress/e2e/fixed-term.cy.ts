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

  describe('funds the lender account', () => {
    it('airdrops SOL', () => {
      airdrop('SOL', 'SOL');
    });
    it('airdrops USDC', () => {
      airdrop('USDC', 'USDC');
    });

    it('deposits SOL', () => {
      deposit('SOL', 1);
    });

    it('deposits USDC', () => {
      deposit('USDC', 50000);
    });
  });

  it('creates a borrower account', () => {
    // Account 2 = Borrower
    createAccount();
  });

  describe('funds the borrower account', () => {
    it('airdrops SOL', () => {
      airdrop('SOL', 'SOL');
    });
    it('airdrops USDC', () => {
      airdrop('USDC', 'USDC');
    });

    it('deposits SOL', () => {
      deposit('SOL', 1);
    });

    it('deposits USDC', () => {
      deposit('USDC', 50000);
    });
  });

  it('selects the lender account', () => {
    cy.contains('ACCOUNT 1').as('lenderAccount');
    cy.get('@lenderAccount').click();
  });

  describe('can create one fixed rate lend order', () => {
    const lendAmtString = '100';
    const interestString = '10';

    it('selects the offer loan tab', () => {
      const lendLink = cy.contains('.nav-link', 'Lend');
      lendLink.click();
      cy.contains('lend now'); // ensure we loaded the page fully
    });

    it('inputs the lend amount', () => {
      const amountInput = cy.get('.fixed-term .offer-loan .input-amount input');
      amountInput.click().type(lendAmtString);
    });

    it('inputs the interest rate', () => {
      const interestInput = cy.get('.fixed-term .offer-loan .input-rate input');
      interestInput.click().type(interestString);
    });
    it('clicks the button once enabled', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('successfully receives confirmation', () => {
      cy.contains(`Your lend offer for ${lendAmtString} USDC at ${interestString}% was created successfully`);
    });
  });

  describe('can create a second fixed rate lend order', () => {
    const lendAmtString = '100';
    const interestString = '10';

    it('selects the offer loan tab', () => {
      const lendLink = cy.contains('.nav-link', 'Lend');
      lendLink.click();
      cy.contains('lend now'); // ensure we loaded the page fully
    });

    it('inputs the lend amount', () => {
      const amountInput = cy.get('.fixed-term .offer-loan .input-amount input');
      amountInput.click().type(lendAmtString);
    });

    it('inputs the interest rate', () => {
      const interestInput = cy.get('.fixed-term .offer-loan .input-rate input');
      interestInput.click().type(interestString);
    });
    it('clicks the button once enabled', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('successfully receives confirmation', () => {
      cy.contains(`Your lend offer for ${lendAmtString} USDC at ${interestString}% was created successfully`);
    });
  });

  it('selects the borrower account', () => {
    cy.contains('ACCOUNT 2').as('borrowerAccount');
    cy.get('@borrowerAccount').click();
  });

  describe('can create one fixed rate borrow order', () => {
    const borrowAmtString = '100';
    const interestString = '10';

    it('selects the offer loan tab', () => {
      const lendLink = cy.contains('.nav-link', 'Borrow');
      lendLink.click();
      cy.contains('borrow now'); // ensure we loaded the page fully
    });

    it('inputs the lend amount', () => {
      const amountInput = cy.get('.fixed-term .request-loan .input-amount input');
      amountInput.click().type(borrowAmtString);
    });

    it('inputs the interest rate', () => {
      const interestInput = cy.get('.fixed-term .request-loan .input-rate input');
      interestInput.click().type(interestString);
    });
    it('clicks the button once enabled', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('successfully receives confirmation', () => {
      cy.contains(`Your borrow offer for ${borrowAmtString} USDC at ${interestString}% was created successfully`);
    });
  });

  describe('can create a second fixed rate borrow order', () => {
    const borrowAmtString = '100';
    const interestString = '10';

    it('selects the offer loan tab', () => {
      const lendLink = cy.contains('.nav-link', 'Borrow');
      lendLink.click();
      cy.contains('borrow now'); // ensure we loaded the page fully
    });

    it('inputs the lend amount', () => {
      const amountInput = cy.get('.fixed-term .request-loan .input-amount input');
      amountInput.click().type(borrowAmtString);
    });

    it('inputs the interest rate', () => {
      const interestInput = cy.get('.fixed-term .request-loan .input-rate input');
      interestInput.click().type(interestString);
    });
    it('clicks the button once enabled', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('successfully receives confirmation', () => {
      cy.contains(`Your borrow offer for ${borrowAmtString} USDC at ${interestString}% was created successfully`);
    });
  });

  it('selects the lender account', () => {
    cy.contains('ACCOUNT 1').as('lenderAccount');
    cy.get('@lenderAccount').click();
  });

  describe('issues a lend now order', () => {
    const lendAmtString = '100';

    it('selects the lend now tab', () => {
      const lendLink = cy.contains('.nav-link', 'Lend');
      lendLink.click();
      const lendNow = cy.contains('lend now');
      lendNow.click();
      cy.contains('Lend 1 day USDC');
    });

    it('enters the amount', () => {
      const amountInput = cy.get('.fixed-term .lend-now .input-amount input').should('not.be.disabled');
      amountInput.click();
      amountInput.type(lendAmtString);
      amountInput.clear();
    });

    it('submits the order', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('received the correct notification', () => {
      cy.contains(`Your lend order for ${lendAmtString} USDC was filled successfully`);
    });
  });

  it('selects the borrower account', () => {
    cy.contains('ACCOUNT 2').as('borrowerAccount');
    cy.get('@borrowerAccount').click();
  });

  describe('issues a borrow now order', () => {
    const lendAmtString = '100';
    it('selects the borrow now tab', () => {
      const borrowLink = cy.contains('.nav-link', 'Borrow');
      borrowLink.click();
      const borrowNowTab = cy.contains('borrow now');
      borrowNowTab.click();
      cy.contains('Borrow 1 day USDC');
    });

    it('enters the loan amount', () => {
      const amountInput = cy.get('.fixed-term .borrow-now .input-amount input').should('not.be.disabled');
      amountInput.click();
      amountInput.type(lendAmtString);
      amountInput.clear();
    });
    it('submits the transaction', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('receives the correct notification', () => {
      cy.contains(`Your borrow order for ${lendAmtString} USDC was filled successfully`);
    });
  });

  describe('can cancel an outstanding order', () => {
    it('selects the lender account', () => {
      cy.contains('ACCOUNT 1').as('lenderAccount');
      cy.get('@lenderAccount').click();
    });

    it('submits the order cancellation', () => {
      cy.get('.debt-detail tr .anticon-close').first().click();
    });
    it('receives the correct notification', () => {
      cy.contains('Order Cancelled');
    });
  });

  describe('can repay and outstanding borrow', () => {
    it('selects the borrower account', () => {
      cy.contains('ACCOUNT 2').as('borrowerAccount');
      cy.get('@borrowerAccount').click();
      cy.contains('You owe');
    });

    it('enters the repayment amount', () => {
      const repayInput = cy.get('.assets-to-settle input').should('not.be.disabled');
      repayInput.click();
      repayInput.type('110');
    });

    it('submits the order', () => {
      const repayButton = cy.contains('Repay Now');
      repayButton.click();
    });
    it('receives the correct notification', () => {
      cy.contains('Repay Successful');
    });
  });
});
