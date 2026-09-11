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
| Tes gapless, crossfade, micro-fade lulus | ✅ lewat sink offline | ✅ CI Ubuntu (commit `f22e6c6`) |
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

- **Tes otomatis** (`cargo test --workspace`): 40 tes lulus, yaitu 11 di `src-tauri`, 17 unit dan 12 integrasi di `onsa-audio`. Tes integrasi memakai fixture yang dibuat saat tes berjalan di folder berisi aksara Jepang dan spasi:
  - Gapless: dua potongan satu sinus, output sama persis dengan sinus utuh (galat < 1e-6). Dengan resampling 44,1 → 48 kHz, output sama dengan file utuh (galat < 1e-5).
  - Crossfade: level di titik silang 0,3536 untuk equal-power (0,5 × 0,7071) dan 0,25 untuk linear, durasi ≈ 1 detik. Lagu berurutan satu album tidak di-crossfade.
  - Micro-fade: tidak ada lompatan sampel saat pause/resume, seek, dan skip (langkah maksimum ≤ langkah alami sinus). Seek mendarat tepat di sampel tujuan.
  - Pergantian output (simulasi perangkat berganti ke 44,1 kHz) melanjutkan dari posisi yang sama. File rusak dilaporkan lalu dilewati.
  - **Gapless lossy** (`tests/lossy_gapless.rs`, fixture dibuat dengan ffmpeg saat tes; dilewati dengan pesan bila ffmpeg tidak ada): dua potongan satu sinus di-encode ke MP3 (LAME 320k) dan M4A (AAC 256k). Panjang tiap potongan tepat sama dengan aslinya, sinyal selaras dengan sinus asli (galat 0,0002 MP3 / 0,0006 AAC), tidak ada lubang di sambungan, hening setelah akhir, dan output identik dengan decode ffmpeg atas file yang sama. Tes ini sempat menemukan bahwa symphonia 0.6 tidak membuang priming AAC (1024 frame per lagu); sekarang `onsa-audio` membaca edit list MP4 dan iTunSMPB sendiri (lihat DECISIONS).
- **Perangkat nyata (Windows, WASAPI)**: `onsa-cli queue` memutar dua potongan sinus 44,1 kHz (di-resample ke 48 kHz) lewat perangkat VB-Audio Cable sampai antrean habis dalam 6,6 detik untuk 6 detik audio, lalu dua potongan 48 kHz dengan crossfade 2 detik dalam 4,3 detik. `render-wav` atas pasangan yang sama menghasilkan 6 detik penuh tanpa lompatan di titik sambung. Jalur berisi aksara Jepang dan spasi berjalan normal.
- **Pemeriksaan**: `scripts/check.ps1` dan `scripts/check.sh` (Git Bash) hijau: fmt, clippy `-D warnings`, tes, dan `svelte-check`.

### Tertunda / belum diverifikasi

- **Linux (ALSA) belum diverifikasi**: tidak ada mesin Linux di sini (WSL hanya berisi `docker-desktop`). Build dan seluruh tes sudah lulus di CI Ubuntu (commit `f22e6c6`); pemutaran lewat ALSA ke perangkat audio menunggu verifikasi di WSL.
- **Uji dengar oleh pemilik proyek**: gapless dengan album sungguhan (MP3/AAC sudah diuji otomatis), crossfade, klik saat pause/seek, dan pencabutan headphone dengan file musik sungguhan (langkahnya ada di laporan M1).
- Mode sample rate "samakan dengan sumber" ditunda ke M4 (lihat DECISIONS).
- Dither TPDF, volume, dan rantai DSP adalah bagian M2.

---

## M2. Rantai DSP — selesai (2026-09-11)

**Kriteria selesai**: tes DSP (§12) lulus, dan mengubah EQ saat lagu berjalan tidak menimbulkan glitch atau klik.

| Kriteria | Windows 11 | Linux |
|---|---|---|
| Tes DSP (§12) lulus | ✅ | ⏳ menunggu CI Ubuntu setelah push M2 |
| Mengubah EQ saat lagu berjalan tanpa klik | ✅ tes otomatis (unit dan lewat engine) + perangkat nyata | ⏳ belum diverifikasi |
| Semua bisa dikendalikan dari `onsa-cli` | ✅ | ⏳ belum diverifikasi |

### Yang dibuat

- `onsa-audio/src/dsp/`:
  - **Biquad** RBJ (peaking, low/high shelf, low/high pass, notch), state f64, dengan clamp supaya preset liar tidak membuat filter tidak stabil.
  - **EQ** grafis (10 band tetap, Q 1,41) dan parametrik (maksimal 16 band), kurva respons pada grid logaritmik, dan **preamp otomatis** (−puncak positif kurva).
  - **Import/export AutoEQ** (`ParametricEQ.txt`): baris tidak dikenal dilewati dengan peringatan, maksimal 16 filter, export yang bisa dibaca kembali tanpa kehilangan isi.
  - **Limiter** lookahead 5 ms, ceiling −0,1 dBFS dijamin, release bisa diatur, delay konstan walau limiter dimatikan.
  - **Dither** TPDF ±1 LSB, hanya untuk output integer 16-bit atau lebih sempit.
  - **ReplayGain**: mode mati/track/album/otomatis, preamp, nilai cadangan, pencegahan clipping dari peak. Tag dibaca dari ID3 TXXX, Vorbis comment, atom MP4, dan APE.
  - **Rantai callback**: urutan EQ/preamp/limiter bisa diubah, volume dan dither selalu terakhir. Parameter lock-free (`ChainParams` lewat `rtrb`), perubahan EQ di-crossfade 20 ms, preamp dan volume memakai penghalus dua kutub. Saat hening dan filter reda, rantai dilewati.
