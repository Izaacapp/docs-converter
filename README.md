# docs-converter

Upload a PDF and convert it to **Markdown, HTML, JSON, LaTeX, Word (.docx), or PDF** — with **OCR built in** for scanned documents. A small [Tauri](https://v2.tauri.app) + [Svelte](https://svelte.dev) desktop app over a thin Rust **sidecar** that drives [Docling](https://github.com/docling-project/docling) (the engine) and [pandoc](https://pandoc.org).

## How it works

```
 Svelte UI ──Command.sidecar──▶ doc-convert (Rust)
                                   │
                  ┌────────────────┴───────────────────┐
                  ▼                                     ▼
            Docling  ── md/html/json ──▶  (md) ──▶ pandoc ──▶ tex / docx / pdf
        (OCR · tables · layout)                      (xelatex for pdf)
```

- **Docling does the hard part** — OCR (RapidOCR/EasyOCR/Tesseract), table structure, reading order, layout. Runs either as a local `docling` CLI or against a remote **`docling-serve`** instance on your homelab.
- **pandoc** only runs the Markdown → `tex`/`docx`/`pdf` leg; **xelatex** renders PDF (Unicode-safe; ligatures like `ﬁ`/`ﬂ` are NFKC-folded for `pdflatex`).
- The **`doc-convert` sidecar** is a thin orchestrator with the classic contract: payload → stdout, progress → stderr (`>> phase=…`), typed exit codes. It is bundled into the Tauri app via `externalBin` and invoked from the frontend through `@tauri-apps/plugin-shell`.

## Repository layout

```
docs-converter/
├── src/                     Svelte 5 frontend (App.svelte, lib/convert.ts)
├── src-tauri/               Tauri 2 shell (externalBin: binaries/doc-convert)
│   ├── binaries/            doc-convert-<triple> (built locally; gitignored)
│   └── capabilities/        shell:allow-execute for the sidecar
├── server/sidecars/
│   └── doc-convert/         the Rust sidecar (Docling + pandoc orchestrator)
└── scripts/
    ├── build-sidecar.sh     cargo build --release → src-tauri/binaries/
    └── ctx7.sh              fetch library docs via context7 (uses .env key)
```

## Setup

### 1. The Docling engine — choose one

**A) Homelab (recommended).** Run `docling-serve` where your GPU + LaTeX already live, then point the app at it (Advanced → Docling server URL, or `DOCLING_SERVE_URL` in `.env`):

```bash
pip install "docling-serve[ui]"
docling-serve run --port 5001
```

**B) Local CLI.** Install Docling into a venv; the app finds it via `DOCLING_BIN`:

```bash
python3 -m venv .venv
.venv/bin/pip install docling
# Apple Silicon: the sidecar runs Docling with --device cpu to avoid the MPS
# float64 bug; a GPU box can use --device cuda.
```

### 2. Format tools (for tex/docx/pdf)

```bash
brew install pandoc          # md → html/tex/docx, and md → pdf via LaTeX
# TeX Live / MacTeX provides xelatex (PDF output)
```

### 3. App

```bash
corepack enable pnpm
pnpm install
pnpm build:sidecar     # builds doc-convert → src-tauri/binaries/doc-convert-<triple>
pnpm tauri dev         # run the desktop app
# pnpm tauri build     # produce a distributable bundle
```

Copy `.env.example` → `.env` and fill in `CONTEXT7_API_KEY`, `DOCLING_SERVE_URL` (or `DOCLING_BIN`).

## The sidecar CLI

The app shells out to this; it is also usable on its own:

```
doc-convert --input <PDF> --to <md|html|json|tex|docx|pdf>
            [--ocr auto|force|off] [--ocr-lang en] [--ocr-engine easyocr]
            [--engine auto|cli|serve] [--serve-url URL]
            [--output PATH] [--standalone] [--pdf-engine xelatex|lualatex|pdflatex]
            [--device cpu|cuda|mps] [--image-mode placeholder|embedded|referenced]
            [--no-tables] [--json-progress] [-q]
```

`md`/`html`/`json` stream to stdout (or `--output`); `docx`/`pdf` require `--output`.

**Exit codes:** `0` ok · `1` input unreadable · `2` Docling failed · `4` a required tool is missing (names it) · `5` pandoc/LaTeX failed · `64` usage.

```bash
# local CLI engine
DOCLING_BIN=./.venv/bin/docling doc-convert -i paper.pdf -t md
# homelab engine
doc-convert -i scan.pdf -t docx -o out.docx --serve-url http://homelab:5001 --ocr force
```

## Tested against

Validated end-to-end on a corpus of technical-writing books (`md/html/json/tex/docx/pdf`), including:

- a 324-page digital book → 671 KB structured Markdown (Preface/section/Tip headings preserved);
- a Type-3-font book → LaTeX with ligatures normalized (no `ﬁ`/`ﬂ`);
- an Internet-Archive **scanned** page → real OCR text via Docling+RapidOCR.

Run the sidecar's tests:

```bash
cd server/sidecars/doc-convert
cargo test                      # fast contract tests
DOC_CONVERT_E2E=1 DOCLING_BIN=../../../.venv/bin/docling cargo test   # + real conversions
```

## Docs

`scripts/ctx7.sh` fetches current library docs via [context7](https://context7.com) using the key in `.env`:

```bash
scripts/ctx7.sh library "Tauri" "sidecar externalBin"
scripts/ctx7.sh docs /docling-project/docling-serve "convert file endpoint"
```

## Known limitations

- First local Docling run downloads layout/OCR models (hundreds of MB).
- CPU conversion of a long book takes minutes; use a GPU box via `docling-serve`.
- `tex`/`docx`/`pdf` need `pandoc` (+ a LaTeX engine for `pdf`); missing tools give a clear exit-4 message.

## License

MIT
