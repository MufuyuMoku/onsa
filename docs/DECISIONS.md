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

## 2026-09-12 · Font tema dibundel (M4a)

- **Konteks**: setiap tema menyebut keluarga fontnya sendiri, dan font harus dibundel beserta lisensinya, tidak dimuat dari internet (SPEC §9.4, §14). Di M0 ini ditunda; pemilik proyek memberi izin mengunduhnya di M4a.
- **Sumber dan versi yang dipatok**:
  - `google/fonts` commit `8e44913e4ff26fc997e6856c1ec40ff4791c98c5`: Chakra Petch (Regular, SemiBold), Share Tech Mono (Regular), B612 (Regular, Bold), B612 Mono (Regular), Barlow (Regular, Medium), Barlow Condensed (Medium, SemiBold), masing-masing dengan `OFL.txt`-nya.
  - `notofonts/noto-cjk` tag `Sans2.004`: `Sans/Variable/OTF/Subset/NotoSansJP-VF.otf` beserta `LICENSE`.
- **Hanya bobot yang benar-benar dipakai tema** yang diambil. Totalnya sekitar 9 MB, dan 8,1 MB di antaranya adalah Noto Sans JP.
- **Noto Sans JP memakai versi variable OTF** dari repo noto-cjk karena itu bentuk resmi yang paling kecil: 8,1 MB, dibanding 9,6 MB untuk TTF variable di google/fonts dan sekitar 9 MB untuk dua file statis Regular + Medium.
- **Tanpa WOFF2**: tidak ada rilis WOFF2 resmi di kedua repo, dan mesin build tidak punya alat konversinya (fontTools/brotli maupun `woff2_compress`). Font dimuat dari dalam aplikasi, jadi ukurannya hanya memengaruhi ukuran binary, bukan waktu unduh.
- **Aksara di luar Noto Sans JP** (Hangul, Devanagari, dan lainnya) memakai font sistem sebagai fallback terakhir pada stack font.

## 2026-09-12 · Irisan uji M4a

- **Konteks**: pemilik proyek ingin menguji hasil M1–M3 lewat aplikasi, bukan CLI. Irisan ini adalah bagian M4 yang sesungguhnya, dibangun di atas arsitektur final, bukan kode sementara yang dibuang.
- **Dua tambahan di `onsa-audio`** (disetujui pemilik proyek):
  - `Engine::set_output` untuk mengganti perangkat atau sample rate tanpa menghentikan lagu. Jalurnya sama dengan pemulihan saat perangkat hilang: posisi disimpan, output lama diredupkan lalu ditutup, output baru dibuka, dan pemutaran dilanjutkan.
  - `Engine::take_events` menyerahkan receiver event ke thread pendengar. Tanpa itu, `src-tauri` harus memegang `Engine` di balik mutex dan melakukan polling, yang melanggar target CPU nol saat idle (SPEC §13.1).
- **Library dengan dua koneksi**: satu koneksi melayani UI, satu lagi dimiliki thread library yang men-scan dan menerapkan perubahan dari pemantau folder. Menjelajah tidak menunggu scan, dan scan pertama berjalan di latar (SPEC §13.1). Thread-nya tidur di channel pekerjaan, jadi library yang diam tidak memakai CPU.
- **Tiga dependensi kecil**: `tauri-plugin-dialog` (pemilih folder dan file preset), `tauri-plugin-opener` (tombol "buka folder log"), dan `tracing-appender` (file log harian). Dua yang pertama plugin resmi Tauri, dan ketiganya hanya dipakai dari sisi Rust sehingga tidak menambah izin JavaScript.
- **Volume lewat `settings_set_dsp`**, bukan perintah tersendiri. Volume adalah bagian dari `DspSettings`; kalau ada dua jalur, pengaturan DSP yang dikirim berikutnya bisa mengembalikan volume ke nilai lama.
- **Kurva EQ dan export AutoEQ dihitung backend** dari `DspPrefs` yang sama. Dengan begitu UI tidak menduplikasi Q tetap EQ grafis (1,41) maupun aturan preamp otomatis.
- **Cover lewat custom protocol** `onsa://cover/<id>/<ukuran>` (SPEC §2), bukan base64 di event. CSP diperlebar khusus untuk skema itu.
- **Pengaturan disimpan sebagai JSON** di tabel `settings` milik library, satu kunci per kelompok. Nilai yang rusak atau hilang jatuh ke default per field, dengan peringatan di log, sehingga satu setelan rusak tidak menghalangi aplikasi terbuka.
- **Fitur yang belum dikerjakan tidak ditampilkan sama sekali**: tidak ada tombol mati atau placeholder di UI.
- **Log**: file harian di folder log aplikasi, disimpan tujuh hari terakhir, plus toggle "Log debug" di Tentang yang mengganti filter `tracing` saat aplikasi berjalan lewat reload layer.

## 2026-09-12 · Sisa M4: antrean, integrasi OS, dan jendela

