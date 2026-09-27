# Deploying Trapiks to Cloudflare Pages or Vercel

Trapiks is a fully static site. `web/dist` holds all of it:

- `index.html`
- hashed JS, CSS and `.wasm` files under `assets/`
- the map at `maps/metro-manila.bin.gz`
- the challenge files under `challenges/`

It has no server, needs no environment variables at runtime, and needs no special headers:
- The sim is single-threaded and does not use `SharedArrayBuffer`, so it needs no COOP/COEP headers.
- Routing uses the URL hash (`#/sandbox`), so it needs no SPA rewrites.

## The one catch: building needs Rust

`pnpm --dir web build` first runs `scripts/build-wasm.sh`. That script needs:

| Tool | Version |
|---|---|
| Rust | 1.94.1 with the `wasm32-unknown-unknown` target |
| `wasm-bindgen-cli` | exactly 0.2.129, which must match `Cargo.lock` |
| binaryen (`wasm-opt`) | 132 |
| Node | 22 |
| pnpm | 10 |

The build images of Cloudflare Pages and Vercel ship none of the wasm tools. Installing them in the platform's build step works, but it takes several minutes per deploy and has to be kept in step with the pinned versions.

**The recommended setup:** build in GitHub Actions, where `.github/workflows/pages.yml` already installs the exact toolchain. Then upload the finished `web/dist` with the host's CLI. The host only serves files.

## Option A: GitHub Actions builds and the host serves (recommended)

### Cloudflare Pages

1. In the Cloudflare dashboard, go to **Workers & Pages → Create → Pages → Direct Upload**. Create a project, for example `trapiks`. You don't upload anything yet.
2. Create an API token with the **Cloudflare Pages: Edit** permission (**My Profile → API Tokens**). Note your **Account ID**, shown on the Workers & Pages overview.
3. In the GitHub repo, go to **Settings → Secrets and variables → Actions** and add:
   - `CLOUDFLARE_API_TOKEN`
   - `CLOUDFLARE_ACCOUNT_ID`
4. Add `.github/workflows/cloudflare.yml`:

```yaml
name: Cloudflare Pages

on:
  workflow_dispatch:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: 1.94.1
          targets: wasm32-unknown-unknown
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@v2
        with:
          tool: wasm-bindgen@0.2.129
      - name: Install binaryen
        run: |
          curl -sSL -o "$RUNNER_TEMP/binaryen.tar.gz" https://github.com/WebAssembly/binaryen/releases/download/version_132/binaryen-version_132-x86_64-linux.tar.gz
          tar -xzf "$RUNNER_TEMP/binaryen.tar.gz" -C "$RUNNER_TEMP"
          echo "$RUNNER_TEMP/binaryen-version_132/bin" >> "$GITHUB_PATH"
      - uses: pnpm/action-setup@v4
        with:
          package_json_file: web/package.json
      - uses: actions/setup-node@v4
        with:
          node-version: 22
          cache: pnpm
          cache-dependency-path: web/pnpm-lock.yaml
      - run: pnpm --dir web install --frozen-lockfile
      - run: pnpm --dir web build
      - name: Cache headers
        run: |
          cat > web/dist/_headers <<'EOF'
          /assets/*
            Cache-Control: public, max-age=31536000, immutable
          /maps/*
            Cache-Control: public, max-age=3600, must-revalidate
          EOF
      - uses: cloudflare/wrangler-action@v3
        with:
          apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
          accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
          command: pages deploy web/dist --project-name=trapiks --branch=main
```

Each push to `main` deploys to `https://trapiks.pages.dev`. Deploying with any other `--branch` value creates a preview URL.

### Vercel

1. Install the CLI with `npm i -g vercel`. Run `vercel link` once from the repo root and create a new project. Set the framework to **Other**.
   - `vercel link` writes `.vercel/project.json`, which holds the `orgId` and `projectId`.
   - Don't commit `.vercel/`.
