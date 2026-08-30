# Binary-distribution licence and notice review

Status: initial review and the separately approved remediation gate completed on
2026-08-30. The rebuilt licence payloads are accepted; signing, publisher
identity, upload, hosting, publication, distribution, and release remain
separate gates and are not authorized by this record.

This is a bounded engineering publication review, not legal advice or a
comprehensive legal-compliance certification. It changes no application,
package, dependency, lockfile, licence, or distribution state.

## Scope and exact evidence

The review record was prepared from clean `main` at:

- commit `29275cd2ed5033c6b7f57d878d30c4dcd70d3ce5`;
- tree `3656fe67c8d9a1a97b4ee9491c675d75ccb031aa`; and
- sole origin `https://github.com/gregweir/ai-engine-room.git`.

The reviewed packages were built from application-source commit
`2293b336eaa314f4fd285737a8470a5b9abd151a`, tree
`6f5df75b612f8b159f172a05766ab5bd2bec491f`. Later commits through the review
record changed documentation only. The exact local, unsigned, unpublished
artifacts were:

| Platform | Artifact | Size | SHA-256 |
| --- | --- | ---: | --- |
| Ubuntu 24.04 x86-64 | `AI Engine Room_0.1.0_amd64.deb` | 4,276,214 bytes | `f32531f9f22ea6dfff2fdc8a8b2631544fdfc5c9d0017bc0f012c0c5e3083d17` |
| Windows 11 x86-64 | `AI Engine Room_0.1.0_x64-setup.exe` | 2,535,913 bytes | `2811b704a6ac23f3069b476b9afc4eba58cf82952d8b166f750b11167d676581` |

The review used read-only package/archive inspection, locked npm metadata, and
target-specific Cargo dependency trees. It did not install, launch, rebuild,
sign, upload, publish, or distribute either artifact. It did not access a
provider or run inference.

## Governing publication references

The engineering release bar adopts the following primary-source requirements:

- Apache License 2.0 section 4 requires recipients to receive a copy of the
  licence and preserves applicable notices, including NOTICE attributions when
  present: <https://www.apache.org/licenses/LICENSE-2.0>.
- Debian Policy section 2.3 requires a package's distribution licences in
  `/usr/share/doc/PACKAGE/copyright` and addresses notices for files compiled
  into shipped object code:
  <https://www.debian.org/doc/debian-policy/ch-archive.html#copyright-considerations>.
  The optional machine-readable format is documented at
  <https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/>.
- MPL 2.0 section 3.2 requires the covered source corresponding to distributed
  executable form to be available and recipients to be told how to obtain it:
  <https://www.mozilla.org/en-US/MPL/2.0/>. Mozilla's FAQ applies that rule to
  externally distributed executable programs and libraries compiled from
  unchanged MPL source: <https://www.mozilla.org/en-US/MPL/2.0/FAQ/>.
- Unicode License v3 requires its copyright and permission notice to accompany
  copies or associated documentation: <https://www.unicode.org/license.txt>.
- NSIS identifies its general licence as zlib/libpng, but identifies its LZMA
  compression module as Common Public License 1.0; the published CPL text
  includes object-code source-availability conditions. Its explicit special
  exception permits static or dynamic linking to that module without applying
  CPL 1.0 to the linked product; modifications or additions to the module remain
  subject to CPL 1.0:
  <https://nsis.sourceforge.io/License>.

These sources establish the minimum review inputs. Each exact locked component's
own distributed licence and notice files remain authoritative for the final
generated materials.

## Findings

### Application material

The repository has the full Apache-2.0 text in `LICENSE` and an application
`NOTICE` naming `Tartanleaf.com Inc.` and 2026. The application package metadata
also declares Apache-2.0. Neither reviewed binary artifact carries those files
in its installed payload.

The workspace crate `aer-core` is the sole Cargo component in the Windows graph
without declared licence metadata. Its source is part of this Apache-2.0
repository, but `crates/aer-core/Cargo.toml` does not currently declare that
licence. This is a metadata defect to correct before generating a dependable
manifest.

### Frontend material

The production frontend compiles code from the locked packages Svelte 5.56.9
(MIT), `@tauri-apps/api` 2.11.1 (Apache-2.0 OR MIT), and
`@tauri-apps/plugin-clipboard-manager` 2.3.2 (MIT OR Apache-2.0). Svelte's
placement under `devDependencies` does not make its compiled production code a
build-only publication input. The final inventory must use shipped-code scope,
not npm dependency-section labels alone.

### Linux package

Read-only `.deb` inspection found only `/usr/bin/aiengineroom`, a desktop entry,
and three icon files. It found no `/usr/share/doc/ai-engine-room/copyright`,
application `LICENSE`, application `NOTICE`, third-party licence/attribution
inventory, or covered-source availability notice.

The target-specific locked Cargo tree produced a conservative 299-component
candidate inventory. Its declared expressions include permissive licences,
Unicode-3.0, and five MPL-2.0 components: `cssparser-macros 0.6.1`,
`cssparser 0.36.0`, `dtoa-short 0.3.5`, `option-ext 0.2.0`, and
`selectors 0.36.1`. A Cargo tree is not proof that every listed build or macro
component's object code is present in the final executable; final generation
must distinguish shipped runtime code from build-only inputs.

The package dynamically links to Ubuntu-provided GTK, WebKitGTK,
JavaScriptCore, libsoup, GStreamer, and other host libraries. Those host
packages were not archived into this `.deb`; they must remain distinguished
from statically compiled or embedded application dependencies.

### Windows installer

