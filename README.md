# <img src="web/static/favicon.svg" alt="" width="36" height="36" align="absmiddle"> uploadn't

Send files from one browser to another without uploading them anywhere.

Transfers are [end-to-end encrypted](https://uploadnt.com/rest#handshake), and the key is part of the link. Try it on [uploadnt.com](https://uploadnt.com) or read [how it works](https://uploadnt.com/rest).

## Self-host

Build the image with [Docker](https://docs.docker.com/get-docker) or [Podman](https://podman.io):

```sh
just web && bun deploy/provenance.ts web/build
docker build -f deploy/selfhost/Dockerfile -t localhost/uploadnt-lobby .
```

On the server, put `deploy/selfhost/compose.yaml` in a folder with `deploy/relay` as `relay`, and add:

- `relay/turn_secret`: a random string
- `tunnel.env`: `TUNNEL_TOKEN=` and your [Cloudflare tunnel](https://developers.cloudflare.com/cloudflare-one/connections/connect-networks/) token
- your relay's hostname in `TURN_URLS` in `compose.yaml`

Then run `docker compose up -d` and open UDP and TCP 3478 and UDP 49160-49999 for the relay.

You can replace the tunnel with any reverse proxy that supports WebSockets. The relay is optional, but without it there is no [plan b](https://uploadnt.com/rest#connection) for people who can't connect directly.

## Cloudflare Workers

Set your domain in `deploy/wrangler.toml` and run `just deploy`. For [plan b](https://uploadnt.com/rest#connection), add `TURN_KEY_ID` and the `TURN_KEY_API_TOKEN` secret for [Cloudflare's TURN](https://developers.cloudflare.com/realtime/turn/), or `TURN_URLS` and `TURN_SECRET` for your own [coturn](https://github.com/coturn/coturn).

## Verify

```sh
just verify https://uploadnt.com
```

Checks that the live site matches the files GitHub Actions built and [attested](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations).

## Build

You need [Rust](https://rustup.rs), [wasm-pack](https://rustwasm.github.io/wasm-pack), [Bun](https://bun.sh) and [just](https://just.systems).

```sh
bun install
just serve
```

Then open `localhost:8787`.
