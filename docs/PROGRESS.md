# Progres

Status tiap milestone dari `SPEC.md` §15. Diperbarui di akhir setiap milestone.

---

## M0. Kerangka proyek — selesai (2026-09-11)

**Kriteria selesai**: jendela kosong bertema terbuka di kedua OS, dan CI hijau.

| Kriteria | Windows 11 | Linux |
|---|---|---|
| Jendela kosong bertema terbuka | ✅ terverifikasi | ⏳ belum diverifikasi |
| CI hijau (lewat skrip lokal) | ✅ `scripts/check.ps1` dan `scripts/check.sh` (Git Bash) lulus | ✅ CI Ubuntu dan `scripts/check.sh` di WSL Ubuntu 26.04 |

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
| `onsa-cli` memutar daftar file secara gapless | ✅ terverifikasi (WASAPI) | ✅ WSL: ALSA → PulseAudio WSLg |
| Tes gapless, crossfade, micro-fade lulus | ✅ lewat sink offline | ✅ CI Ubuntu (commit `f22e6c6`) |
| Skrip pemeriksaan (`check.ps1`, `check.sh`) | ✅ hijau | ✅ `check.sh` hijau di WSL |

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

- **Linux (ALSA) belum diverifikasi**: tidak ada mesin Linux di sini (WSL hanya berisi `docker-desktop`). Build dan seluruh tes lulus di CI Ubuntu (commit `f22e6c6`), dan pemutaran lewat ALSA terverifikasi di WSL (lihat bagian M2). Uji cabut/colok perangkat di Linux tetap belum diverifikasi sampai M12.
- **Uji dengar oleh pemilik proyek**: gapless dengan album sungguhan (MP3/AAC sudah diuji otomatis), crossfade, klik saat pause/seek, dan pencabutan headphone dengan file musik sungguhan (langkahnya ada di laporan M1).
- Mode sample rate "samakan dengan sumber" ditunda ke M4 (lihat DECISIONS).
- Dither TPDF, volume, dan rantai DSP adalah bagian M2.

---

## M2. Rantai DSP — selesai (2026-09-11)

**Kriteria selesai**: tes DSP (§12) lulus, dan mengubah EQ saat lagu berjalan tidak menimbulkan glitch atau klik.

| Kriteria | Windows 11 | Linux |
|---|---|---|
| Tes DSP (§12) lulus | ✅ | ✅ CI Ubuntu dan `check.sh` di WSL |
| Mengubah EQ saat lagu berjalan tanpa klik | ✅ tes otomatis (unit dan lewat engine) + perangkat nyata | ✅ tes otomatis di WSL; perubahan live lewat ALSA |
| Semua bisa dikendalikan dari `onsa-cli` | ✅ | ✅ WSL |

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
- **Linux, lewat WSL** (Ubuntu 26.04.1, kernel WSL2 6.18, rustc 1.98.1, Node 24.21.0 lewat nvm, gcc 15.2):
  - `scripts/check.sh` hijau dalam 120 detik: 83 tes yang sama lulus, `svelte-check` bersih, fmt dan clippy bersih. Build release `onsa-cli` berhasil.
  - Pemutaran lewat ALSA ke PulseAudio WSLg (`~/.asoundrc` mengarahkan `default` ke `pulse`): antrean gapless dua potongan sinus 44,1 kHz, di-resample ke 48 kHz, habis dalam 6,137 detik untuk 6 detik audio. Meter membaca −28,0/−31,0 dB (sinus 0,4 pada volume −20 dB: teori −27,96/−30,97 dB). EQ, preamp otomatis, dan volume diganti lewat stdin saat lagu berjalan. Fixture berada di folder berisi aksara Jepang dan spasi.
  - Saat enumerasi perangkat, libasound mencetak pesan tentang plugin yang tidak ada di WSL (JACK, OSS, card 0). Pesan itu berasal dari ALSA sendiri; daftar perangkat Onsa tetap benar (`default`, `pulse`, `null`).
- **CI GitHub Actions** hijau di Windows dan Ubuntu untuk commit M2 (`a321ccf`).
- **Menit CI** (repo privat; menit Windows dihitung dua kali). Durasi job:

  | Run | Windows | Ubuntu |
  |---|---|---|
  | `f22e6c6`, tanpa cache | 24 menit 45 detik | 9 menit 42 detik |
  | `a321ccf` (M2), dengan cache Rust dan npm | 8 menit 22 detik | 3 menit 32 detik |
  | `479caf6`, cache tetap disimpan saat gagal, build aplikasi debug | 4 menit 56 detik | 33 menit 39 detik |
  | `c2cfa0c` (M3), dependensi library baru | 10 menit 44 detik | 6 menit 35 detik |

  Angka Ubuntu di `479caf6` bukan akibat perubahan workflow. 31 menit 21 detik habis di langkah `apt-get` (mirror runner yang lambat; di run sebelumnya 33 detik), sedangkan semua langkah Rust sama atau lebih cepat (build aplikasi 1:00 → 0:39). Tanpa langkah apt, job Ubuntu sekitar 2 menit 18 detik.

  Commit yang hanya mengubah `docs/` atau `*.md` tidak menjalankan CI.

### Tertunda / belum diverifikasi

- **Linux**: uji cabut/colok perangkat tetap belum diverifikasi sampai M12 (sesuai arahan pemilik proyek).
- **Uji dengar oleh pemilik proyek**: menggeser EQ saat lagu berjalan, preset AutoEQ untuk headphone sendiri, preamp otomatis, limiter, dan ReplayGain (langkahnya ada di laporan M2).
- Preset EQ bawaan, penyimpanan preset, dan tampilan kurva di UI ada di M4. Visualizer dan meter di UI ada di M5.
- Analisis loudness EBU R128 untuk lagu tanpa tag tetap opsional (SPEC §4.1).

---

## M3. Library — selesai (2026-09-11)

**Kriteria selesai**: tes library lulus, dan scan ulang folder yang tidak berubah selesai jauh lebih cepat daripada scan pertama.

| Kriteria | Windows 11 | Linux |
|---|---|---|
| Tes library (§12) lulus | ✅ 21 tes (14 unit, 7 integrasi) | ✅ `check.sh` di WSL, pemantau lewat inotify |
| Scan ulang folder tak berubah jauh lebih cepat | ✅ folder musik nyata, 620 file: 8,06 detik → 0,021 detik (377×) | ✅ tes otomatis di WSL (ambang ≥ 5×) |

### Yang dibuat

- **Database** (`onsa-library`, rusqlite dengan SQLite bundel):
  - skema v1 lewat migrasi berversi (`PRAGMA user_version`), WAL, foreign key aktif;
  - database dari versi yang lebih baru ditolak tanpa diubah;
  - tabel `folders`, `tracks`, `albums`, `covers`, `overrides`, `stats`, `plays`, `settings`, view `track_view` (override di atas nilai file), dan indeks FTS5 `tracks_fts` yang dijaga trigger.
- **Tag** lewat lofty:
  - judul, artis, album, album artist, nomor trek dan disk (beserta totalnya), tahun, genre, komposer, ReplayGain, dan gambar tertanam;
  - properti stream: durasi, codec, sample rate, bit depth, kanal, bitrate.
- **Scanner bertahap**:
  - file dilewati bila mtime dan ukurannya sama;
  - penulisan per 200 file dalam satu transaksi, dengan callback progres;
  - status `ok`/`missing`/`failed`; file yang hilang atau folder yang tidak terjangkau ditandai `missing`, tidak dihapus (`purge_missing` atas perintah pengguna);
  - album dibuat otomatis dari judul dan album artist.
