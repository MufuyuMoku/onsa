# Progres

Status tiap milestone dari `SPEC.md` §15. Diperbarui di akhir setiap milestone.

---

## Garis rilis (keputusan pemilik proyek, 2026-09-19)

**v1**: pemutar lengkap, unduhan satu per satu, tanpa Opus. Isinya M0–M8 dan M10a.
Sesudah M8 tidak ada fitur baru yang masuk v1 — yang tersisa hanya penyiapan rilis.

**v1.1**: M7b (hapus/ekspor sampul, menanam sampul ke berkas beserta aturan 1200 px,
"(beragam)", dan tiga field yang belum bisa di-override), M10b (Deno dan ffmpeg, antrean
paralel, antrean yang disimpan, tombol update yt-dlp), M11 (Opus dan kualitas lanjutan),
dan M12 (paket rilis).

M9 (scrobble) tetap dilewati dan belum punya tanggal.

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

### Halaman Rapikan, diuji sendiri di folder buatan (2026-09-14)

Sesuai permintaan pemilik proyek: berkas ujinya dibuat sendiri, seluruh alurnya diuji sendiri sampai tuntas, dan cakupan foldernya dibuktikan mengunci — sebelum ia mencobanya.

**Antarmukanya** (`Rapikan` di sidebar, tanpa jaringan sama sekali): edit massal satu field, tulis ke file, dan ganti nama berpola. Ketiganya berjalan dengan pola yang sama — sebutkan yang diinginkan, baca ringkasannya, lalu tekan tombol yang ada **di bawah** ringkasan itu. Riwayat di bagian bawah bisa membatalkan tiap run.

- **Spanduk cakupan** selalu ada di atas pekerjaan. Dibatasi folder: warna aktif, nama folder besar, path lengkap di bawahnya. Tanpa batas: warna peringatan, tulisan "SELURUH LIBRARY", dan satu kalimat yang menyebutkan akibatnya.
- **Ringkasan** menyebut berapa lagu terkena, berapa nilai field berubah, berapa file ditulis ulang, berapa dipindah — dan berapa yang dilewati beserta alasannya (di luar folder, melewati batas, sudah sama, namanya bentrok).

**Berkas ujinya** dibuat dengan ffmpeg dari nada uji, bukan dari musik siapa pun, di folder sementara sistem — tidak di repo, tidak di folder musik: FLAC bercover dengan judul dan folder beraksara Jepang, MP3 bertag sebagian, M4A tanpa tag sama sekali, WAV bernama sangat panjang berspasi, OGG dengan tag berisi karakter yang tidak boleh ada di path (`AC/DC`, `B:Side`, `What? *Really*: yes`), dua MP3 yang akan mendarat di nama yang sama setelah rename, satu FLAC yang sengaja rusak, dan dua berkas di folder lain yang tidak boleh tersentuh.

**Pengujiannya** dijalankan terhadap **library sekali pakai** lewat `ONSA_DATA_DIR`, jadi library sungguhan pemilik proyek tidak pernah dibuka. Diperiksa sesudahnya: dua kali jalan terakhir memakai folder data sementara, dan tidak satu berkas pun di folder musik yang tersentuh.

36 pemeriksaan pada aplikasi rilis, semuanya lulus:

- berkas yang bukan berkas suara ditandai `failed` saat scan, dan saat penulisan ia **ditolak dengan alasan** sementara tujuh lainnya tetap ditulis;
- hanya lagu di dalam folder cakupan yang ditawarkan;
- **perintah yang diarahkan langsung ke berkas di luar folder ditolak** — nol perubahan, dihitung sebagai di luar cakupan, tidak ada run yang tercatat, dan lagu di luar itu tetap seperti semula;
- ringkasan dihitung sebelum apa pun terjadi, dan tidak mengubah apa pun;
- yang diterapkan sama persis dengan yang dikatakan ringkasan;
- tag di dalam berkas benar-benar berubah di disk (diperiksa dengan ffprobe, bukan dengan Onsa sendiri), dan berkas di luar folder tidak;
- pembatalan mengembalikan tag persis seperti semula, dan berkasnya masih bisa diputar;
- **membatalkan run yang sama dua kali ditolak**;
- rename ditampilkan lebih dulu: dua berkas yang akan bentrok ditandai dan tidak dijalankan, tag berisi `AC/DC` tidak berubah jadi dua folder, judul dan folder beraksara Jepang tersusun benar;
- pembatalan rename mengembalikan nama lama dan menghapus yang baru, berikut path di library;
- **setelah aplikasi ditutup dan dibuka lagi**: cakupan foldernya masih sama, run yang belum dibatalkan masih ada, bisa dibatalkan, dan berkasnya kembali persis seperti semula.

Pemeriksa tata letak dijalankan ulang dengan halaman ini ikut di dalamnya — 6 lebar × 11 halaman + 4 panel, bersih.

**Temuan**: regex pemisah path yang disalin di banyak komponen kehilangan sebuah backslash **empat kali dalam satu sesi**. Sekarang `fileName()` dan `relativeTo()` ada satu-satunya di `ui/src/lib/format.ts` dengan tesnya, dan tidak ada komponen yang menulis regex path lagi.

### Mencari tahu lewat internet: AcoustID, MusicBrainz, Cover Art Archive (2026-09-15)

Semuanya masuk lewat pintu yang sudah ada. Pencocokan hanya menghasilkan **daftar usulan**; yang dicentang dikirim ke perintah yang sama dengan editan manual, jadi cakupan folder, ringkasan sebelum tombol, dan riwayat yang bisa dibatalkan berlaku dengan sendirinya. Tidak ada jalur pintas — itu yang diuji, bukan sekadar diklaim.

**Pengelola program luar** (`onsa-downloader::programs`) dibuat sekali dan dipakai bersama: `fpcalc` sekarang, yt-dlp/ffmpeg/Deno di M10. Array argumen tanpa shell, `CREATE_NO_WINDOW` di Windows, tenggat dengan kill di ujungnya, keluaran dibaca di thread sendiri. Dicari di pilihan pengguna → folder `bin` Onsa → PATH sistem. Mengunduh binary-nya sendiri ditunda ke M10, tempat tiga program lain butuh pembongkaran arsip yang sama.

**Tingkat keyakinan menentukan perilaku.** Sidik suara memakai skor AcoustID; pencarian teks MusicBrainz ditahan di bawah ambang centang kalau tidak sama dengan judul *dan* artis yang ada di file; tebakan dari nama file bernilai 0,6 atau 0,35. Di bawah 0,85 tidak ada yang tercentang. Kalau sidik suara dan nama file berbeda, **keduanya ditampilkan dan tidak ada yang tercentang**; kalau usulan keduanya terlalu lemah untuk ditampilkan (misalnya berkas bernama `03 kosong` yang "berjudul" kosong), ia tidak dihitung sebagai perselisihan — kalau tidak, hampir semua lagu tanpa tag harus diputuskan satu per satu.

**Cover** diambil sekali per rilis, ditampilkan sebagai gambar lewat protokol `onsa://` sebelum disetujui, masuk ke database (bukan ke dalam berkas), dan bisa dibatalkan seperti perubahan lain. Menanamkan gambar ke dalam tag belum dikerjakan.

**Pengujiannya** memakai layanan tiruan yang berdiri di 127.0.0.1 dan `fpcalc` tiruan di folder `bin` sekali pakai. Alamat layanan hanya bisa dipindahkan ke loopback — di luar itu diabaikan dan dicatat — jadi seluruh alurnya diuji terhadap **build rilis**, lewat HTTP sungguhan dengan klien dan batas laju yang sama, tanpa key sungguhan, tanpa internet, dan tanpa menyentuh musik siapa pun. Berkas ujinya bertambah dua: nama unduhan tanpa tag, satu beraksara Latin dan satu beraksara Jepang.

48 pemeriksaan pada perintah, 18 pada antarmuka, semuanya lulus. Yang terpenting:

- **fitur internet mati = tidak ada satu pun permintaan** yang sampai ke layanan, dan tombolnya mati;
- mematikannya di tengah jalan **menghentikan run yang sedang berjalan**, dan yang sudah ditemukan tetap ada;
- lagu tanpa tag dikenali lewat suaranya, lengkap dengan tahun dari MusicBrainz, dan **sudah tercentang** karena keyakinannya 96%;
- lagu yang nama berkasnya bertentangan dengan suaranya **ditandai, dan tidak satu field pun tercentang**;
- nama berkas dibaca tanpa `[dQw4w9WgXcQ]` dan tanpa `(Official Video)`;
- berkas yang bukan berkas suara menyebut alasannya dan tidak menghentikan yang lain;
- usulan 55% ditampilkan tapi tidak tercentang;
- sampul tampil sebagai gambar yang benar-benar termuat, dipasang, lalu dibatalkan sampai kembali kosong;
- **aplikasi tetap menjawab dalam 2 ms** sementara layanan yang lambat membuat run berjalan puluhan detik;
- layanan yang mati di tengah jalan: run tetap selesai, tiap lagu menyebut apa yang terjadi, dan yang sudah selesai tidak hilang;
- ringkasan muncul hanya setelah diminta, tombolnya di bawah angka-angkanya, dan **mengubah satu centang langsung menghapus ringkasannya**;
- run yang diterapkan sebelum aplikasi ditutup **masih bisa dibatalkan setelah dibuka lagi**, sementara daftar usulannya sendiri tidak ikut disimpan;
- perintah yang diarahkan langsung ke berkas di luar folder tetap ditolak.

Pemeriksa tata letak dijalankan ulang dengan daftar usulan terbuka di dalamnya — 6 lebar × 12 halaman + 4 panel.

**Temuan**: heredoc bash di lingkungan ini memakan satu backslash, **untuk kelima kalinya dalam dua sesi** — kali ini di sebuah skrip uji, yang membuat folder cakupan menjadi omong kosong. Kebetulan itu membuktikan cakupannya tidak tertipu (nol lagu, nol perubahan), tapi pelajarannya tetap: berkas yang mengandung backslash ditulis lewat alat tulis berkas, tidak pernah lewat heredoc.

### Perapihan otomatis, tanpa jaringan (2026-09-15)

Chip kelima di halaman Rapikan: `Rapikan otomatis`. Library membaca dirinya sendiri dan menyebutkan apa yang akan ditulisnya berbeda — sisa nama unduhan yang menempel di judul, judul yang berteriak atau berbisik, `ft`/`FEAT`/`Featuring` yang jadi satu bentuk, nama artis yang tertulis beda-beda di library yang sama, dan album tanpa artis album. Tiap baris menyebut alasannya.

Diterapkan lewat pintu yang sama seperti sebelumnya: ringkasan dulu, tombol di bawahnya, riwayat yang bisa dibatalkan. Yang jelas perbaikan tercentang; yang menyangkut huruf besar-kecil tidak — sering kali itu memang ditulis begitu dengan sengaja, dan halaman itu mengatakannya.

Diuji pada folder buatan yang sama, 14 pemeriksaan pada perintah dan 9 pada antarmuka, semuanya lulus:

- nama yang ditulis tiga cara menetap pada ejaan yang dipakai paling banyak — dan **seri tidak menghasilkan usulan apa pun**;
- judul kehilangan `(Official Video)` dan berhenti berteriak, dengan dua alasan tercantum;
- `Mina ft Lilith` menjadi `Mina feat. Lilith`;
- tidak ada satu pun usulan untuk lagu di luar folder cakupan;
- yang diterapkan sama persis dengan ringkasannya, file di disk tidak tersentuh, dan pembatalan mengembalikan ejaan lamanya;
- bertanya lagi sesudahnya tidak menemukan lagi apa yang baru saja diperbaiki.

**Temuan dari melihat daftarnya sendiri**: baris pertama mengusulkan `AC/DC → Ac/Dc`. Satu kata dalam satu jenis huruf sama seringnya adalah cara menulis nama, bukan teriakan; sekarang usulan huruf besar-kecil hanya berlaku untuk frasa dua kata atau lebih. `LILITH` dan `deadmau5` ikut selamat karena aturan yang sama.

### Skema tema diperdalam, dan tiga tema terang (2026-09-18)

**Skemanya** kini lima lapis, bukan tiga (SPEC §9.3). Yang bertambah: warna tepi (`color.edge`), bentuk kontrol (`soft`, `square`, `pill`, `bevel`), bingkai panel (`hairline`, `inset`, `raised`, `none`), gaya baris daftar (`plain`, `lines`, `stripes`), scrollbar (`thin`, `classic`, `hidden`), tooltip (`plain`, `panel`), dialog (`flat`, `raised`, `titled`), dan gerak (`motion.fast`, `motion.slow`, `motion.ease`).

Aturannya: **kalau sebuah tema butuh sesuatu, skemanya yang diperdalam, bukan kodenya yang diberi pengecualian.** Tidak ada satu pun aturan CSS yang menyebut nama tema. Dan tiap field baru default-nya persis seperti tampilan Onsa sebelum field itu ada — ada tesnya, supaya tema yang sudah ditulis siapa pun tidak berubah sendiri.

Tema boleh mengatur tempo, tidak boleh mengatur perangai: kedua durasi dijepit ke rentang yang tetap cepat, dan keempat kurva yang tersedia dibuktikan tidak memantul.

Onsa juga menggambar **tooltip-nya sendiri** sekarang untuk catatan pendek di atas instrumen; tooltip sistem tetap dipakai untuk path panjang.

**Tiga tema bawaan yang lama** kini menyebutkan dirinya lewat kosakata baru itu: Kaca asap lembut dan cekung dan bergerak pelan; Kokpit kaca bersudut siku, timbul, dan bergerak patah; Deck malam bertepi timbul, berbaris belang, dengan scrollbar yang bisa dipegang pakai sarung tangan.

**Tiga tema baru** (atas persetujuan pemilik proyek, SPEC §1 diperbarui jadi enam bawaan — tiga gelap, tiga terang; Onsa tetap membuka dengan yang gelap):

- **Kubikel biru** — kantor awal 2000-an: panel krem-abu, kontrol siku bertepi timbul, bilah judul dialog biru, scrollbar lebar, garis tipis antar baris.
- **Kilau milenium** — plastik bening: kontrol kapsul, kaca, glow, playhead merah muda, baris berselang-seling.
- **Musim dingin utara** — pagi bersalju: putih kebiruan, meter jarum bermuka krem, aksen tembaga, panel sedikit cekung, label kapital kecil.

Ketiganya **terinspirasi, bukan meniru**: tidak ada logo, ikon, wallpaper, atau aset asli dari mana pun, dan namanya tidak menyebut produk atau negara. Tidak ada font baru — enam font yang sudah dibundel dipilih ulang untuk masing-masing watak.

**Diperiksa**: keenam tema dibuka satu per satu dan dibuktikan benar-benar memakai apa yang dikatakannya (atribut di root, token bernilai, tepi yang benar-benar timbul, garis baris yang benar-benar ada); nama dialog dibuktikan terbaca di atas bilahnya di keenam tema; dan pemeriksa tata letak dijalankan ulang dua kali lagi — sekali di tema berscrollbar lebar, sekali di tema terang — 6 lebar × 13 halaman + 4 panel, bersih.

**Temuan**: sebuah backslash hilang lagi, kali ini dimakan template literal JavaScript di skrip pemeriksa, membuat regex-nya diam-diam tidak pernah cocok sehingga pemeriksaan kontras membaca latar yang salah. Pelajarannya sama dengan heredoc: di tempat yang memproses escape, jangan menulis regex — pemeriksaannya ditulis ulang tanpa satu backslash pun.

## M7a. Metadata — selesai (2026-09-18)

**Kriteria selesai** (SPEC §15): tes metadata lulus, termasuk simulasi kegagalan penulisan.

| Kriteria | Hasil |
|---|---|
| Tes metadata lulus | ✅ 23 tes integrasi di `crates/onsa-library/tests/edits.rs`, ditambah tes unit `write`, `rename`, `clean`, `auto`, `edits` |
| Simulasi kegagalan penulisan | ✅ `a_write_that_fails_leaves_the_original_alone`: berkas yang tidak bisa dibaca lofty ditolak, aslinya tetap **byte per byte** seperti semula, dan tidak ada berkas sementara yang tertinggal di sebelahnya |
| Tes (§12) | ✅ 273 tes Rust dan 29 tes antarmuka; `cargo fmt --check`, clippy `-D warnings`, dan `svelte-check` bersih |
| CI | ✅ hijau di windows-latest dan ubuntu-latest |

