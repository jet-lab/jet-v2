#![cfg(feature = "localnet")]

feature_groups! {
    "batch_all";
    "batch1" {
        mod liquidate;
        mod liquidate_with_swap;
        mod lookup_table;
    }
    "batch2" {
        mod fixed_term;
        mod load;
        mod pool_overpayment;
        mod rounding;
        mod sanity;
        mod route_swap;
        mod swap;
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