- **Cache cover**:
  - sumber: gambar tertanam, lalu `cover.*`, `folder.*`, `front.*`;
  - thumbnail JPEG 128 dan 512 px, satu per hash SHA-256, ditulis atomik;
  - cover yang gagal di-decode (misalnya AVIF) hanya dicatat sebagai peringatan; lagunya tetap masuk tanpa cover.
- **Pemantau folder** (notify-debouncer-full, debounce 750 ms):
  - tambah/ubah, hapus, dan pindah/ganti nama untuk file maupun folder;
  - pemindahan mempertahankan id lagu, sehingga statistik dan riwayat ikut;
  - pasangan hapus+buat dalam satu batch dikenali sebagai pemindahan (cara Windows melaporkan pemindahan antar-folder).
- **Jelajah dan pencarian**:
  - daftar per halaman: lagu (8 pilihan urutan), album beserta lagunya, artis, genre, folder;
  - pencarian FTS5 berawalan, tanpa membedakan diakritik, dikelompokkan menjadi lagu/album/artis;
  - input pengguna selalu parameter terikat, dan sintaks FTS yang diketik diperlakukan sebagai teks biasa.
- **Override**: disimpan dan langsung terlihat di daftar maupun pencarian. Penulisan ke file ada di M7.
- **Statistik dan riwayat**: jumlah putar, jumlah skip, terakhir diputar, rating 0–5, dan riwayat putar per lagu.
- Contoh `scan` untuk mengukur scan pertama dan scan ulang pada folder sungguhan (lihat README).

### Cara verifikasi

- **Tes otomatis** (`cargo test --workspace`): 104 tes lulus, yaitu 83 dari M2 ditambah 21 di `onsa-library`. Tes integrasi (`tests/library.rs`) membuat fixture WAV bertag saat tes berjalan, di folder berisi aksara Jepang dan spasi:
  - **Scan folder fixture**:
    - 7 file terlihat: 6 terbaca, dan 1 FLAC palsu tercatat `failed`; `notes.txt` diabaikan;
    - album dikelompokkan dengan benar, termasuk album artist yang kosong;
    - dua cover unik, masing-masing dengan dua thumbnail; cover rusak dilewati tanpa menggagalkan lagunya;
    - pencarian awalan, aksara Jepang, dan "cafe creme" menemukan "Café Crème".
  - **Scan ulang bertahap**:
    - tanpa perubahan, 0 file dibaca ulang;
    - dengan 1 file diubah, 1 dihapus, dan 1 ditambah, tepat 2 file dibaca dan 1 ditandai `missing`, dan id lagu yang diubah tetap sama;
    - file yang kembali mendapatkan identitas lamanya;
    - folder yang tidak terjangkau membuat semua lagunya `missing`, lalu `purge_missing` membersihkannya.
  - **Kecepatan**: 160 file dengan cover, 1,31 detik untuk scan pertama dan 4,8 milidetik untuk scan ulang (build debug). Tes mensyaratkan scan ulang minimal 5× lebih cepat.
  - **Pemantau nyata**: file yang ditambah terdeteksi, file yang dipindah mempertahankan id dan jumlah putarnya, dan file yang dihapus menjadi `missing`. Ada juga tes deterministik untuk rename file, rename folder, pasangan hapus+buat (file dan folder), dan file lain yang tidak boleh dianggap pemindahan.
  - **Override dan statistik**: override tampil dan bisa dicari, dan menghapusnya mengembalikan nilai file. Rating dibatasi 5, riwayat terbaru tampil lebih dulu, dan statistik bertahan setelah file dan foldernya dipindah.
- **Temuan saat tes**:
  - di Windows, pemindahan file antar-folder dilaporkan sebagai hapus+buat, bukan rename, sehingga lagu sempat kehilangan statistiknya; ini diperbaiki dengan mencocokkan pasangan tersebut (lihat DECISIONS);
  - query pencarian sempat gagal karena nama kolom yang ambigu.
- **Folder musik nyata** (Windows, build release, `examples/scan.rs`): 620 file (3,5 GB) dan 97 album, tanpa file gagal. Scan pertama 8,057 detik, scan ulang 0,021 detik.
- **Setiap commit M3** lulus `cargo clippy -p onsa-library --all-targets -- -D warnings` secara terpisah.
- **Pemeriksaan**: `cargo fmt --check`, clippy workspace, dan `svelte-check` (0 error) bersih di Windows.
- **Linux, lewat WSL**: `scripts/check.sh` hijau dalam 88 detik, dengan 104 tes yang sama lulus. Di inotify, pemindahan dalam folder yang dipantau dilaporkan sebagai rename sungguhan. File yang masuk dari luar folder hanya muncul sebagai perubahan pada folder induknya, lalu ditemukan saat folder itu ditelusuri.

### Tertunda / belum diverifikasi

- Library belum disambungkan ke `src-tauri` dan UI. Command, event, dan tampilan library ada di M4.
- Smart playlist dan M3U8 ada di M6. Menulis override ke file ada di M7. Opus masuk bersama decoder-nya di M11. Tabel playlist, lirik, scrobble, dan unduhan dibuat lewat migrasi di milestone masing-masing.
- File yang dipindah saat Onsa tidak berjalan tercatat sebagai `missing` ditambah lagu baru, sehingga statistiknya tidak ikut pindah.
- Tidak ada yang tertunda dari sisi CI. Lihat baris `c2cfa0c` di tabel menit CI (bagian M2).

**CI GitHub Actions** hijau di Windows dan Ubuntu untuk commit M3 (`c2cfa0c`, sembilan commit di-push sekaligus). Job Windows makan 10 menit 44 detik, yang sebagian besar adalah biaya sekali:
- 3 menit 22 detik untuk menyimpan cache Rust baru, karena `Cargo.lock` berubah;
- sebagian dari 2 menit 40 detik `cargo test` untuk mengompilasi dependensi baru, termasuk SQLite dalam C.

Run berikutnya dengan dependensi yang sama diperkirakan kembali ke sekitar 5–6 menit.

---

## M4a. Irisan uji dari M4 — selesai (2026-09-12)

Bagian M4 yang dikerjakan lebih dulu atas permintaan pemilik proyek, supaya hasil M1–M3 bisa diuji lewat aplikasi, bukan lewat CLI. Semua isinya adalah bagian M4 yang sesungguhnya, dibangun di atas arsitektur final, bukan kode sementara.

**Kriteria yang dipakai**: seluruh alur pilih folder, scan, jelajah, putar, atur EQ, dan ganti tema bisa dilakukan tanpa CLI.

| Kriteria | Windows 11 | Linux |
|---|---|---|
| Alur lengkap tanpa CLI | ✅ terverifikasi pada build rilis portable | ⏳ belum diverifikasi (bagian kriteria M4 penuh) |
| Pemeriksaan (fmt, clippy, tes, `svelte-check`) | ✅ | ✅ CI Ubuntu |

### Yang dibuat

- **`onsa-audio`** (dua tambahan, disetujui pemilik proyek):
  - `Engine::set_output`: ganti perangkat atau sample rate tanpa menghentikan lagu, memakai jalur yang sama dengan pemulihan perangkat hilang;
  - `Engine::take_events`: menyerahkan receiver event supaya aplikasi tidak perlu polling.
