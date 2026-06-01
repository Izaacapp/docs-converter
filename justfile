# docs-converter — task runner. Run `just` to list tasks.
# Loads .env (CONTEXT7_API_KEY, CONVERTER_API_URL, CONVERTER_SERVER_*, …).
set dotenv-load := true
set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

# put nvm's node bin (where corepack's pnpm lives) on PATH for every recipe,
# since `just` runs a non-login shell that doesn't load nvm.
node_bin := `ls -d "$HOME"/.nvm/versions/node/*/bin 2>/dev/null | tail -1`
export PATH := if node_bin == "" { env_var('PATH') } else { node_bin + ":" + env_var('PATH') }

# list tasks
_default:
    @just --list

# install JS deps
install:
    pnpm install

# build the doc-convert binary (the server service / CLI) locally
sidecar:
    bash scripts/build-sidecar.sh

# run the desktop app (a thin HTTP client to the server)
dev:
    pnpm tauri dev

# build the distributable desktop bundle
build:
    pnpm tauri build

# build just the frontend (-> .artifacts/frontend)
web:
    pnpm build

# sidecar tests (in-process, fast)
test:
    cargo test --manifest-path server/sidecars/doc-convert/Cargo.toml

# lint + format the sidecar
lint:
    cargo clippy --manifest-path server/sidecars/doc-convert/Cargo.toml --all-targets -- -D warnings
fmt:
    cargo fmt --manifest-path server/sidecars/doc-convert/Cargo.toml

# real edge-case harness — converts the sample corpus off the live server
e2e:
    scripts/e2e.sh

# fetch library docs via context7:  just docs library "Tauri" "sidecar"
docs *ARGS:
    scripts/ctx7.sh {{ARGS}}

# --- converter server ---

# build + (re)start the doc-convert HTTP service on the server
deploy:
    deploy/serve.sh

# is the converter server up?
server-health:
    curl -fsS "${CONVERTER_API_URL%/}/health" && echo " ok" || echo "down"

# stop + remove the service on the server
teardown:
    deploy/teardown.sh
