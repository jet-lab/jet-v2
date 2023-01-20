#![cfg(feature = "localnet")]

feature_groups! {
    "batch_all";
    "batch1" {
        mod liquidate;
        mod liquidate_with_swap;
    }
    "batch2" {
        mod fixed_term;
        mod load;
        mod pool_overpayment;
        mod rounding;
        mod sanity;
        mod swap;
        mod client_sanity;
        mod client_fixed_term;
    }
}

macro_rules! feature_groups {
    (
		$parent:literal;
		$($group_name:literal {
			$(mod $mod_name:ident;)*
		})*
	) => {
        $($(
			#[cfg(any(feature = $parent, feature = $group_name))]
			mod $mod_name;
		)*)*
    };
}
use feature_groups;