- **`onsa-library`**: `TrackRow` membawa codec, sample rate, bit depth, dan kanal untuk strip jalur sinyal; `album_count` untuk virtual list; `cover_file` untuk protokol cover.
- **`src-tauri`**: 34 command dan lima event (status pemutar, posisi, meter, progres scan, perubahan library). Player dengan thread pendengar tanpa polling, library dengan dua koneksi (UI dan thread scan/pemantau), pengaturan JSON di tabel `settings`, protokol `onsa://cover/<id>/<ukuran>`, dan log harian dengan toggle debug.
- **UI (SvelteKit)**:
  - layar pertama: pilih folder, pilih tema dengan pratinjau langsung, lalu scan dengan progres;
  - library: daftar lagu dan album dengan virtual list dan pengurutan per kolom, halaman album, serta pencarian yang dikelompokkan;
  - transport: putar/jeda, lagu sebelumnya/berikutnya, bar posisi, volume, dan peak meter L/R dengan indikator CLIP dan LIM (varian batang, segmen, atau jarum VU sesuai tema);
  - panel antrean: menyorot lagu yang berjalan dan bisa diklik untuk melompat;
  - strip jalur sinyal: format sumber, resample, ReplayGain, EQ, limiter, dan perangkat output; klik untuk menyalakan atau mematikan, klik kanan untuk membuka pengaturannya;
  - pengaturan Output & Kualitas (perangkat, sample rate, kualitas resampler, buffer, crossfade, ReplayGain, dither) dan DSP & EQ (EQ grafis 10 band dengan kurva respons, mode parametrik sampai 16 filter, import dan export AutoEQ lewat pemilih file, preamp otomatis, limiter);
  - Tampilan (tema dan bahasa) dan Tentang (versi, buka folder log, toggle log debug);
  - pintasan keyboard dasar: spasi, panah, Ctrl+panah, Ctrl+F, Ctrl+koma.
- **Font tema dibundel** di `ui/static/fonts/` beserta lisensinya: Chakra Petch, Share Tech Mono, B612, B612 Mono, Barlow, Barlow Condensed, dan Noto Sans JP (lihat DECISIONS untuk sumber dan versi yang dipatok).
- **`docs/UJI-M4a.md`**: panduan uji coba untuk pemilik proyek, dalam bahasa sehari-hari.

### Cara verifikasi

Semuanya dijalankan pada build rilis portable (`dist-test/Onsa-M4a.exe`) di Windows 11, dengan folder musik sungguhan berisi 620 file, dan dikendalikan lewat port debug WebView2 supaya setiap langkah tercatat:

- **Layar pertama**: dialog folder Windows terbuka dengan judul dari kamus UI, folder terpilih, dan scan 620 file selesai dalam 3,8 detik dengan progres yang terlihat bertambah.
- **Scan ulang**: saat aplikasi dibuka lagi, 620 file yang tidak berubah diperiksa dalam 13 milidetik.
- **Library**: daftar lagu dan album tampil lewat virtual list, urutannya sama persis dengan urutan backend, dan judul beraksara Jepang tampil benar.
- **Pemutaran**: klik dua kali memutar lagu yang tepat; `next` maju satu per satu (1, 2, 3) dan lompat ke antrean nomor 10 mendarat di lagu yang benar; jeda, lanjut, geser posisi ke detik 45, dan volume −12 dB semuanya diterapkan.
- **Strip jalur sinyal** menampilkan MP3 48 kHz, EQ, limiter, dan perangkat WASAPI yang sedang dipakai, dan meter bergerak mengikuti sinyal.
- **EQ saat lagu berjalan**: menggeser dua fader (+9 dB di 31 Hz, −6 dB di 1 kHz) langsung mengubah kurva dan diterapkan ke mesin.
- **AutoEQ**: export menulis preset 10 filter, lalu import membacanya kembali dan memindahkan EQ ke mode parametrik dengan nilai yang sama.
- **Ganti perangkat output** saat lagu berjalan berpindah dari satu perangkat ke perangkat lain, lagu tetap berjalan dan posisinya tidak terputus.
- **Tema**: ketiga tema berganti langsung, termasuk bentuk meter (segmen, batang, jarum VU) dan indikator tahap (glow, outline, LED).
- **Bahasa**: tampilan Indonesia dan Inggris, keduanya lengkap.
- **Tersimpan setelah ditutup dan dibuka lagi**: tema, bahasa, volume, EQ, dan status log debug.
- **Pemeriksaan**: `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace`, dan `svelte-check` bersih.

### Tertunda / belum diverifikasi

- **Linux belum diverifikasi untuk UI**: build dan tes lewat CI Ubuntu, tetapi jendela, dialog, dan audio lewat aplikasi belum dicoba di Linux. Itu bagian kriteria M4 penuh.
- **Uji dengar oleh pemilik proyek** sesuai `docs/UJI-M4a.md`: gapless, crossfade, klik saat jeda dan seek, cabut/colok headphone, EQ saat lagu berjalan, preset AutoEQ, preamp otomatis, limiter, dan ReplayGain.
- **Satu kejadian yang belum bisa dijelaskan**: pada percobaan pertama, klik dua kali di baris pertama berakhir memutar lagu ke-282 dalam antrean. Tidak terulang dalam tiga percobaan berikutnya, penomoran antrean terbukti benar (`next` dan lompat ke indeks tertentu), dan log tidak menunjukkan lagu yang gagal atau dilewati. Kalau terjadi lagi saat uji dengar, log debug sekarang mencatat setiap lagu yang mulai diputar.
- **Belum masuk irisan ini** (bagian M4 dan M5 berikutnya): visualizer dan warna nada, Now Playing, mini player, sleep timer, kontrol media OS dan tray, single instance, asosiasi file dan drag-and-drop, pemulihan antrean beserta posisi terakhir, tema buatan pengguna, pengaturan pintasan, dan pengurutan ulang antrean.

---

## M4. UI dasar dan integrasi OS — selesai (2026-09-12)

Lanjutan dari irisan uji M4a, yang sudah lulus uji dengar pemilik proyek untuk M1–M3.

**Kriteria selesai**: seluruh alur dari pilih folder, scan, jelajah, putar, atur EQ, sampai ganti tema bisa dilakukan tanpa CLI di kedua OS, dan ketiga tema sesuai `palet-preview.html`.

| Kriteria | Windows 11 | Linux |
|---|---|---|
| Alur lengkap tanpa CLI | ✅ terverifikasi pada build rilis portable | ✅ jendela terbuka lewat WSLg, library dan output ALSA jalan; dialog, tray, dan uji dengar belum dicoba di Linux |
| Ketiga tema sesuai acuan visual | ✅ termasuk font yang dibundel dan varian meter per tema | ⏳ belum diperiksa langsung |
| Tes (§12) | ✅ 114 tes | ✅ 114 tes di WSL |

### Yang dibuat (di luar yang sudah ada di M4a)

