# rpass

A Password Manager written in **Rust** 🔒

This project contains three crates:

⚙️ `rpass_server` — the main storage for passwords

📕 `rpass as lib` — library to interact with `rpass_server`

💻 `rpass as bin` — *CLI* for end users

## Features progress

|       Feature       | server | lib  | cli  |
| :-----------------: | :----: | :--: | :--: |
|    Registration     |   ✅    |      |      |
|        Login        |   ✅    |      |      |
|  Clear user's data  |   ✅    |      |      |
|     New record      |   ✅    |      |      |
|    Delete record    |   ✅    |      |      |
|     Show record     |   ✅    |      |      |
| List user's records |   ✅    |      |      |
|  Dump user's data   |        |      |      |
| Password generation |   —    |      |      |
|   SSL encryption    |        |      |      |

## Building

Use the next command to make release build:

```bash
cargo build --release
```

