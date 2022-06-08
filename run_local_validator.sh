#!/bin/bash

rm -rf test-ledger

#rm ../target/deploy/*.so

#anchor build -- --features devnet && \
solana-test-validator --reset \
  --bpf-program JPCtrLreUqsEbdhtxZ8zpd8wBydKz4nuEjX5u9Eg5H8  ./target/deploy/jet_control.so \
  --bpf-program JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ  ./target/deploy/jet_margin.so \
  --bpf-program JPPooLEqRo3NCSx82EdE2VZY5vUaSsgskpZPBHNGVLZ  ./target/deploy/jet_margin_pool.so \
  --bpf-program JPMAa5dnWLFRvUsumawFcGhnwikqZziLLfqn9SLNXPN  ./target/deploy/jet_margin_swap.so \
  --bpf-program JPMetawzxw7WyH3qHUVScYHWFBGhjwqDnM2R9qVbRLp  ./target/deploy/jet_metadata.so \
  --bpf-program FT9EZnpdo3tPfUCGn8SBkvN9DMpSStAg3YvAqvYrtSvL ./target/deploy/pyth.so \
  --bpf-program 9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP ./deps/orca_token_swap_v2.so \
  --bpf-program 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin ./deps/serum_dex_v3.so \
  --bpf-program 4bXpkKSV8swHSnwqtzuboGPaPDeEgAn4Vt8GfarV5rZt ./deps/spl_token_faucet.so \
  --bpf-program SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8  ./deps/spl_token_swap.so