- **Jelajah** Artis, Genre, dan Folder, masing-masing dengan daftar per halaman dan halaman isinya.
- **Antrean** (SPEC §6.1): putar berikutnya, tambah ke akhir, hapus satu entri, tarik untuk mengurutkan ulang, kosongkan, shuffle yang menyimpan urutan asli, dan repeat mati/semua/satu. Semua perubahan diterapkan **tanpa memotong lagu yang sedang berbunyi**.
- **Sedang diputar**: cover besar, judul, artis, album, bar posisi, serta format sumber dan perangkat output.
- **Mini player**: jendela yang sama diringkas jadi satu baris dan selalu di atas, dengan cover, judul, kontrol, posisi, dan meter.
- **Sleep timer** (SPEC §3.5): setelah N menit, di akhir lagu, atau setelah N lagu; volume diturunkan perlahan lalu dikembalikan, dan aksinya jeda, berhenti, atau tutup Onsa.
- **Integrasi OS** (SPEC §13): kontrol media sistem (SMTC di Windows, MPRIS di Linux) lengkap dengan judul, artis, album, cover, dan posisi; ikon tray dengan menu berbahasa UI dan opsi tutup-ke-tray; single instance; file dan folder yang diseret ke jendela atau dibuka lewat "Buka dengan Onsa"; serta deklarasi asosiasi file.
- **Pemulihan keadaan**: antrean, lagu dan posisi terakhir (dalam keadaan jeda), volume, tema, bahasa, EQ, ukuran dan posisi jendela, serta mode mini.
- **Mode sample rate "samakan dengan sumber"** (ditunda dari M1), dengan strip jalur sinyal yang menunjukkan tahap Resample hilang saat perangkat mengikuti rate lagu.
- **Tema buatan pengguna** dari `<app config>/themes/*.json`, dengan tombol pembuka foldernya di Tampilan.
- **Pintasan keyboard** bawaan SPEC §13, termasuk Ctrl+M untuk mini player dan Ctrl+N untuk Sedang diputar.

### Cara verifikasi

Dijalankan pada build rilis portable di Windows 11, dikendalikan lewat port debug WebView2, dengan folder musik sungguhan berisi 620 file:

- **Antrean**: memutar album, menambah album lain ke akhir, menyisipkan satu lagu sebagai berikutnya, memindahkan entri, menghapus entri, lalu menyalakan dan mematikan shuffle. Setiap langkah diperiksa: posisi putar terus berjalan dan statusnya tetap "playing".
- **Repeat**: tersimpan dan diterapkan; tes mesin membuktikan repeat seluruh antrean kembali ke lagu pertama dan repeat satu lagu menghasilkan tiga putaran yang sama persis dengan satu sinus utuh (galat < 1e-6).
- **Jelajah**: 110 artis, 1 genre, dan 4 folder dari library nyata tampil, termasuk nama folder beraksara Jepang, dan membukanya menampilkan lagunya.
- **Sleep timer**: dipasang 1 menit, sisa waktunya terlihat, lalu dibatalkan.
- **Mini player**: jendela menyusut, tetap bisa memutar, dan mode itu kembali saat aplikasi dibuka lagi.
- **Pemulihan keadaan**: setelah ditutup dan dibuka lagi, antrean kembali dalam keadaan jeda, begitu pula mode jendela.
- **Single instance dan buka file**: menjalankan exe untuk kedua kalinya dengan sebuah file tidak membuka jendela kedua; jendela yang ada maju ke depan dan memutar file itu. Ini juga jalur yang sama dengan drag-and-drop.
- **Tutup ke tray**: menutup jendela menyembunyikannya dan prosesnya tetap hidup; menjalankan exe lagi memunculkannya kembali.
- **Samakan dengan sumber**: file 44,1 kHz membuat perangkat dibuka ulang di 44,1 kHz, tahap Resample hilang, dan pemutaran berlanjut.
- **Tema pengguna**: sebuah file tema di folder tema muncul di daftar bersama tiga tema bawaan.
- **Kontrol media**: log mencatat "system media controls ready" di kedua OS, dan di Linux `org.mpris.MediaPlayer2.onsa` benar-benar terdaftar di session bus.
- **Linux (WSL Ubuntu + WSLg)**: build rilis berhasil, 114 tes lulus, jendela Onsa terbuka (terlihat sebagai jendela WSLg dari sisi Windows), database dibuat, dan output ALSA terbuka.
- **Pemeriksaan**: `cargo fmt --check`, clippy `-D warnings`, `cargo test --workspace` (114 tes), dan `svelte-check` (202 file) bersih.

### Temuan saat pengujian (sudah diperbaiki)

- Mode "samakan dengan sumber" membuka ulang output tepat saat lagu mulai, dan saat itu belum ada posisi putar yang bisa dilanjutkan, sehingga antrean langsung dianggap habis. Sekarang lagunya dimulai dari awal dalam kasus itu, dengan tes yang menutupnya.
- Tata letak mini player terlalu sempit pada ukuran awal, sehingga judul dan waktu berdesakan; ukurannya dan proporsinya diperbaiki.

### Tertunda / belum diverifikasi

- **Linux**: dialog pemilih file, ikon tray, drag-and-drop, dan uji dengar lewat aplikasi belum dicoba. Uji cabut/colok perangkat di Linux tetap menunggu M12.
- **Uji dengar oleh pemilik proyek** untuk fitur baru (langkahnya ada di `docs/UJI-M4.md`), terutama mengubah antrean saat lagu berjalan, repeat satu lagu, sleep timer, tombol media, dan tray.
- **Repeat satu lagu tidak memancarkan event "lagu dimulai" baru**; kalau M9 (scrobble) butuh batas pemutaran, mesin perlu penanda pemutaran keberapa (lihat DECISIONS).
- **Asosiasi file** baru terdaftar lewat installer (M12).
- Visualizer, warna nada, dan mode hemat daya ada di M5. Lirik di M8. Playlist, termasuk menyimpan antrean sebagai playlist, ada di M6. Pengaturan pintasan keyboard menyusul.

---

## Perbaikan setelah uji coba M4 (2026-09-12)

### Lagu yang terdengar berbeda dengan yang terlihat, setelah antrean diedit cepat

Dilaporkan pemilik proyek dan bisa diulang: klik dua kali satu lagu, lalu hapus baris antrean cepat-cepat termasuk baris yang sedang berbunyi. Hasilnya UI dan suara tidak sejalan, dan baru pulih setelah memilih lagu lain lalu kembali.

**Akar masalahnya satu**: nomor urut antrean dipakai sebagai identitas lagu, di tiga tempat sekaligus — mesin audio menggeser nomor yang diingatnya dengan satu selisih yang hanya benar untuk perubahan sebelum lagu yang berbunyi; aplikasi menghitung sendiri nomor lagu yang diputar, sejajar dengan perhitungan mesin; dan UI mengirim perintah memakai nomor baris dari daftar yang sudah basi saat klik berikutnya datang. Rinciannya di `docs/DECISIONS.md`.

**Perbaikannya**: setiap entri antrean sekarang membawa id sendiri (`QueueId`) yang dibuat sekali dan hidup selama entri ada. Mesin audio yang menentukan lagu mana yang sedang diputar; ia mencari entrinya kembali lewat id dan memetakan ulang semua nomor yang diingatnya, sedangkan UI menyorot baris berdasarkan id dari mesin dan mengirim perintah hapus, pindah, dan lompat memakai id entri, bukan nomor baris. Antrean aplikasi dipindah ke modul `src-tauri/src/queue.rs` supaya aturannya bisa diuji langsung.

**Perilaku yang ditetapkan** (lihat DECISIONS): menghapus lagu yang sedang berbunyi langsung pindah ke lagu berikutnya yang masih ada, tanpa klik, karena memakai jalur peredupan yang sama dengan seek; berhenti kalau tidak ada lagi sesudahnya; kembali ke lagu pertama kalau repeat seluruh antrean menyala.

**Tes yang menutupnya** (semuanya gagal pada kode lama):

