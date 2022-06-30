FROM ubuntu:latest as programs

ARG SOLANA_VERSION="1.10.8"

COPY . /v2
WORKDIR /v2

RUN if [ ! -d "./target/deploy" ]; then echo "Deployment programs not found"; exit 1; fi && \
    apt-get update && \
    apt-get install -y curl pkg-config build-essential libudev-dev && \
    sh -c "$(curl -sSfL https://release.solana.com/v${SOLANA_VERSION}/install)"

# -------------------------------------------

FROM ubuntu:latest

RUN apt-get update && \
    apt-get install -y bzip2 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=programs /root/.local/share/solana/install/active_release/bin/solana-test-validator /usr/bin/solana-test-validator
COPY --from=programs /v2/target/deploy/*.so /root/programs/
COPY --from=programs /v2/deps/*.so /root/programs/
COPY --from=programs /v2/deps/*/*.so /root/programs/

EXPOSE 1024
EXPOSE 1027
EXPOSE 8899

CMD solana-test-validator --reset \
      --bpf-program JPCtrLreUqsEbdhtxZ8zpd8wBydKz4nuEjX5u9Eg5H8  /root/programs/jet_control.so \
      --bpf-program JPMRGNgRk3w2pzBM1RLNBnpGxQYsFQ3yXKpuk4tTXVZ  /root/programs/jet_margin.so \
      --bpf-program JPPooLEqRo3NCSx82EdE2VZY5vUaSsgskpZPBHNGVLZ  /root/programs/jet_margin_pool.so \
      --bpf-program JPMAa5dnWLFRvUsumawFcGhnwikqZziLLfqn9SLNXPN  /root/programs/jet_margin_swap.so \
      --bpf-program JPMetawzxw7WyH3qHUVScYHWFBGhjwqDnM2R9qVbRLp  /root/programs/jet_metadata.so \
      --bpf-program FT9EZnpdo3tPfUCGn8SBkvN9DMpSStAg3YvAqvYrtSvL /root/programs/pyth.so \
      --bpf-program 9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin /root/programs/serum_dex_v3.so \
      --bpf-program 4bXpkKSV8swHSnwqtzuboGPaPDeEgAn4Vt8GfarV5rZt /root/programs/spl_token_faucet.so \
      --bpf-program 9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP /root/programs/mainnet_9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP_2022-06-20.so \
      --bpf-program DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1 /root/programs/mainnet_DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1_2022-06-20.so \
      --bpf-program SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8  /root/programs/crates.io-spl_token_swap==2.0.0.so
