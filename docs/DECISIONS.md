# Keputusan

Catatan keputusan yang diambil saat spesifikasi kurang jelas. Format: tanggal, konteks, keputusan, alasan. Keputusan terkunci ada di `SPEC.md` §1 dan tidak dicatat ulang di sini.

---

## 2026-09-10 · Identifier aplikasi

- **Konteks**: SPEC §1 memakai placeholder `io.github.<pemilik>.onsa`.
- **Keputusan**: `io.github.mufuyumoku.onsa`, sesuai arahan pemilik proyek. SPEC sudah diperbarui.

## 2026-09-10 · Edisi Rust dan versi minimum

- **Konteks**: SPEC hanya menyebut "Rust stable".
- **Keputusan**: edisi 2021, `rust-version = "1.82"` di workspace.
- **Alasan**: edisi 2021 paling sedikit gesekan dengan ekosistem Tauri 2. Versi minimum di bawah MSRV dependensi yang dipakai, dan CI memakai stable terbaru.

## 2026-09-10 · Isi crate modul di M0

- **Konteks**: M0 meminta workspace sesuai §2, tapi isi tiap modul baru dikerjakan di milestone masing-masing.
- **Keputusan**: crate `onsa-audio`, `onsa-library`, `onsa-downloader`, `onsa-lyrics`, dan `onsa-scrobble` hanya berisi tipe error (`thiserror`) dan alias `Result`. `src-tauri` belum bergantung pada crate-crate ini; dependensi ditambahkan saat modulnya benar-benar dipakai.
- **Alasan**: tidak ada kode mati atau dependensi yang tidak dipakai, dan aturan batas crate tetap terlihat dari struktur folder.

## 2026-09-10 · Letak pemuat tema

- **Konteks**: §2 tidak menyebut crate khusus tema.
- **Keputusan**: pemuat tema menjadi modul `theme` di `src-tauri`. Tema bawaan dikompilasi ke dalam binary dengan `include_str!` dari `themes/*.json`.
- **Alasan**: pilihan paling sederhana yang konsisten dengan §2. Tema bawaan selalu tersedia walau folder data rusak.

## 2026-09-10 · Validasi tema

- **Konteks**: §9.3 meminta field yang hilang atau salah diganti default, dan tema tidak boleh merusak aplikasi.
- **Keputusan**: tema dibaca sebagai `serde_json::Value`, lalu setiap field diambil satu per satu dengan default dan peringatan. Nilai warna hanya diterima dalam bentuk hex, `rgb()/rgba()`, `hsl()/hsla()`, atau `transparent`. Nilai varian yang tidak dikenal kembali ke default. Angka di luar rentang dijepit ke rentangnya. File yang bukan objek JSON ditolak utuh.
- **Alasan**: deserialisasi serde biasa gagal total pada satu field yang salah. Pembatasan notasi warna mencegah tema menyelipkan CSS lain lewat nilai variabel.

## 2026-09-10 · Tema awal sebelum layar pertama

- **Konteks**: pilihan tema baru ada di layar pertama (M4).
- **Keputusan**: sampai saat itu, jendela memakai `kaca-asap` (`DEFAULT_THEME_ID`). Pilihan tema belum disimpan.

## 2026-09-10 · Font belum dibundel di M0

- **Konteks**: font setiap tema harus dibundel di `ui/static/fonts/` beserta lisensinya (§9.4). M0 hanya meminta jendela kosong bertema.
- **Keputusan**: di M0 font belum dibundel. Stack font sudah menyebut keluarga dari tema, lalu jatuh ke font sistem. Font dan lisensinya ditambahkan di M4 bersama sistem tema lengkap.
- **Alasan**: font adalah bagian dari kecocokan visual dengan `palet-preview.html`, yang baru menjadi kriteria di M4. Tidak ada yang dimuat dari internet.

## 2026-09-10 · Log di M0

- **Konteks**: §13 meminta log bergulir di folder log aplikasi.
- **Keputusan**: M0 hanya menulis log `tracing` ke stderr, difilter lewat env `ONSA_LOG`. File log bergulir dikerjakan di M4 bersama integrasi OS.