- `onsa-audio`: menghapus entri yang sedang berbunyi lalu memeriksa lagu mana yang benar-benar terdengar; menghapus entri sesudahnya tanpa mengganggu yang berbunyi; empat penghapusan beruntun secepat perintah bisa dikirim; mengosongkan antrean saat lagu berjalan; dan mengurutkan ulang antrean sambil memeriksa nomor yang dilaporkan mesin.
- `src-tauri`: sembilan tes unit untuk aturan antrean — hapus yang sedang diputar, hapus yang terakhir (dengan dan tanpa repeat), hapus semua sesudahnya, penghapusan beruntun termasuk id yang sama dua kali, kosongkan, tarik-lepas, putar berikutnya dan tambah ke akhir, shuffle beserta kembalinya urutan asli, dan mesin sebagai penentu akhir.

### Ukuran folder target dan cache CI

Job Ubuntu sempat gagal di langkah penyimpanan cache dengan `No space left on device`, padahal semua langkah nyata lulus. Runner kini dibersihkan dari toolchain yang tidak dipakai sebelum build (lebih dari 20 GB), dan debug info dimatikan untuk profil `dev` dan `test` di `Cargo.toml` workspace supaya `target/` dan cache-nya jauh lebih kecil.

---

## M5. Visualizer, meter, warna nada — selesai (2026-09-13)

**Kriteria selesai**: saat visualizer disembunyikan atau aplikasi di-minimize, tap analisis berhenti (terverifikasi lewat log atau tes), dan warna nada bisa dinyalakan dan dimatikan.

| Kriteria | Hasil |
|---|---|
| Tap berhenti saat tidak ada yang menampilkannya | ✅ terlihat di log aplikasi dan ditutup dua tes |
| Warna nada bisa dinyalakan dan dimatikan | ✅ lewat Pengaturan → Tampilan, terverifikasi pada build rilis |
| Tes (§12) | ✅ 132 tes |

### Yang dibuat

- **Visualizer spektrum** di layar Sedang Diputar, dengan **tiga varian tema** sesuai `palet-preview.html`: `segment` (blok bertumpuk, segmen mati tetap samar — Kaca asap), `bar` (batang polos di atas sumur samar — Kokpit kaca), dan `soft` (kolom hangat yang memudar dari bawah — Deck malam). Semuanya menggambar 64 band beserta garis peak-hold dalam warna lit temanya, dan ikut menyala saat tema memakai glow.
- **Warna nada** (SPEC §9.5): spectral centroid dari thread analisis dihaluskan dengan rata-rata bergerak sekitar satu detik, lalu dipetakan ke `hue-rotate` dalam rentang yang ditentukan tema. Suara terang bergeser ke rona dingin, suara berat ke rona hangat. Bisa dimatikan di Pengaturan → Tampilan, dan ikut mati saat mode hemat daya menyala.
- **Analisis mengikuti apa yang terlihat**: setiap meter dan visualizer mendaftarkan dirinya selama ada di layar. Tanpa satu pun yang terdaftar, atau saat jendela di-minimize atau disembunyikan ke tray, tap di callback output mati dan thread analisis tidur. Kalau hanya meter yang tampil, FFT di balik spektrum tidak dijalankan sama sekali.
- **Mode hemat daya** (SPEC §3.4): buffer Besar, resampler Cepat, frame analisis 20 per detik, event posisi tiap 500 ms, dan warna nada mati. Pilihan buffer dan resampler milik pengguna tetap tersimpan dan berlaku lagi saat mode ini dimatikan.
- **Laju frame mengikuti kebutuhan**: 60 per detik saat visualizer tampil, 30 saat hanya meter, 20 saat hemat daya.

### Cara verifikasi

Dijalankan pada build rilis portable di Windows 11 dengan library 620 lagu, dikendalikan lewat port debug WebView2:

- **Visualizer**: kolomnya benar-benar berubah mengikuti musik, tiap tema menggambar variannya sendiri, dan warna yang dihitung browser cocok dengan acuan visual: Kaca asap teal bermasker segmen di atas sumur samar, Kokpit kaca hijau 85% dengan penanda puncak putih, Deck malam gradien hangat dengan penanda puncak merah.
- **Warna nada**: `hue-rotate(15,4°)` saat menyala, `none` setelah dimatikan lewat Pengaturan → Tampilan, dan kembali setelah dinyalakan lagi.
- **Tap berhenti** (kriteria selesai), dari log aplikasi:
  - masuk Sedang Diputar → `analysis enabled=true spectrum=true fps=60`
  - kembali ke library → `analysis enabled=true spectrum=false fps=30`
  - jendela di-minimize → `analysis enabled=false`
  - jendela dikembalikan → `analysis enabled=true`
- **Laju event diukur langsung**: dalam 2 detik, mode biasa menghasilkan 60 frame analisis dan 15 event posisi; mode hemat daya 40 frame dan 4 event posisi — persis 20 fps dan 500 ms.
- **Tes**: `Analyzer` tanpa spektrum hanya menghitung level tanpa FFT; mesin tetap mengirim frame setelah serangkaian perubahan setelan analisis; aturan "analisis hanya untuk yang terlihat" diuji langsung; dan pemetaan mode hemat daya diuji termasuk kembalinya pilihan pengguna.
- **Pemeriksaan**: `cargo fmt --check`, clippy `-D warnings`, `cargo test --workspace` (132 tes), dan `svelte-check` (204 file) bersih.

### Temuan saat pengujian (sudah diperbaiki)

- Efek yang menghaluskan centroid membaca dan menulis state yang sama, sehingga Svelte masuk ke loop pembaruan (`effect_update_depth_exceeded`) dan **seluruh UI berhenti diperbarui** — meter ikut membeku, padahal backend tetap mengirim 60 frame per detik. Rata-rata bergeraknya sekarang disimpan di luar graf reaktif. Ini hanya ketahuan lewat uji coba pada aplikasi sungguhan, bukan lewat tes.

### Tertunda / belum diverifikasi

- **Linux**: visualizer dan warna nada belum dicoba lewat WSLg.
- **Uji dengar dan uji pandang oleh pemilik proyek** (langkahnya di `docs/UJI-M5.md`).
- **Penyempurnaan mode hemat daya** ada di M11, termasuk mengikuti keadaan baterai sistem.
- Warna nada baru menyentuh elemen yang disebut tema; ketiga tema bawaan menyebut spektrum saja.

---

## Perbaikan setelah uji coba M5 (2026-09-13)

### Keadaan jendela hilang saat bolak-balik mini player

Dilaporkan pemilik proyek: dari jendela yang dimaksimalkan, masuk mini player lalu kembali menghasilkan jendela normal, bukan maximized.

**Akar masalahnya**: keluar dari mini player selalu memasang ukuran tetap 1180×760, dan tidak ada apa pun yang mengingat jendela sebelumnya. Mode jendela juga hidup di tiga tempat sekaligus (backend, store UI, dan jendela sungguhan).

**Perbaikannya**: `WindowMode` sekarang menyimpan mode **dan** jendela yang dipinjam mini player, dan menjadi satu-satunya sumber kebenaran. Semua jalan masuk lewat `session::set_mini`, yang menerapkan bentuknya, mencatat modenya, menyimpan sesi, lalu memancarkan event `window://mode` yang diikuti UI. Rinciannya di `docs/DECISIONS.md`.

**Dua bug lain ikut ketahuan saat memverifikasinya**:

1. Setelah kembali maximized, ukuran "restore" yang diingat Windows adalah strip mini player selebar 660 piksel — menekan tombol restore memberi sliver jendela. Sekarang ukuran penuh selalu dipasang lebih dulu, baru dimaksimalkan.
2. `outer_size()` dan `set_size()` tidak simetris di Windows: yang satu melaporkan bingkai luar, yang satu mendarat di ukuran dalam. Jendelanya **tumbuh 18×47 piksel setiap kali kembali dari mini player**, terukur persis begitu tiga putaran berturut-turut. Sekarang keduanya memakai ukuran dalam.

