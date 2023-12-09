###### Going through https://os.phil-opp.com/freestanding-rust-binary

###### When running tests comment out both occurences of the following line in the `Cargo.toml`:
```
panic = "abort" # Disable stack unwinding on panic
```
###### This is due to [this](https://github.com/rust-lang/cargo/issues/7359) issue.

