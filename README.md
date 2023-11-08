# avfallsor-homeassistant
HomeAssistant MQTT integration for Avfall Sør.

## Dependencies
- Rust

## Building
```
cargo build --release
```

## Running
1. Copy the binary (`target/release/avfallsor-homeassistant` if you compiled it yourself) to your desired location.
2. Create a `.env` file in the same directory. See `.env.example` for available config variables.
3. Run `avfallsor-homeassistant`.

In your HomeAssistant's MQTT integration, Avfall Sør should now be added, with one sensor per type of garbage collection. Sensors will report the date of the next garbage collection.

Set up a weekly cron job / systemd timer / scheduled task. That's it!

## Configuration
### ADDRESS
Your address.

### COLLECTION_TIME
Integer between 0 and 23. Approximate hour of the day when garbage is collected. This is used for the time component of the sensor in HomeAssistant. Defaults to 8.

### MQTT_HOST
MQTT host / IP. Defaults to localhost. 

### MQTT_PORT
MQTT port. Defaults to 1883.