## 2026-09-10 · SPA tanpa prerender

- **Konteks**: §1 meminta `adapter-static` dan `ssr = false`.
- **Keputusan**: `ssr = false`, `prerender = false`, dengan `fallback: 'index.html'`. Konfigurasi SvelteKit ada di `ui/vite.config.ts` (format bawaan `sv create` terbaru).
- **Alasan**: satu dokumen fallback untuk seluruh jendela adalah bentuk SPA paling sederhana dan tidak bentrok dengan prerender.

## 2026-09-10 · Error lewat batas command

- **Konteks**: teks UI tidak boleh ditulis langsung di komponen (§9.6), termasuk pesan error dari backend.
- **Keputusan**: command mengembalikan error sebagai objek berkode stabil, misalnya `{"code":"theme_not_found"}`. UI memetakan kode ke kunci kamus.

## 2026-09-10 · Ikon

- **Konteks**: ikon final belum ada (§16).
- **Keputusan**: ikon di `src-tauri/icons/` dibuat dengan `cargo tauri icon` dari `assets/icon.svg`. Hanya ukuran untuk Windows dan Linux yang disimpan (tanpa iOS, Android, macOS, dan logo Microsoft Store).

## 2026-09-10 · Frontend harus di-build sebelum pemeriksaan Rust

- **Konteks**: `tauri::generate_context!` membaca `frontendDist` (`ui/build`) saat kompilasi.
- **Keputusan**: CI dan skrip `scripts/check.*` menjalankan `npm run build` di `ui/` sebelum `cargo clippy` dan `cargo test`.

## 2026-09-10 · Build aplikasi di CI tanpa bundling

- **Konteks**: §12 meminta CI membangun aplikasi. Bundling installer adalah pekerjaan M12.
- **Keputusan**: CI menjalankan `tauri build --no-bundle` lewat `@tauri-apps/cli` yang dipasang sebagai devDependency di `ui/`. Bundling (NSIS, deb, AppImage) masuk CI di M12.
- **Alasan**: memverifikasi build rilis tanpa mengunduh alat bundling tambahan.

## 2026-09-10 · Advisori `npm audit`

- **Konteks**: `npm audit` melaporkan advisori tingkat rendah pada `cookie` lewat `@sveltejs/kit`.
- **Keputusan**: dibiarkan. Advisori itu menyangkut parsing cookie di server, sedangkan Onsa tidak punya server (SPA statis). Perbaikan otomatis yang ditawarkan justru menurunkan SvelteKit ke versi lama.

## 2026-09-10 · Akhir baris

- **Keputusan**: `.gitattributes` memaksa LF untuk semua teks, kecuali `*.ps1`, `*.cmd`, dan `*.bat` yang memakai CRLF.
- **Alasan**: `scripts/check.sh` harus tetap bisa dijalankan di Linux walau di-checkout dari Windows.

## 2026-09-11 · `clap` untuk `onsa-cli`

- **Konteks**: `onsa-cli` di M1 butuh subperintah `play`, `queue`, `devices`, dan `render-wav`. `clap` tidak tercantum di SPEC.
- **Keputusan**: disetujui pemilik proyek, **khusus untuk `onsa-cli`** (fitur `derive`). Crate lain tidak memakainya.
- **Alasan**: parsing argumen, bantuan, dan pesan error yang rapi tanpa parser buatan sendiri.

## 2026-09-11 · Verifikasi Linux dikumpulkan per milestone

- **Konteks**: M0 hanya diverifikasi di Windows.
- **Keputusan**: status Linux tetap "belum diverifikasi". Verifikasi Linux dilakukan di milestone yang memang bergantung pada Linux: M1 (ALSA), M4 (MPRIS/tray), M12 (installer). Build dan tes Linux ditangani CI Ubuntu setelah repositori dipush.

## 2026-09-11 · Advisori `cookie` dan font

- Pemilik proyek menyetujui advisori `cookie` dibiarkan dan font ditunda ke M4 (lihat dua keputusan sebelumnya).

## 2026-09-11 · Versi crate audio dan Rust minimum