2. In the Vercel project settings, turn off automatic Git deployments (**Settings → Git**). Otherwise Vercel tries to build the repo itself and fails for lack of Rust.
3. Create a token under **Account Settings → Tokens**. Add these GitHub secrets:
   - `VERCEL_TOKEN`
   - `VERCEL_ORG_ID`
   - `VERCEL_PROJECT_ID`
4. Add `.github/workflows/vercel.yml`. Copy the steps from the Cloudflare workflow up to and including `pnpm --dir web build`, then finish with:

```yaml
      - name: Vercel config
        run: |
          cat > web/dist/vercel.json <<'EOF'
          {
            "headers": [
              {
                "source": "/assets/(.*)",
                "headers": [{ "key": "Cache-Control", "value": "public, max-age=31536000, immutable" }]
              },
              {
                "source": "/maps/(.*)",
                "headers": [{ "key": "Cache-Control", "value": "public, max-age=3600, must-revalidate" }]
              }
            ]
          }
          EOF
      - name: Deploy
        env:
          VERCEL_ORG_ID: ${{ secrets.VERCEL_ORG_ID }}
          VERCEL_PROJECT_ID: ${{ secrets.VERCEL_PROJECT_ID }}
        run: npx vercel@latest deploy web/dist --prod --yes --token=${{ secrets.VERCEL_TOKEN }}
```

Leave out `--prod` to get preview deployments, for example on pull requests.

## Option B: build locally and deploy from your machine

If the toolchain is installed (see `CLAUDE.md` → Gotchas), build first:

```bash
pnpm --dir web install --frozen-lockfile
pnpm --dir web build
pnpm --dir web exec vite preview
```

The last command is optional: it checks the build at http://localhost:4173.

Then deploy with one of these:

```bash
npx wrangler login
npx wrangler pages deploy web/dist --project-name=trapiks
```

```bash
npx vercel deploy web/dist --prod
```

## Option C: let the platform build (not recommended)

This only works if the platform's build command installs the toolchain first:

```bash
curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain 1.94.1 --target wasm32-unknown-unknown \
  && . "$HOME/.cargo/env" \
  && curl -sSL https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.129/wasm-bindgen-0.2.129-x86_64-unknown-linux-musl.tar.gz | tar -xz \
  && export PATH="$PWD/wasm-bindgen-0.2.129-x86_64-unknown-linux-musl:$PATH" \
  && curl -sSL https://github.com/WebAssembly/binaryen/releases/download/version_132/binaryen-version_132-x86_64-linux.tar.gz | tar -xz \
  && export PATH="$PWD/binaryen-version_132/bin:$PATH" \
  && corepack enable && pnpm --dir web install --frozen-lockfile && pnpm --dir web build
```

- Set the output directory to `web/dist` and the Node version to 22.
- Expect cold builds of 5–10 minutes. The platforms don't cache `~/.cargo` or `target/` between builds.

## Settings that matter

| Setting | Value | Why |
|---|---|---|
| Base path | Leave `TRAPIKS_BASE` unset; it defaults to `/` | Both hosts serve from the domain root. Set `TRAPIKS_BASE=/sub/` only when serving from a sub-path, as GitHub Pages does |
| `.wasm` content type | `application/wasm` | Both hosts do this automatically |
| `maps/*.bin.gz` | Served as a plain file | The worker detects gzip by its magic bytes. It works whether or not the host adds `Content-Encoding: gzip` |
| File sizes | The map is about 5.5 MB | Under the Cloudflare Pages limit of 25 MiB per file |
| Caching | `assets/` is immutable (hashed names); `maps/` revalidates | The map keeps its name when regenerated |

## After deploying

1. Open the site. The title screen should show "© OpenStreetMap contributors (ODbL)". The ODbL requires this attribution, together with `maps/LICENSE`, which is deployed with the map.
2. Start the Sandbox. The Metro Manila map should load in a few seconds, and vehicles should appear once you raise the demand slider.
3. `?map=synthetic` loads the small test city. It is handy for checking that a deploy works on slow connections.
