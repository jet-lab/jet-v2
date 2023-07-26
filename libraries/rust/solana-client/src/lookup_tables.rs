//! Functions to optimize lookup table selections to minimize transaction sizes.
//!
//! These functions properly handle when multiple lookup tables provide some of
//! the same addresses. They will only include a lookup table in the return if
//! it actually provides a real benefit relative to the other returned tables.

use std::collections::HashSet;
use std::hash::Hash;

use anchor_lang::prelude::Pubkey;
use itertools::Itertools;
use solana_sdk::{
    address_lookup_table_account::AddressLookupTableAccount, instruction::Instruction,
};

use crate::util::data::{intersect, retain, retain_cloned};

/// Selects a subset of lookup tables for the provided instructions to minimize
/// tx size.
///
/// This is useful whenever you want to figure out what lookup tables to include
/// in a transaction.
///
/// Strikes a balance between accuracy and performance. The output is typically
/// perfect, but for very complex inputs, it may sacrifice accuracy for
/// performance.
///
/// The output is guaranteed to be optimal as long as the number of tables
/// meeting this criteria is less than 10:
/// - provides at least one address needed by an instruction.
/// - is not a subset of another table.
///
/// If that number >= 10, the output will typically be good, but may not be
/// perfect.
pub fn optimize_lookup_tables<'a>(
    instructions: impl IntoIterator<Item = &'a Instruction>,
    tables: &[AddressLookupTableAccount],
) -> Vec<AddressLookupTableAccount> {
    if tables.is_empty() {
        return vec![];
    }
    let table_intersections = get_table_intersections(instructions, tables);

    // exclude redundant tables
    let deduped_intersections = exclude_subsets_and_useless(table_intersections, 0);
    if deduped_intersections.len() == 1 {
        return vec![tables[deduped_intersections[0].0].clone()];
    }

    // optimize remaining tables.
    if deduped_intersections.len() < 10 {
        // perfect but slow, so avoid using it when input is large
        retain_cloned(tables, perfectly_optimize(deduped_intersections))
    } else {
        // fast but imperfect, should work reasonably well
        exclude_subsets_and_useless(deduped_intersections, 1)
            .into_iter()
            .map(|(index, _)| tables[index].clone())
            .collect()
    }
}

/// Always selects the ideal set of lookup tables for the provided instructions.
///
/// This completes in 2^n time, which can be expensive if you have large numbers
/// of lookup tables.
///
/// This is useful if:
/// - guaranteeing perfectly optimized transaction sizes is critical
/// - you know there aren't enough tables to cause a performance problem
pub fn perfectly_optimize_lookup_tables<'a>(
    instructions: impl IntoIterator<Item = &'a Instruction>,
    tables: &[AddressLookupTableAccount],
) -> Vec<AddressLookupTableAccount> {
    if tables.is_empty() {
        return vec![];
    }
    let table_intersections = get_table_intersections(instructions, tables);

    // exclude redundant tables
    let deduped_intersections = exclude_subsets_and_useless(table_intersections, 0);
    if deduped_intersections.len() == 1 {
        return vec![tables[deduped_intersections[0].0].clone()];
    }

    // optimize remaining tables.
    retain_cloned(tables, perfectly_optimize(deduped_intersections))
}

/// Basic optimization that excludes tables which are definitely useless for the
/// provided instructions.
///
/// This is useful when you are executing a first-pass filter of a large number
/// of lookup tables, but you don't yet know how the instructions will be
/// divided or grouped into transactions.
///
/// Excludes tables with either...
/// - no more than 1 account that is needed by any instructions
/// - 0 accounts that are not provided by another table with at least as many matches
pub fn exclude_useless_lookup_tables<'a>(
    instructions: impl IntoIterator<Item = &'a Instruction>,
    tables: &[AddressLookupTableAccount],
) -> Vec<AddressLookupTableAccount> {
    basic_optimization(instructions, tables, 0)
}