- **Konteks**: SPEC §1 mengunci pustaka audio tanpa versi.
- **Keputusan**: versi terbaru saat M1 dimulai: `symphonia` 0.6, `rubato` 5, `cpal` 0.18, `rtrb` 0.4. `rust-version` workspace naik dari 1.82 ke **1.87** karena `rubato` 5 membutuhkannya.
- **Alasan**: memulai dari API terbaru menghindari migrasi besar nanti. Mesin dan CI memakai stable (1.93).

## 2026-09-11 · Thread kontrol dan thread decode digabung

- **Konteks**: SPEC §3.1 menggambar thread kontrol dan thread decode terpisah.
- **Keputusan**: satu thread mesin (`onsa-audio`) menangani perintah sekaligus decode. Perintah datang lewat channel dan diproses di sela pengisian buffer. Handle `Engine` di sisi pemanggil hanya mengirim perintah, jadi pemanggil tidak pernah terblokir.
- **Alasan**: paling sederhana dan tanpa lock antar-thread. Saat pause atau stop, thread tidur di channel perintah sehingga CPU idle nol (SPEC §13.1). Saat bermain, thread bangun tiap seperempat panjang buffer (5–50 ms).

## 2026-09-11 · Lane untuk gapless dengan resampling

- **Konteks**: gapless harus tetap mulus meski sumber di-resample.
- **Keputusan**: lagu berurutan dengan sample rate sama dan tanpa crossfade disambung di dalam satu *lane* yang memakai satu resampler, sehingga resampler melihat sinyal kontinu. Seek, skip, crossfade, atau pergantian sample rate memulai lane baru. Delay resampler dipangkas di awal, dan ekornya di-flush sampai jumlah frame keluaran tepat `round(input × rasio)`.

## 2026-09-11 · Tempat micro-fade

- **Keputusan**: pause dan resume di-fade di callback output (raised cosine, 60 ms), tanpa membuang buffer. Seek, stop, dan pergantian antrean memakai *flush*: callback fade-out, membuang isi ring buffer, lalu fade-in saat audio baru datang. Skip manual saat bermain memakai crossfade di dalam stream (default 0,3 detik, minimal sepanjang micro-fade), jadi tidak ada flush. Skip saat pause memakai flush tanpa crossfade.
- **Alasan**: pause dan seek terasa langsung (~60 ms), sedangkan skip mendapat crossfade yang diminta SPEC. Latensi skip sama dengan panjang buffer.

## 2026-09-11 · Crossfade butuh panjang lagu

- **Keputusan**: crossfade dimulai saat sisa lane kurang dari durasi crossfade. Kalau panjang lagu tidak diketahui (container tanpa jumlah frame), lagu berikutnya disambung tanpa crossfade. Aturan album memakai override di `QueueItem` bila ada, lalu tag album dan nomor trek dari file. "Berurutan" berarti album sama (tidak peka huruf besar/kecil) dan nomor trek naik satu.

## 2026-09-11 · Preset resampler dan ukuran buffer

- **Keputusan**: Cepat = `Async` polinomial kubik; Seimbang (default) = `Fft` sinkron; Terbaik = `Async` sinc 256 tap dengan oversampling 256 dan jendela Blackman-Harris². Buffer Rendah/Normal/Besar = 50/150/500 ms ring buffer antara mesin dan callback; buffer perangkat memakai default cpal.

## 2026-09-11 · Mode sample rate output di M1

- **Konteks**: SPEC §3.4 menyebut ikuti perangkat, samakan dengan sumber, atau nilai tetap.
- **Keputusan**: M1 menyediakan "ikuti perangkat" (default) dan "nilai tetap" (`--rate` di CLI). "Samakan dengan sumber" butuh membangun ulang stream setiap sample rate lagu berubah dan menjadi pengaturan di M4, jadi ditunda ke M4.

## 2026-09-11 · Pemulihan dan pengikutan perangkat