- **Repeat ada di mesin audio, bukan di aplikasi.** Mengulang satu lagu harus tetap gapless, dan itu hanya mungkin kalau mesin yang memilih lagu berikutnya saat pre-roll. Repeat "satu lagu" karena itu tidak memancarkan event "lagu dimulai" yang baru: bagi mesin itu lagu yang sama berlanjut, dan yang terlihat di UI adalah posisi putar yang kembali ke awal. Kalau nanti scrobble (M9) butuh batas pemutaran, mesin perlu penanda pemutaran ke berapa.
- **Mengedit antrean tidak memotong lagu.** `Engine::update_queue` mengganti isi antrean dan menggeser semua nomor antrean yang diingat mesin (timeline, lane, dan info lagu) mengikuti perpindahan lagu yang sedang berbunyi. Satu batasannya: lagu yang sudah di-pre-roll saat pengeditan terjadi tetap akan diputar berikutnya.
- **Shuffle ada di aplikasi, bukan di mesin.** Aplikasi menyimpan urutan asli, mengacak sisanya, lalu mengirim urutan baru lewat `update_queue`. Lagu yang sedang berbunyi tidak terputus, dan mematikan shuffle mengembalikan urutan semula.
- **Pemulihan keadaan** (SPEC §13) disimpan di tabel `settings`: kunci `queue` berisi id lagu, posisi antrean, dan status shuffle, sedangkan kunci `position` berisi detik pemutaran dan ditulis paling sering setiap 10 detik. Keduanya dipisah supaya playhead bisa sering disimpan tanpa menulis ulang seluruh antrean. Antrean selalu kembali dalam keadaan **jeda**: membuka Onsa tidak pernah langsung mengeluarkan suara.
- **Kontrol media OS** memakai `souvlaki` (SPEC §13). Di Windows objek SMTC terikat pada thread jendela, jadi objeknya disimpan di thread local milik thread utama dan setiap pembaruan dikirim lewat `run_on_main_thread`. Tombol dari sistem masuk seperti perintah biasa.
- **Label menu tray dikirim dari UI.** Semua teks UI hanya boleh ada di kamus (SPEC §9.6), jadi UI menyerahkan labelnya setelah tahu bahasanya, dan menyerahkannya lagi setiap bahasa diganti.
- **Mini player memakai jendela yang sama**, hanya diubah ukurannya dan disetel selalu di atas. Jendela kedua akan menggandakan pemulihan keadaan dan pengaturan posisi jendela tanpa keuntungan yang sepadan.
- **Sleep timer** memeriksa keadaan tiap 500 milidetik, tapi hanya selama timer aktif; saat mati tidak ada yang berjalan. Volume diturunkan bertahap lalu **dikembalikan** ke nilai pengaturan, supaya pemutaran berikutnya tidak senyap.
- **Samakan dengan sumber** (SPEC §3.4, ditunda dari M1): saat lagu dengan sample rate berbeda mulai, mesin mengirim perintah ke dirinya sendiri untuk membuka ulang output pada rate itu. Lewat channel perintah, bukan panggilan langsung, supaya tidak ada rekursi. Uji coba menemukan bug di sini: output yang dibuka ulang tepat di awal lagu belum punya posisi putar, sehingga antrean dianggap habis. Sekarang mesin memulai lagunya dari awal dalam kasus itu, dan ada tesnya.
- **File dari luar** (argumen "Buka dengan Onsa", instance kedua, dan drag-and-drop) ditangani satu jalur: folder masuk ke library, file audio langsung diputar. File yang belum dikenal library tetap bisa diputar dengan id −1, dan karena itu tidak ikut dipulihkan saat aplikasi dibuka lagi.
- **Asosiasi file** dideklarasikan di `tauri.conf.json`. Pendaftarannya ke Windows baru terjadi lewat installer, jadi efeknya baru terasa di M12.

## 2026-09-12 · Identitas entri antrean (perbaikan bug)

- **Konteks**: pemilik proyek menemukan bug yang bisa diulang. Saat baris antrean dihapus cepat-cepat selagi lagu berjalan, lagu yang terlihat di UI berbeda dengan lagu yang terdengar. Penelusuran menemukan **tiga sebab**, semuanya satu akar: nomor urut antrean dipakai sebagai identitas lagu.
  1. **Mesin audio**: `Engine::update_queue` menggeser semua nomor yang diingatnya sebesar satu selisih (`current` baru dikurangi `current` lama). Selisih itu hanya benar kalau perubahan terjadi sebelum lagu yang berbunyi. Saat lagu yang sedang berbunyi sendiri dihapus, selisihnya nol, sehingga mesin terus memutar lagu lama sementara nomor itu sudah menunjuk lagu lain.
  2. **Aplikasi**: `Player` menghitung sendiri nomor lagu yang sedang diputar pada setiap pengeditan, sejajar dengan perhitungan mesin. Dua tebakan atas satu fakta yang sama, dan keduanya bisa berbeda.
  3. **UI**: perintah antrean dikirim memakai nomor baris (`remove(index)`, `move(from, to)`, `jump(index)`). Daftar di layar baru diperbarui setelah perintah selesai, jadi klik cepat berikutnya mengirim nomor dari daftar yang sudah basi dan menghapus baris yang salah.
- **Keputusan: setiap entri antrean punya id sendiri.** `QueueItem` di `onsa-audio` membawa `QueueId`, dibuat sekali saat entri dibuat dan hidup selama entri ada di antrean. Nomor urut tetap ada, tapi hanya sebagai posisi, tidak pernah sebagai identitas.
- **Mesin audio adalah sumber kebenaran** soal lagu yang sedang diputar. `update_queue` tidak lagi menerima `current`: mesin mencari sendiri entri yang sedang dimainkannya lewat id, lalu memetakan ulang semua nomor yang diingatnya (timeline, lane, dan info lagu) ke posisi barunya. Event `TrackStarted` membawa id entri, dan aplikasi memakai id itu, bukan hitungannya sendiri.
- **UI selalu mengikuti mesin**: entri antrean dikirim ke UI sebagai `{ entryId, track }`, dan `currentId` di snapshot menentukan baris mana yang disorot. Perintah hapus, pindah, dan lompat memakai `entryId`. Menghapus entri yang sudah tidak ada tidak melakukan apa-apa, sehingga klik ganda atau klik cepat beruntun tidak pernah mengenai baris yang salah.
- **Perilaku saat lagu yang sedang berbunyi dihapus**: pemutaran **langsung pindah ke lagu berikutnya** yang masih ada di antrean. Alasannya, pendengar baru saja membuang lagu itu, jadi membiarkannya selesai adalah jawaban yang salah, sementara berhenti sama sekali mengagetkan padahal antrean masih berisi. Kalau tidak ada lagi lagu sesudahnya, pemutaran berhenti; kalau repeat "seluruh antrean" menyala, pemutaran kembali ke lagu pertama. Aturan yang sama dipakai mesin dan aplikasi.
- **Perpindahan itu tidak berbunyi klik**: mesin memakai jalur yang sama dengan seek, yaitu output diredupkan lalu isinya dibuang sebelum lane baru dimulai.
- **Kasus batas lain**: menghapus lagu-lagu sesudah yang sedang berbunyi tidak mengganggu apa pun (hanya pre-roll yang dihitung ulang); mengosongkan antrean saat lagu berjalan menghentikan pemutaran; menghapus banyak baris beruntun dengan cepat aman karena setiap perintah menyebut id.
- **Jalur lain yang ikut diperiksa**: tarik untuk mengurutkan ulang, shuffle, "putar berikutnya", dan "tambah ke antrean" kini tidak lagi menghitung nomor lagu yang sedang diputar sama sekali — identitasnya tidak berubah ke mana pun entri itu berpindah. Satu-satunya nomor yang tersisa di perintah adalah **tujuan** tarik-lepas, yang memang posisi menurut definisinya; salah tempat karena daftar basi hanya menggeser posisi, tidak pernah menukar lagu.
- **Antrean aplikasi pindah ke modulnya sendiri** (`src-tauri/src/queue.rs`) supaya aturannya bisa diuji langsung tanpa menjalankan aplikasi.
- Keputusan ini menggantikan poin "Mengedit antrean tidak memotong lagu" pada catatan sisa M4 di atas.

