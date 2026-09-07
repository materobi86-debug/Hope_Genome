# HERMES SERVERLESS — WASM & Browser Capability Matrix

A **HERMES SERVERLESS** egy 100%-ban böngészőben futó, **WASM-first**, **local-first** digitális társ.

## Browser / Native Capability Boundary

| Capability | Scope / Adapter | Browser Status | Notes / Limitations |
| :--- | :--- | :--- | :--- |
| **Memory Recall & Store** | `WasmMemoryAdapter` (Microscope) | **REAL WASM** | Működik local WebAssembly memóriablokkokkal. |
| **Agent State Machine** | `AgentWorker` (Web Worker) | **REAL WORKER** | Non-blocking háttérszálon futó determinisztikus ciklus. |
| **Event Bus & Spine** | `BrowserSpineBus` | **REAL BROWSER** | Monoton sorszámozású, deduplikált typed event bus. |
| **Policy Engine** | `BrowserOctopusPolicy` | **REAL POLICY** | Capability, scope, user approval és evidence ellenőrzés. |
| **Evidence Verification** | Web Crypto API (SHA-256) | **REAL NATIVE API** | Kriptográfiai igazolás a műveletek eredményéről. |
| **Persistence & Snapshots** | OPFS & IndexedDB | **REAL BROWSER** | Snapshots, session audit log és bináris munkaterület tárolás. |
| **User File Access** | File System Access API | **REAL BROWSER** | Kizárólag az operátor által kijelölt mappára/fájlra. |
| **OS Process Execution** | `process.spawn` / Shell | **UNSUPPORTED** | A böngésző sandbox nem ad közvetlen shell hozzáférést. |
| **Arbitrary Native IPC** | `native.mmap` | **UNSUPPORTED** | A böngésző memóriakorlátai miatt nem szimulált. |

---
**BROWSER-ONLY · WASM AGENT · SANDBOXED TOOLS · EXPLICIT CONTROL**
