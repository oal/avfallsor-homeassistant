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

In your HomeAssistant's MQTT integration, Avfall Sør should now be added, with one sensor per type of garbage collection. Sensors will report the date of the next garbage collection.

Set up a weekly cron job / systemd timer / scheduled task. That's it!
