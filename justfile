core:
    wasm-pack build crates/uploadnt-core --release --target web --out-dir ../../packages/core/pkg --out-name core --no-pack -- --features wasm

web: core
    cd web && bun run build

dev: core
    cd web && bun run dev

lobby:
    cargo run -p lobby --release

serve: web lobby

deploy: web
    bun deploy/provenance.ts web/build
    cd deploy && bunx wrangler deploy

verify url="https://uploadnt.dev" repo="":
    bun deploy/verify.ts {{url}} {{repo}}