Read-only NSIS archive inspection found the installed `aiengineroom.exe` and
installer components named `System.dll`, `modern-wizard.bmp`, `nsDialogs.dll`,
`nsis_tauri_utils.dll`, `StartMenu.dll`, and `NSISdl.dll`. It found no payload
filename identifying an application licence, NOTICE, third-party
licence/copyright inventory, attribution, or source-availability file. The
final inventory must also establish the bitmap's provenance and licence. Static
string inspection found no identifiable substantive licence page, though that
read-only method cannot conclusively rule out every dynamically rendered page.

The installer identifies NSIS 3.11 and LZMA solid compression. Therefore the
final distribution inventory must include the NSIS and `nsis_tauri_utils`
components actually carried by the installer and must establish and satisfy
the applicable module-level LZMA/CPL source-availability terms. The NSIS special
exception avoids CPL propagation to the linked product, but it does not make
the carried module disappear from the distribution inventory. Treating bundler
helpers as outside the distributed artifact is not sufficient.

The Windows target-specific locked Cargo tree contained 253 unique nonblank
components after Cargo duplicate-marker removal. Apart from `aer-core`, all had
a declared expression. It included the same five MPL-2.0 components and 18
components whose exact expression is Unicode-3.0, plus `unicode-ident 1.0.24`
whose expression is `(MIT OR Apache-2.0) AND Unicode-3.0`; 19 expressions
therefore include Unicode-3.0. The graph also included MIT, Apache-2.0, BSD,
Zlib, Boost, Unlicense, CC0/MIT-0, and 0BSD choices. Alternative expressions are
not final licence selections; a reproducible process must select a permitted
branch and preserve the corresponding texts and required notices.

## Remediation acceptance evidence

The separately approved implementation gate was completed on clean public
`main` at commit `2fa639dfee75f81ea63b98d3d00086708e3e571a`, tree
`dbe9209ef8b6791488a678c268b968892fd0cfc0`, with the sole origin
`https://github.com/gregweir/ai-engine-room.git`. The implementation:

- records a checksum-verified conservative inventory of 471 locked Rust
  registry components, the 22-package locked production frontend dependency
  closure, and the NSIS 3.11 plus `nsis_tauri_utils 0.5.3` installer baseline;
- pins non-lock inputs by immutable revision and SHA-256, including SPDX licence
  texts, both installer source archives, and the NSIS wizard bitmap provenance;
- generates `THIRD-PARTY-LICENSES.txt`, `THIRD-PARTY-SOURCES.txt`, Debian
  `copyright`, and a self-verifying provenance manifest;
- declares `aer-core` as Apache-2.0 and packages application `LICENSE`, `NOTICE`,
  third-party licence material, and source-location material in both formats;
- provides Debian documentation under `/usr/share/doc/ai-engine-room`; and
- makes deterministic CI regenerate and compare the materials, verify both
  exact target dependency graphs, build real `.deb` and NSIS packages, extract
  them, and compare their licence payloads with the accepted source files.

GitHub Actions run
<https://github.com/gregweir/ai-engine-room/actions/runs/33306199586> passed all
five jobs: frontend and repository contracts, Rust on Ubuntu and Windows, and
actual Linux and Windows package-licence verification.

The exact rebuilt, unsigned, unpublished host artifacts accepted by this gate
are:

| Platform | Build/review host | Artifact | Size | SHA-256 |
| --- | --- | --- | ---: | --- |
| Ubuntu 24.04 x86-64 | Linux verification host | `AI Engine Room_0.1.0_amd64.deb` | 4,722,944 bytes | `0fe182d2680db444c936870e3a6c68e1345963a34054463610ed8749e148b256` |
| Windows 11 x86-64 | Independent Windows review host | `AI Engine Room_0.1.0_x64-setup.exe` | 2,649,494 bytes | `99020327a743493ff460d0ad694d6d99483021d12304ea673d85ae3ab813b09e` |

Linux package verification confirmed byte-identical installed licence resources
and the three Debian documentation files. Independent Windows review confirmed
all seven declared installer payloads, byte-identical application and
third-party licence resources, and a wizard bitmap matching the hash derived
from pinned NSIS source. Neither host installed, launched, signed, uploaded,
published, or distributed the rebuilt artifact, and neither accessed a provider
or ran inference.

## Gate decision

**The binary licence-and-notice remediation gate passes.** The earlier artifact
hashes remain rejected for publication and cannot be substituted for the
rebuilt artifacts above. This acceptance establishes package-content evidence;
it does not itself authorize signing, upload, hosting, publication,
distribution, or release.

The approved implementation gate was evaluated against these criteria:

1. Define a reproducible, locked, platform-specific shipped-component inventory
   covering Rust code, compiled frontend code, Tauri components, and installer
   components, while excluding verified build-only and host-provided libraries.
2. Correct the missing `aer-core` licence metadata and make explicit,
   consistent choices for dual- or multi-licensed dependencies.
3. Generate and review complete third-party copyright, attribution, licence-text,
   and source-availability material from the exact locked versions. Preserve
   upstream NOTICE content where applicable.
4. Provide timely source availability and clear recipient instructions for the
   exact MPL-covered source, and resolve the corresponding NSIS LZMA/CPL source
   obligation before distributing the Windows installer.
5. Include the application's `LICENSE` and `NOTICE` plus the reviewed
   third-party material in both installed products. The `.deb` must include
   `/usr/share/doc/ai-engine-room/copyright`; the Windows installation must make
   the same material readily accessible to recipients.
6. Add deterministic repository checks that fail when generated material is
   stale, incomplete, or absent from either package.
7. Rebuild both formats from one newly accepted source identity, inspect their
   actual payloads, record new sizes and hashes, rerun deterministic CI, and
   obtain independent review. The current artifact hashes cannot be carried
   forward after payload changes.

Signing and publisher identity are the next separate gate. Upload, hosting,
publication, distribution, and public release remain separately prohibited
until explicitly approved.