**Tes**: enam tes unit untuk aturan modenya (mini mengembalikan jendela yang dipinjamnya, maximized kembali maximized, sesi menyimpan jendela penuh bukan strip, jendela maximized disimpan pada ukuran sebelum dimaksimalkan, dan jendela yang tidak masuk akal jatuh ke ukuran wajar).

**Verifikasi langsung** pada build rilis, diukur dari luar aplikasi lewat Win32 (`GetWindowRect`, `IsZoomed`):

- maximized → mini → kembali: maximized lagi, 1550×974 sama persis;
- tombol restore setelah itu memberi jendela penuh, bukan strip;
- jendela biasa → mini → kembali: ukuran dan posisi sama persis, dan tetap sama setelah tiga putaran;
- Ctrl+M: sama seperti tombolnya, maximized tetap kembali;
- ditutup dan dibuka lagi saat maximized: kembali maximized;
- ditutup dan dibuka lagi saat mini: kembali mini, dan keluar dari mini memberi jendela maximized yang ditinggalkan;
- instance kedua (jalur yang sama dengan tray): memunculkan jendela tanpa mengubah modenya;
- meminta mode yang sedang dipakai: tidak mengubah apa pun.

### Warna nada terlalu tipis

Dilaporkan pemilik proyek, terutama di Deck malam.

**Akar masalahnya**: `hue-rotate` hampir tidak terlihat pada krem hangat, dan rentang 70° yang sama dipakai ketiga tema.

**Perbaikannya**: warna nada sekarang **menggeser warna di antara dua warna yang disebut temanya sendiri** (`warm` dan `cool`), bukan memutar rona, dan kekuatannya bisa dipilih di Pengaturan → Tampilan: mati, halus, sedang (bawaan), atau kuat. Deck malam menggeser krem lampunya ke merah jarum untuk suara berat dan ke baja dingin untuk suara terang, serta menggeser bar posisinya juga; dua tema lainnya cukup dengan spektrum. Meter tidak pernah ikut bergeser, karena warnanya berarti "mendekati batas".

**Verifikasi**: matematika pergeserannya diperiksa terhadap ketiga tema bawaan (setiap tingkat kekuatan menggeser lebih jauh, suara berat dan terang ke arah berlawanan, dan ujungnya tidak pernah melewati warna tema), lalu pada aplikasi sungguhan: "mati" meninggalkan warna tema apa adanya, "sedang" menggeser 57 sampai 143 dari 255 pada musik nyata, bar posisi hanya bergeser di Deck malam, dan mode hemat daya menghentikan pergeseran sepenuhnya.

### Tata letak berantakan saat jendela dikecilkan (2026-09-13)

Dilaporkan pemilik proyek: pada ukuran jendela paling kecil, judul kolom saling menindih, panel antrean memaksa lebarnya, strip jalur sinyal pecah dua baris, dan tombol kanan transport terdorong keluar layar.

**Akar masalahnya dua**: tidak ada tahap penyesuaian sama sekali (semua panel memaksa lebarnya, kolom menyempit sampai nol tapi tetap digambar), dan **ukuran minimum jendela dipasang dalam piksel fisik** padahal `tauri.conf.json` menyebutnya logis — di layar 125% minimum 880×560 itu sebenarnya 704×448. Rinciannya di `docs/DECISIONS.md`.

**Yang dikerjakan**:

- Semua ukuran jendela kini dalam piksel logis, dan minimumnya 820×600. Mini player ikut terkoreksi: selama ini 660×146 sebenarnya tampil 528×117 di layar 125%.
- Tahapan lebar ada di satu tempat (`ui/src/lib/layout.svelte.ts`): antrean punya kolom sendiri di ≥1160, sidebar berkata-kata di ≥1040, jadi ikon di 880–1039, dan jadi laci di bawah itu. Antrean dan laci yang menimpa hanya menutupi area konten, sehingga transport tetap terjangkau.
- Kolom daftar lagu dibuang menurut prioritas — tahun, lalu album, lalu artis — dengan judul dan durasi selalu bertahan, dan setiap sel dipotong elipsis.
- Strip jalur sinyal selalu satu baris: melepas angkanya, lalu memendekkan namanya, lalu melipat ReplayGain dan Limiter ke tombol "…".
- Transport selalu menyisakan tombol putar, posisi, dan volume; meter, cover, dan tombol tambahan mundur satu per satu.
- Halaman pengaturan, DSP & EQ, dan Sedang Diputar memakai container query terhadap lebar area konten.

**Cara verifikasi**: sebuah pemeriksa otomatis menjalankan aplikasi rilis di enam lebar jendela (1546, 1266, 1086, 946, 806, dan 686 piksel CSS — dua terakhir pada dan di bawah lantai minimum) dan tujuh halaman, lalu pada tiap kombinasi memeriksa empat hal di dalam halaman: tidak ada scroll mendatar, tidak ada elemen yang melewati kotaknya, tidak ada teks yang terpotong tanpa elipsis, dan **tidak ada dua elemen bersebelahan yang saling menindih**. Panel antrean, laci, dan kedua tombol "…" juga diperiksa dalam keadaan terbuka. Semuanya bersih, dan tangkapan layar tiap kombinasi disimpan.

Pemeriksaan tumpang tindih itu langsung berguna: ia menemukan cover Sedang Diputar duduk di atas judul lagu pada lebar sempit, karena baris grid tidak bisa menghitung tinggi dari `aspect-ratio`.

## M6. Playlist — selesai (2026-09-13)

**Kriteria selesai**: tes playlist pintar dan M3U8 lulus, dan playlist pintar ikut berubah saat library berubah.

| Kriteria | Hasil |
|---|---|
| Tes playlist pintar lulus | ✅ 7 tes integrasi di `crates/onsa-library/tests/playlists.rs` |
| Tes M3U8 lulus | ✅ ekspor–impor pulang pergi, relatif dan absolut, plus entri yang tidak ditemukan |
| Playlist pintar ikut berubah saat library berubah | ✅ lagu baru yang cocok masuk sendiri setelah scan, tanpa playlist-nya disentuh |
| Tes (§12) | ✅ 150 tes Rust dan 15 tes antarmuka |

### Yang dibuat

- **Skema versi 2**: tabel `playlists` dan `playlist_items`, lewat migrasi berversi, jadi library yang sudah ada ikut naik tanpa kehilangan apa pun. `playlist_items` memakai `(playlist_id, position)` sebagai primary key dan `ON DELETE CASCADE` ke `tracks`.
- **Playlist biasa** (SPEC §6.2): buat, ganti nama, gandakan, hapus, tambah, hapus baris, pindahkan baris, dan ganti isinya sekaligus. Lagu yang sama boleh dua kali.
- **Playlist pintar** (SPEC §6.3): aturan JSON (`Rules`) dikompilasi jadi **SQL berparameter**, dengan 13 kolom, 17 operator yang terikat jenis kolomnya, urutan (termasuk acak), dan batas. Isinya tidak pernah disimpan — selalu ditanyakan ulang, jadi ia mengikuti library.
- **M3U8 keluar dan masuk** (SPEC §6.4): ekspor menulis `#EXTM3U` dengan `#EXTINF` per lagu dan path relatif terhadap file playlist bila bisa; impor mencocokkan kembali ke library dan **melaporkan entri yang tidak ditemukan** alih-alih membuangnya.
- **Perintah dan antarmuka**: 15 perintah baru, halaman **Playlist** di sidebar, halaman satu playlist dengan tarik-lepas dan tombol naik/turun/hapus, **editor aturan** dengan pratinjau langsung, klik kanan sebuah lagu → **Tambah ke playlist**, dan tombol **simpan antrean sebagai playlist** di panel antrean.
- **Satu event** `library://playlists` memberi tahu antarmuka saat daftarnya berubah; daftar juga dimuat ulang saat library berubah, karena jumlah lagu playlist pintar ikut berubah dengan sendirinya.

