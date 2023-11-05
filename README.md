# avfallsor-homeassistant
HomeAssistant MQTT integration for Avfall Sør.

## Dependencies
- Rust

## Building
```
cargo build --release
```

## Running
1. Copy `target/release/avfallsor-mqtt` to your desired location.
2. Create a `.env` file in the same directory. See `.env.example` for available config variables.
3. Run `avfallsor-mqtt`.
