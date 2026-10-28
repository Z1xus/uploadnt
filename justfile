core:
    wasm-pack build crates/uploadnt-core --release --target web --out-dir ../../packages/core/pkg --out-name core --no-pack -- --features wasm

web: core
    cd web && bun run build

dev: core
    cd web && bun run dev

lobby:
    cargo run -p lobby --release

serve: web
    bun deploy/provenance.ts web/build
    just lobby

deploy-vps host: web
    bun deploy/provenance.ts web/build
    podman build -f deploy/selfhost/Dockerfile -t uploadnt-lobby .
    podman save localhost/uploadnt-lobby:latest | ssh {{host}} docker load
    ssh {{host}} mkdir -p uploadnt/relay
    rsync deploy/selfhost/compose.yaml {{host}}:uploadnt/
    rsync deploy/relay/compose.yaml deploy/relay/turnserver.conf {{host}}:uploadnt/relay/
    ssh {{host}} 'cd uploadnt && docker compose up -d --remove-orphans'

deploy: web
    bun deploy/provenance.ts web/build
    cd deploy && bunx wrangler deploy

verify url="https://uploadnt.com" repo="Z1xus/uploadnt":
    bun deploy/verify.ts {{url}} {{repo}}