### Cara verifikasi

- **Tes integrasi** dengan fixture yang dibuat saat tes berjalan, di folder beraksara Jepang dan berspasi: urutan playlist biasa bertahan lewat pindah-hapus-gandakan; lagu yang dibuang dari library ikut hilang dari playlist sementara file yang hanya *missing* tetap tinggal; aturan dikompilasi ke SQL tanpa satu pun nilai pengguna di dalam teksnya; `'; DROP TABLE tracks; --` sebagai nilai aturan tidak menyentuh library; `%` diperlakukan sebagai karakter, bukan wildcard; lagu baru yang cocok masuk sendiri ke playlist pintar setelah scan; dan M3U8 pulang pergi dalam path relatif, absolut, dan relatif yang naik keluar foldernya.
- **Pada build rilis** dengan library 620 lagu, dikendalikan lewat port debug WebView2 — 24 pemeriksaan, semuanya lulus: urutan playlist lewat perintah dan lewat tombol di antarmuka, lagu yang sama dua kali, memutar playlist dari baris yang diklik, menyimpan antrean, aturan pintar beserta batas dan pratinjaunya, suntikan SQL, editor aturan yang tidak menyimpan apa pun sampai disuruh, dan halaman playlist pada ukuran jendela terkecil.
- **M3U8 lewat dialog file yang sungguhan** (dijawab dari luar aplikasi lewat Win32): ekspor menulis file tanpa BOM dengan garis miring depan dan sepasang baris per lagu; impor membacanya kembali menjadi playlist dengan lagu yang sama dalam urutan yang sama, tanpa satu pun entri hilang.
- **Pemeriksa tata letak** dijalankan ulang dengan dua halaman playlist ikut di dalamnya: 6 lebar jendela × 10 halaman + 4 panel yang menimpa — 64 pemeriksaan, semuanya bersih.
- **Pemeriksaan**: `cargo fmt --check`, clippy `-D warnings`, `cargo test --workspace` (148 tes), dan `svelte-check` bersih.

### Temuan saat pengujian (sudah diperbaiki)

- **Dua bug ditangkap tesnya sebelum ada yang memakainya**: kolom "folder" memanggil fungsi SQL `parent_of` padahal yang terdaftar bernama `onsa_parent` (setiap aturan folder akan gagal), dan memindahkan baris playlist menabrak primary key-nya karena SQLite tidak menjanjikan urutan baris saat UPDATE.
- **Daftar lagu playlist pintar tidak tergambar**: halamannya memakai baris grid tetap, sementara baris pemberitahuannya hanya kadang ada, jadi daftarnya jatuh ke baris setinggi isinya sendiri dan mengerut jadi 30 piksel. Sekarang halamannya kolom flex. Ketahuan lewat tangkapan layar verifikasi, lalu ditutup dengan pemeriksaan tinggi daftar.
- **Jendela yang diminimalkan menimpa sesi dengan geometri omong kosong** — lihat di bawah.

### Perbaikan yang ikut masuk: sesi kehilangan ukuran jendela

Ketahuan saat memverifikasi M6, bukan dilaporkan. Sesi disimpan saat jendela kehilangan fokus; jendela yang **diminimalkan** kehilangan fokus juga, dan melaporkan ukuran nol di tempat jauh di luar layar. Yang tersimpan: `{"width":0,"height":0,"x":-25600,"y":-25600}` — dan Onsa berikutnya terbuka sekecil mungkin.

`WindowMode` sekarang menolak geometri yang tidak layak jadi jendela penuh dan menyimpan jendela terakhir yang layak; setiap ukuran yang dilewati jendela selagi terlihat dicatat, jadi meminimalkan tepat sebelum menutup tidak menghilangkan tempat terakhir yang sungguhan. Ditutup empat tes unit.

### Tertunda / belum diverifikasi

- **Linux**: halaman playlist dan dialog file belum dicoba lewat WSLg.
- **Uji coba oleh pemilik proyek** (langkahnya di `docs/UJI-M6.md`).
- Playlist belum bisa **ditarik-lepas dari daftar lagu**; menambahkan lewat klik kanan sudah ada, satu album atau satu artis sekaligus belum.
- Daftar playlist belum muncul **di dalam sidebar**; sidebar baru punya satu pintu masuk ke halamannya.
- **Folder pintar** (SPEC §6.5) dan ekspor playlist **saat keluar** belum dikerjakan.

## Perbaikan setelah uji coba M6 (2026-09-14)

### Tarik-lepas untuk mengurutkan tidak berfungsi

Dilaporkan pemilik proyek: mengurutkan playlist dengan tarik-lepas tidak jalan, hanya tombol panah.

**Akar masalahnya ada di Tauri.** Target file-drop miliknya memegang drag loop Windows, dan dokumentasinya sendiri menyebut bahwa mematikannya adalah syarat agar HTML5 drag-and-drop bekerja di Windows. Onsa membutuhkan file-drop untuk SPEC §13, jadi HTML5 drag-and-drop memang tidak tersedia — dan panel antrean punya masalah yang sama karena memakai API yang sama.

**Perbaikannya**: satu mekanisme berbasis pointer event (`ui/src/lib/reorder.ts`) dipakai antrean dan playlist, lengkap dengan garis penanda tempat jatuh, gulir otomatis di tepi daftar, dan **Alt+panah** sebagai jalan lewat keyboard. Rinciannya di `docs/DECISIONS.md`.

**Cara verifikasi** — dengan **mouse sungguhan**, digerakkan lewat Win32 di luar aplikasi, bukan lewat kejadian buatan di dalam halaman (itu justru melewati jalur OS yang rusak dan akan selalu "berhasil"):

- menjatuhkan baris pertama di paruh atas baris keempat menaruhnya tepat di atas baris itu; di paruh bawah, satu tempat lebih jauh;
- menariknya kembali ke depan mengembalikan urutan semula;
- klik-dua-kali tetap memutar baris yang ditekan dan tidak mengubah urutan;
- antrean ikut terurut dengan cara yang sama;
- Alt+panah memindahkan baris yang difokus.

Ditambah lima tes unit untuk hitungannya (`ui/src/lib/reorder.test.ts`).

### Lebar kolom yang bisa diatur

- Pembatas kolom bisa ditarik; lebarnya disimpan **per tampilan** (Lagu, Album, Artis, Genre, Folder, playlist, pencarian) dan pulih setelah aplikasi dibuka lagi.
- Menu di ujung kanan kepala tabel memilih kolom mana yang tampil, dengan satu jalan kembali ke bawaan.
- Panah kiri/kanan menggeser pembatas yang difokus, jadi ini pun tidak butuh tetikus.
- Lebar pilihan pengguna dihormati **selama muat**; kalau tidak, kolom dibuang menurut prioritas lama — tahun, lalu album, lalu artis — dan judul serta durasi selalu bertahan.

