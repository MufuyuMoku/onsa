# Progres

Status tiap milestone dari `SPEC.md` §15. Diperbarui di akhir setiap milestone.

---

## M0. Kerangka proyek — selesai (2026-09-11)

**Kriteria selesai**: jendela kosong bertema terbuka di kedua OS, dan CI hijau.

| Kriteria | Windows 11 | Linux |
|---|---|---|
| Jendela kosong bertema terbuka | ✅ terverifikasi | ⏳ belum diverifikasi |
| CI hijau (lewat skrip lokal) | ✅ `scripts/check.ps1` dan `scripts/check.sh` (Git Bash) lulus | ⏳ belum diverifikasi |

### Yang dibuat

- Repositori git lokal (`main`) dengan `.gitignore` dan `.gitattributes`. Belum ada remote.
- Identifier aplikasi `io.github.mufuyumoku.onsa` di SPEC §1 dan `tauri.conf.json`.
- Workspace Cargo sesuai §2: `onsa-audio`, `onsa-library`, `onsa-downloader`, `onsa-lyrics`, `onsa-scrobble`, `onsa-cli`, dan `src-tauri`. Crate modul baru berisi tipe error; isinya menyusul di milestone masing-masing.
- `src-tauri`: jendela Tauri 2 dengan CSP ketat, capability minimal (`core:default`), command `app_info`, `theme_list`, `theme_get`, dan log `tracing` ke stderr (filter lewat `ONSA_LOG`).
- Pemuat tema (stub): tiga tema bawaan dikompilasi ke binary, dibaca dengan parser toleran (field hilang atau salah jadi default dan dilaporkan sebagai peringatan).
- `ui/`: SvelteKit (Svelte 5 runes, TypeScript strict, `adapter-static`, SPA). Tema diterapkan sebagai CSS variables `--onsa-*` dan atribut `data-onsa-*`. Kamus i18n `id` dan `en`, default mengikuti locale OS. Panel M0 sengaja kosong, hanya ada pilihan tema dan bahasa untuk memverifikasi pemuat.
- Ikon dari placeholder `assets/icon.svg` (ukuran Windows dan Linux saja).
- Workflow GitHub Actions `.github/workflows/ci.yml` (matriks `windows-latest` dan `ubuntu-latest`), plus `scripts/check.ps1` dan `scripts/check.sh`.
- `README.md` (cara build, dependensi sistem Linux), `docs/DECISIONS.md`, dan file ini.

### Cara verifikasi

- **Jendela bertema (Windows)**:
  - `tauri dev`: jendela "Onsa" terbuka. DOM di webview dibaca lewat port remote debugging WebView2: `data-onsa-theme="kaca-asap"`, `data-onsa-panel-texture="glass"`, `--onsa-role-active: #5CF2CF`, bahasa `en` sesuai locale OS.
  - Ketiga tema diganti lewat tombol, lalu di-screenshot dari webview. Kaca asap (kaca, teal), Kokpit kaca (bezel, hijau/putih), dan Deck malam (logam sikat, LED biru, label kapital) semuanya tampil sesuai tokennya.
  - Build rilis `tauri build --no-bundle` dijalankan tanpa dev server: halaman `http://tauri.localhost/` termuat dari frontend yang dibundel, dengan tema Kaca asap.
- **CI hijau (Windows)**: `scripts/check.ps1` dan `sh scripts/check.sh` sama-sama selesai dengan "All checks passed": `svelte-check` 0 error 0 warning, `cargo fmt --check` bersih, `cargo clippy --workspace --all-targets -- -D warnings` bersih, `cargo test --workspace` 11 tes lulus.

### Tertunda / belum diverifikasi

- **Linux belum diverifikasi**: jendela bertema dan skrip `scripts/check.sh` belum dijalankan di Linux.
- **GitHub Actions belum pernah berjalan** karena repositori belum punya remote. Workflow ditulis supaya langsung aktif setelah push.
- Font tema belum dibundel (M4, lihat DECISIONS).
- Log ke file bergulir (M4).
- Tema buatan pengguna di `<app_config>/themes/` dan penyimpanan pilihan tema (M4).

---

## M1. Mesin audio inti (tanpa UI) — selesai (2026-09-11)

**Kriteria selesai**: `onsa-cli` memutar daftar file secara gapless di kedua OS, dan tes gapless, crossfade, serta micro-fade (§12) lulus.

