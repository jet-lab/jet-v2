import { airdrop, borrow, deposit, loadPageAndFundSol, createAccount } from '../support/actions';

describe('Fixed Term Market', () => {
  describe('Setup', () => {
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

    describe('funds the lender account', () => {
      it('creates a lender account', () => {
        // Account 1 = Lender
        createAccount();
      });

      it('airdrops USDC', () => {
        airdrop('USDC', 'USDC');
      });

      it('deposits USDC', () => {
        deposit('USDC', 50000);
      });
    });

    describe('funds the borrower account', () => {
      it('creates a borrower account', () => {
        // Account 2 = Borrower
        createAccount();
      });
      it('airdrops USDC', () => {
        airdrop('USDC', 'USDC');
      });

      it('deposits USDC', () => {
        deposit('USDC', 50000);
      });
    });
  });

  describe('can create one fixed rate lend order', () => {
    it('selects the lender account', () => {
      cy.contains('ACCOUNT 1').as('lenderAccount');
      cy.get('@lenderAccount').click();
    });

    it('selects the offer loan tab', () => {
      const lendLink = cy.contains('.nav-link', 'Lend');
      lendLink.click();
      cy.contains('lend now'); // ensure we loaded the page fully
    });

    it('inputs the lend amount', () => {
      const amountInput = cy.get('.fixed-term .offer-loan .input-amount input');
      amountInput.click();
      cy.wait(500)
      amountInput.type('1000');
      cy.wait(500)
      amountInput.blur();
      cy.wait(1000) // wait for debounce
    });

    it('inputs the interest rate', () => {
      const interestInput = cy.get('.fixed-term .offer-loan .input-rate input');
      interestInput.click();
      interestInput.type('10');
      interestInput.blur();
      cy.wait(1000) // wait for debounce
    });
    it('clicks the button once enabled', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('successfully receives confirmation', () => {
      cy.contains(`Your lend offer for 1000.00 USDC at 10.00% was created successfully`);
    });
  });

  describe('can create one fixed rate borrow order', () => {
    it('selects the borrower account', () => {
      cy.contains('ACCOUNT 2').as('borrowerAccount');
      cy.get('@borrowerAccount').click();
    });

    it('selects the request loan tab', () => {
      const borrowLink = cy.contains('.nav-link', 'Borrow');
      borrowLink.click();
      cy.contains('borrow now'); // ensure we loaded the page fully
    });

    it('inputs the borrow amount', () => {
      const amountInput = cy.get('.fixed-term .request-loan .input-amount input');
      amountInput.clear();
      amountInput.click();
      cy.wait(500)
      amountInput.type('100');
      cy.wait(500)
      amountInput.blur();
      cy.wait(1000) // wait for debounce
    });

    it('inputs the interest rate', () => {
      const interestInput = cy.get('.fixed-term .request-loan .input-rate input');
      interestInput.clear();
      interestInput.type('5');
      interestInput.blur();
      cy.wait(1000) // wait for debounce
    });

    it('clicks the button once enabled', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('successfully receives confirmation', () => {
      cy.contains(`Your borrow offer for 100.00 USDC at 5.00% was created successfully`);
    });
  });

  describe('issues a lend now order', () => {
    it('selects the lender account', () => {
      cy.contains('ACCOUNT 1').as('lenderAccount');
      cy.get('@lenderAccount').click();
    });

    it('selects the lend now tab', () => {
      const lendLink = cy.contains('.nav-link', 'Lend');
      lendLink.click();
      const lendNow = cy.contains('lend now');
      lendNow.click();
      cy.contains('Lend 1 day USDC');
    });

    it('enters the amount', () => {
      const amountInput = cy.get('.fixed-term .lend-now .input-amount input').should('not.be.disabled');
      amountInput.clear();
      amountInput.click();
      amountInput.type('20');
      amountInput.blur();
      cy.wait(1000) // wait for debounce
    });

    it('clicks the button once enabled', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('received the correct notification', () => {
      cy.contains(`Your lend order for 20.00 USDC was filled successfully`);
    });
  });

  describe('issues a borrow now order', () => {
    it('selects the borrower account', () => {
      cy.contains('ACCOUNT 2').as('borrowerAccount');
      cy.get('@borrowerAccount').click();
    });

    it('selects the borrow now tab', () => {
      const borrowLink = cy.contains('.nav-link', 'Borrow');
      borrowLink.click();
      const borrowNowTab = cy.contains('borrow now');
      borrowNowTab.click();
      cy.contains('Borrow 1 day USDC');
    });

    it('enters the loan amount', () => {
      const amountInput = cy.get('.fixed-term .borrow-now .input-amount input').should('not.be.disabled');
      amountInput.clear();
      amountInput.click();
      amountInput.type('30');
      cy.wait(1000) // wait for debounce
    });
    it('submits the transaction', () => {
      const submitButton = cy.get('.fixed-term .submit-button').should('not.be.disabled');
      submitButton.click();
    });

    it('receives the correct notification', () => {
      cy.contains(`Your borrow order for 30.00 USDC was filled successfully`);
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
      repayInput.clear();
      repayInput.click();
      cy.wait(500)
      repayInput.type('110');
      cy.wait(500)
      repayInput.blur();
      cy.wait(1000) // wait for debounce
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