/// Basic optimization that quickly attempts to estimate an optimal set of
/// lookup tables for the provided instructions. This may exclude tables that
/// would actually be included in the perfectly optimal set of lookup tables for
/// these instructions.
///
/// This is useful if:
/// - you just need a ballpark upper bound estimate for a transaction size
///   including these instructions
/// - you are not worried about finding the perfectly optimal set of tables
///
/// Excludes tables with no more than 1 account that is...
/// - needed by an instructions
/// - not provided by another table with at least as many matches
pub fn roughly_optimize_lookup_tables<'a>(
    instructions: impl IntoIterator<Item = &'a Instruction>,
    tables: &[AddressLookupTableAccount],
) -> Vec<AddressLookupTableAccount> {
    basic_optimization(instructions, tables, 1)
}

/// perfect brute force optimization, but it completes in 2^n time
fn perfectly_optimize<T: Clone>(intersections: Vec<(T, HashSet<Pubkey>)>) -> Vec<T> {
    let mut scored: Vec<(isize, Vec<T>)> = vec![];
    for combo in intersections.into_iter().powerset() {
        let combined_isect: HashSet<Pubkey> = combo.iter().flat_map(|x| x.1.clone()).collect();
        let byte_adjustment = (combo.len() as isize - combined_isect.len() as isize) * 32
            + combined_isect.len() as isize;
        scored.push((byte_adjustment, combo.iter().map(|x| x.0.clone()).collect()));
    }
    scored.sort_by_key(|x| x.0);
    scored.swap_remove(0).1
}

/// fast optimization that returns a useful subset of lookup tables, but it is
/// not always optimal.
fn basic_optimization<'a>(
    instructions: impl IntoIterator<Item = &'a Instruction>,
    tables: &[AddressLookupTableAccount],
    max_novel: usize,
) -> Vec<AddressLookupTableAccount> {
    if tables.is_empty() {
        return vec![];
    }
    let table_intersections = get_table_intersections(instructions, tables);
    exclude_subsets_and_useless(table_intersections, max_novel)
        .into_iter()
        .map(|(index, _)| tables[index].clone())
        .collect()
}

/// returns tuples with:
/// - 0 = the index of the lookup table in the input
/// - 1 = the intersection of the accounts from *that* table with the set of all
///       accounts in all instructions
fn get_table_intersections<'a>(
    instructions: impl IntoIterator<Item = &'a Instruction>,
    tables: &[AddressLookupTableAccount],
) -> Vec<(usize, HashSet<Pubkey>)> {
    let accounts = instructions
        .into_iter()
        .flat_map(|ix| ix.accounts.iter().map(|acc| acc.pubkey))
        .collect::<HashSet<_>>();
    tables
        .iter()
        .map(|table| intersect(&accounts, &table.addresses))
        .filter(|intersection| intersection.len() > 1)
        .enumerate()
        .collect::<Vec<_>>()
}

/// A "useless" set has no more than 1 in its intersection. These are excluded
/// from the return.
///
/// If max_novel == 0, this excludes any set that is a subset of another.
///
/// If max_novel > 0, this removes any set that would be a subset of another,
/// aside from up to max_novel of its items.
fn exclude_subsets_and_useless<T: Clone>(
    mut intersections: Vec<(T, HashSet<Pubkey>)>,
    max_novel: usize,
) -> Vec<(T, HashSet<Pubkey>)> {
    intersections.sort_by_key(|v| v.1.len());
    let mut to_keep = vec![];
    'outer: for (sorted_index, (_, isect0)) in intersections.iter().enumerate() {
        if isect0.len() < 2 {
            continue;
        }
        for (_, isect1) in intersections.iter().skip(sorted_index + 1) {
            if num_not_in(isect0, isect1) <= max_novel {
                continue 'outer;
            }
        }
        to_keep.push(sorted_index);
    }
    retain(intersections, to_keep)
}

/// returns the number of items in "sub" that are not in "sup"
fn num_not_in<T: Eq + Hash>(sub: &HashSet<T>, sup: &HashSet<T>) -> usize {
    sub.iter().filter(|v| !sup.contains(v)).count()
}

#[cfg(test)]
mod test_optimize_lookup_tables {
    use std::collections::{HashMap, HashSet};

