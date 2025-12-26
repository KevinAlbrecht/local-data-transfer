# Local data transfer
🧸 Yet another playground for rust 🦀 learning purpose

## Description
CLI application to experiment with TCP and to send/receive files over local network.

## Examples
- Receive:

```bash
cargo run -- recv --port 9000 --output outputs/received.txt
```

- Send:

```bash
cargo run -- send --host 10.0.1.10 --port 9000 --input "some text"
```