- **Keputusan**: bila cpal melaporkan stream hilang (perangkat dicabut, stream invalid), mesin mencatat posisi putar, menutup stream, lalu mencoba membuka perangkat default setiap 1 detik. Setelah terbuka, mesin melanjutkan dari posisi yang sama dengan fade-in. Untuk pilihan "default sistem", selama bermain mesin juga memeriksa ID perangkat default setiap 2 detik dan pindah bila berubah. Laporan `DeviceChanged` dari cpal (backend sudah memindahkan stream sendiri) tidak memicu pembangunan ulang.

## 2026-09-11 · Fixture tes audio

- **Konteks**: SPEC §12 menyebut fixture WAV/FLAC.
- **Keputusan**: tes mesin M1 memakai WAV float 32-bit yang ditulis saat tes berjalan (penulis WAV sendiri di `onsa_audio::wav`), di folder bernama Jepang dan berspasi. Belum ada encoder FLAC atau lossy di dependensi, jadi gapless untuk MP3/AAC (delay encoder) diverifikasi pemilik proyek dengan telinga (langkah di laporan M1). Fixture FLAC ditambahkan saat library (M3) membutuhkannya.

## 2026-09-11 · Tombol "sebelumnya"

- **Keputusan**: bila posisi lebih dari 3 detik, "sebelumnya" mengulang lagu dari awal; kalau tidak, pindah ke lagu sebelumnya. Perilaku umum di pemutar musik.

## 2026-09-11 · Persetujuan pemilik proyek atas keputusan M1

- "Samakan dengan sumber" untuk sample rate output ditunda ke M4: **disetujui**.
- `rust-version` workspace 1.87 (dari 1.82, karena `rubato` 5): **disetujui**.

## 2026-09-11 · Trim encoder MP4 dibaca sendiri

- **Konteks**: tes gapless lossy menunjukkan setiap potongan M4A (AAC dari ffmpeg) diputar 1024 frame terlalu panjang. symphonia 0.6 mem-parse `elst` tapi tidak memakainya, dan tidak membaca `iTunSMPB`, sehingga priming AAC ikut terdengar di setiap sambungan.
- **Keputusan**: `onsa-audio` membaca sendiri trim untuk file MP4 (`.m4a`, `.m4b`, `.mp4`, `.m4p`) bila symphonia tidak melaporkan delay: `iTunSMPB` dari tag lebih dulu, lalu edit list (`media_time` untuk delay; panjang dari `segment_duration` bila timescale film minimal sama dengan sample rate, selain itu dari durasi `mdhd` dikurangi delay). Priming dibuang di awal, pembacaan dibatasi sampai panjang yang bisa diputar, dan target seek digeser sebesar priming. Parser box-nya kecil (hanya `moov/mvhd`, `trak/mdia/hdlr`, `mdhd`, `edts/elst`) dan tidak menambah dependensi.
- **Alasan**: tanpa ini album M4A tidak gapless. Bila versi symphonia berikutnya menangani edit list sendiri (melaporkan `delay`), jalur ini otomatis tidak dipakai.
- **Tes lossy**: `lossy_gapless` memeriksa lima hal. (1) Panjang tiap potongan tepat. (2) Keselarasan dengan sinus asli di bagian stabil (galat < 0,02). (3) Tidak ada lubang (RMS per jendela > 0,2). (4) Seluruh sinyal sama dengan decode ffmpeg atas file yang sama, disambung dengan panjang asli (selisih < 0,005). (5) Hening setelah akhir. Area sambungan tidak dibandingkan dengan sinus murni, karena encoder membuat transien sendiri saat sumbernya dipotong keras: encoder AAC ffmpeg menyimpang sampai 0,48 di awal potongan, dan decode ffmpeg menunjukkan hal yang sama. Sambungan Onsa identik bit-per-bit dengan decode ffmpeg.

## 2026-09-11 · Repositori GitHub privat

- **Keputusan**: repositori `MufuyuMoku/onsa` dibuat **privat**, sesuai pilihan pemilik proyek, karena lisensi belum diputuskan (SPEC §16). Kuota GitHub Actions privat berlaku (runner Windows dihitung 2×). Visibilitas bisa diubah kapan saja.

## 2026-09-11 · Rantai DSP (M2)

