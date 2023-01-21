``` 
cargo watch -cx "run --release"
```

```
RUSTFLAGS='--cfg use_canvas' cargo build --profile wasm-release
```

``` 
wasm-bindgen --out-dir ./out/ --target web ./target/wasm32-unknown-unknown/wasm-release/legacy_game.wasm
```

```
wasm-opt -Oz --output ./out/optimized.wasm ./out/legacy_game_bg.wasm
mv optimized.wasm ./target/legacy_game_bg.wasm
```