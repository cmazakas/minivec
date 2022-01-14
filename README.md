# MiniVec

`std::vec::Vec` is a cool class but it's just too big! `MiniVec` is only the size of a pointer.

## Acknowledgements

This library is dedicated to the great Glen Joseph Fernandes whose constant tutelage has been
instrumental in making me the programmer that I am.

We would also like to thank the following for their contributions:
* [DoumanAsh](https://github.com/DoumanAsh)
* [hbina](https://github.com/hbina)
* [berkus](https://github.com/berkus)
* [Plecra](https://github.com/Plecra)

## Why use an alternative to `std::vec::Vec`?

It's an interesting choice to replace a container as ubiquitous as `Vec` with an alternative implementation.

For that reason, it's not a wise design choice to use `MiniVec` itself in public interfaces as it'll create
friction with the rest of the ecosystem. Instead, `MiniVec` is ideal for internal implementation details.

Its smaller size penalizes users less for using it as a data member and it can expose some Nightly `Vec` APIs
in a stable way. It also contains myriad extensions to the standard `Vec` interface and the project aims to be
community-driven, i.e. any feature a user wants, they're likely to have their PR merged.

To be competetive performance-wise with the standard library, we must use several currently Nightly features.
Namely, specialization and access to other intrinsics. To compile `MiniVec` with such optimizations, one must use:
```console
cargo +nightly build --features minivec_nightly
```

[Serde](https://crates.io/crates/serde) is similarly supported via:
```console
cargo build --features serde
```
