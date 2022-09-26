var sourcesIndex = JSON.parse('{\
"jet_margin":["",[["instructions",[],["accounting_invoke.rs","adapter_invoke.rs","close_account.rs","close_position.rs","create_account.rs","liquidate_begin.rs","liquidate_end.rs","liquidator_invoke.rs","refresh_position_metadata.rs","register_position.rs","update_position_balance.rs","verify_healthy.rs"]],["state",[["account",[],["positions.rs"]]],["account.rs"]]],["adapter.rs","events.rs","instructions.rs","lib.rs","state.rs","syscall.rs","util.rs"]],\
"jet_margin_pool":["",[["instructions",[],["close_loan.rs","collect.rs","configure.rs","create_pool.rs","deposit.rs","margin_borrow.rs","margin_refresh_position.rs","margin_repay.rs","register_loan.rs","repay.rs","withdraw.rs"]]],["events.rs","instructions.rs","lib.rs","state.rs","util.rs"]],\
"jet_margin_swap":["",[["instructions",[],["margin_spl_swap.rs"]]],["instructions.rs","lib.rs"]]\
}');
createSourceSidebar();