    use anchor_lang::prelude::Pubkey;
    use solana_sdk::{
        address_lookup_table_account::AddressLookupTableAccount,
        instruction::{AccountMeta, Instruction},
    };

    use super::{
        optimize_lookup_tables, perfectly_optimize_lookup_tables, roughly_optimize_lookup_tables,
    };

    #[test]
    fn no_tables() {
        run_test(&[&["true", "false"]], &[], &[], AllMustBePerfect)
    }

    #[test]
    fn no_instructions() {
        run_test(
            &[],
            &[("good", &["true", "false"]), ("bad", &["true", "red"])],
            &[],
            AllMustBePerfect,
        )
    }

    #[test]
    fn basic() {
        run_test(
            &[&["true", "false"]],
            &[("good", &["true", "false"]), ("bad", &["true", "red"])],
            &["good"],
            AllMustBePerfect,
        )
    }

    #[test]
    fn hierarchy() {
        run_test(
            &[&["a", "b", "c"], &["d", "e", "f", "g", "h"]],
            &[
                ("complete", &["a", "b", "c", "d", "e", "f", "g", "h"]),
                ("-1", &["b", "c", "d", "e", "f", "g", "h"]),
                ("-2", &["c", "d", "e", "f", "g", "h"]),
                ("-3", &["d", "e", "f", "g", "h"]),
                ("-4", &["e", "f", "g", "h"]),
                ("-5", &["f", "g", "h"]),
                ("-6", &["g", "h"]),
                ("-7", &["h"]),
            ],
            &["complete"],
            AllMustBePerfect,
        )
    }

    #[test]
    fn diagonal_hierarchy() {
        run_test(
            &[&["a", "b", "c"], &["d", "e", "f", "g", "h"]],
            &[
                ("best", &["a", "b", "c", "d", "e", "f", "g"]),
                ("-1", &["b", "c", "d", "e", "f", "h"]),
                ("-2", &["c", "d", "e", "g", "h"]),
                ("-3", &["d", "f", "g", "h"]),
                ("-5", &["f", "g", "h"]),
                ("-6", &["g", "h"]),
                ("-7", &["h"]),
            ],
            &["best"],
            AllMustBePerfect,
        )
    }

    #[test]
    fn partial_double_diagonal_hierarchy() {
        run_test(
            &[&["a", "b", "c"], &["d", "e", "f", "g", "h"]],
            &[
                ("best", &["a", "b", "c", "d", "e", "f"]),
                ("-1", &["b", "c", "d", "e", "f", "h"]),
                ("-2", &["c", "d", "e", "g", "h"]),
                ("-3", &["d", "f", "g", "h"]),
                ("-5", &["f", "g", "h"]),
                ("-6", &["g", "h"]),
                ("-7", &["h"]),
            ],
            &["best", "-3"],
            RoughCanBe(&["-1"]),
        )
    }

    #[test]
    fn trimmed_partial_double_diagonal_hierarchy() {
        run_test(
            &[&["a", "b", "c"], &["d", "e", "f", "g", "h"]],
            &[
                ("best", &["a", "b", "c", "d", "e", "f"]),
                ("-1", &["c", "d", "e", "f"]),
                ("-2", &["c", "d", "e", "g"]),
                ("-3", &["d", "f", "g", "h"]),
                ("-5", &["f", "g", "h"]),
                ("-6", &["g", "h"]),
                ("-7", &["h"]),
            ],
            &["best", "-3"],
            AllMustBePerfect,
        )
    }

    #[test]
    fn sloppy_double_diagonal_hierarchy() {
        run_test(
            &[&["a", "b", "c"], &["d", "e", "f", "g", "h"]],
            &[
                ("best", &["a", "b", "c", "d", "e", "f"]),
                ("-1", &["c", "d", "e", "f", "g"]),
                ("-2", &["c", "d", "e", "h"]),
                ("-3", &["d", "f", "g"]),
                ("-5", &["f", "h"]),
                ("-6", &["g"]),
                ("-7", &["h"]),
            ],
            &["best"],
            AllMustBePerfect,
        )
    }