| Kriteria | Windows 11 | Linux |
|---|---|---|
| `onsa-cli` memutar daftar file secara gapless | ✅ terverifikasi (WASAPI) | ⏳ belum diverifikasi (ALSA) |
| Tes gapless, crossfade, micro-fade lulus | ✅ lewat sink offline | ⏳ akan dijalankan CI Ubuntu setelah push |
| Skrip pemeriksaan (`check.ps1`, `check.sh`) | ✅ hijau | ⏳ belum diverifikasi |

### Yang dibuat

- `onsa-audio`:
  - Decode lewat symphonia 0.6 (FLAC, MP3, AAC/M4A, ALAC, OGG Vorbis, WAV, AIFF; Opus di M11), dengan pemangkasan delay/padding encoder untuk gapless, seek akurat per sampel, dan pembacaan tag album/nomor trek.
  - Resampler streaming di atas rubato 5 dengan preset Cepat/Seimbang/Terbaik. Jumlah frame keluaran selalu tepat.
  - Pemetaan kanal (mono, stereo, downmix 5.1/7.1 ke stereo tanpa clipping).
  - Output stage real-time safe di callback cpal: ring buffer `rtrb`, komunikasi hanya lewat atomics, tanpa alokasi/lock/I/O/log. Micro-fade 60 ms saat pause, resume, seek, dan stop.
  - Sink perangkat (semua format sampel cpal) dan sink offline (dipakai semua tes).
  - Thread mesin: antrean, pre-roll, *lane* untuk gapless (termasuk saat resampling), crossfade equal-power/linear 0–12 detik, aturan album berurutan, crossfade 0,3 detik saat skip, seek, stop, event (lagu mulai, posisi ≤ 10 Hz, status, seek, gagal, antrean habis, output), pemulihan saat perangkat hilang dengan posisi dipertahankan, dan mengikuti perangkat default sistem.
- `onsa-cli`: `devices`, `play`, `queue` (file atau `--list`), `render-wav`, dengan kontrol keyboard berbasis baris.

### Cara verifikasi

- **Tes otomatis** (`cargo test --workspace`): 34 tes lulus, yaitu 11 di `src-tauri`, 13 unit dan 10 integrasi di `onsa-audio`. Tes integrasi memakai fixture yang dibuat saat tes berjalan di folder berisi aksara Jepang dan spasi:
  - Gapless: dua potongan satu sinus, output sama persis dengan sinus utuh (galat < 1e-6). Dengan resampling 44,1 → 48 kHz, output sama dengan file utuh (galat < 1e-5).
  - Crossfade: level di titik silang 0,3536 untuk equal-power (0,5 × 0,7071) dan 0,25 untuk linear, durasi ≈ 1 detik. Lagu berurutan satu album tidak di-crossfade.
  - Micro-fade: tidak ada lompatan sampel saat pause/resume, seek, dan skip (langkah maksimum ≤ langkah alami sinus). Seek mendarat tepat di sampel tujuan.
  - Pergantian output (simulasi perangkat berganti ke 44,1 kHz) melanjutkan dari posisi yang sama. File rusak dilaporkan lalu dilewati.
- **Perangkat nyata (Windows, WASAPI)**: `onsa-cli queue` memutar dua potongan sinus 44,1 kHz (di-resample ke 48 kHz) lewat perangkat VB-Audio Cable sampai antrean habis dalam 6,6 detik untuk 6 detik audio, lalu dua potongan 48 kHz dengan crossfade 2 detik dalam 4,3 detik. `render-wav` atas pasangan yang sama menghasilkan 6 detik penuh tanpa lompatan di titik sambung. Jalur berisi aksara Jepang dan spasi berjalan normal.
- **Pemeriksaan**: `scripts/check.ps1` dan `scripts/check.sh` (Git Bash) hijau: fmt, clippy `-D warnings`, tes, dan `svelte-check`.

### Tertunda / belum diverifikasi

- **Linux (ALSA) belum diverifikasi**: tidak ada mesin Linux di sini (WSL hanya berisi `docker-desktop`). Build dan tes Linux akan dijalankan CI Ubuntu setelah repositori dipush. Pemutaran ALSA/PipeWire perlu dicoba di mesin Linux.
- **Uji dengar oleh pemilik proyek**: gapless untuk MP3/AAC (delay encoder), crossfade, klik saat pause/seek, dan pencabutan headphone dengan file musik sungguhan (langkahnya ada di laporan M1).
- Mode sample rate "samakan dengan sumber" ditunda ke M4 (lihat DECISIONS).
- Dither TPDF, volume, dan rantai DSP adalah bagian M2.