Milestone ini ditutup sebagai **M7a** atas keputusan pemilik proyek (2026-09-18): sisa §8 yang belum dikerjakan dicatat sebagai **M7b** di bawah, dan **editor tag satuan pindah ke M8** karena lirik adalah salah satu field yang diedit di sana — membangun editornya dua kali tidak masuk akal.

### Yang dibuat

- **Skema versi 3 dan 4**: `edit_batches` dan `edit_steps` — catatan apa yang Onsa ubah, supaya bisa dikembalikan. `track_id` sengaja bukan foreign key: catatannya harus hidup lebih lama daripada baris track-nya. Versi 4 melebarkan catatan itu agar bisa memuat sampul.
- **Tiga lapis yang bisa dibatalkan** (SPEC §8): editan masuk `overrides` dulu (tidak ada berkas yang tersentuh), "tulis ke berkas" aksi kedua, memindahkan berkas aksi ketiga. Pembatalan menuruni tangga yang sama, langkah terakhir lebih dulu.
- **Penulisan aman**: salin ke berkas sementara di folder yang sama, tulis, `fsync`, **baca ulang setiap field sebelum menggantikan aslinya**, lalu rename atomik. Bila gagal di mana pun, yang asli tidak pernah tersentuh.
- **Rename berpola dengan dry-run**: pola `{album_artist}/{album}/{track:02} {title}`, karakter yang tidak boleh ada di path diganti (termasuk pemisah path di dalam nilai tag, supaya `AC/DC` tidak menjadi dua folder), bentrokan ditampilkan dan tidak dijalankan, dan rencananya disusun ulang di backend saat diterapkan.
- **Pengaman** (diminta pemilik proyek sebelum apa pun ditulis): cakupan folder yang selalu terlihat, batas per sekali jalan (50, langit-langit 500), ringkasan sebelum tombol, dan riwayat yang bisa dibatalkan — termasuk setelah aplikasi ditutup dan dibuka lagi.
- **Halaman Rapikan** dengan lima pekerjaan: edit massal, tulis ke berkas, ganti nama berpola, **rapikan otomatis** (tanpa jaringan), dan **cari data online**.
- **AcoustID, MusicBrainz, dan Cover Art Archive** di belakang pintu yang sama persis: pencocokan hanya menghasilkan usulan, dan yang dicentang dikirim ke perintah yang sama dengan editan manual. Tingkat keyakinan menentukan apa yang tercentang sendiri; dua sumber yang berbeda ditampilkan berdua tanpa satu pun tercentang.
- **Pengelola program luar** (`onsa-downloader::programs`), dibuat sekali untuk M7 dan M10.
- **Klien HTTP tunggal** (SPEC §1): User-Agent, timeout, batas ukuran jawaban, batas laju per layanan, dan kegagalan yang dikembalikan sebagai nilai.

### Cara verifikasi

Semuanya di `docs/PROGRESS.md` bagian-bagian di atas: 36 pemeriksaan untuk halaman Rapikan, 48 + 19 untuk pencarian online, 14 + 9 untuk perapihan otomatis — seluruhnya terhadap **build rilis** dengan library sekali pakai (`ONSA_DATA_DIR`), berkas uji buatan sendiri, dan layanan tiruan di loopback. Tag diperiksa dengan ffprobe, bukan dengan Onsa sendiri.

### M7b — utang §8 yang belum dikerjakan

Dikerjakan **setelah M10 penuh**, atas keputusan pemilik proyek:

- **Sampul belum bisa dihapus atau diekspor.** Sekarang hanya bisa diganti (dari Cover Art Archive).
- **Sampul belum ditanam ke berkas.** Yang ada baru pointer di database plus thumbnail 128/512 di cache; aturan §8 "gambar besar diperkecil dulu, maksimal 1200 px, dengan opsi mempertahankan asli" belum berlaku karena belum ada yang menulis gambar ke tag.
- **"(beragam)" belum ada.** Edit massal menyetel satu nilai untuk semua lagu; field yang nilainya berbeda antar-lagu belum ditampilkan sebagai "(beragam)" dan dibiarkan.
- **Empat field belum bisa di-override**: nomor trek total, nomor disk total, dan komentar. (Lirik ikut M8.)

### Pindah ke M8

- **Editor tag satuan (per lagu)** — lirik adalah salah satu field di dalamnya.

## M10a. Pengambil binary dan yt-dlp — selesai (2026-09-18)

Dikerjakan lebih awal atas keputusan pemilik proyek (lihat `docs/DECISIONS.md`): bagian M10 yang menentukan rilis pertama, dikerjakan sebelum M8.

**Kriteria selesai M10** belum terpenuhi seluruhnya — itu milik M10b. Yang sudah terpenuhi dari kriteria itu:

| Kriteria M10 | Keadaan |
|---|---|
| URL satu video diunduh sampai muncul di library | ✅ diuji ujung ke ujung terhadap build rilis |
| URL satu playlist | ⏳ pratinjau playlist sudah ada dan bisa dicentang per item; antrean paralel M10b |
| Tidak ada jendela konsol di Windows | ✅ `CREATE_NO_WINDOW` di semua proses luar sejak pengelola program dibuat |
| Tombol batal menghentikan semua proses turunan | ✅ Job Object di Windows, process group di Linux — diuji dengan menghitung proses sebelum, saat, dan sesudah |
| Pengelolaan binary: persetujuan, checksum, opsi program sistem, versi | ✅ untuk berkas tunggal (yt-dlp). Update = unduh ulang. Deno dan ffmpeg (arsip) M10b |

### Yang dibuat

- **Pengambil binary** (`onsa-downloader::install`): katalog sumber resmi yang **ditulis di kode** (URL, nama berkas di daftar checksum, perkiraan ukuran), pembacaan `SHA2-256SUMS`, dan pemasangan yang menulis ke berkas sementara, `fsync`, memberi izin jalan di Unix, lalu rename. **Yang checksum-nya tidak cocok tidak ditulis ke mana pun.**
- **Halaman Unduhan** dengan persetujuan §7.1 sejak awal: nama, sumber, perkiraan ukuran, lalu tombol — tidak ada yang diambil sebelum ditekan. Program yang berbentuk arsip menyebutkan dirinya begitu, bukan diam-diam hilang. Opsi "pakai program sistem" ada di halaman itu, dan nilainya datang dari yang tersimpan, bukan dari tebakan.
- **Pengambilan bertahap** lewat klien HTTP yang sama dengan seluruh proyek, dengan batas ukuran, laporan kemajuan, dan tombol hentikan.
- **Penghentian sampai ke anak-anaknya** (`onsa-downloader::platform::Group`): Job Object di Windows (dengan `KILL_ON_JOB_CLOSE`, jadi unduhan tidak hidup lebih lama daripada Onsa) dan process group di Linux. Tidak ada dependensi baru: `windows-sys` dan `libc` sudah ada di pohon dependensi.
- **Aliran baris** (`onsa-downloader::runner`): tiap baris yt-dlp dibaca saat dicetak, stderr dikuras di thread sendiri supaya pipa penuh tidak membuat semuanya berhenti, dan permintaan berhenti dijawab di antara baris.
- **yt-dlp** (`src-tauri::ytdlp`): pratinjau `-J --flat-playlist` (satu video maupun playlist), argumen §7.2 **di satu tempat** supaya prioritas format yang harus berubah di M11 satu baris, parser progres yang toleran (baris asing masuk log, tidak menggagalkan apa pun), dan `--print after_move:filepath` sebagai satu-satunya sumber nama berkas hasil.
- **Antrean** satu per satu di memori, dengan status per baris (menunggu, sedang diunduh, selesai, gagal, dihentikan), kecepatan, dan sisa waktu. Berkas yang selesai diserahkan ke library, jadi lagunya langsung muncul.

### Cara verifikasi

**Berkas uji dibuat sendiri dan dilayani dari mesin ini**: sepotong nada 90 detik dibuat dengan ffmpeg lalu disajikan lewat HTTP di 127.0.0.1, dengan varian yang sengaja lambat. Jadi seluruh alur unduhan diuji tanpa mengambil musik siapa pun dan tanpa bergantung pada layanan mana pun.

