/// <reference types="vitest/config" />
import { defineConfig } from 'vite'
import { svelte } from '@sveltejs/vite-plugin-svelte'
import { svelteTesting } from '@testing-library/svelte/vite'

// The frontend dev server binds to loopback by default and never binds all
// interfaces. LAN preview is an explicit, developer-initiated mode: the
// developer supplies the private interface address to bind (and optionally a
// free port) through non-client-exposed env vars (AER_*; not the VITE_ prefix,
// so nothing is leaked into the browser bundle). No private IP, subnet,
// hostname, or current dev-machine address is committed here.
//
// `strictPort` is false so the server tolerates a free port when 5173 is busy
// (an unrelated process may occupy it); Vite logs the chosen port. The Tauri
// devUrl default remains loopback:5173; native launch is conditional on a
// graphical session (see docs/dev/lan-preview.md).

const lanPreview = process.env.AER_LAN_PREVIEW === '1'
const lanHost = process.env.AER_HOST
const lanPort = process.env.AER_PORT ? Number(process.env.AER_PORT) : 5173

if (lanPreview && !lanHost) {
  throw new Error(
    'LAN preview is explicit: supply the private interface address to bind. ' +
      'Example: AER_HOST=192.168.1.10 AER_LAN_PREVIEW=1 npm run dev:lan ' +
      '[AER_PORT=5173]. The default `npm run dev` stays loopback-only and ' +
      'never binds all interfaces.',
  )
}

export default defineConfig({
  plugins: [svelte(), svelteTesting()],
  server: {
    // `false` = loopback only. LAN mode binds the developer-supplied address.
    host: lanPreview ? lanHost : false,
    port: lanPreview ? lanPort : 5173,
    strictPort: false,
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/tests/setup.ts'],
    include: ['src/**/*.{test,spec}.ts'],
  },
})