## 2026-09-12 · Ruang disk runner Ubuntu dan ukuran folder target

- **Konteks**: setelah aplikasi punya dependensinya, job Ubuntu gagal di `Post Run Swatinem/rust-cache` dengan `No space left on device`, padahal semua langkah nyata lulus.
- **Runner dibersihkan lebih dulu**: langkah "Free disk space" menghapus toolchain bawaan runner yang tidak dipakai Onsa (dotnet, Android, GHC, Boost, Swift, dan tool cache), yang membebaskan lebih dari 20 GB. Setelah itu kedua job hijau.
- **Debug info dimatikan untuk profil `dev` dan `test`** di `Cargo.toml` workspace. Itu isi terbesar `target/`, dan tes maupun build pengembangan tidak membutuhkannya: panic tetap menyebut nama fungsinya. `strip` saja tidak cukup — `strip` hanya merapikan binary jadi, bukan library yang ditinggalkan setiap crate, dan justru library itu yang membuat cache besar. Pasang `debug = true` lagi secara lokal saat butuh debugger.
- **Cache tetap menyertakan `target/`** (bukan hanya dependensi). Dengan dua langkah di atas, ruangnya cukup, dan membuang cache target akan menambah waktu tiap run.

## 2026-09-12 · M5: visualizer, warna nada, dan mode hemat daya

- **Analisis mengikuti apa yang terlihat, bukan apa yang berjalan.** Setiap meter dan visualizer "mendaftar" selama ada di layar dan mencabut pendaftarannya saat pergi; aplikasi menjumlahkannya lalu memberi tahu mesin. Tanpa satu pun yang terdaftar, tap di callback output mati dan thread analisis tidur, jadi tidak ada beban sama sekali (SPEC §4.4, §13.1).
- **Spektrum terpisah dari meter.** `AnalysisSettings` sekarang punya `spectrum`: kalau hanya meter yang tampil, FFT tidak dijalankan sama sekali dan frame hanya membawa level. Spectral centroid ikut FFT, jadi warna nada butuh spektrum menyala — dan memang hanya ada gunanya saat ada yang diwarnai.
- **Jendela yang tidak terlihat diputuskan di Rust, bukan di UI.** Minimize dan restore tidak memancarkan event DOM yang bisa diandalkan di WebView; Tauri melaporkannya sebagai resize, jadi `remember_window` memeriksa `is_minimized`/`is_visible` dan mematikan analisis. Menyembunyikan ke tray juga langsung mematikannya, dan memunculkan kembali dari tray menyalakannya lagi.
- **Frame analisis lewat satu event.** Spektrum, peak-hold, meter, dan centroid dikirim bersama di `player://meter`, bukan dua aliran event terpisah, supaya UI selalu melihat satu potret yang konsisten dan jumlah pesan tidak berlipat.
- **Laju frame mengikuti kebutuhan**: 60 per detik saat visualizer tampil, 30 saat hanya meter, 20 saat mode hemat daya (SPEC §4.4).
- **Mode hemat daya menimpa, bukan menimpa permanen.** Saat menyala, mesin memakai buffer Besar, resampler Cepat, event posisi tiap 500 ms, dan warna nada dimatikan. Pilihan buffer dan resampler milik pengguna tetap tersimpan apa adanya dan berlaku lagi begitu mode ini dimatikan.
- **Interval event posisi jadi setelan mesin** (`PlaybackSettings.position_interval`), bukan konstanta, karena mode hemat daya perlu menurunkannya.
- **Warna nada dihitung di UI.** Mesin mengirim centroid mentah; UI-lah yang menghaluskannya dengan rata-rata bergerak (sekitar satu detik) lalu memetakannya ke `hue-rotate`. Pemetaannya: 1,2 kHz sebagai titik tengah tanpa geseran, lalu ±2,5 oktaf ke tepi rentang yang ditentukan tema (`toneColor.range`). Suara terang naik ke rona dingin, suara berat turun ke rona hangat. Hanya elemen yang disebut `toneColor.targets` yang ikut bergeser; ketiga tema bawaan menyebut spektrum saja.
- **Tiga varian spektrum** mengikuti tema dan acuan visual `palet-preview.html`: `segment` (blok bertumpuk dengan segmen mati yang tetap samar, Kaca asap), `bar` (batang polos di atas sumur samar, Kokpit kaca), dan `soft` (kolom hangat yang memudar dari bawah dengan penanda puncak merah, Deck malam).
- **Spektrum memakai warna lit tema, bukan gradien level meter.** Hijau–kuning–merah punya arti di peak meter (mendekati batas) dan tidak punya arti di spektrum; acuan visualnya pun memakai satu warna per tema. Penanda peak-hold-lah yang berbeda per tema: lit untuk Kaca asap, warna label untuk Kokpit kaca, warna clip untuk Deck malam.
- **Visualizer hanya ada di Sedang Diputar.** Menaruhnya juga di transport akan membuatnya berjalan terus-menerus di setiap layar, yang persis berlawanan dengan tujuan tap yang bisa dimatikan.