- **Pengambil binary, 15 pemeriksaan**: halaman menyebut sumber dan ukuran sebelum apa pun terjadi; yt-dlp benar-benar diunduh dari rilis resminya; **hasilnya dibandingkan byte per byte dengan checksum yang diterbitkan rilis itu** (diperiksa di luar Onsa, dengan hash sendiri); tidak ada `.part` atau `.old` tertinggal; versinya terbaca; memasang ulang mengganti; opsi program sistem berpengaruh; program yang belum bisa dipasang ditolak.
- **yt-dlp, 16 pemeriksaan**: URL ditanyakan tanpa mengunduh; yang bukan alamat web ditolak; antrean menerima, berjalan, dan selesai; berkasnya benar-benar ada, utuh, di folder yang disebutkan; **muncul di library**; lalu unduhan lambat dihentikan — **jumlah proses yt-dlp dan ffmpeg sebelum dan sesudah sama**, dan barisnya menyebut dirinya dihentikan.
- **Antarmuka, 8 pemeriksaan**: bagian URL muncul hanya setelah yt-dlp ada; daftar isinya bisa dicentang; catatan "konversi tidak menambah kualitas" muncul hanya saat konversi dipilih; antrean menunjukkan yang sedang berjalan lalu selesai.
- **Pemeriksa tata letak** dijalankan ulang dengan halaman Unduhan di dalamnya.

### Temuan saat pengujian (sudah diperbaiki)

- Bar kemajuan menggambar **penuh** selama panjang berkas belum diketahui — terbaca seperti sudah selesai. Sekarang kosong sampai ada yang bisa diisi.
- Dua pembacaan daftar program bisa berjalan bersamaan, dan yang lambat bisa mendarat di atas yang baru — saklar "pakai program sistem" tampak tidak berfungsi. Sekarang jawaban yang lebih baru selalu menang.
- Saklar itu juga digambar dari tebakan, bukan dari yang tersimpan; sekarang nilainya ikut dalam jawaban yang sama dengan daftarnya.
- Daftar programnya kosong selama beberapa detik pertama (memeriksa versi tiap program butuh waktu), tanpa mengatakan apa-apa. Sekarang ia mengatakannya.

### Belum dikerjakan (M10b)

- Deno dan ffmpeg: keduanya arsip, jadi butuh pembongkar arsip beserta keputusan dependensinya.
- Antrean paralel (bawaan 2) dan antrean yang disimpan di database.
- Menanyakan apakah folder keluaran mau ditambahkan ke library bila ia di luar library (sekarang hasil unduhan selalu ditaruh di dalam folder library pertama, dan Onsa menolak mengunduh bila belum ada folder library sama sekali).
- Tombol update khusus yt-dlp (`yt-dlp -U`) — sekarang "perbarui" berarti mengunduh ulang rilis terbaru, yang hasilnya sama.

## M8. Lirik dan editor tag satuan — selesai (2026-09-19)

Termasuk **editor tag satuan**, pindahan dari M7 atas keputusan pemilik proyek: lirik salah satu field yang diedit di sana, jadi membangunnya dua kali tidak masuk akal.

**Kriteria selesai M8** ("lirik `.lrc` lokal dan lirik dari LRCLIB tampil sinkron, dan offset tersimpan") terpenuhi, diuji ujung ke ujung terhadap build rilis.

### Yang dibuat

- **Format LRC** (`onsa-lyrics::lrc`): parser yang memaafkan apa yang benar-benar ditulis orang — beberapa waktu dalam satu baris, titik dua di tempat titik, tag tentang berkas (`[ti:]`, `[offset:]`), baris yang urutannya terbalik, timing per kata `<mm:ss.xx>`, dan berkas yang bukan UTF-8. Tidak ada yang ditolak, dan yang tak terbaca tetap jadi baris teks, bukan hilang.
- **Empat sumber, yang terdekat lebih dulu** (`src-tauri::lyrics`): suntingan sendiri → `.lrc` di sebelah lagu → tag di dalam lagu → LRCLIB. Tiga yang pertama dibaca tiap kali lagu berganti (membaca berkas kecil lebih murah daripada mengingat); hanya yang keempat yang keluar dari mesin ini.
- **LRCLIB** (`onsa-lyrics::lrclib`): pertanyaan dan pembacaan jawaban ada di crate; yang mengambil adalah klien HTTP yang sama dengan seluruh proyek. Yang dikirim hanya artis, judul, album, dan durasi. Hasil pencarian yang durasinya meleset lebih dari tiga detik dianggap rekaman lain.
- **Cache dan offset dalam satu baris** (migrasi 5). Yang diingat hanya jawaban dari internet — termasuk "tidak ada liriknya", supaya lagu itu tidak ditanyakan terus. Offset yang disetel pendengar hidup lebih lama daripada liriknya: mengganti atau melupakan lirik tidak menghapus setengah detik yang disetel dengan telinga.
- **Panel kanan jadi dua tab** (Antrean / Lirik, SPEC §9.2), dan lirik yang sama muncul besar di layar Sedang Diputar. Baris yang sedang dinyanyikan memakai warna yang di seluruh jendela berarti "di sini"; klik baris untuk melompat; lirik menggulir sendiri, kecuali saat pendengar sedang menggulirnya.
- **Tombol ± per lagu** menggeser lirik terhadap lagu (0,25 detik sekali tekan, dijepit ±30 detik) dan nilainya tersimpan.
- **Menyimpan `.lrc` di sebelah lagu**, baik lewat tombol maupun otomatis untuk hasil dari internet (opsional, mati secara bawaan) — supaya liriknya jadi milik pendengar dan tetap ada tanpa internet.
- **Pengaturan → Lirik**: satu saklar untuk jaringan (mati secara bawaan) dan satu untuk menyimpan `.lrc`. Mematikannya juga **menghentikan pencarian yang sedang di udara**: apa pun yang kembali dibuang, tidak disimpan dan tidak ditampilkan.
- **Editor tag satuan** (`src-tauri::editor`): sepuluh field termasuk lirik, dengan tanda pada yang nilainya milik Onsa dan tombol untuk mengembalikannya ke isi berkas. Suntingan masuk ke `overrides` dulu; "tulis ke berkas" tetap tombol terpisah. Cakupan satu jalannya adalah **folder lagu itu sendiri**, jadi editor yang terbuka atas satu lagu tidak bisa menyentuh yang lain. ReplayGain ditampilkan dan tidak ditawarkan untuk diubah.
- **Lirik jadi field tag biasa**, jadi undo, riwayat, dan "tulis ke berkas" dipakai bersama. ID3 menyimpannya di `USLT`, Vorbis dan MP4 di nama biasa; format yang tidak bisa menampungnya (RIFF INFO di WAV) membuat penulisan **ditolak**, bukan liriknya diam-diam hilang.

### Cara verifikasi

**Berkas uji dibuat sendiri**: lima nada tiga detik dibuat dengan ffmpeg dan ditandai (`lirik uji/`), satu dengan `.lrc` di sebelahnya, satu dengan lirik tertanam di tag, satu yang liriknya hanya ada di layanan tanpa waktu, satu yang tidak dikenal siapa pun, dan satu yang liriknya **hanya** ada di layanan dan **bersinkron**. LRCLIB berdiri di 127.0.0.1. Tidak ada musik siapa pun yang disentuh dan tidak ada yang keluar dari mesin ini.

