# llama.cpp passive verification

Aligned re-verification date: 2026-08-21
Aligned AI Engine Room source commit: `6fbd802c59d56ade4b37d73d8b6f50399ead92fe`

## Scope

This record captures bounded developer-operated verification of AI Engine Room's passive llama.cpp integration on the tested Ubuntu 24.04 LTS x86_64 baseline. It is evidence for one package/build/mode/endpoint contract, not a broad llama.cpp or platform compatibility claim.

Direct traditional single-model llama-server detection and served-model display are implemented for the tested llama.cpp Snap `b9969`, revision `307`, at the fixed same-machine endpoint `127.0.0.1:8080`. Bounded Ubuntu passive verification passed with an developer-operated existing server. One passive Refresh preserved the observed server/model state. No llama.cpp inference or model/server management was performed.

No llama.cpp inference or POST request ran. AI Engine Room performed no server, model, process, filesystem, backend, or configuration management. Clipboard was not used during the aligned re-verification.

## Tested provider contract

| Item             | Tested value                          |
| ---------------- | ------------------------------------- |
| Environment      | Ubuntu 24.04 LTS x86_64               |
| Distribution     | Snap `llama-cpp`, `latest/stable`     |
| Version          | `b9969`                               |
| Snap revision    | `307`                                 |
| Mode             | Traditional single-model llama-server |
| Endpoint         | `http://127.0.0.1:8080`               |
| Served-model ID  | `lfm2.5-1.2b`                         |
| Provider marker  | `llamacpp`                            |
| API scope        | `same_machine_loopback`               |
| Compute location | `not_independently_verified`          |

The developer operated the existing server. AI Engine Room did not start, stop, configure, discover, or manage it. The served-model ID is the validated provider-reported `data[].id` from this test only; it is not a product-required model name. No model path is needed or retained.

## Aligned passive re-verification

Before the application Refresh, the developer observed:

- `GET /health` returned HTTP 200 with `status = ok`.
- `GET /v1/models` reported exactly one `data` entry.
- The entry reported `id = lfm2.5-1.2b` and `owned_by = llamacpp`.
- An existing `llama-server` process was bound to `127.0.0.1:8080`.

After one AI Engine Room passive Refresh, the developer observed:

- the same server process remained bound to `127.0.0.1:8080`;
- `GET /health` still reported `status = ok`;
- `GET /v1/models` still reported exactly one `data` entry with `id = lfm2.5-1.2b` and `owned_by = llamacpp`.

The developer also confirmed in the application:

- llama.cpp displayed **Ready**;
- the served-model ID displayed as `lfm2.5-1.2b`;
- no filesystem path appeared;
- Ollama and LM Studio remained independently visible as applicable;
- no inference occurred;
- no provider, model, or server management occurred;
- Report remained provider-free and unchanged;
- no raw HTTP, error, or endpoint text appeared in the UI;
- keyboard order, visible focus, narrow layout, and state readability/accessibility checks passed.

No raw JSON, private model path, full process command line, provider log, or generated output is retained in this evidence. The matching bounded pre/post observations support passive behavior for this tested build and traditional single-model configuration; they do not establish a universal side-effect guarantee for other builds or configurations.

## Historical verification retained

The initial bounded verification occurred on 2026-08-20 against pre-alignment AI Engine Room source commit `fc78e6b9d4155a6f003af55c6aece1ffc3b8866f`. That event used the same Snap label `b9969`, revision `307`, traditional single-model mode, fixed endpoint, and provider-reported identity. It additionally recorded server version `1` and server build commit `76f2798`.

During that earlier event, the developer manually started the existing server and model, confirmed ready health and one served-model identity before and after the application check, and confirmed Ready display, no path exposure, no llama.cpp inference or management, provider coexistence, unchanged Report, no clipboard use, and bounded keyboard/layout/readability checks. The earlier event remains historical evidence.

The 2026-08-21 event supersedes only the parser/source verification reference: it verifies the aligned `data[].id`/`model_id` implementation at commit `6fbd802c59d56ade4b37d73d8b6f50399ead92fe`. It does not erase or rewrite the earlier event.

## Privacy and compute qualification

AI Engine Room accepts only a validated provider-reported served-model ID. It does not display, derive, hash, truncate, normalize, or persist filesystem paths as model identity. Unsafe or incompatible identity produces a controlled incompatible state.

The tested API was reached through same-machine loopback. This does not independently verify CPU, GPU, backend, offload, or other compute placement. No such placement claim is made.

## Limitations

Broader llama.cpp versions, router/multi-model mode, authentication, TLS, custom endpoints/ports, LAN/remote access, Windows llama.cpp support, model compatibility, inference, and compute placement are not established.

This verification does not establish:

- full llama.cpp support or compatibility with arbitrary GGUF models;
- llama.cpp inference through AI Engine Room;
- router or multi-model support;
- authentication, TLS, custom endpoint/port, LAN, or remote support;
- server or model management;
- compatibility with other llama.cpp packages, builds, configurations, platforms, or Ubuntu systems;
- Windows llama.cpp support;
- local compute placement, performance, benchmark validity, production readiness, or release-candidate status.

The browser/mock path remains artificial-fixture only. Report and clipboard boundaries are unchanged.
