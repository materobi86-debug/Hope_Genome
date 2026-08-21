# 🧬 Bio-Binaries — Rust Workspace

Biomimetikus rendszermodulok Rust workspace-je: **21 bio-inspirált crate** + közös primitívek (`bio-shared`) + protokoll-híd (`bio-bridge`), amely a modulokat az eredeti [Bio-Binaries](https://github.com/silentnoisehun/Bio-Binaries) v2 **BioMessage** protokolljához köti.

Minden modul egy biológiai elvet fordít le rendszer-szintű mechanizmusra: crash-álló snapshot, agy nélküli útvonal-optimalizálás, adaptív immunitás, hot-patching, önorganizáció, rejtett tárolás, graceful self-destruct.

---

## Architektúra

```text
┌────────────────────────────────────────────────────────────────┐
│                        bio-bridge                              │
│  21 modul egyetlen BioBridge "testbe" szerelve                  │
│  modul-esemény → BioOp → BioMessage v2 wire bájt               │
│  (vendorelt bio_protocol: CRC64 + BLAKE3 auth + nonce replay)  │
└──────────────┬─────────────────────────────────────────────────┘
               │ UDP (kompatibilis az omega-master 8888 portjával)
┌──────────────┴─────────────────────────────────────────────────┐
│                        bio-shared                              │
│  BLAKE3 hash · SealedFrame (tamper-proof) · BioMessage envelope│
└──────────────┬─────────────────────────────────────────────────┘
               │
   ┌───────────┼───────────────────────────────┐
   │           │                               │
┌──┴───┐   ┌───┴────────────┐          ┌───────┴──────┐
│Túlélés│   │Hálózat/Erőforrás│         │Védelem/Álcázás│
└───────┘   └────────────────┘          └──────────────┘
```

---

## Modulok

### Túlélés és állapot

| crate | biológiai elv | mechanizmus |
|---|---|---|
| `tardigrade-tun` | Medveállatka anhydrobiosis | Futási állapot (thread-stack, memória-watchdog, nyitott UDP workflow-k) BLAKE3-lel szilárdított Tun-Frame snapshotba dermesztése; újrainduláskor `revive()` |
| `blastema-regen` | Axolotl végtag-regeneráció | Sérült blob újjáépítése XOR-paritás szilánkokból (k shard → rekonstrukció) |
| `yamanaka-deaging` | Yamanaka-tényezők | Szeneszcens állapot (töredezettség, szivárgás, drift) nullázása a tanult mátrixok megtartásával, újraindítás nélkül |
| `apoptosis-cascade` | Programozott sejthalál | Kompromittálódásnál kulcsok zeroize-álása + szomszédok értesítése a maradványok eltakarítására |

### Hálózat és erőforrás

| crate | biológiai elv | mechanizmus |
|---|---|---|
| `physarum-path` | Sárga nyálkagomba | Cső-vezetőképesség oszcilláció: lassú/csomagvesztő útvonalak visszaszorítása, leggyorsabbak erősítése — központi vezérlés nélkül |
| `quorum-signal` | Baktérium kvórum-érzékelés | Erőforrás-igényes művelet csak akkor indul, ha a peer-szavazatok elérik a küszöböt |
| `mycorrhiza-trade` | Fagomba hálózat (Wood Wide Web) | CPU/RAM headroom átvállalása a terhelt edge-csomópontoktól a bőséges csomópontok felé |
| `symbiont-engine` | Endoszimbiózis (mitokondrium) | Heterogén eszközök (GPU/NPU/MCU) integrálása; mátrix/FFT feladatok kiosztása, eredmény BLAKE3 verifikációval |
| `electrocyte-burst` | Elektromos angolna | Sok kis rész-eredmény szinkronizált összegzése egyetlen számítási tüskévé |
| `magnetosome-nav` | Magnetotaktikus baktériumok | Célponthoz orientálódás a legolcsóbb gradiens mentén (késleltetés + sávszélesség) |
| `biophoton-telepathy` | Biophoton koherencia | Memórialapok tükrözése verziókövetéssel (a tényleges zero-copy transzport RDMA/PCIe backend mögött) |
| `morpho-electric-mesh` | Bioelektromos morfogenezis (Levin) | Csomópontok ön-differenciálódása szív/tüdő/ideg szerepre feszültség-gradiensből, omega-master nélkül |

### Védelem, álcázás, memória

| crate | biológiai elv | mechanizmus |
|---|---|---|
| `tcell-sentinel` | Adaptív immunrendszer | Fenyegetés-szignatúrák (memóriaszivárgás, csomag-anomália) rögzítése globális immunitási adatbázisban |
| `crispr-patch` | CRISPR génszerkesztés | Sérült kódvonalak karanténba zárása/letiltása hot-patch registry-n át, újraindítás nélkül |
| `plasmid-conjugate` | Horizontális géntranszfer | Képességek (kulcsok, szűrőmátrixok) P2P átadása BLAKE3 integritás-ellenőrzéssel, központi szerver nélkül |
| `transposon-hive` | Retrovirális transzposonok | Megoldott feladatok bytecode-képességként való beégetése a hive genomjába (eBPF/wasm codec) |
| `epigenetic-switch` | Epigenetika | A bináris self-hash érintetlen; a futásidejű metilációs profil kapcsol Stealth/Hyperdrive/Fortress módok között |
| `chameleon-stealth` | Tintahal kromatofórák | BioMessage csomagok DNS/NTP/HTTPS utánzatú álcázása, természetes port-poolból |
| `mycelial-void-vault` | Gombaspórák rejtőzködése | Titkok elrejtése zaj-hordozóban (BFSK-szerű XOR-moduláció), külső megfigyelő számára láthatatlanul |
| `xenobot-swarm` | Xenobot raj-intelligencia | Elárvult temp fájlok, naplómaradványok, memóriaszivárgások autonóm összegyűjtése |
| `chimera-fusion` | Kiméra szuper-organizmus | Python/C++/Wasm/eBPF modulok bekebelezése egyetlen dispatch-felületbe |

### Közös réteg

| crate | szerep |
|---|---|
| `bio-shared` | BLAKE3 hash-segédek, `SealedFrame` (magic + kind + len + 32 B digest, tamper-detektálás), `BioMessage` envelope + `ReplayGuard` |
| `bio-bridge` | A 21 modul egyetlen `BioBridge` struktúrába szerelve; modul-esemény → `BioOp` leképezés; valós v2 wire-bájtok generálása |

---

## Protokoll (bio-bridge)

A `bio-bridge/src/protocol.rs` az eredeti Bio-Binaries `bio_protocol.rs` vendorelt másolata — **byte-azonos** wire-formátum, így az `omega-master` (UDP 8888) változtatás nélkül fogadja az üzeneteket.

```text
Header (60 bájt):
[magic:2][op:1][flags:1][generation:4][nonce:8][auth_tag:32][payload_len:4][checksum:8]
```

- **CRC64** (ECMA-182) minden üzeneten
- **BLAKE3 keyed hash** auth tag (32 bájt)
- **nonce** (ns időbélyeg) + `NonceWindow` replay-védelem

Modul-esemény leképezések:

| modul | BioOp |
|---|---|
| tardigrade | `FREEZE` (0x40) |
| crispr | `CRISPR_PATCH` (0x60) |
| sentinel | `IMMUNE_ALERT` (0x61) |
| apoptosis | `APOPTOSIS` (0x3F) |
| physarum | `TASK` (0x11) |
| morpho | `HOMEO_SYNC` (0x80) |

---

## Build és teszt

```powershell
# Teljes workspace fordítása
cargo check --workspace

# Összes teszt (59 teszt a 22 crate-ben)
cargo test --workspace

# Csak a bridge (protokoll roundtrip + integrációs tesztek)
cargo test -p bio-bridge
```

### Eredeti Bio-Binaries repó

A klón a `_biorepo/` mappában van, saját workspace-ként (a fő workspace `exclude`-olja):

```powershell
cd _biorepo
cargo check        # 33 bináris, 0 hiba
cargo run --release --bin omega-master -- status
```

---

## Teszt-lefedettség

| terület | tesztek |
|---|---|
| bio-shared (frame tamper/truncation, replay, hash) | 6 |
| bio-bridge protokoll (roundtrip, checksum, auth, nonce window) | 9 |
| bio-bridge integráció (module mount, op mapping, wire encode, JOIN) | 4 |
| bio-bridge end-to-end (`tests/integration.rs`: immune cascade, freeze/revive, apoptosis, quorum gating, Wood Wide Web, epigenetika) | 7 |
| modulok (physarum útvonal-konvergencia, blastema paritás, quorum, mycorrhiza, plasmid, vault, …) | 33 |
| **összesen** | **59** |

---

## Biztonsági megjegyzések

- A `SealedFrame` és a BioMessage CRC64+BLAKE3 integritása **tamper-detektálást** ad, nem titkosítást.
- A `mycelial-void-vault` egyszerűsített szteganográfiai modell (XOR-moduláció); éles használat előtt valódi hordozó-rejtés és kulcskezelés kell.
- A `transposon-hive` genom-szerializációt ad; a tényleges önmódosító bináris viselkedés eBPF/wasm runtime mögött valósul meg (a legtöbb OS memóriavédelme a futó kód írását tiltja).
- Az `apoptosis-cascade` zeroize-álja a kulcsokat memóriában; a RAM fizikai maradványaira (cold-boot) nem ad garanciát.

---

## Státusz

- ✅ 22 crate fordul (`cargo check --workspace` → 0 hiba)
- ✅ 45 teszt zöld (`cargo test --workspace` → exit 0)
- ✅ `_biorepo` klón külön fordítható (33 bináris)
- ✅ bio-bridge wire-kompatibilis az omega-master protokolljával