- **Dari perintah, 43 pemeriksaan**: urutan sumber (yang terdekat menang, termasuk atas layanan); tag `[ti:]` tidak jadi baris lagu; waktu terbaca benar; dengan jaringan mati **tidak ada satu pun permintaan** dan "cari lagi" ditolak; dengan jaringan hidup jawabannya kembali seketika sementara layanan ditanya di thread lain; jawabannya diingat dan tidak ditanyakan berulang; **layanan yang mati tidak dicatat sebagai "lagu ini tidak punya lirik"**, dan liriknya datang sendiri setelah layanan kembali; mematikan jaringan saat pencarian di udara membuang hasilnya; offset tersimpan dan dijepit; `.lrc` bisa ditulis dan langsung jadi sumbernya; editor menyimpan ke Onsa tanpa menyentuh berkas (**diperiksa dengan ffprobe di luar Onsa**), lalu menulis ke berkas saat diminta, dan satu jalannya bisa dibatalkan lewat riwayat yang sama.
- **Dari jendela, 22 pemeriksaan**: dua tab di panel kanan; lagu tanpa lirik mengatakannya; baris yang sedang dinyanyikan menyala dan berpindah saat lagu berjalan; klik baris melompat; tombol ± menggeser dan jendela menyebut berapa; layar Sedang Diputar menampilkan lirik yang sama; menu klik-kanan membuka editor; sepuluh field; menyimpan mengatakan "berkas belum diubah"; mengembalikan ke isi berkas jalan; lirik yang diketik di editor muncul di panel tanpa berganti lagu; halaman pengaturan menyebut apa yang dikirim.
- **Editor di jendela terkecil, 6 pemeriksaan** (686×483 dan 1000×700): tidak ada yang tumpah, tidak ada yang menggulir ke samping, tombol tetap terjangkau.
- **Kriteria selesai M8 diuji tersendiri, 8 pemeriksaan**: lagu yang liriknya tidak ada di mana pun kecuali di layanan menerima lirik **bersinkron**, tiap barisnya punya waktu sendiri, baris yang sedang dinyanyikan menyala di jendela dan berpindah saat lagu berjalan, dan **offset yang disetel masih ada setelah jendela dimuat ulang**.
- **Pemeriksa tata letak** dijalankan ulang dengan halaman Pengaturan → Lirik di dalamnya: ALL CLEAR.

### Temuan saat pengujian (sudah diperbaiki)

- **Layanan yang menjawab 503 dianggap menjawab.** `ask_json` menelan status, jadi layanan yang tumbang terlihat persis seperti layanan yang berkata "tidak ada liriknya" — dan lagunya dicatat tak berlirik gara-gara jaringan sedang buruk satu menit. Status kini ikut dibawa, dan tiap pemanggil menentukan artinya sendiri.
- **Lirik memakai warna lampu.** Satu blok lirik mengambil amber yang seharusnya untuk indikator, jadi semua baris berteriak dan baris yang sedang dinyanyikan tidak punya sisa untuk membedakan diri. Sekarang lirik adalah teks; yang sedang dinyanyikan memakai warna "di sini".
- **Jawaban yang salah urutan.** Panel menjawab "pencarian lewat internet mati" padahal pertanyaan pertama pendengar adalah "ada liriknya tidak?". Sekarang yang pertama dijawab dulu, yang kedua ditambahkan di bawahnya.
- **Lirik yang baru diketik tidak sampai ke panel** sampai lagunya berganti. Panel kini ikut mendengarkan kabar "library berubah" yang sudah dikirim editor.
- **Dialog panjang mendorong tombolnya sendiri keluar layar.** `max-height: 100%` pada panel tidak punya tinggi pasti untuk dijadikan persentase. Diperbaiki di `Modal` yang dipakai bersama, jadi semua dialog ikut benar.

### Belum dikerjakan

- **Total trek, total disk, dan komentar** belum bisa di-override, jadi belum ada di editor — bagian dari **M7b** (bersama hapus/ekspor sampul dan aturan 1200 px), dikerjakan setelah M10 penuh.
- **Edit banyak lagu sekaligus lewat editor ini** (dengan "(beragam)") juga M7b; edit massal yang sudah ada tetap lewat halaman Perapihan.
- Timing per kata dibaca dan disimpan, tapi belum digambar per kata — barisnya yang menyala, bukan katanya.

## Penyiapan rilis v1 (2026-09-19)

Tidak ada fitur baru. Tiga hal: menyisir janji yang tidak bisa ditepati, menjalankan dari keadaan benar-benar baru, dan membangun rilis untuk kedua sistem.

### Janji yang dicabut atau ditepati

Temuan lengkap dan keputusannya ada di `docs/DECISIONS.md`. Yang terbesar: **unduhan gagal seluruhnya di mesin tanpa ffmpeg** — termasuk format "Asli" yang tidak mengonversi apa pun — karena `-x` dan penanaman metadata/sampul semuanya pekerjaan ffmpeg. Sekarang ketiganya hanya diminta bila ffmpeg ada; tanpa ffmpeg audionya diambil apa adanya dan halamannya mengatakan apa yang hilang. Selain itu: ffprobe digabung ke baris ffmpeg, Deno dihapus dari daftar, kata "belum" diganti keterangan cara memasang sendiri, prioritas format tidak lagi punya penadah yang bisa membawa Opus, alasan gagal dibawa sampai ke jendela, dan rujukan `docs/PROGRESS.md` di editor tag dihapus.

**Cara verifikasi**: aplikasi dijalankan dengan PATH yang benar-benar bersih (`C:\Windows\system32;C:\Windows`), jadi yt-dlp pun tidak bisa menemukan ffmpeg di mana pun — satu-satunya bukti yang berlaku, karena mematikan "pakai program sistem" hanya menghentikan Onsa mencari, bukan yt-dlp. 12 pemeriksaan: daftar berisi dua program, konversi dilaporkan tidak tersedia, "Asli" terunduh sampai muncul di library, konversi yang tetap diminta hanya tidak dikonversi, dan kegagalan menyebut alasannya dengan kalimat. Lalu 5 pemeriksaan di mesin yang **punya** ffmpeg: tag tertanam kembali ada dan konversi ke MP3 tetap jalan.

### Dari keadaan benar-benar baru

Tanpa database, tanpa pengaturan, tanpa folder library, dan dengan PATH bersih:

- Layar pertama muncul, menolak mulai sebelum ada folder, dan menunjukkan keenam tema sebelum dipilih.
- Ke-15 halaman dibuka satu per satu, **dua kali**: dalam bahasa Inggris dan dalam bahasa Indonesia. Semuanya terbuka, mengatakan sesuatu yang benar untuk library kosong, tidak ada yang menggulir ke samping, tidak ada kunci kamus yang bocor ke layar, dan tidak ada yang melempar kesalahan.
- Sedang Diputar tidak bisa dibuka saat tidak ada yang diputar — tombolnya mati, bukan membuka layar kosong. Mini player terbuka dan mengembalikan jendela.

### Build rilis

Ditambahkan `.github/workflows/release-build.yml`, dijalankan hanya bila diminta.

- **Windows**: biner rilis dijalankan dari PATH bersih sepanjang pengujian di atas. Tidak ada `vcruntime`/`msvcp` di antara 27 DLL yang diimpornya — hanya DLL sistem Windows dan penerus UCRT, jadi tidak perlu Visual C++ redistributable dan tidak perlu toolchain.
- **Linux**: dibangun di runner Ubuntu yang bersih, lalu **dijalankan di Ubuntu 26.04** (WSLg) dari salinan artefak: 123 pustaka, **tidak ada yang hilang**, jendela bernama "Onsa" benar-benar terbuka, database dan log dibuat, dan mesin audio membuka output ALSA.

Catatan: build Linux tidak bisa dijalankan langsung di WSL mesin ini karena jaringan WSL-nya mati (cargo tidak bisa mengunduh crate), jadi yang membangun adalah CI dan yang dijalankan adalah artefaknya.

## v1.0.1 — enam perbaikan dari sesi audit (2026-09-19)

Tidak ada fitur baru dan tidak ada yang diambil dari v1.1. Semuanya berasal dari sesi audit: hal-hal yang Onsa lakukan tanpa berkata apa-apa kepada orang yang tidak punya penulisnya di sebelahnya.

### Yang diperbaiki