## 2026-09-13 · Mode jendela punya satu sumber kebenaran (perbaikan bug)

- **Konteks**: dilaporkan pemilik proyek. Dari jendela yang dimaksimalkan, masuk mini player lalu kembali menghasilkan jendela normal, bukan maximized seperti semula.
- **Akar masalahnya**: keluar dari mini player selalu memasang ukuran tetap 1180×760. Tidak ada apa pun yang mengingat jendela sebelum mini player mengambil alih — tidak ukurannya, tidak posisinya, apalagi keadaan maximized-nya. Selain itu, mode jendela hidup di tiga tempat sekaligus: sebuah `AtomicBool` di backend, `mini` di store UI, dan jendela sungguhan itu sendiri.
- **Keputusan**: `WindowMode` menyimpan mode **dan** jendela yang dipinjam mini player. Ia satu-satunya yang tahu Onsa sedang di mode mana. Semua jalan masuk — tombol, Ctrl+M, dan pemulihan sesi — lewat `session::set_mini`, yang menghitung bentuk tujuannya, menerapkannya, mencatat modenya, menyimpan sesi, lalu **memancarkan event `window://mode`**. UI tidak lagi menyimpulkan modenya sendiri: ia memakai jawaban perintah dan event itu.
- **Masuk mini dari jendela maximized meng-unmaximize dulu**, baru membaca geometrinya. Jendela yang dimaksimalkan seukuran layar dan tidak mengatakan apa-apa tentang tempatnya saat tidak dimaksimalkan; yang dicatat adalah yang sebenarnya akan dikembalikan nanti.
- **Keluar dari mini selalu memasang ukuran dulu, baru maximize kalau perlu.** Ukuran itulah yang dipakai Windows saat pengguna menekan tombol restore; kalau dibiarkan, yang tersisa di sana adalah strip mini player selebar 660 piksel.
- **Sesi menyimpan jendela penuh, bukan strip**, dan menyimpan jendela maximized pada ukuran sebelum dimaksimalkan. Membuka Onsa lagi lalu menekan restore memberi jendela yang dulu dipakai, bukan jendela seukuran layar.
- **Ukuran dibaca dari `inner_size`, bukan `outer_size`.** Di Windows, `set_size` mendarat pada ukuran dalam sementara `outer_size` melaporkan bingkai di sekelilingnya. Membaca yang satu lalu menulis yang lain membuat jendela **tumbuh 18×47 piksel setiap kali kembali dari mini player** — terukur persis begitu, tiga putaran berturut-turut, sebelum diperbaiki.

## 2026-09-13 · Warna nada: dua warna per tema dan kekuatan yang bisa diatur

- **Konteks**: pemilik proyek hampir tidak menyadari warna nada, terutama di Deck malam.
- **Akar masalahnya**: `hue-rotate` bekerja baik pada warna pekat seperti fosfor teal, tapi hampir tidak terlihat pada krem hangat yang saturasinya rendah — memutar rona krem tetap menghasilkan krem. Rentang 70° yang sama dipakai ketiga tema, padahal artinya berbeda-beda.
- **Keputusan: geseran warna, bukan putaran rona.** Setiap tema menyebut dua warnanya sendiri di `toneColor`: `warm` (tujuan suara berat) dan `cool` (tujuan suara terang). Warna elemen digeser dari warna aslinya ke salah satu ujung itu. Cara ini bekerja untuk palet mana pun, dan hasilnya tidak pernah mendarat di warna yang tidak dipilih temanya. `range` dalam derajat dihapus dari skema tema.
  - **Kaca asap**: teal fosfor, hangat ke amber (filter kedua tema ini), dingin ke biru fosfor `#4FA8FF`.
  - **Kokpit kaca**: hijau display, hangat ke amber "hati-hati" miliknya, dingin ke cyan "bisa diatur" miliknya.
  - **Deck malam**: krem lampu hangat, hangat ke merah jarum VU `#C8401F`, dingin ke baja dingin `#7FA6C8`.
- **Pencampuran dilakukan lewat cahaya** (kuadrat nilai kanal), bukan lewat angka mentahnya, supaya campuran setengah jalan benar-benar terlihat setengah jalan.
- **Kekuatan diatur pengguna**: mati / halus / sedang / kuat, dengan **sedang sebagai bawaan** — bukan lagi sekadar hidup-mati. Halus setengah jalan, sedang sepenuh rentang tema, kuat 1,6 kali (tetap berhenti di warna ujung tema). Pada tiga tema bawaan, "sedang" menggeser warna sejauh 37 sampai 79 dari 255 pada dua pertiga rentang — jelas terlihat tanpa jadi lampu disko.
- **Rentang centroid dipersempit** dari ±2,5 oktaf ke ±2,2 oktaf di sekitar 1,2 kHz, sehingga musik biasa memakai lebih banyak bagian rentangnya.
- **Elemen yang ikut bergeser ditentukan per tema.** Meter **tidak pernah** ikut: hijau, kuning, dan merahnya berarti "mendekati batas", dan menggesernya akan merusak arti itu. Deck malam menggeser bar posisinya juga, karena spektrumnya lebih kecil dan lebih hangat daripada dua tema layar lainnya; dua tema itu cukup dengan spektrum.
- **Setelan lama yang tersimpan** (`toneColor: true`) tidak bisa dibaca sebagai kekuatan, jadi kelompok setelan tampilan jatuh ke bawaannya, yaitu "sedang" — persis yang diinginkan.

## 2026-09-13 · Tata letak yang menyesuaikan lebar jendela

