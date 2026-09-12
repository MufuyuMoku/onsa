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