- **Database yang tidak bisa dibuka kini punya jendela.** Sebelumnya `setup()` Tauri mengembalikan `Err` dan prosesnya mati sebelum satu jendela pun ada: mengklik ikon tidak menghasilkan apa-apa, dan satu-satunya keterangan ada di berkas log yang tidak diketahui siapa pun. Sekarang kegagalannya ditangkap, jendelanya tetap dibuka, dan isinya (`startup.rs`, `StartupTrouble.svelte`) membedakan **dua hal yang berbeda**: database dari Onsa yang lebih baru (skemanya dikenali, versinya di depan) dan database yang tidak terbaca. Keduanya menyebutkan apa yang terjadi, di mana berkasnya, dengan tombol untuk membuka foldernya, apa yang bisa dilakukan, dan kalimat mentah dari mesinnya di bagian yang bisa dibuka. **Onsa tidak menghapus dan tidak memperbaiki database itu sendiri** — itu data pengguna, dan jendelanya mengatakan bahwa tidak ada yang disentuh.
- **Berkas lagu yang hilang ditandai, bukan didiamkan.** Saat mesin audio gagal membuka berkas, Onsa sudah mengatakannya di transport; yang belum ada adalah tanda di barisnya. Sekarang `Library::mark_missing` dipanggil dari jalur kegagalan pemutaran (hanya bila berkasnya memang tidak ada di disk), lalu kabar "library berubah" dikirim supaya daftarnya ikut berubah tanpa memindai ulang.
- **`downloads.notAUrl` akhirnya sampai ke layar.** Kalimatnya sudah ada di kamus sejak lama tapi tidak punya kode kesalahan yang mengantarkannya. Ditambah `NotAUrl` dan `UrlRefused`, jadi "itu bukan alamat web" dan "yt-dlp tidak bisa membaca URL itu" adalah dua kalimat yang berbeda. Pesannya ditaruh di bagian URL, bukan di tempat pesan bersama, karena yang di sana terhapus tiap kali status program dibaca ulang.
- **Berkas tema yang cacat mengeluh di layar.** Pembacanya sudah mencatat tiap berkas yang gagal di log, lengkap dengan nama dan alasannya; yang belum ada adalah jalan ke jendela. `read_user_dir` kini mengembalikan daftar keluhan bersama temanya, dan Pengaturan → Tampilan menampilkannya. Di halaman yang sama ada **tombol untuk menyalin tema yang sedang dipakai** ke folder tema sebagai titik mulai — sebelumnya orang harus menulis JSON tema dari nol tanpa contoh.
- **Alasan gagal menulis tag tidak lagi kosong.** Kalimatnya dulu berisi tempat untuk alasan yang tidak pernah diisi. Sekarang alasannya dibawa dari `report.failed[0]` dan ditampilkan di bawahnya; kalimat utamanya tidak lagi punya lubang.
- **Halaman Library mengatakan batasnya.** Satu baris: folder belum bisa dihapus di versi ini.

### Cara verifikasi

Tiap perbaikan diuji dengan cara yang sama persis seperti waktu ditemukan, pada aplikasi yang berjalan, bukan lewat tes.

- **Database**: berkas `library.db` ditimpa dengan teks yang bukan SQLite → jendela terbuka dan berkata "Onsa tidak bisa membuka pustakanya" dengan jenis `unreadable`. Lalu `PRAGMA user_version` dinaikkan ke angka di depan skema yang dikenal → jendela yang sama dengan kalimat yang lain, jenis `fromNewerOnsa`. Tombol foldernya membuka Explorer di tempat yang benar, dan berkas databasenya **tidak berubah** sesudahnya (ukuran dan isinya diperiksa dari luar Onsa).
- **Berkas lagu hilang**: mengganti nama berkas di folder yang sama ternyata dibaca pengintai folder sebagai **perpindahan** — dan memang begitu, berkasnya masih ada. Jadi diuji dua kali lagi: (a) berkasnya disalin ke luar lalu dihapus → barisnya berbunyi "Tiga hilang" dalam hitungan detik, dan kembali normal setelah berkasnya dikembalikan dan dipindai ulang; (b) lagu di drive yang **dicabut saat Onsa berjalan** (`subst`), yaitu satu-satunya jalur di mana pengintai folder tidak melihat apa pun dan hanya mesin audio yang tahu → transport berkata "Lagu tidak bisa diputar dan dilewati: lagu.flac" dan barisnya jadi `missing`.
- **URL**: menempel "halo dunia ini bukan alamat" → "Itu bukan alamat web." dan kalimatnya **tetap ada** setelah enam detik (inilah yang dulu terhapus). Menempel alamat yang tidak bisa dibaca yt-dlp → "yt-dlp tidak bisa membaca URL itu." Dua kalimat berbeda untuk dua keadaan berbeda.
- **Tema**: dua berkas cacat ditaruh di folder tema yang sebenarnya (`%APPDATA%\io.github.mufuyumoku.onsa\themes`) — satu JSON terpotong, satu kosong → keduanya muncul di Pengaturan → Tampilan dengan nama berkas dan alasan dari parsernya. Tombol salin menghasilkan `tema-saya.json` di folder itu dan halamannya mengatakan berkas apa yang dibuat. Berkas ujinya dihapus lagi sesudahnya.
- **Menulis tag**: berkas dikunci dari proses lain (`FileShare.None`) lalu "tulis ke berkas" ditekan → "Tidak bisa menulis ke berkas." dengan alasan dari sistem operasi di bawahnya.
- **Halaman Library**: kalimatnya ada di layar.

### Dua temuan audit yang ternyata salah, dan itu salahku

- **"Berkas hilang tidak dikatakan di transport"** — Onsa mengatakannya, dan sudah sejak sebelum v1.0.0: "Lagu tidak bisa diputar dan dilewati: 03 Tiga.flac". Pemeriksaku mencari kata Indonesia sementara jendelanya sedang berbahasa Inggris, jadi yang kulaporkan adalah keheningan pemeriksaku sendiri. Yang benar-benar kurang hanyalah tanda di barisnya, dan itulah yang diperbaiki.
- **"Tema cacat didiamkan"** — log-nya memperingatkan per berkas, dengan nama dan alasan. Berkas ujiku kutaruh di `<ONSA_DATA_DIR>/config/themes`, tempat yang tidak pernah dibaca Onsa: `ONSA_DATA_DIR` mengalihkan data, cache, dan log, tapi folder tema selalu di folder konfigurasi aplikasi yang sebenarnya. Yang benar-benar kurang adalah jalan dari log ke layar, dan contoh untuk memulai.

### Ditunda ke v1.1

- **Pesan mentah yt-dlp** masih muncul apa adanya untuk sebab yang belum diterjemahkan.
- **Folder library yang hilang** (drive dicabut, folder dipindah) masih butuh pindai ulang manual; belum ada yang mengatakannya sendiri.
- **Baris ffmpeg di halaman Unduhan versus saklar "pakai program sistem"**: keduanya benar sendiri-sendiri tapi belum menjelaskan hubungannya.
- **Tur atau menu bantuan** untuk pemakai pertama kali.

### Perilaku yang belum diketahui

- **Disk penuh saat mengunduh atau saat menulis tag** belum bisa diuji di mesin ini, jadi belum diketahui apa yang dikatakan Onsa. Penanganannya tidak dikarang tanpa bisa diuji; dicatat di sini sampai ada cara mengujinya.

## v1.1 — Onsa menjelaskan dirinya (2026-09-20)

Tidak ada kemampuan baru. Tujuannya satu: orang yang baru pertama membuka Onsa bisa memakainya tanpa bertanya kepada pemiliknya dan tanpa membaca README.

### 1. Layar pertama menyebutkan apa saja yang ada

Sesudah scan selesai, layar pertama menambahkan satu langkah penutup, **"Selebihnya"**: tiga baris, masing-masing satu kalimat, dengan nama halamannya bisa diklik — Rapikan, Unduhan, dan Lirik. Bukan tur berlangkah banyak: satu layar, bisa dilewati lewat tombol "Buka library" tepat di bawahnya, dan bisa dibuka lagi dari halaman Bantuan. Layar itu kini tahu kalau ia dibuka ulang: langkah penutupnya langsung terlihat, kalimat pembukanya berganti, dan tombolnya jadi "Tutup".

**Cara verifikasi**: dijalankan dari keadaan benar-benar baru — tanpa database, tanpa pengaturan, tanpa folder library. Layar demi layar: layar pertama (Inggris), bahasa diganti dari layar itu juga, dialog folder Windows yang sungguhan dijawab dengan jalur folder musik, scan, lalu langkah penutupnya muncul. Mengklik "Rapikan" benar-benar membuka halaman Rapikan, bukan sekadar menutup layar pertama. Diulang di jendela terkecil (686×483): tidak ada gulir ke samping dan tombol terakhirnya tetap terjangkau.

### 2. Bantuan dua lapis

**Lapis 1**: tombol tanda tanya di sudut yang sama di semua halaman, membuka panel berisi halaman itu saja — satu paragraf "untuk apa", tiap kontrol dengan kalimatnya sendiri, apa yang terjadi pada berkas asli bila halaman itu bisa menyentuhnya, dan batasan yang berlaku di sana. Panelnya berdiri di kolom sendiri bila jendelanya ≥900 px dan menumpuk di atas halaman bila tidak, tanpa scrim dan tanpa jebakan fokus. Nama kontrol di bantuan memakai kunci kamus yang dipakai kontrol itu sendiri, jadi bantuannya selalu menyebut kata yang benar-benar ada di layar.

