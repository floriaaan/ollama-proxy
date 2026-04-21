# ollama-proxy

`ollama-proxy` is a local reverse proxy for remote Ollama instances that require bearer-token authentication.

## usage

```bash
ollama-proxy \
  --host https://ollama.floriaaan.fr \
  --token MY_SECRET_TOKEN \
  --port 11434 \
  --log-level info
```

## startup output

```text
ollama-proxy v0.1.0

remote  : https://ollama.floriaaan.fr
local   : http://127.0.0.1:11434
auth    : bearer token
log     : info

ready.
```

## request logs

```text
127.0.0.1 "POST /api/generate" 200 123ms
127.0.0.1 "GET /api/tags" 200 12ms
```

## architecture

```text
src/
├── application/
├── domain/
├── infrastructure/
├── interfaces/
└── main.rs
```

## development

```bash
cargo fmt
cargo test
cargo build
```
