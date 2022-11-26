# Shoes

**An (incomplete) implementation of SOCKS5 proxy, done for education and fun**

* To run the SOCKS proxy:

```
RUST_LOG=debug cargo run --bin shoes
```

* To run an example client:

```
cargo run --bin client
```

* If you do not have TCP target host for testing the client, do not despair:

```
cargo run --bin target
```