**Lapis 2**: halaman Bantuan di navigasi. Daftar isi 25 entri (tiap halaman, halaman rincian, hasil pencarian, dan tiga perabot tetap jendela); mengklik satu membuka halamannya sekaligus penjelasannya. Di bawahnya hal-hal yang tidak menempel di satu halaman: urutan empat sumber lirik dan geseran per lagu, isi berkas tema `.json` dengan contoh yang bisa disalin, arti "belum ditulis ke berkas" beserta daftar empat hal yang bisa menyentuh berkas, yt-dlp dan ffmpeg, batasan versi ini, cara menyalakan log debug dan mengirimnya, dan tombol membuka layar pertama lagi.

**Cara verifikasi**: tiap halaman dibuka satu per satu dengan **library kosong dan library terisi**, panelnya dibuka, isinya dibaca apa adanya. Lalu pemeriksaan yang lebih keras: tiap kontrol di layar didaftar dan dicocokkan dengan isi panelnya — 206 kontrol di 25 halaman (library terisi) dan 142 kontrol di 20 halaman (library kosong), **nol yang belum disebut**. Kontrol yang namanya data (judul lagu, nama album, nama artis, nama halaman di daftar isi) diwakili satu entri umum, dan aturan perwakilan itu tertulis di pemeriksanya serta ikut tercetak per kontrol. Pemeriksaan itu menemukan 54 kontrol yang tidak tersebut; semuanya diperbaiki, bukan ditambal: tiap pilihan di dalam sebuah setelan kini disebut namanya, tiap tahap jalur sinyal disebut, dan tiga entri baru ditambahkan.

**Tidak menghalangi halaman**: di jendela 686×483 dengan panel menutupi sebagian baris lagu (352 dari 657 px yang terlihat), lagu itu diklik dua kali lewat bagian yang tersisa dan benar-benar diputar, sementara panelnya tetap terbuka. Di empat lebar (1400, 1000, 880, 686): tidak ada gulir ke samping, panel tidak pernah menutupi baris atas, tombol tutupnya bekerja.

**Istilah**: seluruh teks bantuan disisir untuk kata yang harus dijelaskan dulu. *offset*, *sidecar*, *transcode*, dan *gapless* tidak dipakai sama sekali; *tag*, *scan*, *cakupan*, *genre*, dan *.lrc* selalu dijelaskan di kalimat yang sama.

### 3. Dua sisa audit

- **Alasan gagal unduh** kini punya sembilan sebab berkalimat manusia, dan **kalimat asli yt-dlp disimpan di balik "Lihat detail"**. Berlaku juga di pemeriksaan URL ("Lihat isinya"), tempat orang pertama kali bertemu penolakan — dulu semuanya dijawab satu kalimat umum dan kalimat yt-dlp dibuang ke log.
- **Baris ffmpeg** tidak lagi berkata "dari sistem" saat saklar program sistem dimatikan. Sekarang: "ada di sistem, dipakai yt-dlp sendiri", dengan kalimat yang menjelaskan bahwa saklar itu hanya mengatur program yang dijalankan Onsa, sementara yt-dlp mencari ffmpeg atas namanya sendiri.

**Cara verifikasi**: lima alamat yang benar-benar gagal pada aplikasi yang berjalan — bukan alamat sama sekali (0,5 dtk), situs tak dikenal yt-dlp (4,6 dtk), alamat tanpa yang menjawab (3,6 dtk), alamat yang menjawab 404 (1,5 dtk), dan alamat yang menjawab 500 (1,5 dtk) — masing-masing dengan kalimatnya di layar dan kalimat yt-dlp di balik "Lihat detail". Lalu saklar program sistem dimatikan dan dinyalakan lagi sambil membaca baris yt-dlp dan ffmpeg.

**Yang belum bisa dibuktikan**: empat sebab lain (butuh masuk akun, ditolak karena negara, ditolak situs 403, terlalu banyak permintaan 429) hanya diuji lewat pola terhadap kalimat yt-dlp yang sudah dikenal bentuknya, karena membuktikannya berarti menyentuh situs berhak cipta. Dicatat untuk dibuktikan di v1.2.

### 4. Folder library yang hilang

Halaman Library menanyakan ke disk tiap folder — saat daftarnya dibaca, dan **setiap empat detik selama halaman itu terbuka**, karena sebuah drive bisa dicabut sementara halamannya di layar. Folder yang tidak ada ditandai di daftar, dengan kalimat tentang sebabnya dan apa yang bisa dilakukan. **Tidak ada data yang dihapus**: foldernya tetap terdaftar, lagunya tetap di library.

**Cara verifikasi**: sebuah drive dibuat dengan `subst`, ditambahkan sebagai folder library, lalu dicabut sungguhan selagi halaman Library terbuka — tanpa menyentuh apa pun, halamannya berkata "tidak ada di tempatnya" dalam **2,6 detik**. Sesudah itu diperiksa dari luar: foldernya masih terdaftar, lagunya masih ada, statusnya masih `ok`. Drive dicolokkan lagi, "Scan ulang" ditekan seperti yang disuruh kalimat di layar, dan halamannya kembali normal.

### Belum dikerjakan

- Tur berlangkah banyak: memang tidak dibuat, atas permintaan pemilik proyek.
- Paket untuk tester (installer Windows, berkas untuk tester, dan pembuktian tiap jalur internet terhadap layanan sungguhan) adalah v1.2.

## v1.2 — paket untuk tester (2026-09-21)

Tidak ada fitur baru. Tiga hal: membuktikan tiap jalur internet terhadap layanan aslinya, membuat berkas pemasang untuk Windows, dan menulis berkas untuk relawan tester.

### A. Tiap jalur internet, terhadap layanan sungguhan

Selama ini semuanya diuji terhadap layanan tiruan di 127.0.0.1 — itu tetap dipertahankan untuk CI, karena CI tidak boleh bergantung pada internet. Sebelum diserahkan, tiap jalur dibuktikan sekali terhadap layanan aslinya, di mesin uji tanpa satu pun pengalihan `ONSA_*_URL`.

| Jalur | Hasil | Lama |
|---|---|---|
| Pengambil binary | yt-dlp 2026.08.19 dari rilis resminya, di mesin yang belum punya apa-apa | 2,5 detik |
| yt-dlp | satu lagu dari Internet Archive sampai muncul di library dan diputar | 3,6 dtk membaca isi + 61,8 dtk mengunduh |
| MusicBrainz | dua lagu dikenali; release-group id yang dijawabnya sama dengan yang tertulis di halaman sumbernya | 9,1 dtk untuk 4 lagu |
| Cover Art Archive | sampul 500×500 diambil dan ditawarkan sebagai usulan | bagian dari 9,1 dtk itu |
| AcoustID | kunci diterima layanannya (`works`); **lagu tidak dikenali** — lihat "yang tidak terbukti" | 0,6 dtk |
| LRCLIB | 21 baris bersinkron, baris pertama 5,06 dtk, terakhir 113,3 dtk, baris yang dinyanyikan menyala | 1,0 dtk |

**Checksum diperiksa dari luar Onsa**: SHA-256 berkas yang ditulis Onsa sama persis dengan yang diterbitkan `SHA2-256SUMS` rilis yt-dlp itu.

**Sumber unduhan**: [HAZE 077] Faustino Goyena — *De Neatins* (HAZE netlabel, 2009) di Internet Archive, halamannya menyatakan lisensi **Creative Commons Attribution-Noncommercial-No Derivative Works 3.0**. Rilis netlabel, bukan situs berbagi video, dan bukan karya yang diambil tanpa izin.

**Kalau layanannya lambat atau menolak** (dibuktikan lewat layanan tiruan yang menjawab 403, 429, 503, dan yang menjawab sangat lambat): unduhan menyerah di 23 dtk, lirik di 21 dtk, "Cari data online" di 61 dtk untuk satu lagu, "Coba kunci" di 30 dtk. Semuanya berakhir dengan kalimat yang bisa ditindaklanjuti, tidak ada yang menggantung. 403 dan 429 — dua sebab yang di v1.1 baru berupa pola — kini terbukti diterjemahkan dengan benar.