- **Konteks**: dilaporkan pemilik proyek. Saat jendela dikecilkan sampai ukuran paling kecil, judul kolom saling menindih, panel antrean tetap memaksa lebarnya, strip jalur sinyal pecah jadi dua baris, dan tombol kanan transport terdorong keluar layar.
- **Akar masalahnya ada dua.** Pertama, tidak ada satu pun tahap penyesuaian: semua panel memaksa lebarnya, dan kolom yang menyempit sampai nol tetap digambar sehingga teksnya saling menindih. Kedua, **ukuran minimum jendela dipasang dalam piksel fisik** (`PhysicalSize`) sementara `tauri.conf.json` menyebutnya dalam piksel logis; di layar 125%, minimum 880×560 itu sebenarnya hanya 704×448 — jauh lebih kecil daripada yang dimaksud.
- **Semua ukuran jendela sekarang dalam piksel logis**: minimum, ukuran mini player, ukuran bawaan, dan geometri yang disimpan. Itu satuan yang dipakai `tauri.conf.json`, yang dipakai UI untuk menata dirinya, dan yang dilihat pengguna. Efek sampingnya: mini player 660×146 selama ini sebenarnya tampil 528×117 di layar 125%, dan sekarang seukuran yang dimaksud.
- **Minimum jendela 820×600 logis**, naik dari 880×560 fisik yang menyesatkan itu. Itu lantai keras; di bawahnya tata letaknya tidak punya tempat lagi untuk melangkah mundur.
- **Tahapannya, dalam piksel CSS lebar jendela**: ≥1160 antrean punya kolom sendiri; ≥1040 sidebar dengan kata-katanya; 880–1039 sidebar jadi strip ikon; <880 sidebar jadi laci yang dibuka lewat tombol. Semua angka itu ada di satu tempat (`ui/src/lib/layout.svelte.ts`), bukan tersebar di CSS tiap komponen.
- **Panel yang menimpa hanya menutupi area konten**, bukan seluruh jendela: transport dan strip jalur sinyal tetap terjangkau saat antrean atau laci terbuka.
- **Kolom daftar lagu dibuang menurut prioritas, bukan disempitkan**: tahun (di bawah 880), lalu album (740), lalu artis (560); judul dan durasi selalu bertahan. Komponen mengukur **kotaknya sendiri**, bukan lebar jendela, karena sidebar dan antrean sudah mengambil bagiannya lebih dulu. Setiap sel juga dipotong dengan elipsis, jadi bahkan saat transisi tidak ada yang bisa menindih.
- **Strip jalur sinyal selalu satu baris** (`flex-wrap: nowrap` dan `overflow: hidden`). Saat menyempit ia melepas angka-angkanya (≥1060 penuh), lalu memendekkan namanya (≥900), lalu melipat ReplayGain dan Limiter ke tombol "…" yang membukanya di tempat (<780). Yang dilipat adalah tahap yang paling jarang disentuh; sumber, EQ, dan output selalu terlihat.
- **Transport menjaga tombol putar, posisi, dan volume** di lebar berapa pun. Meter hilang di bawah 1000, cover di bawah 760, dan tombol tambahan (Sedang diputar, sleep timer, mini player) pindah ke tombol "…" di bawah 1120.
- **Halaman pengaturan memakai container query**, bukan media query: yang menentukan bukan lebar jendela melainkan lebar area konten yang tersisa setelah sidebar dan antrean. Label pindah ke atas kontrolnya di bawah 520, dan fader EQ menyempit. Tabel EQ parametrik adalah satu-satunya yang menyimpan scroll mendatarnya sendiri: deretan angka itu tidak bisa dilipat tanpa berbohong soal isinya, dan scroll-nya tertahan di dalam kotaknya.
- **Sedang Diputar menumpuk** cover di atas keterangan di bawah 700. Cover diberi lebar **dan** tinggi: sebuah baris grid tidak bisa menghitung tinggi dari `aspect-ratio`, dan akibatnya cover sempat duduk di atas judul — ketahuan lewat pemeriksaan tumpang tindih, bukan lewat mata.

## 2026-09-13 · M6: playlist biasa, playlist pintar, dan M3U8

- **Aturan playlist pintar dikompilasi jadi SQL berparameter, selalu.** `Rules` (JSON) diterjemahkan ke satu statement dengan `?` untuk setiap nilai; tidak ada satu pun nilai dari pengguna yang pernah masuk ke teks SQL-nya. Nama kolom tidak pernah dibangun dari input: setiap `Field` memetakan ke satu ekspresi kolom yang sudah tertulis di kode. Ada tes yang menyimpan `'; DROP TABLE tracks; --` sebagai nilai aturan dan memastikan library-nya utuh setelahnya, dan tes lain yang memastikan `%` diperlakukan sebagai karakter biasa, bukan wildcard (`ESCAPE '\'`).
- **Playlist pintar tidak pernah menyimpan daftar lagunya.** Yang tersimpan hanya aturannya; lagunya ditanyakan ulang setiap kali playlist dibuka, jadi ia otomatis ikut berubah saat library berubah (SPEC §6.3). Batas atasnya 10.000 baris supaya aturan tanpa limit tidak menarik seluruh library sekaligus ke UI.
- **Operator terikat pada jenis kolomnya.** Kolom teks hanya menerima operator teks, kolom angka hanya operator angka, kolom tanggal hanya operator tanggal. Aturan yang tidak cocok ditolak saat dikompilasi, bukan diam-diam dianggap tidak cocok — dan editornya hanya menawarkan operator yang berlaku, jadi aturan yang ditolak backend tidak bisa dibangun dari UI.
- **Aturan "tidak" ikut mencakup lagu yang kolomnya kosong.** "Genre tidak mengandung pop" harus juga memuat lagu tanpa genre; kalau tidak, aturan negatif akan diam-diam membuang lagu yang belum ditandai.
- **Posisi playlist dinomori ulang seluruhnya saat sebuah baris dipindahkan.** `(playlist_id, position)` adalah primary key, jadi menggeser baris satu per satu akan menabrak tetangganya (SQLite tidak menjanjikan urutan baris saat UPDATE). Menulis ulang urutannya sederhana dan jelas benar; panjang playlist tidak sampai membuat itu terasa.
- **Lagu yang sama boleh dua kali dalam satu playlist biasa** (SPEC §6.2), jadi identitas barisnya adalah posisinya, bukan track id-nya.
- **Menghapus lagu dari library menghapusnya dari playlist**, lewat `ON DELETE CASCADE`. Tapi file yang hilang hanya ditandai `missing` dan **tetap ada di playlist**, sesuai SPEC §5.2 — ia baru pergi kalau pengguna sendiri membersihkannya.
- **M3U8 ditulis dengan path relatif kalau bisa**, supaya playlist yang disimpan di sebelah musiknya ikut berpindah bersama musiknya. Selalu UTF-8 tanpa BOM, selalu garis miring depan, dan selalu diakhiri baris baru — itu yang dibaca pemutar lain di kedua sistem. Ekspor dengan path absolut tetap tersedia di lapisan library.
- **Impor melaporkan apa yang tidak ditemukan, tidak membuangnya diam-diam** (SPEC §6.4). `#EXTINF` dibaca hanya untuk tempatnya: tag milik library lebih baik daripada tulisan di file playlist. Path relatif yang naik keluar foldernya (`../`) tetap diikuti, dan di Windows path dibandingkan tanpa memandang besar-kecil huruf.
- **Perintah playlist menulis lewat koneksi pembaca**, bukan lewat thread library. Perubahan playlist adalah perbuatan pengguna sendiri dan harus terlihat begitu perintahnya kembali; databasenya WAL dengan busy timeout, jadi scan yang sedang berjalan tetap bisa membaca sementara transaksi pendek ini di-commit.
- **Satu event untuk daftar playlist** (`library://playlists`). UI memuat ulang daftarnya saat event itu datang, dan juga saat library berubah — karena jumlah lagu playlist pintar ikut berubah tanpa ada yang menyentuh playlist-nya.
- **Antrean bisa disimpan jadi playlist** lewat tombol di panel antrean (SPEC §6.2); namanya diusulkan berisi tanggal hari ini dan tinggal diganti.

