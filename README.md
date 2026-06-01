# docs-converter

Upload a PDF and convert it to **Markdown, HTML, LaTeX, Word (.docx), or PDF**.
A [Tauri](https://v2.tauri.app) + [Svelte](https://svelte.dev) desktop app over a
small Rust converter — [`pdf_oxide`](https://crates.io/crates/pdf_oxide) +
[pandoc](https://pandoc.org) — that runs as a tiny HTTP service **on your
server**. One binary, three modes.

## How it works

```
 Svelte UI ──sidecar──▶ doc-convert (client) ──HTTP──▶ doc-convert serve ──▶ PDF
                                                        (pdf_oxide + pandoc)
```

- Conversion runs **on the server, on the CPU** — `pdf_oxide` is ~0.8 ms/page, so
  a 726-page book converts in ~1 s. The **GPU is never touched** (it stays free
  for your LLM — which is exactly what ran out of VRAM when heavier ML tools
  tried to grab it). Your laptop does no conversion at all.
- The same `doc-convert` Rust binary runs in three modes:
  - `doc-convert serve --port 8088` — the HTTP API (deployed on the server)
  - `doc-convert -i in.pdf -t md --api-url http://server:8088` — forward to it
  - `doc-convert -i in.pdf -t md` — convert locally, in-process
- `pdf_oxide` (the fastest Rust PDF crate) extracts to Markdown; pandoc does the
  `md → {html,tex,docx,pdf}` leg; **xelatex** renders pdf (ligatures NFKC-folded
  so pdflatex doesn't choke).

## Layout

```
docs-converter/
├── src/                        Svelte 5 frontend (App + lib/components, lib/convert.ts)
├── src-tauri/                  Tauri 2 shell (bundles the client sidecar)
├── server/sidecars/doc-convert/  the Rust converter (CLI · client · serve)
├── deploy/                     serve.sh / teardown.sh — run it on the server
├── scripts/                    build-sidecar.sh · e2e.sh · ctx7.sh
├── justfile · .mcp.json · .envrc
```

## Setup

```bash
cp .env.example .env     # set CONVERTER_SERVER_* + CONVERTER_API_URL, CONTEXT7_API_KEY
just deploy              # build + run doc-convert on the server (:8088)
just server-health       # -> ok
just install && just dev # run the desktop app (it forwards to the server)
```

`just deploy` installs Rust + pandoc on the server (once), builds the binary
natively, and starts `doc-convert serve`. `just teardown` removes it. Needs
`sshpass` locally.

## The API

```
GET  /health                                  -> ok
POST /convert?to=md|html|tex|docx|pdf          body = raw PDF -> converted bytes
```

```bash
curl --data-binary @paper.pdf "http://your-server:8088/convert?to=docx" -o paper.docx
```

Your other services can hit the same endpoint.

## CLI / tests

```bash
# convert off the server
CONVERTER_API_URL=http://your-server:8088 doc-convert -i paper.pdf -t md
# or locally, in-process
doc-convert -i paper.pdf -t md

cd server/sidecars/doc-convert && cargo test      # in-process, fast
just e2e                                           # the 8-book corpus, off the live server
```

## Docs

`scripts/ctx7.sh` (and the project `.mcp.json` Context7 server) fetch current
library docs using the key in `.env`:

```bash
just docs library "pdf_oxide" "to_markdown"
```

## License

MIT
