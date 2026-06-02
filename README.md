# docs-converter

Convert **between** PDF, Markdown, HTML, LaTeX, and Word (.docx) — any input to
any output, one file or a whole folder. A [Tauri](https://v2.tauri.app) +
[Svelte](https://svelte.dev) desktop app over a small Rust converter —
[`pdf_oxide`](https://crates.io/crates/pdf_oxide) + [pandoc](https://pandoc.org) —
that runs as a tiny HTTP service **on your server**.

## How it works

```
 Svelte UI ──HTTP──▶ doc-convert serve ──▶ converted file(s)
 (pick / drop)       (pdf_oxide + pandoc, on your server)
```

- The app is a **thin HTTP client** — it reads the files you pick or drop and
  POSTs them to the server. Pick one file (save-as dialog) or many / a folder
  (batch into an output folder). Your laptop does no conversion at all.
- Conversion runs **on the server, on the CPU**. `pdf_oxide` is ~0.8 ms/page, so
  a 726-page book extracts in ~1 s, and the **GPU is never touched** (it stays
  free for your LLM).
- **Any input → any output.** PDF input is extracted to Markdown by `pdf_oxide`
  (pandoc can't read PDF); every other input (md/html/tex/docx) is read by
  pandoc directly — so `docx → pdf`, `html → md`, `tex → docx`, … all work.
  **xelatex** renders pdf, and extracted text is sanitized (control bytes
  stripped, stray `\n`/`$`/`<tag>` kept literal) so LaTeX never chokes.
- The same `doc-convert` binary is also a CLI:
  - `doc-convert serve --port 8088` — the HTTP API (deployed on the server)
  - `doc-convert -i in.docx -t pdf --api-url http://server:8088` — off the server
  - `doc-convert -i in.docx -t pdf` — locally, in-process

## Layout

```
docs-converter/
├── src/                        Svelte 5 frontend (App + lib/components, lib/convert.ts, lib/files.ts)
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
just install && just dev # run the desktop app (a thin client to the server)
```

`just deploy` installs Rust + pandoc on the server (once), builds the binary
natively, and starts `doc-convert serve`. `just teardown` removes it. Needs
`sshpass` locally.

## The API

```
GET  /health                                                  -> ok
POST /convert?from=pdf|md|html|tex|docx&to=md|html|tex|docx|pdf
     body = raw document bytes -> converted bytes   (from is sniffed if omitted)
```

```bash
curl --data-binary @report.docx "http://your-server:8088/convert?from=docx&to=pdf" -o report.pdf
```

Your other services can hit the same endpoint.

## CLI / tests

```bash
# convert off the server (any input -> any output)
CONVERTER_API_URL=http://your-server:8088 doc-convert -i report.docx -t pdf
# or locally, in-process
doc-convert -i notes.md -t docx

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