### Dua bug yang ditangkap tesnya sebelum ada yang memakainya

- Kolom "folder" memanggil fungsi SQL `parent_of`, padahal yang didaftarkan ke koneksi bernama `onsa_parent`. Setiap aturan folder akan gagal.
- Memindahkan baris playlist menabrak primary key-nya, seperti dijelaskan di atas.

## 2026-09-13 · Jendela yang diminimalkan tidak menimpa sesi (perbaikan bug)

- **Konteks**: ketahuan saat memverifikasi M6. Onsa terbuka sekecil-kecilnya di pojok layar, padahal ditinggalkan dalam jendela besar. Yang tersimpan di sesi: `{"width":0,"height":0,"x":-25600,"y":-25600}`.
- **Akar masalahnya**: sesi disimpan saat jendela kehilangan fokus dan saat ditutup. Jendela yang **diminimalkan** kehilangan fokus juga — dan jendela yang diminimalkan melaporkan ukuran nol dan tempat jauh di luar layar. Itulah yang tersimpan.
- **Keputusan**: `WindowMode` menolak geometri yang tidak layak jadi jendela penuh (`usable`) dan menyimpan jendela terakhir yang layak sebagai gantinya. Selain itu setiap ukuran yang dilewati jendela **selagi terlihat** dicatat, jadi meminimalkan tepat sebelum menutup tidak menghilangkan tempat terakhir yang sungguhan. Strip mini player otomatis ikut tertolak oleh aturan yang sama, karena lebih kecil daripada jendela penuh terkecil.
- Kalau tidak ada satu pun yang layak diingat, sesi menyimpan ukuran bawaan tanpa posisi, sehingga sistem yang menempatkannya.

## 2026-09-14 · Tarik-lepas tidak memakai HTML5, dan satu mekanisme untuk semua

- **Konteks**: dilaporkan pemilik proyek. Mengurutkan playlist dengan tarik-lepas tidak berfungsi; hanya tombol panah yang jalan.
- **Akar masalahnya ada di Tauri, bukan di kode Onsa.** Target file-drop milik Tauri memegang drag loop milik Windows. Dokumentasi Tauri sendiri menyebutnya: *"Disabling it is required to use HTML5 drag and drop on the frontend on Windows"*, dan bawaannya menyala. Onsa membutuhkannya untuk "jatuhkan file ke jendela untuk memutarnya" (SPEC §13), jadi mematikannya bukan pilihan. Panel antrean punya masalah yang sama persis karena memakai API yang sama.
- **Keputusan: satu mekanisme berbasis pointer event** (`ui/src/lib/reorder.ts`), dipakai antrean dan playlist. Pointer event tidak dicegat siapa pun dan berperilaku sama di Windows maupun Linux. Baris membawa nomor urutnya di `data-index`, jadi daftar yang hanya menggambar yang terlihat (antrean) bekerja sama seperti yang menggambar semuanya.
- **Tanpa `setPointerCapture`.** Menangkap pointer akan mengirim klik dan klik-dua-kali ke wadahnya, bukan ke barisnya — dan klik-dua-kali adalah cara memutar sebuah baris. Pendengarnya dipasang di `window` selama penekanan berlangsung.
- **Ada jalan lewat keyboard**: Alt+panah memindahkan baris yang sedang difokus, di antrean maupun di playlist.
- **Tanda jatuh digambar lewat atribut** (`data-drop`, `data-grabbed`) yang dipasang langsung ke elemennya, bukan lewat state reaktif, supaya daftar tervirtualisasi ikut terlayani. Selektornya di CSS memakai `:global(...)` karena Svelte tidak bisa melihat atribut yang dipasang saat berjalan.

## 2026-09-14 · Lebar kolom milik pengguna, dan aturan penyempitan