- **Latensi konstan 5 ms**: delay lookahead limiter selalu berjalan, juga saat limiter dimatikan. Kalau delay-nya ikut hilang dan muncul, menyalakan atau mematikan limiter di tengah lagu akan menggeser sinyal dalam waktu dan terdengar klik. Tes yang membandingkan sampel secara presisi memperhitungkan latensi ini.
- **Limiter aktif secara default**: di bawah ceiling −0,1 dBFS limiter transparan bit-per-bit (hanya tertunda 5 ms), dan melindungi dari clipping keras saat EQ atau preamp menaikkan level.
- **Jaminan ceiling**: gain limiter adalah rata-rata bergerak dari minimum bergerak kebutuhan gain di sepanjang jendela lookahead, sehingga gain yang dipakai tidak pernah melebihi kebutuhan frame yang sedang keluar. Release memakai satu kutub yang hanya boleh naik.
- **Perubahan tanpa klik**: perubahan EQ di-crossfade 20 ms antara bank filter lama dan baru (bank baru mewarisi memori filter lama). Preamp dan volume memakai penghalus dua kutub berjenjang (critically damped, total sekitar 10 ms), sehingga kurva gain tidak punya sudut sama sekali.
- **Parameter lock-free**: pengaturan diterjemahkan di thread mesin menjadi `ChainParams` berukuran tetap dan dikirim ke callback lewat `rtrb`. Callback tidak pernah mengalokasi, mengunci, atau menunggu.
- **Dither**: TPDF ±1 LSB hanya saat perangkat memakai integer 16-bit atau lebih sempit. Output float dan 24/32-bit tidak di-dither.
- **Hemat CPU**: setelah 1 detik input hening dan semua filter reda, rantai DSP dilewati dan output tetap hening digital (tanpa derau dither). Thread analisis diparkir saat tidak aktif, bukan polling (SPEC §13.1).
- **ReplayGain otomatis**: gain album dipakai bila seluruh antrean adalah satu album berurutan menurut `album` dan `track_number` di `QueueItem`. Info itu diisi pemanggil: library di M3/M4, dan di CLI dari tag file. Kalau ada lagu tanpa info itu, dipakai gain per lagu. Preamp ReplayGain hanya berlaku untuk lagu bertag; lagu tanpa tag memakai nilai cadangan.
- **Preset EQ bawaan dan penyimpanan preset** ada di M4 bersama halaman pengaturan. M2 menyediakan import/export format AutoEQ dan pengaturan lewat CLI.

## 2026-09-11 · Dependensi library (M3)

- **Konteks**: cache cover (SPEC §5.2) butuh decode, resize, dan encode gambar, serta hash isi gambar. SPEC tidak menyebut crate untuk keduanya.
- **Keputusan pemilik proyek**:
  - **`image`** tanpa fitur default, hanya decode JPEG, PNG, dan WebP, serta encode JPEG untuk thumbnail. WebP ikut karena sebagian file (terutama hasil unduhan) menyematkan cover WebP. Cover yang gagal di-decode atau berformat lain (misalnya AVIF) **tidak menggagalkan scan**: lagunya tetap masuk library tanpa cover, dan kejadiannya dicatat di log sebagai peringatan.
  - **`sha2` (SHA-256)** untuk hash isi cover. Alasannya: M10 tetap butuh SHA-256 untuk memverifikasi `SHA2-256SUMS` rilis yt-dlp (SPEC §7.1), jadi seluruh proyek cukup memakai satu algoritma hash.
- **Pemantau folder**: `notify` 8.2 dan `notify-debouncer-full` 0.7, versi stabil terbaru. Versi yang lebih baru di crates.io masih RC.

## 2026-09-11 · Rancangan library (M3)