### Tiga cacat yang ditemukan pengujian ini

1. **"Coba kunci" menjawab "AcoustID menerima kunci ini" kepada layanan yang sedang tumbang.** Status HTTP-nya tidak dilihat sama sekali. Sekarang hanya 2xx dan 400 (cara AcoustID menolak permintaan) yang dibaca sebagai jawaban tentang kunci; sisanya "tidak bisa dihubungi".
2. **Panel lirik berkata "Tidak ada lirik untuk lagu ini" ketika layanannya tidak bisa dihubungi.** Yang tercatat di database sudah benar sejak M8 — kegagalan tidak pernah diingat — tapi jendelanya tidak punya cara tahu. Sekarang: "Layanan liriknya tidak bisa dihubungi. Lagunya mungkin punya lirik; coba lagi nanti."
3. **Kegagalan mengunduh program luar berakhir sebagai "Detailnya ada di log".** Sebabnya sudah diketahui dan dibuang di tengah jalan. Sekarang sama seperti kegagalan mengunduh lagu: kalimat yang menyebut sebab dan menyuruh coba lagi, dengan kalimat asli di balik "Lihat detail".

### Yang tidak bisa dibuktikan jujur

- **AcoustID mengenali satu lagu dari bunyinya.** fpcalc dipasang (Chromaprint 1.6.1 resmi), sidik jari dihitung, permintaan terkirim dan dijawab — tapi dua rekaman yang boleh dipakai (lagu netlabel itu dan nada uji buatan sendiri) memang tidak ada di basis data AcoustID. Jalurnya terbukti sampai "layanan menjawab"; "lagu dikenali" tidak, karena membuktikannya berarti memegang rekaman komersial orang.
- **Ditolak karena butuh masuk akun, dan ditolak karena negara.** Keduanya sengaja tidak dipaksakan: mengujinya berarti sengaja mencari sumber yang diproteksi. Keduanya tetap berupa pola yang belum terbukti.

### B. Berkas pemasang Windows

`Onsa_1.1.0_x64-setup.exe`, **11,6 MB**, dari build rilis. Dipasang per pengguna ke `%LOCALAPPDATA%\Onsa` — **tidak butuh hak administrator**, tidak menyentuh Program Files, dan pemasangannya selesai dalam 5 detik.

- **Tidak butuh toolchain apa pun.** Binernya meminta 27 DLL, semuanya milik Windows; **tidak ada satu pun runtime Visual C++** (`vcruntime*`, `msvcp*`), hanya penerus UCRT yang memang bagian dari Windows 10 dan 11.
- **Dijalankan dari keadaan benar-benar baru** — data folder kosong, dan PATH hanya `C:\Windows\system32;C:\Windows`, jadi tidak ada yt-dlp dan tidak ada ffmpeg: layar pertama muncul, folder dipilih, 5 lagu terbaca, lagu diputar, dan strip jalur sinyal menunjukkan rantai yang sebenarnya. Halaman Unduhan menyebut keduanya "belum ada" dan menjelaskan cara memasang masing-masing.
- **Nama benar di tiga tempat**: jendela pemasang "Onsa Setup", pintasan Start Menu "Onsa", judul jendela aplikasinya "Onsa", dan di daftar program: Onsa 1.1.0.
- **Dua hal diperbaiki setelah diperiksa**: berkas pemasangnya semula memakai ikon bawaan NSIS (gambar globe Nullsoft), sekarang memakai ikon Onsa; dan di daftar program ia semula tercatat dengan penerbit "github" (dari bagian tengah identifier), sekarang "MufuyuMoku".
- **Pencopotan bersih**: sesudah uninstaller dijalankan, folder pemasangan, pintasan Start Menu, dan entri di daftar program ketiganya hilang.

**SmartScreen**: berkasnya **tidak ditandatangani** (`NotSigned`) — menandatanganinya butuh sertifikat berbayar. Ditandai sebagai berkas yang diunduh dari internet (Mark of the Web) lalu dijalankan, di mesin ini **tidak muncul peringatan SmartScreen**; proses SmartScreen memang berjalan, jadi berkasnya dinilai dan dibiarkan lewat. Itu tidak bisa dijadikan janji: keputusan SmartScreen bergantung pada reputasi berkas itu di layanan Microsoft, setelan Windows di mesin yang memakainya, dan berkas ini sudah beberapa kali dijalankan di mesin ini. Berkas tak bertanda tangan yang baru biasanya memang dihadang. Karena itu `docs/UNTUK-TESTER.md` tetap menjelaskan peringatan itu beserta apa yang harus diklik.

**Sebersih apa lingkungan ujinya**: Windows Sandbox tidak ada di Windows 11 Home, dan membuat akun pengguna baru butuh hak administrator — jadi pemasangannya diuji di mesin pengembang ini sendiri, dengan data folder baru dan PATH yang dibersihkan. Yang membuktikan "tidak butuh toolchain" bukan lingkungan itu melainkan daftar DLL yang diminta binernya.

### Kunci AcoustID di dalam build rilis

**Build rilis di mesin ini membawa kunci AcoustID pemiliknya ke dalam binernya.** `keys.rs` memakai `option_env!`, yang dibaca saat kompilasi: variabel `ONSA_ACOUSTID_API_KEY` yang ada di lingkungan mesin ini ikut tertanam. Diperiksa tiga kali: kunci itu ada di dalam `onsa.exe` hasil build, ada di dalam salinan yang dipasang berkas pemasang, dan salinan yang dipasang itu — dijalankan tanpa kunci apa pun di lingkungannya — berkata `source: "build"`, yaitu "Memakai kunci yang dipasang saat build".

Artinya: berkas pemasang yang dibangun sambil variabel itu ada akan memberi tiap tester kunci pribadi pemiliknya, yang bisa dibaca siapa pun dari binernya. Build kedua dibuat dengan variabel itu dikosongkan, dan di situ kuncinya tidak ada di mana pun: halaman Metadata berkata "Belum ada kunci. Pengenalan lewat AcoustID tidak akan berjalan."

**Apa yang dilihat tester tanpa kunci dan tanpa fpcalc**: halaman Metadata menyebut keduanya belum ada, menjelaskan kuncinya gratis beserta tempat mengambilnya, dan menyebut fpcalc ada di paket Chromaprint. Halaman Rapikan menampilkan dua kalimat sebelum apa pun dijalankan: "Belum ada API key AcoustID. Tanpa itu, pencocokan lewat suara tidak bisa jalan." dan "fpcalc belum terpasang. Tanpa itu, hanya lagu yang sudah bertag yang bisa dicari." Tombol "Mulai cari" tetap hidup, dan menjalankannya tetap berguna: dari lima lagu uji, empat mendapat usulan dari MusicBrainz lewat keterangan lagunya; hanya yang memang perlu dikenali dari bunyinya yang kembali dengan sebab `noKey`.

### Berkas pemasang datang dari CI, bukan dari mesin pemilik

Karena build lokal membawa kunci pemiliknya, berkas pemasang untuk orang lain tidak lagi dibuat di mesin itu. `release-build.yml` sekarang juga berjalan pada tag versi, membangun berkas pemasang Windows, dan mengunggahnya sendiri ke Rilis GitHub — jadi berkas yang sampai ke tester adalah berkas yang dibangun CI, tanpa melewati mesin siapa pun.

Yang membuktikan tidak ada kunci di dalamnya ada tiga lapis, saling bebas:

1. **Penjaga saat kompilasi** di `keys.rs`: dengan `ONSA_KEYLESS` disetel, kompilasi **gagal** kalau ada kunci yang akan ikut tertanam. Diuji dua arah di mesin pengembang — dengan kunci di lingkungan, `cargo build` berhenti dengan pesan "this build was told to carry no keys, but an AcoustID key was in the build environment"; tanpa kunci, build jalan seperti biasa.
2. **Satu tes biasa** yang mengatakan hal yang sama saat dijalankan, dijalankan CI sesudah build.
3. **Langkah di workflow** yang memeriksa lingkungannya sendiri sebelum apa pun dibangun, dan berhenti kalau salah satu variabel kunci berisi sesuatu.

Versi dinaikkan ke **1.2.0**, karena berkas pemasang bernama 1.1.0 padahal isinya sudah lewat tag v1.1.0.