    #[test]
    fn multi() {
        run_test(
            &[&["red", "green", "blue", "true", "false"]],
            &[
                ("bool", &["true", "false"]),
                ("color", &["red", "green", "blue"]),
            ],
            &["bool", "color"],
            AllMustBePerfect,
        )
    }

    #[test]
    fn weak_overlap_loses() {
        run_test(
            &[&["red", "green", "blue", "true", "false", "n", "s", "e", "w"]],
            &[
                ("bool", &["true", "false"]),
                ("color", &["red", "green", "blue"]),
                ("direction", &["n", "s", "e", "w"]),
                ("weak", &["red", "green", "true", "n", "s", "e"]),
            ],
            &["bool", "color", "direction"],
            RoughCanBe(&["weak"]),
        )
    }

    #[test]
    fn strong_overlap_wins() {
        run_test(
            &[&["red", "green", "blue", "true", "false", "n", "s", "e", "w"]],
            &[
                ("bool", &["true", "false"]),
                ("color", &["red", "green", "blue"]),
                ("direction", &["n", "s", "e", "w"]),
                ("strong", &["red", "green", "true", "false", "n", "s", "e"]),
            ],
            &["strong"],
            AllMustBePerfect,
        )
    }

    #[test]
    fn weak_pair_loses() {
        run_test(
            &[&["red", "green", "blue", "true", "false", "n", "s", "e", "w"]],
            &[
                ("bool", &["true", "false"]),
                ("color", &["red", "green", "blue"]),
                ("direction", &["n", "s", "e", "w"]),
                ("one", &["true", "red", "n"]),
                ("two", &["false", "green", "s", "e"]),
                ("empty", &[]),
            ],
            &["bool", "color", "direction"],
            RoughCanBe(&["one", "two", "color", "direction"]),
        )
    }

    #[test]
    fn strong_pair_wins() {
        run_test(
            &[&["red", "green", "blue", "true", "false", "n", "s", "e", "w"]],
            &[
                ("bool", &["true", "false"]),
                ("color", &["red", "green", "blue"]),
                ("direction", &["n", "s", "e", "w"]),
                ("one", &["true", "red", "n"]),
                ("two", &["false", "green", "s", "e", "w"]),
            ],
            &["one", "two"],
            RoughCanBe(&["one", "two", "color"]),
        )
    }

    #[test]
    fn strong_pair_wins2() {
        run_test(
            &[&["red", "green", "blue", "true", "false", "n", "s", "e", "w"]],
            &[
                ("bool", &["true", "false"]),
                ("color", &["red", "green", "blue"]),
                ("direction", &["n", "s", "e", "w"]),
                ("one", &["red", "true", "false"]),
                ("two", &["green", "n", "s", "e", "w"]),
            ],
            &["one", "two"],
            RoughCanBe(&["one", "two", "color"]),
        )
    }

    #[test]
    fn over_10_not_subsets_partial() {
        run_test(
            &[
                &["10", "11", "12", "13", "14", "15", "16", "17", "18", "19"],
                &["20", "21", "22", "23", "24", "25", "26", "27", "28", "29"],
                &["30", "31", "32", "33", "34", "35", "36", "37", "38", "39"],
                &["40", "41", "42", "43", "44", "45", "46", "47", "48", "49"],
                &["50", "51", "52", "53", "54", "55", "56", "57", "58", "59"],
            ],
            &[
                (
                    "10",
                    &["10", "11", "12", "13", "14", "15", "16", "17", "18", "19"],
                ),
                (
                    "20",
                    &["20", "21", "22", "23", "24", "25", "26", "27", "28", "29"],
                ),
                (
                    "30",
                    &["30", "31", "32", "33", "34", "35", "36", "37", "38", "39"],
                ),
                (
                    "40",
                    &["40", "41", "42", "43", "44", "45", "46", "47", "48", "49"],
                ),
                (
                    "50",
                    &["50", "51", "52", "53", "54", "55", "56", "57", "58", "59"],
                ),
                ("0", &["10", "20", "30", "40", "50"]),
                ("1", &["11", "21", "31", "41", "51"]),
                ("2", &["12", "22", "32", "42", "52"]),
                ("3", &["13", "23", "33", "43", "53"]),
                ("4", &["14", "24", "34", "44", "54"]),
                ("5", &["15", "25", "35", "45", "55"]),
                ("6", &["16", "26", "36", "46", "56"]),
                ("7", &["17", "27", "37", "47", "57"]),
                ("8", &["18", "28", "38", "48", "58"]),
                ("9", &["19", "29", "39", "49", "59"]),
                (
                    "all_but1",
                    &[
                        "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "22", "23",
                        "24", "25", "26", "27", "28", "29", "30", "31", "33", "34", "35", "36",
                        "37", "38", "39", "40", "41", "42", "44", "45", "46", "47", "48", "49",
                        "50", "51", "52", "53", "55", "56", "57", "58", "59",
                    ],
                ),
            ],
            &["10", "20", "30", "40", "50"],
            RoughAndBalancedCanBe(&["all_but1"]),
        )
    }