- **Skema v1 hanya berisi yang dipakai M3**: `folders`, `tracks`, `albums`, `covers`, `overrides`, `stats`, `plays`, `settings`, dan `tracks_fts`. Tabel playlist, lirik, antrean scrobble, dan unduhan dibuat lewat migrasi baru di milestone yang memakainya, supaya rancangannya diputuskan saat kebutuhannya jelas. Versi skema disimpan di `PRAGMA user_version`. Database dengan versi lebih baru dari yang dikenal build ini ditolak utuh, tidak dimodifikasi.
- **Nilai yang tampil**: view `track_view` memilih override bila ada, selain itu nilai dari file. FTS5 mengindeks dari view yang sama, sehingga pencarian melihat hasil editan. Input pencarian diubah menjadi istilah awalan yang dikutip (`"kata"*`), sehingga sintaks FTS yang diketik pengguna diperlakukan sebagai teks biasa. Nilainya selalu parameter terikat.
- **Scan bertahap**: file dilewati bila mtime (milidetik) dan ukurannya sama, dan statusnya bukan `missing`. File yang gagal dibaca dicatat dengan status `failed` supaya terlihat oleh pengguna, dan tidak dibaca ulang selama file-nya tidak berubah. Penulisan dilakukan per 200 file dalam satu transaksi, dan progres dilaporkan setiap batch.
- **Hilang, bukan dihapus**: file yang tidak ditemukan, termasuk seluruh isi folder yang tidak bisa diakses (misalnya drive dilepas), diberi status `missing`. Penghapusan hanya lewat `purge_missing` atas perintah pengguna.
- **Pindah atau ganti nama**: pemantau memperbarui path lagu (atau semua lagu di bawah folder yang dipindah) tanpa mengubah id-nya, sehingga statistik dan riwayat putar tetap utuh. Windows melaporkan setiap pemindahan antar-folder sebagai "hapus" dan "buat", bukan rename. Karena itu, dalam satu batch pemantau, pasangan hapus dan buat dianggap pemindahan bila file di lokasi baru belum dikenal library dan ukuran serta mtime-nya sama dengan yang lama (pemindahan mempertahankan keduanya). Untuk folder, cukup satu file di dalamnya yang dicocokkan. Bila pasangan itu terpisah ke dua batch, atau file dipindah saat Onsa tidak berjalan, lagu lama menjadi `missing` dan file di lokasi baru masuk sebagai lagu baru.
- **Album**: diidentifikasi dari judul album ditambah album artist (atau artis lagu bila album artist kosong). Tahun dan cover album diambil dari lagu pertama yang memilikinya.
- **Cover**: gambar tertanam (cover depan bila ditandai), lalu `cover.*`, `folder.*`, `front.*`. Cover disimpan sekali per hash SHA-256. Cover yang gagal di-decode dicatat sekali per hash, lalu tidak dicoba lagi di scan yang sama.
- **Path**: disimpan sebagai path absolut (`std::path::absolute`, tanpa prefiks `\?\` di Windows) dalam UTF-8. Path yang bukan Unicode valid dilewati dengan peringatan. Folder tersembunyi dan folder symlink tidak ditelusuri, supaya loop symlink tidak membuat scan macet.
- **Format**: scanner mengikuti SPEC §3.2. Opus ikut masuk bersama decoder-nya di M11.
- **Pemantau**: debounce 750 ms. Handler berjalan di thread debouncer dan hanya menyerahkan daftar perubahan. Library diterapkan oleh thread pemiliknya lewat `apply_changes`, karena koneksi SQLite tidak dibagi antar-thread.
- **Pengukuran di mesin pemilik**: `onsa-cli` hanya menguji mesin audio dan tidak bergantung pada `onsa-library` (modul fitur hanya disatukan di `src-tauri`). Karena itu pengukuran scan pertama dan scan ulang disediakan sebagai contoh di crate library (`cargo run -p onsa-library --release --example scan`).

## 2026-09-11 · Batas waktu langkah apt di CI

- **Konteks**: di run `479caf6`, langkah `apt-get` di job Ubuntu memakan 31 menit 21 detik karena mirror paket yang lambat. Biasanya langkah ini selesai di bawah satu menit (55 detik di run M3).
- **Keputusan**: langkah itu diberi `timeout-minutes: 10`. Bila mirror macet, job gagal cepat dan bisa dijalankan ulang, alih-alih menghabiskan menit Actions. Tidak memakai action pihak ketiga untuk cache paket apt.
- **Alasan**: satu baris konfigurasi sudah membatasi kerugian terburuk. Cache apt pihak ketiga menambah dependensi pada kode luar, padahal langkah ini biasanya cepat.
