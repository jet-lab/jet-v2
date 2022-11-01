import { loadPageAndCreateAccount, airdrop, deposit, borrow, withdraw, repay } from '../support/actions';

describe('Main Flows', () => {
  it('Connects a new test wallet and creates an account', () => {
    loadPageAndCreateAccount();
  });

  it('Airdrop USDC and deposit collateral', () => {
    airdrop('USDC', 'USDC');
    deposit('USDC', 1);
  });

  it('Deposit and withdraw SOL', () => {
    deposit('SOL', 0.5);
    withdraw('SOL', 0.3);
  });

  it('Borrow and repay SOL from wallet', () => {
    airdrop('SOL', 'SOL');
    deposit('SOL', 0.5);
    borrow('SOL', 0.3);
    repay('SOL', 0.3, false);
  });

  it('Borrow and repay SOL from existing margin account', () => {
    deposit('SOL', 0.5);
    borrow('SOL', 0.3, true);
    repay('SOL', 0.3, true);
  });

  it('can create multiple fixed rate lend orders', () => {
    airdrop('SOL', 'SOL');
    airdrop('USDC', 'USDC');

    deposit('SOL', 1);
    deposit('USDC', 50000);

    const lendLink = cy.contains('Lend');
    lendLink.click();

    const amountInput = cy.get('input.ant-input[placeholder="enter order value"]');
    const interestInput = cy.get('input.ant-input[placeholder="enter interest"]');
    amountInput.click().type(`1000`);
    interestInput.click().type(`5`);

    cy.contains('button', 'Create Lend Order').should('not.be.disabled').click();

    cy.contains('Lend Order Created');

    amountInput.focus().clear();
    amountInput.click().type(`2000`);
    interestInput.focus().clear();
    interestInput.click().type(`10`);

    cy.contains('button', 'Create Lend Order').should('not.be.disabled').click();
    cy.contains('Lend Order Created');
  });

  it('can create multiple fixed rate borrow orders', () => {
    const borrowLink = cy.contains('Borrow');
    borrowLink.click();

    const submitButton = cy.contains('button', 'Create Borrow Order').should('not.be.disabled');
    const amountInput = cy.get('input.ant-input[placeholder="enter order value"]');
    const interestInput = cy.get('input.ant-input[placeholder="enter interest"]');

    amountInput.click().type(`1000`);
    interestInput.click().type(`5`);
    submitButton.click();
    cy.contains('Borrow Order Created');

    amountInput.focus().clear();
    amountInput.click().type(`2000`);
    interestInput.focus().clear();
    interestInput.click().type(`10`);

    submitButton.click();
    cy.contains('Borrow Order Created');
  });
});

describe('Error Flows', () => {
  it('Connects a new test wallet and creates an account', () => {
    cy.on('uncaught:exception', (err, runnable, promise) => {
      return false;
    });

    cy.clearLocalStorage();
    loadPageAndCreateAccount();
  });

  it('All lend and borrow transactions should be disabled, because SOL in wallet is under fees buffer amount', () => {
    const disabledInput = () => {
      cy.get('.ant-modal-content input.ant-input').should('be.disabled');
    };
    const notEnoughSolMessage = () => {
      cy.contains('Please make sure you have a buffer of at least');
    };
    const closeModal = () => {
      cy.get('button.ant-modal-close').click();
    };
    cy.get('.SOL-pools-table-row').click();
    cy.contains('button', 'Deposit').should('not.be.disabled').click();
    cy.contains('100%').click();
    cy.contains('.ant-modal-body button', 'Deposit').should('not.be.disabled').click();
    cy.contains('deposit successful');

    cy.contains('button', 'Withdraw').click();
    disabledInput();
    notEnoughSolMessage();
    closeModal();

    cy.contains('button', 'Borrow').click();
    disabledInput();
    notEnoughSolMessage();
    closeModal();

    cy.contains('button', 'Repay').click();
    disabledInput();
    notEnoughSolMessage();
    closeModal();
  });
});