    #[test]
    fn under_10_not_subsets_partial() {
        run_test(
            &[
                &["10", "11", "12", "13", "14", "15", "16", "17", "18", "19"],
                &["20", "21", "22", "23", "24", "25", "26", "27", "28", "29"],
                &["30", "31", "32", "33", "34", "35", "36", "37", "38", "39"],
                &["40", "41", "42", "43", "44", "45", "46", "47", "48", "49"],
                &["50", "51", "52", "53", "54", "55", "56", "57", "58", "59"],
            ],
            &[
                (
                    "10",
                    &["10", "11", "12", "13", "14", "15", "16", "17", "18"],
                ),
                (
                    "20",
                    &["20", "21", "22", "23", "24", "25", "26", "27", "28"],
                ),
                (
                    "30",
                    &["30", "31", "32", "33", "34", "35", "36", "37", "38"],
                ),
                (
                    "40",
                    &["40", "41", "42", "43", "44", "45", "46", "47", "48"],
                ),
                (
                    "50",
                    &["50", "51", "52", "53", "54", "55", "56", "57", "58"],
                ),
                ("0", &["10", "20", "30", "40", "50"]),
                ("1", &["11", "21", "31", "41", "51"]),
                ("2", &["12", "22", "32", "42", "52"]),
                (
                    "all_but1",
                    &[
                        "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "22", "23",
                        "24", "25", "26", "27", "28", "29", "30", "31", "33", "34", "35", "36",
                        "37", "38", "39", "40", "41", "42", "44", "45", "46", "47", "48", "49",
                        "50", "51", "52", "53", "55", "56", "57", "58", "59",
                    ],
                ),
            ],
            &["all_but1"],
            AllMustBePerfect,
        )
    }

    #[test]
    fn over_10_all_subsets_comprehensive() {
        run_test(
            &[
                &["10", "11", "12", "13", "14", "15", "16", "17", "18", "19"],
                &["20", "21", "22", "23", "24", "25", "26", "27", "28", "29"],
                &["30", "31", "32", "33", "34", "35", "36", "37", "38", "39"],
                &["40", "41", "42", "43", "44", "45", "46", "47", "48", "49"],
                &["50", "51", "52", "53", "54", "55", "56", "57", "58", "59"],
            ],
            &[
                (
                    "10",
                    &[
                        "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20",
                    ],
                ),
                (
                    "20",
                    &[
                        "20", "21", "22", "23", "24", "25", "26", "27", "28", "29", "30",
                    ],
                ),
                (
                    "30",
                    &[
                        "30", "31", "32", "33", "34", "35", "36", "37", "38", "39", "40",
                    ],
                ),
                (
                    "40",
                    &[
                        "40", "41", "42", "43", "44", "45", "46", "47", "48", "49", "50",
                    ],
                ),
                (
                    "50",
                    &[
                        "50", "51", "52", "53", "54", "55", "56", "57", "58", "59", "10",
                    ],
                ),
                ("0", &["10", "20", "30", "40", "50"]),
                ("1", &["11", "21", "31", "41", "51"]),
                ("2", &["12", "22", "32", "42", "52"]),
                ("3", &["13", "23", "33", "43", "53"]),
                ("4", &["14", "24", "34", "44", "54"]),
                ("5", &["15", "25", "35", "45", "55"]),
                ("6", &["16", "26", "36", "46", "56"]),
                ("7", &["17", "27", "37", "47", "57"]),
                ("8", &["18", "28", "38", "48", "58"]),
                ("9", &["19", "29", "39", "49", "59"]),
                (
                    "all",
                    &[
                        "10", "11", "12", "13", "14", "15", "16", "17", "18", "19", "20", "21",
                        "22", "23", "24", "25", "26", "27", "28", "29", "30", "31", "32", "33",
                        "34", "35", "36", "37", "38", "39", "40", "41", "42", "43", "44", "45",
                        "46", "47", "48", "49", "50", "51", "52", "53", "54", "55", "56", "57",
                        "58", "59",
                    ],
                ),
            ],
            &["all"],
            AllMustBePerfect,
        )
    }