**Cara verifikasi**: delapan tes unit untuk hitungannya, dua tes Rust untuk penyimpanannya (termasuk setelan yang ditulis versi sebelum kolom ada), lalu pada aplikasi rilis dengan mouse sungguhan — 14 pemeriksaan: menarik pembatas melebarkan kolom (170 → 280 piksel), lebarnya tersimpan untuk daftar itu saja, daftar lain tetap memakai bawaannya, menu menyembunyikan kolom dan mengingatnya, pada jendela terkecil tidak ada kolom yang menindih atau menyempit di bawah batasnya, kolom yang dilebarkan melampaui ruang mendorong keluar yang berprioritas lebih rendah, dan jalan kembali ke bawaan mengembalikan semuanya. Susunan kolom juga diperiksa **setelah aplikasi benar-benar ditutup dan dibuka lagi**.

**Pemeriksa tata letak dijalankan ulang**: 6 lebar jendela × 10 halaman + 4 panel yang menimpa — 64 pemeriksaan, bersih. Ia menemukan satu bug nyata dalam proses: padding baris dan celah antar-sel tidak ikut dihitung, sehingga kepala tabel menjorok keluar tepat 88 piksel.

**Tes antarmuka sekarang ikut CI** (`npm test` di `ui/`, dengan `node --test`).

### Membawa lagu ke playlist, dan satu album sekaligus

Diminta bersama perbaikan tarik-lepas, karena mekanismenya sama.

- Sebuah baris daftar lagu bisa **ditarik ke playlist di sidebar**; sidebar sekarang menampilkan enam playlist yang terakhir berubah, lalu "Lihat semua". Sebuah lencana mengikuti kursor, dan playlist yang dilewati menyala.
- Halaman album, artis, genre, dan folder punya tombol **Tambah ke playlist** yang memasukkan seluruhnya sekaligus, termasuk ke playlist baru yang dibuat saat itu juga.

**Cara verifikasi**, dengan mouse sungguhan pada build rilis: membawa sebuah baris ke playlist di sidebar menambahkannya; melepasnya di tempat yang bukan sasaran tidak menambah apa pun; dan tombol di halaman album memasukkan ketiga lagunya sekaligus. Pemeriksa tata letak dijalankan ulang dengan sidebar yang sudah berisi playlist — 64 pemeriksaan, bersih.

### Awal M7: kunci layanan luar, klien HTTP, dan kolom aturan baru

- **Klien HTTP untuk seluruh proyek** masuk SPEC §1 sebagai keputusan terkunci: `reqwest` dengan TLS bawaannya (rustls), klien blocking. Satu tempat memegang aturannya — User-Agent, timeout koneksi dan total, batas ukuran jawaban, batas redirect, dan kegagalan yang dikembalikan sebagai nilai. URL tidak pernah masuk log, karena di situlah kunci API menumpang.
- **Kunci AcoustID** datang dari Pengaturan, variabel lingkungan `ONSA_ACOUSTID_API_KEY`, atau lingkungan saat build — yang pertama ada menang. Kunci tidak pernah kembali ke antarmuka; yang bisa ditanyakan hanya *apakah* ada dan *dari mana*. Halaman Pengaturan → Metadata punya sakelar internet yang **mati secara bawaan**, kolom kunci, dan tombol **Coba kunci** yang benar-benar bertanya ke AcoustID (lewat lookup `trackid`, jadi tidak perlu membaca audio sama sekali).
- **Lima kolom aturan baru** untuk mengumpulkan file yang perlu dirapikan: sample rate, kedalaman bit, bitrate, punya cover, punya tag artis. Pada library 624 lagu milik pemilik proyek: 455 lagu tanpa tag artis, 473 tanpa cover, 436 di atas 44,1 kHz, 114 di bawah 192 kbit/s.
- **Pratinjau aturan menampilkan delapan lagu pertama**, bukan hanya jumlahnya.

**Cara verifikasi**: 159 tes Rust dan 20 tes antarmuka; rantai kunci diperiksa pada aplikasi sungguhan (setelan menang atas lingkungan, mengosongkannya jatuh kembali, dan nilainya tidak muncul di jawaban perintah mana pun maupun di log); tombol Coba kunci menjawab `offline` saat sakelarnya mati dan `refused` untuk kunci palsu — jawaban sungguhan dari AcoustID, jadi jalur TLS-nya ikut terbukti; perintah itu berjalan di `spawn_blocking`, dan perintah lain tetap dijawab dalam 1 ms selagi ia menunggu. Pemeriksa tata letak dijalankan ulang (64 pemeriksaan) beserta pemeriksa khusus untuk editor aturan di tiga ukuran jendela.

**Temuan saat pengujian (sudah diperbaiki)**: `fileName()` disalin di tujuh komponen, dan satu salinannya kehilangan sebuah backslash sehingga path Windows tampil utuh sebagai judul lagu di pratinjau aturan. Sekarang fungsinya satu di `ui/src/lib/format.ts` dengan tesnya sendiri — kesalahan yang sama sempat terjadi dua kali dalam satu sesi.

### Pengaman metadata, sebelum apa pun ditulis (2026-09-14)

Diminta pemilik proyek sebagai syarat sebelum metadata otomatis dijalankan ke library sungguhan. Semuanya ada di lapisan library dan mesin, dengan tesnya, **sebelum** kode pencocokan ditulis.

| Yang diminta | Keadaannya |
|---|---|
| Batalkan ada dan teruji sebelum penulisan massal pertama, termasuk nama file yang sudah diubah dan dipindahkan | ✅ jurnal per batch, pembatalan mundur, mencakup override, tag di dalam file, dan pemindahan file |
| Mode uji coba di folder terbatas | ✅ `Scope` dengan folder; diperiksa dua kali untuk pemindahan |
| Tingkat keyakinan ditampilkan, yang rendah tidak tercentang otomatis | ✅ ambang 0,85; di bawah itu tidak satu pun field tercentang |
| Batas jumlah lagu per sekali jalan | ✅ bawaan 50, langit-langit keras 500 |
| Batas laju MusicBrainz dan AcoustID, tanpa membuat aplikasi tersendat | ✅ 1/detik dan 3/detik, ditunggu di thread pekerja |

**Yang dibuat**: skema versi 3 (`edit_batches`, `edit_steps`); `edits.rs` (cakupan folder, batas, penerapan berjurnal, pembatalan); `write.rs` (penulisan tag yang aman lewat salinan sementara, pembacaan ulang sebelum mengganti, pemindahan file); `rename.rs` (pola, dry-run, daftar bentrok, penerapan berjurnal); `proposal.rs` (model usulan dengan keyakinan); batas laju di `net.rs`.

**Cara verifikasi**: 186 tes Rust, di antaranya 12 tes integrasi khusus pengaman ini — run yang dibatasi folder tidak menyentuh folder lain, cakupan tidak bisa diakali lewat `..` atau nama folder yang mirip, batas per run ditaati, run bisa dibatalkan utuh (termasuk mengembalikan nilai override yang sebelumnya ada, bukan sekadar menghapusnya), penulisan ke file bisa dibatalkan dan file tetap bisa diputar sesudahnya, penulisan yang gagal tidak menyentuh file asli dan tidak meninggalkan sampah, rename punya dry-run yang mendaftar bentrokan, dan pemindahan bisa dikembalikan berikut path di library.

**Temuan**: lofty kehilangan tag ID3v2 di dalam WAV pada penulisan kedua berturut-turut. Pendekatan salin-lalu-ganti menghindarinya, dan pembacaan ulang sebelum penggantian menangkapnya seandainya kasus serupa muncul di format lain.

**Tertunda**: belum ada apa pun yang bisa diklik. Pengaman ini fondasi; pencocokan MusicBrainz/AcoustID, daftar usulan, dan tombol batalkan di antarmuka menyusul.
