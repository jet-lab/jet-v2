#![cfg(feature = "localnet")]

feature_groups! {
	"batch_all";
	"batch1" {
		mod liquidate;
	}
	"batch2" {
		mod load;
		mod pool_overpayment;
		mod positions;
		mod rounding;
		mod sanity;
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
