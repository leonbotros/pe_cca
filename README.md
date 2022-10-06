# Efficient and Generic Transformations for Chosen-Ciphertext Secure Predicate Encryption

This is the supplementary code package to the similarly named paper.
Specifically, this package contains code to benchmark several instantiation of
CCA transformations applied to two predicate encryption schemes:

- Chen-Gay-Wee anonymous IBE (CGW).
- Rouselakis-Waters (RW13).

# Running the benchmarks

First make sure the Rust toolchain is installed.
If not, you can install it using:

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Test all schemes with:

```
cargo test --all-features
```

Then run the benchmarks using:

```
cargo bench --all-features
```