    fn run_test(
        instructions: &[&[&str]],
        tables: &[(&str, &[&str])],
        expected: &[&str],
        exceptions: AltAssert<&[&str]>,
    ) {
        let names = tables
            .iter()
            .map(|t| (addr(t.0), t.0))
            .collect::<HashMap<Pubkey, &str>>();
        let instructions = instructions
            .iter()
            .map(|ix| test_ix(ix))
            .collect::<Vec<_>>();
        let tables = tables
            .iter()
            .map(|(key, addrs)| test_table(key, addrs))
            .collect::<Vec<_>>();
        let balanced = optimize_lookup_tables(&instructions, &tables);
        let perfect = perfectly_optimize_lookup_tables(&instructions, &tables);
        let rough = roughly_optimize_lookup_tables(&instructions, &tables);
        assert_tables("perfect", expected, perfect, &names).unwrap();
        match exceptions {
            AllMustBePerfect => {
                assert_tables("balanced", expected, balanced, &names).unwrap();
                assert_tables("rough", expected, rough, &names).unwrap();
            }
            RoughCanBe(alt) => {
                assert_tables("balanced", expected, balanced, &names).unwrap();
                assert_tables("rough", alt, rough, &names).unwrap();
            }
            RoughAndBalancedCanBe(alt) => {
                assert_tables("balanced", alt, balanced, &names).unwrap();
                assert_tables("rough", alt, rough, &names).unwrap();
            }
        }
    }

    fn test_ix(addresses: &[&str]) -> Instruction {
        Instruction {
            program_id: Pubkey::default(),
            accounts: addresses
                .iter()
                .map(|s| AccountMeta {
                    pubkey: addr(s),
                    is_signer: false,
                    is_writable: false,
                })
                .collect(),
            data: vec![],
        }
    }

    fn assert_tables(
        name: &str,
        expected: &[&str],
        actual: Vec<AddressLookupTableAccount>,
        names: &HashMap<Pubkey, &str>,
    ) -> anyhow::Result<()> {
        let e = expected.iter().collect::<HashSet<_>>();
        let a = actual
            .into_iter()
            .map(|table| names.get(&table.key).unwrap())
            .collect::<HashSet<_>>();
        if e != a {
            anyhow::bail!("{name}\n expected: {e:?}\n   actual: {a:?}\n")
        }
        Ok(())
    }

    fn test_table(key: &str, addresses: &[&str]) -> AddressLookupTableAccount {
        AddressLookupTableAccount {
            key: addr(key),
            addresses: addresses.iter().map(|s| addr(s)).collect(),
        }
    }

    fn addr(s: &str) -> Pubkey {
        Pubkey::find_program_address(&[(s.as_bytes())], &Pubkey::default()).0
    }

    enum AltAssert<T> {
        AllMustBePerfect,
        RoughCanBe(T),
        RoughAndBalancedCanBe(T),
    }
    use AltAssert::*;
}