- **Lebar kolom disimpan per tampilan** (`display.columns` di setelan): Lagu, Album, Artis, Genre, Folder, playlist, dan hasil pencarian masing-masing punya susunannya sendiri. Tampilan yang belum pernah diatur tidak disimpan sama sekali, jadi mengubah bawaan nanti tetap sampai ke orang yang belum pernah berpendapat soal kolom itu.
- **Hanya lebar yang dilepas yang ditulis**, bukan setiap piksel selama menarik.
- **Kolom dibuang hanya kalau benar-benar tidak muat.** Ambang tetap lama (tahun di bawah 880, album 740, artis 560) diganti perhitungan sungguhan: judul boleh menyusut sampai batas minimumnya dulu, baru kolom berprioritas terendah keluar — tahun, lalu album, lalu artis. Judul dan durasi tidak pernah keluar. Akibatnya, pada lebar bawaan sekarang lebih banyak kolom yang bertahan di jendela sempit daripada sebelumnya, dan itu memang lebih baik: dengan lebar yang pasti, kolom tidak lagi bisa saling menindih.
- **Judul memakai `minmax(lantai, 1fr)`**: lebar yang ditarik pengguna jadi lantainya, bukan ukurannya, supaya jendela lebar tidak menyisakan lubang di kanan. Kalau ruangnya kurang dari lantai itu, judul yang mengalah lebih dulu — sampai batas minimumnya sendiri, tidak lebih.
- **Padding baris dan celah antar-sel ikut dihitung.** Dengan kolom `fr` keduanya terserap sendiri; dengan lebar piksel tidak. Tidak menghitungnya membuat kepala tabel menjorok keluar **tepat 88 piksel** (2×14 padding + 6×10 celah) — ketahuan oleh pemeriksa tata letak, bukan oleh mata.
- **Penarik pembatas sengaja menjorok** 7 piksel ke celah di sebelahnya: di situlah garis antara dua kolom berada. Sel kepala karena itu tidak boleh memotong isinya (`overflow: visible`), dan pemeriksa tata letak diberi pengecualian untuk elemen ini, seperti tabel EQ parametrik.
- **Menu kolom menempati kolom sendiri di ujung kanan** (22 piksel), bukan mengambang di atas grid, supaya kepala dan baris memakai kisi yang sama persis.
- **Tes antarmuka masuk ke pemeriksaan.** Bagian yang murni hitungan — di mana sebuah baris mendarat, kolom mana yang muat — diuji dengan `node --test` (`npm test` di `ui/`), dan ikut dijalankan CI. `@types/node` masuk sebagai dependensi pengembangan supaya berkas tesnya ikut diperiksa tipenya.

## 2026-09-14 · Membawa lagu ke playlist, dan playlist di sidebar

- **Membawa dan menjatuhkan memakai dasar yang sama dengan mengurutkan**: pointer event, bukan HTML5 drag-and-drop, karena alasannya sama persis (target file-drop Tauri). Keduanya juga sepakat soal rasanya: sebuah penekanan baru jadi tarikan setelah menempuh jarak tertentu, jadi klik dan klik-dua-kali tetap sampai ke barisnya.
- **Sasaran jatuh mendaftar lewat atribut** (`data-drop-target`), dan yang membawa mencari lewat `elementsFromPoint`. Dengan begitu sasaran tidak perlu tahu apa pun tentang asal muatannya, dan sebaliknya.
- **Yang dibawa adalah satu baris yang ditekan**, bukan seluruh daftar tempatnya berasal. Untuk memasukkan seluruh album atau seluruh artis sekaligus ada tombol tersendiri di kepala halamannya, yang mengirim `PlayContext`-nya apa adanya — backend yang menentukan lagu mana saja, jadi tidak ada daftar yang perlu disusun di UI lebih dulu.
- **Sidebar menampilkan enam playlist terakhir berubah**, lalu "Lihat semua". Enam cukup untuk jadi sasaran jatuh tanpa mendorong pengaturan keluar layar; sisanya satu klik jauhnya. Di mode ikon daftar itu tidak muncul: strip selebar ikon tidak punya tempat untuk nama.
- **Lencana yang mengikuti kursor tidak menerima pointer event** (`pointer-events: none`), supaya ia tidak menutupi sasaran yang ada persis di bawahnya.

## 2026-09-14 · Kunci layanan luar: dari mana, dan apa yang boleh dilihat

- **Satu variabel lingkungan per layanan**, dinamai seperti kunci Last.fm: `ONSA_ACOUSTID_API_KEY`.
- **Tiga sumber, yang pertama menang**: (1) kunci yang diketik pengguna di Pengaturan dan disimpan di database miliknya sendiri, (2) variabel lingkungan aplikasi yang sedang berjalan, (3) variabel lingkungan mesin yang mem-build (lewat `option_env!`). Kunci kosong dianggap tidak ada, jadi mengosongkan kolomnya jatuh kembali ke sumber berikutnya.
- **Kunci tidak pernah kembali ke antarmuka.** UI bisa menulisnya dan bisa bertanya *apakah* ada dan *dari mana*, tapi tidak pernah membacanya lagi. Yang tidak pernah meninggalkan backend tidak bisa berakhir di log atau laporan. Ada tes yang memeriksa bahwa JSON statusnya tidak memuat nilai kuncinya.
- **Log hanya mencatat bahwa sebuah kunci tersimpan**, bukan kuncinya (`acoustid=true`).
- **AcoustID punya dua jenis kunci, dan yang dibutuhkan adalah yang application.** Dokumentasi web service-nya menyebut `client` sebagai *"application's API key"* yang wajib untuk `/v2/lookup`, dan `user` sebagai *"user's API key"* yang hanya tambahan untuk `/v2/submit`, dengan catatan tegas: *"You should not store this key in your application code, each user should provide their own"*. Onsa hanya melakukan lookup, jadi yang dipakai adalah kunci aplikasi dari `acoustid.org/new-application`.
- **Fitur internet mati secara bawaan** (`MetadataPrefs.online = false`). Persetujuan pengguna adalah syarat, jadi keadaan awalnya bukan menyala.
- **Perintah uji ada di `onsa-cli`, bukan di aplikasi**: `onsa-cli acoustid-check` membaca variabel lingkungan dan melaporkan ada/tidak, jumlah karakter, dan keluhan yang lazim (masih ada tanda kutip, ada spasi di dalamnya, ada karakter yang bukan milik kunci AcoustID). Jumlah karakter bukan isi kuncinya dan itulah yang menangkap tempelan yang terpotong.

## 2026-09-14 · Klien HTTP untuk seluruh proyek: reqwest dengan rustls

