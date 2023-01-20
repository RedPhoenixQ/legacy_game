``` 
cargo watch -cx "run --release"
```

``` 
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/release/legacy_game.wasm
```