- **Tap analisis** (`analysis.rs`): aktif hanya bila analisis dinyalakan dan lagu diputar. Menghasilkan spektrum FFT Hann 2048 dengan 64 band log dan peak-hold, meter peak/RMS per kanal, status clip dan limiter, serta spectral centroid, maksimal 60 fps. Thread-nya diparkir saat pause.
- **Engine**: `Engine::set_dsp` dan `Engine::set_analysis`, event `Analysis`, ReplayGain per blok di thread decode dengan glide 10 ms antarlagu.
- **`onsa-cli`**:
  - opsi `--eq`, `--eq-file`, `--preamp`, `--auto-preamp`, `--no-limiter`, `--limiter-release`, `--volume`, `--replaygain`, `--rg-preamp`, `--rg-fallback`, `--no-dither`, `--meter`;
  - perintah saat lagu berjalan: `eq BAND DB`, `eq on/off/flat`, `pre DB`, `auto`, `vol DB`, `lim on/off`, `rg MODE`, `m`;
  - subperintah `eq-check`.

### Cara verifikasi

- **Tes otomatis** (`cargo test --workspace`): 83 tes lulus, yaitu 11 di `src-tauri`, 53 unit, 10 engine, 2 gapless lossy, dan 5 DSP lewat engine di `onsa-audio`, serta 2 di `onsa-cli`:
  - **Biquad**: respons yang diukur dengan sinus sama dengan respons hitungan dalam ±0,1 dB, untuk tujuh filter di tujuh frekuensi. Peaking tepat +6 dB di frekuensi tengah, LP/HP −3,01 dB di frekuensi sudut, notch di bawah −40 dB. Preset liar tetap stabil.
  - **Preamp otomatis** menjaga sinus full-scale yang di-boost +9 dB di bawah 0 dBFS. Tanpa preamp otomatis, puncaknya di atas 2,0.
  - **Parser AutoEQ**: contoh valid terbaca utuh; contoh rusak (tipe tidak dikenal, Fc hilang, gain hilang, BW, baris asing) melewati baris yang salah dan tetap mempertahankan yang benar; round-trip export lalu import identik.
  - **Tanpa klik**: menggeser EQ drastis (+12 dB di 1 kHz, −12 dB di 125 Hz) saat sinus berjalan tidak menimbulkan lonjakan turunan kedua di atas kelengkungan alami sinus, baik di unit chain maupun lewat seluruh engine. Glide volume dan preamp juga mulus.
  - **Limiter** tidak pernah melewati −0,1 dBFS (sinus +12 dB, lonjakan tunggal ×3), transparan bit-per-bit di bawah ceiling, dan pulih setelah burst.
  - **ReplayGain** dari FLAC bertag buatan ffmpeg: mati 0,5, track 0,25, album 0,125, otomatis mengikuti antrean, dan gain +12 dB dengan peak 0,5 dibatasi di 1,0.
  - **Analisis**: sinus 1 kHz di −6 dBFS terbaca peak −6,02 dB, RMS −9,03 dB, centroid ≈ 1 kHz. Frame hanya muncul saat analisis aktif dan lagu diputar, lalu berhenti saat pause dan saat analisis dimatikan.
- **Perangkat nyata** (WASAPI, VB-Audio Cable): meter `--meter` membaca −8,0/−11,0 dB untuk sinus 0,4 (teori −7,96/−10,97 dB). EQ, preamp otomatis, volume, dan limiter diganti lewat stdin saat lagu berjalan dan diterapkan. `eq-check` membaca preset AutoEQ, melaporkan baris rusak, dan mengekspornya ulang.
- **Pemeriksaan**: `scripts/check.ps1` hijau.

### Tertunda / belum diverifikasi

- **Linux**: CI Ubuntu untuk commit M2 berjalan setelah push. Pemutaran ALSA diverifikasi lewat WSL begitu Ubuntu di WSL siap.
- **Uji dengar oleh pemilik proyek**: menggeser EQ saat lagu berjalan, preset AutoEQ untuk headphone sendiri, preamp otomatis, limiter, dan ReplayGain (langkahnya ada di laporan M2).
- Preset EQ bawaan, penyimpanan preset, dan tampilan kurva di UI ada di M4. Visualizer dan meter di UI ada di M5.
- Analisis loudness EBU R128 untuk lagu tanpa tag tetap opsional (SPEC §4.1).