- **Konteks**: SPEC tidak menyebut klien HTTP di mana pun, padahal M7 (MusicBrainz, AcoustID, Cover Art Archive), M8 (LRCLIB), M9 (Last.fm), dan M10 (unduh binary) semuanya membutuhkannya. Karena ini lintas milestone, diputuskan bersama pemilik proyek dan dimasukkan ke SPEC §1 sebagai keputusan terkunci.
- **`reqwest`, dengan TLS bawaannya.** Tauri sendiri sudah mengompilasi reqwest, jadi crate-nya bukan tambahan; yang baru hanya backend TLS-nya. Ternyata **reqwest 0.13 memakai rustls di fitur bawaannya** — Tauri-lah yang mematikannya (`default-features = false`), yang membuat pohon dependensi tampak tanpa TLS sama sekali. Menyalakan fitur bawaan cukup untuk mendapatkan rustls, dan tidak ada OpenSSL maupun native-tls yang masuk (diperiksa di `Cargo.lock`).
- **Blocking, bukan async.** Tidak ada permintaan jaringan di Onsa yang perlu ditunggu di thread yang penting: library punya thread sendiri, perintah yang lama menyerahkan kerjanya ke `spawn_blocking`, dan mesin audio tidak pernah menyentuh jaringan. Klien blocking membuat kode pemanggilnya lurus, dan aturan "jangan panggil dari thread runtime async" ditulis di modulnya.
- **Aturan yang berlaku di semua permintaan**, satu tempat di `src-tauri/src/net.rs`:
  - **User-Agent** `Onsa/<versi> ( <url repo> )`, yang diminta MusicBrainz (SPEC §8);
  - **timeout koneksi 10 detik dan timeout total 30 detik**;
  - **batas ukuran jawaban 8 MB**, diperiksa dua kali: panjang yang diklaim server, lalu pembacaannya sendiri dibatasi — server yang berbohong soal panjang tetap tidak bisa membuat Onsa membaca selamanya;
  - **maksimal 3 redirect**;
  - **kegagalan dikembalikan sebagai nilai**, tidak pernah panic. Klien yang gagal dibangun pun mengembalikan `None`, dan fitur yang membutuhkannya berkata "tidak tersedia".
- **URL tidak pernah masuk log.** Kunci API hidup di query string, jadi pesan error dibersihkan dari URL-nya lebih dulu. Ada tesnya: sebuah error sungguhan dengan kunci di URL-nya harus keluar tanpa kunci itu.
- **Modul ini tumbuh bersama pemakainya.** Yang ada sekarang hanya yang benar-benar dipanggil; batas unduhan dan pembungkus permintaan yang lebih lengkap menyusul bersama pemakai pertamanya di M7 dan M10, supaya tidak ada kode mati yang menunggu.
- **Uji kunci AcoustID tanpa membaca audio**: lookup lewat `trackid` tidak membutuhkan fingerprint, jadi satu-satunya hal yang benar-benar ditanyakan permintaan itu adalah kuncinya. AcoustID tidak mendokumentasikan kode error-nya, jadi jawabannya dibaca apa adanya: `status: ok` berarti kunci lolos, error yang menyebut "api key" berarti tidak, dan error tentang hal lain tetap berarti kuncinya lolos — ia sudah sampai ke tahap mengeluhkan sisa permintaannya.
- **Tombol "Coba kunci" menghormati sakelar internet.** Selama "Ambil metadata dari internet" mati, tombol itu tidak menghubungi apa pun dan berkata begitu. "Bisa dimatikan total" berarti total.

## 2026-09-14 · Kolom aturan tentang file itu sendiri, dan pratinjau yang menampilkan lagunya

- **Diminta pemilik proyek** untuk mengumpulkan file unduhan yang tagnya kosong, sebagai bahan perapian di M7.
- **Lima kolom baru**: sample rate, kedalaman bit, bitrate (ketiganya angka), serta "cover" dan "tag artis" sebagai pertanyaan ya-atau-tidak.
- **Jenis kolom baru, `Flag`**, dengan dua operator sendiri (`yes` dan `no`) dan **tanpa nilai**: pertanyaannya sudah utuh di perbandingannya. Editor tidak menampilkan kolom nilai untuk kolom semacam ini, dan aturannya tidak mengikat satu parameter pun.
- **"Tag artis" berarti tag yang berisi sesuatu**: `TRIM(artist)` yang kosong dihitung tidak ada, karena tag berisi spasi persis sama tidak bergunanya dengan tag yang hilang.
- **Angka yang tidak ada sekarang dihitung "bukan angka itu".** Sebelumnya `tahun != 2000` melewatkan lagu yang tahunnya kosong, karena perbandingan dengan NULL di SQL menghasilkan NULL. Sekarang `Ne` ditulis `(kolom IS NULL OR kolom <> ?)`, sejalan dengan aturan teks "tidak mengandung" yang sudah lebih dulu begitu. Ini mengubah perilaku `!=` pada kolom tahun juga, dan ke arah yang benar.
- **Pratinjau menampilkan delapan lagu pertama**, bukan hanya jumlahnya, supaya aturannya bisa dilihat mengenai lagu yang dimaksud — bukan sekadar dihitung.

## 2026-09-14 · Satu tempat untuk mengambil nama file dari sebuah path

- Tujuh komponen masing-masing punya salinan `fileName()` yang sama. Salah satunya ditulis `/[\/]/` alih-alih `/[\/]/`, sehingga path Windows tidak pernah terpotong dan pratinjau aturan menampilkan `C:\Users\...\lagu.mp3` utuh sebagai judul.
- Sekarang fungsinya satu, di `ui/src/lib/format.ts`, **dengan tesnya sendiri** untuk path Windows, path Linux, path campuran, dan string yang memang sudah berupa nama file.
- Tesnya bukan hiasan: kesalahan yang sama terjadi **dua kali** dalam satu sesi, keduanya karena satu backslash hilang saat berkas ditulis, dan keduanya terlihat benar sampai sebuah path Windows melewatinya.
