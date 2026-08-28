# LAN preview (development only)

The frontend dev server binds to **loopback only** by default. `npm run dev`
never exposes the app on a network interface. There is no setting, and no
default, that binds all interfaces.

LAN preview is an **explicit, opt-in** mode for viewing the browser build on
another device on your local network during development. It is a *preview* of
the frontend only — it does not start, and cannot reach, the Tauri/Rust
backend.

## What LAN preview exposes

Only these, served by Vite over HTTP:

- The frontend dev assets (HTML, JS, CSS).
- Clearly artificial mock/preview fixture data.

The browser data source selects the mock/fixture path automatically in dev
mode (see `src/lib/datasource`). It does not call Tauri `invoke`, does not
contact Ollama or any model server, and does not read real system telemetry.

## What LAN preview never exposes

- Tauri IPC or the Rust backend (no native runtime is loaded in the browser).
- Real system metrics, Ollama, GPU, or runtime data.
- The filesystem, environment variables, or any machine identifier.
- Privileged commands, native internals, or session internals.

The Vite dev server serves only files under the project root. No `VITE_`-prefixed
secrets are used; the LAN flag and bind address use the non-client-exposed
`AER_*` env vars, read in `vite.config.ts` (server-side), so they never enter
the browser bundle.

## Running it

Supply the private interface address you want to bind. Optionally supply a free
port (5173 is the preferred port; if it is busy, Vite picks the next free port
and logs it).

```sh
AER_HOST=<your-private-address> AER_LAN_PREVIEW=1 npm run dev:lan
# optional: AER_PORT=<free-port>
```

Then open `http://<your-private-address>:<chosen-port>` from another device on
the same local network. Vite prints the chosen address and port on startup.

If you omit `AER_HOST`, the command refuses to start — it never falls back to
binding all interfaces.

If a port is already in use by an unrelated process, do not stop or alter that
process. Pass a free port with `AER_PORT`, or let Vite pick the next free one.

## Verification

From a second device, the page should show the mock/preview banner ("Showing
mock / preview fixture data") and representative metric states. No real
system values, paths, or errors are shown.

Loopback preview (default) is the same content, reachable only from this
machine:

```sh
npm run dev
# open http://localhost:<chosen-port>
```

## What this is not

This is not a production server, a tunnel, a public preview, or a way to share
the native app. It does not configure your router, firewall, or any public
service. It is a local-network, development-only view of the frontend with
fixture data.