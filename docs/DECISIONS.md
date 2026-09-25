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

## 2026-09-14 · Pengaman sebelum metadata otomatis menyentuh library

Diminta pemilik proyek: pengaman dulu, baru jaringan. Semua di bawah ini ada dan teruji **sebelum** ada satu pun penulisan massal.

- **Batalkan bekerja per batch, bukan per langkah.** Satu kali jalan adalah satu batch; membatalkan setengahnya lebih buruk daripada kedua ujungnya. Setiap langkah dicatat di `edit_steps` dengan nilai **sebelum** dan **sesudah**, dan pembatalan berjalan mundur dari langkah terakhir.
- **Urutan mundurnya penting.** Sebuah file yang di-retag lalu dipindah harus dipulihkan dengan urutan terbalik: tagnya dikembalikan selagi file masih memakai nama barunya, baru namanya dikembalikan. Itulah sebabnya langkah diurutkan `position DESC`.
- **`track_id` di jurnal sengaja bukan foreign key.** Catatan tentang apa yang terjadi harus hidup lebih lama daripada baris track-nya: sebuah file bisa keluar dari library dan tetap perlu dikembalikan namanya.
- **Tiga lapis, dan pembatalan menuruni tangga yang sama.** Editan masuk `overrides` dulu (tidak ada file yang tersentuh), "tulis ke file" adalah aksi kedua, memindahkan file aksi ketiga. Ini yang diminta SPEC §8, dan kebetulan juga membuat lapisan yang paling sering dipakai adalah yang paling murah dibatalkan.
- **Run yang tidak mengubah apa pun tidak dicatat.** Usulan yang isinya sama dengan yang sudah ada bukan perubahan, jadi tidak meninggalkan langkah — membatalkan run tidak lantas "mengembalikan" sesuatu yang tidak pernah diubah siapa pun.
- **Cakupan folder diuji terhadap path yang sudah dirapikan**, sehingga `salinan/../asli/lagu.mp3` tidak bisa menyelinap, dan folder bertetangga bernama mirip (`salinan-lama`) bukan bagian dari `salinan`. Folder itu sendiri bukan file di dalamnya. Di Windows perbandingannya mengabaikan besar-kecil huruf.
- **Cakupan diperiksa dua kali untuk pemindahan**: sekali saat rencana disusun, sekali lagi saat rencana dijalankan. Sebuah rencana bisa dibuat, disimpan, lalu dijalankan belakangan.
- **Batas per sekali jalan 50, dengan langit-langit keras 500.** Yang menentukan bukan kemampuan mesin melainkan kemampuan orang membaca ulang dan membatalkannya kalau ada yang salah.
- **Penulisan ke file dibaca ulang sebelum menggantikan aslinya.** Salinan sementara ditulis, lalu dibaca kembali; kalau tidak berisi apa yang diminta, salinan itu dibuang dan file aslinya tidak pernah tersentuh. Ini bukan kehati-hatian teoretis: lofty **kehilangan tag ID3v2 di dalam WAV pada penulisan kedua berturut-turut** — terlihat langsung saat menulis di tempat. Pendekatan salin-lalu-ganti menghindarinya, dan pembacaan ulang menangkapnya kalau kasus serupa muncul lagi.
- **Nilai angka dibandingkan sebagai angka** saat pembacaan ulang: tag yang diberi `05` mengembalikan `5`, dan itu nomor trek yang sama, bukan penulisan yang gagal.
- **Nilai ditulis ke semua tag yang dipunyai file.** File yang memegang ID3v2 sekaligus RIFF INFO kalau tidak begitu akan bertentangan dengan dirinya sendiri, dan menentukan mana yang dipercaya pemutar lain bukan hak Onsa.
- **Rename selalu punya dry-run.** Rencananya menyebutkan asal, tujuan, dan alasan sebuah file tidak bisa dipindah — dua lagu yang akan mendarat di nama yang sama, atau nama yang sudah ditempati. Yang bentrok tetap ditampilkan, tidak dibuang diam-diam, dan tidak ikut dijalankan.
- **Pemisah path di dalam nilai tag ikut diganti.** `AC/DC` tidak boleh berubah menjadi dua folder: bentuk path ditentukan polanya, bukan tagnya.
- **Usulan berkeyakinan rendah tidak tercentang.** Ambangnya 0,85. Di bawah itu usulannya tetap ditampilkan untuk dibaca, tapi tidak satu pun field-nya tercentang — menekan "terapkan" tanpa membaca tidak mengubah apa-apa. Field yang mengusulkan persis nilai yang sudah ada juga tidak pernah tercentang.
- **Batas laju ditunggu di thread yang meminta**, bukan dijadwalkan. Thread itu selalu thread pekerja, jadi antrean lookup memperlambat dirinya sendiri, bukan hal yang sedang dilihat pengguna. MusicBrainz 1 permintaan per detik (SPEC §8), AcoustID 3 per detik sesuai dokumentasinya.

## 2026-09-14 · Halaman Rapikan: cakupan yang selalu terlihat, ringkasan sebelum tombol

- **`ONSA_DATA_DIR` memindahkan database, cover, dan log sekaligus.** Dibuat supaya pengujian bisa berjalan terhadap library sekali pakai tanpa pernah membuka library sungguhan pemilik proyek — dan nanti berguna lagi untuk build portable (M12). Satu variabel memindahkan ketiganya, karena library tanpa cover-nya bukan library yang sama.
- **Cakupan disimpan, bukan ditanyakan tiap kali.** Folder yang sedang dibatasi adalah satu-satunya hal yang tidak boleh diragukan, jadi ia harus bisa ditampilkan terus-menerus, juga setelah aplikasi dibuka lagi.
- **Spanduk cakupan diberi warna menurut isinya**: dibatasi folder memakai warna "aktif", sementara "seluruh library" memakai warna peringatan beserta satu kalimat yang menyebutkan akibatnya. Keadaan yang lebih berbahaya harus terlihat lebih berbahaya.
- **Nama folder ditampilkan lebih dulu, path lengkapnya di bawahnya dan dibungkus, bukan dipotong.** Path yang dipotong di tengah menyembunyikan justru ujungnya — bagian yang membedakan satu folder dari yang lain.
- **Tombol terapkan milik komponen ringkasan**, dan secara struktur berada di bawah angka-angkanya. Ringkasan yang harus digulir balik untuk dibaca adalah ringkasan yang tidak dibaca.
- **Ringkasan basi dihapus begitu permintaannya berubah.** Mengganti field, nilai, pola, atau folder tujuan langsung mengosongkan ringkasan; ringkasan basi di atas tombol terapkan persis hal yang harus dihindari.
- **Rencana rename disusun ulang di backend saat diterapkan**, bukan diambil dari antarmuka. Nama yang keburu ditempati sejak rencana dibuat ketahuan sekarang, bukan dipercaya dari semenit lalu.
- **Tujuan rename ditampilkan relatif terhadap folder tujuan.** Semua baris berawalan sama dan terlalu panjang untuk dibaca utuh; yang membedakan satu baris dari yang lain ada di ujungnya.
- **Menulis file dan memindahkan file berjalan di `spawn_blocking`.** Membaca dan menulis ratusan file bukan pekerjaan thread yang menjawab antarmuka.

## 2026-09-14 · Path tidak lagi dipotong sendiri-sendiri di komponen

- Regex pemisah path ditulis ulang di berbagai komponen, dan **empat kali dalam satu sesi sebuah backslash hilang saat berkasnya ditulis** (`/[\/]/` alih-alih `/[\\/]/`). Tiap kali hasilnya terlihat benar sampai sebuah path Windows melewatinya, lalu path utuh muncul di tempat nama berkas seharusnya.
- Sekarang `fileName()` dan `relativeTo()` ada satu-satunya di `ui/src/lib/format.ts`, **dengan tesnya**, dan tidak ada komponen yang menulis regex path lagi. Tesnya yang menangkap kesalahan itu, bukan mata.

## 2026-09-15 · AcoustID, MusicBrainz, dan Cover Art Archive lewat pintu yang sama

- **Usulan dari internet tidak punya jalur terapkan sendiri.** Pencocokan hanya menghasilkan daftar; yang dicentang dikirim ke `tidy_preview_edits` dan `tidy_apply_edits` — perintah yang sama persis dengan nilai yang diketik tangan. Jadi cakupan folder, ringkasan sebelum tombol, dan riwayat yang bisa dibatalkan berlaku otomatis, bukan karena diingat untuk diberlakukan lagi.
- **Pengelola program luar dibuat sekali, di `onsa-downloader`.** `fpcalc` (M7) dan yt-dlp, ffmpeg, Deno (M10) adalah persoalan yang sama: array argumen tanpa shell, `CREATE_NO_WINDOW` di Windows, tenggat waktu dengan kill di ujungnya, dan keluaran dibaca di thread sendiri supaya pipa yang penuh tidak membuat Onsa menunggu selamanya. Urutan pencarian: pilihan pengguna, folder `bin` milik Onsa, lalu PATH sistem (SPEC §7.1).
- **Mengunduh binary-nya sendiri ditunda ke M10.** Di sana ffmpeg, Deno, dan yt-dlp butuh pembongkaran arsip (zip, tar.gz, tar.xz) beserta alur persetujuan dan checksum-nya. Membuat setengahnya sekarang untuk satu program berarti membuatnya dua kali. Untuk M7, fpcalc ditunjuk lewat Pengaturan atau diambil dari PATH, dan kalau tidak ada, hanya lagu yang sudah bertag yang bisa dicari — dikatakan di layar, bukan gagal diam-diam.
- **Sidik suara tidak mengirim audio.** Yang dikirim ke AcoustID adalah ringkasan Chromaprint, panjang lagu, dan API key. Ringkasan itu tidak bisa dikembalikan menjadi rekamannya.
- **Tingkat keyakinan menentukan perilaku, bukan sekadar ditampilkan.** Sidik suara memakai skor AcoustID apa adanya. Pencarian teks MusicBrainz memakai skornya sendiri, **tapi ditahan di bawah 0,85 kalau hasilnya tidak sama dengan judul *dan* artis yang ada di file** — skor pencarian mengatakan dua teks mirip, bukan bahwa ini rekaman yang benar. Tebakan dari nama file bernilai 0,6 (ada artis dan judul) atau 0,35 (judul saja), dan hasil MusicBrainz yang ditanyakan berdasarkan tebakan itu dikalikan dengan nilai tebakannya — rantai hanya sekuat mata rantai terlemahnya.
- **Dua sumber yang berbeda: keduanya ditampilkan, tidak ada yang tercentang.** Itu baris yang harus diputuskan orang, dan memilih salah satunya diam-diam berarti mesin memutuskan sambil tampak tidak memutuskan.
- **Tapi yang terlalu lemah untuk ditampilkan tidak bisa menghalangi.** Berkas bernama `03 kosong` akan mengaku berjudul "kosong"; kalau itu dihitung sebagai "sumber yang berbeda", hampir setiap lagu tanpa tag harus diputuskan satu per satu — padahal justru itu yang mau dibantu. Jadi hanya usulan di atas 0,5 yang bisa berselisih. Perbandingannya mengabaikan besar-kecil huruf dan tanda baca: "BANG BANG" dan "Bang Bang" bukan perselisihan.
- **Satu baris bisa membawa usulan sekaligus keluhan.** Berkas yang suaranya tidak terbaca tetap punya nama yang bisa dibaca; barisnya menyebut keduanya.
- **Cover diambil sekali per rilis, bukan sekali per lagu**, disimpan di `cache/covers/proposed/` dengan nama hash gambarnya, dan disajikan lewat protokol `onsa://` — bukan base64 di dalam event (SPEC §2). Menyetujuinya tidak mengunduh ulang apa pun. Nama yang dipakai di path harus hash buatan Onsa sendiri; selain itu tidak dibaca.
- **Cover masuk ke database, bukan ke dalam file.** Ia berdiri di lapisan yang sama dengan `overrides`: murah dibatalkan, dan "tulis ke file" tetap aksi terpisah. Menanamkan gambar ke dalam tag belum dikerjakan.
- **Mati berarti mati, termasuk yang sedang berjalan.** Mematikan fitur internet menghentikan run yang sedang jalan. Yang sudah ditemukan tetap ada — dipegang aplikasi, bukan thread-nya, jadi berhenti, gagal, atau layanan yang mati di tengah jalan tidak menghapus lagu-lagu yang sudah selesai.
- **Alamat layanan bisa dipindahkan ke mesin ini, dan hanya ke mesin ini.** `ONSA_ACOUSTID_URL`, `ONSA_MUSICBRAINZ_URL`, dan `ONSA_COVERART_URL` hanya dipatuhi kalau menunjuk ke loopback; selain itu diabaikan dan dicatat di log. Dengan begitu seluruh alurnya bisa diuji terhadap build rilis — lewat HTTP sungguhan, klien yang sama, batas laju yang sama — tanpa key, tanpa internet, dan tanpa menyentuh musik siapa pun; sementara sebuah env var tidak bisa dipakai mengirim key ke tempat baru.
- **Nilai tag tidak bisa menjadi bagian dari query.** Judul dan artis masuk ke pencarian MusicBrainz sebagai satu frasa berkutip, dengan tanda kutip dan backslash dibuang lebih dulu. Id MusicBrainz yang masuk ke path harus berbentuk id; selain itu tidak diminta.

## 2026-09-15 · Perapihan otomatis: aturan yang malu-malu

- **Tanpa jaringan sama sekali.** Ini library membaca dirinya sendiri: sisa nama unduhan, huruf besar-kecil, cara menulis `feat.`, dan nama yang tertulis beda-beda. Hasilnya usulan, dan diterapkan lewat perintah yang sama dengan editan manual.
- **Yang terlihat disengaja tidak disentuh.** Teks yang sudah mencampur huruf besar dan kecil (`iPhone`, `dArkSide`), kata yang mengandung angka (`MP3`, `1985`), dan aksara yang tidak punya huruf besar-kecil (`夜明けのうた`) dilewati. Perapih yang menimpa kesengajaan lebih buruk daripada tidak ada perapih.
- **Satu kata bukan teriakan.** `AC/DC`, `LILITH`, `deadmau5` — satu kata dalam satu jenis huruf sama seringnya merupakan cara menulis nama. Usulan huruf besar-kecil hanya untuk frasa, minimal dua kata. Ini ditemukan dengan melihat daftarnya sendiri: `AC/DC → Ac/Dc` muncul di layar, dan itu jelas salah.
- **Kata yang menonjol tetap menonjol.** Di teks yang tidak semuanya huruf besar, kata yang seluruhnya huruf besar (`DJ`, `MINA`) dibiarkan — ia ditulis begitu supaya menonjol. Tapi di teks yang *semuanya* huruf besar, tidak ada yang menonjol, jadi aturan itu tidak berlaku; kalau tidak, `THE END OF THE LINE` hanya akan separuh dirapikan.
- **Penulisan nama ditentukan oleh library, bukan oleh Onsa.** Ejaan dikelompokkan dengan mengabaikan huruf besar-kecil, spasi, dan tanda baca; yang menang adalah ejaan yang dipakai paling banyak lagu — dan selalu ejaan yang memang sudah ada, tidak pernah dikarang. **Seri berarti tidak ada usulan**: kalau library sendiri belum memutuskan, Onsa apalagi.
- **Penulisan nama dihitung dari seluruh library**, bukan dari folder cakupan saja: sepuluh lagu tidak bisa menentukan ejaan mana yang lazim. Yang diusulkan berubah tetap hanya lagu di dalam folder.
- **Nama yang menang tidak diperbaiki huruf besar-kecilnya lagi.** Kalau library-nya menulis `LILITH` di mana-mana, itu ejaannya. Mengoreksinya sesudah itu berarti Onsa berdebat dengan library.
- **Usulan huruf besar-kecil tidak tercentang otomatis**, termasuk kalau di baris yang sama ada perbaikan lain — nilai yang diterapkan adalah hasil rapi seutuhnya, huruf besar-kecil termasuk. Sisanya (sisa unduhan, penulisan `feat.`, penulisan nama, artis album kosong) tercentang.
- **Artis album kosong diisi dengan artis utama** (bagian sebelum `feat.`), supaya satu album tidak terpecah menjadi empat album satu lagu.
- **Memisahkan artis kolaborasi menjadi beberapa artis belum dikerjakan.** Itu butuh library yang tahu satu lagu bisa punya beberapa artis, bukan satu teks — perubahan skema tersendiri. Yang dikerjakan sekarang: penulisan kreditnya diseragamkan dan album difilekan di bawah artis utamanya.

## 2026-09-18 · Skema tema diperdalam, dan tiga tema terang

- **Kosakatanya yang diperdalam, bukan temanya yang di-hardcode.** Supaya sebuah tema bisa tampak dicetak-timbul, berbaris, atau berbilah judul, yang ditambah adalah kosakata skema (§9.3) — bukan kode yang tahu nama tema tertentu. Aturan yang dipegang: kalau sebuah tema butuh sesuatu yang belum ada di skema, skemanya yang diperdalam.
- **Warna tepi (`color.edge`) adalah kuncinya.** Satu sisi kena cahaya, satu sisi tidak. Dengan dua token itu, tombol, panel, scrollbar, tooltip, dan dialog bisa tampak moulded tanpa satu pun aturan CSS yang menyebut nama tema.
- **Semua bawaan baru harus persis seperti sebelumnya.** Tiap field yang ditambahkan default-nya sama dengan tampilan Onsa sebelum field itu ada, dan ada tesnya — kalau tidak, setiap tema yang sudah ditulis orang akan berubah sendiri begitu Onsa diperbarui. `rows` default-nya `plain` justru karena daftar lagu Onsa memang tidak bergaris.
- **Tema boleh mengatur tempo, tidak boleh mengatur perangai.** `motion.fast` dan `motion.slow` dijepit ke rentang yang tetap cepat, dan keempat kurva yang tersedia berhenti tanpa memantul (SPEC §9.1). Ada tes yang membuktikan titik kontrol tiap kurva tidak melewati 0–1.
- **Onsa menggambar tooltip-nya sendiri** untuk catatan pendek di atas instrumen, karena tooltip milik sistem tidak bisa ditema. Tooltip sistem tetap dipakai untuk path panjang: menunggu sebentar lalu membungkus teks adalah yang orang harapkan dari nama berkas.
- **Garis-belang mengikuti nomor baris, bukan urutan baris yang sedang digambar.** Daftar lagu hanya menggambar yang terlihat; belang yang ikut bergeser saat digulir lebih buruk daripada tidak ada belang.
- **Bilah judul dialog memakai warna `adjustable`, tulisannya memakai warna `surface.body`.** Keduanya dipunyai setiap tema dan jaraknya sejauh warna tema itu sendiri, jadi namanya terbaca entah temanya terang atau gelap. Diperiksa di keenam tema.

### Tiga tema terang

- **Keputusan pemilik proyek (2026-09-18)**: tema terang boleh, dan ikut dibundel. SPEC §1 diperbarui menjadi enam tema bawaan (tiga gelap, tiga terang); butir "tema terang dan tema tambahan" dihapus dari §16. Onsa tetap membuka dengan tema gelap.
- **Terinspirasi, bukan meniru.** Tidak ada logo, ikon, wallpaper, atau aset asli dari mana pun. Yang diambil adalah *watak*: kantor awal 2000-an yang bersudut siku dan bertepi timbul, plastik bening milenium, dan pagi bersalju di utara. Nama temanya pun tidak menyebut produk atau negara mana pun.
- **Tidak ada font baru.** Ketiganya memakai font yang sudah dibundel (Barlow, Barlow Condensed, B612, B612 Mono, Chakra Petch, Share Tech Mono), dipilih ulang untuk masing-masing watak. Menambah font berarti mengunduh berkas baru dan menambah lisensi baru ke repo; memilih ulang yang sudah ada memberi tiga suara yang berbeda tanpa keduanya.
- **Tema terang tidak butuh CSS baru.** Seluruh antarmuka sudah memakai token, jadi yang berubah hanya nilainya. Yang diperiksa: panel benar-benar terang, teks benar-benar gelap (ada tesnya), dan pemeriksa tata letak dijalankan di tema terang berscrollbar lebar.

## 2026-09-18 · Urutan milestone diubah pemilik proyek

- **Keputusan**: sesudah M7a, yang dikerjakan adalah **pengambil binary eksternal dari M10** (unduh, verifikasi checksum, bongkar arsip) dengan **yt-dlp sebagai konsumen pertama**, lalu **integrasi yt-dlp** itu sendiri, baru **M8 (lirik)**. **M9 (scrobble) dilewati**, tanpa tanggal.
- **Alasan pemilik proyek**: empat hal yang menentukan rilis pertama adalah tema, tata letak, lirik, dan yt-dlp. Dua yang pertama sudah selesai.
- Ini menyimpang dari `CLAUDE.md` ("kerjakan milestone secara berurutan"). Dicatat di sini supaya jejaknya jujur, bukan supaya aturannya dianggap tidak ada.
- **M7 ditutup sebagai M7a**; sisa §8 (hapus/ekspor sampul, menanam sampul ke berkas, aturan 1200 px, "(beragam)", tiga field yang belum bisa di-override) menjadi **M7b**, dikerjakan setelah M10 penuh. **Editor tag satuan pindah ke M8**, karena lirik adalah salah satu field yang diedit di sana dan membangun editornya dua kali tidak masuk akal.
- **Deno dan ffmpeg menunggu M10 penuh.** Pengambil binary yang dikerjakan sekarang mengambil **berkas tunggal** saja — yt-dlp memang berkas tunggal dengan `SHA2-256SUMS` di rilisnya — jadi tidak ada dependensi baru yang ditambahkan. Keputusan soal pembongkar zip/tar.xz diambil di M10 penuh, bukan sekarang.
- **Job Object di Windows ikut dalam integrasi yt-dlp**, tidak ditunda: tanpa itu kriteria selesai M10 ("tombol batal menghentikan semua proses turunan") tidak terpenuhi. Di Linux process group sudah dipasang sejak pengelola program luar dibuat.
- **Halaman Unduhan dibuat sekarang**, dan daftar persetujuan §7.1 tinggal di sana sejak awal — tidak menumpang di Pengaturan, supaya tidak dipindahkan dua kali.

## 2026-09-18 · Utang yang sengaja dibiarkan, supaya tidak terlupa

- **Prioritas format unduhan tetap `bestaudio[ext=m4a]/bestaudio`** sesuai SPEC §7.2, dan **akan diubah menjadi `bestaudio` di M11**, setelah decoder Opus ada. Sampai saat itu Onsa mengunduh m4a dengan sengaja, bukan karena lupa. Kriteria selesai M11 ("file Opus hasil yt-dlp bisa diputar gapless") justru baru bisa diuji setelah pengunduh ada, jadi urutan yang diubah ini membantu yang satu itu.
- **Nomor migrasi mengikuti urutan pengerjaan, bukan nomor milestone.** Antrean unduhan mendapat versi berikutnya yang kosong, dan lirik mendapat versi sesudahnya — jadi nomor migrasi tidak lagi sejajar dengan nomor milestone. Yang menentukan tetap satu hal: migrasi yang sudah diterapkan tidak pernah diubah.
- **M9 dilewati, dan tidak ada pola penyimpanan rahasia kedua yang dibangun sekarang.** Keyring menunggu M9; `keys.rs` tetap satu-satunya jalan (Pengaturan → env aplikasi → env build). Menyatukannya dengan keyring adalah urusan M9.

## 2026-09-18 · Mengunduh: apa yang dipegang di mana

- **Yang memasang tidak pernah bicara ke jaringan.** `onsa-downloader::install` menyebutkan sumber resmi, membaca daftar checksum, dan menaruh berkasnya; `src-tauri` yang mengambil, dengan klien HTTP yang sama dengan seluruh proyek (SPEC §1). Pembagian yang sama dengan sampul: yang mengambil ada di satu tempat, yang tahu untuk apa ada di tempat lain.
- **Sumber rilis ditulis di kode, bukan diambil dari yang diunduh.** URL, nama berkas yang dicari di daftar checksum, dan perkiraan ukurannya semuanya tetap. Tidak ada yang bisa mengarahkan Onsa ke berkas lain.
- **Checksum diambil lebih dulu, baru berkasnya.** Mengunduh 18 MB untuk kemudian menemukan tidak ada yang bisa dipakai membandingkan adalah unduhan yang terbuang.
- **Yang tidak cocok tidak ditulis ke mana pun**, bahkan tidak untuk dilihat. Binary yang tidak bisa dijamin Onsa bukan binary yang pantas disimpan.
- **Argumen yt-dlp disusun di satu tempat** (`ytdlp::arguments`), supaya bisa dibaca sekali pandang — dan supaya prioritas format yang harus berubah di M11 adalah satu baris, bukan pencarian.
- **Satu-satunya sumber nama berkas hasil adalah `--print after_move:filepath`.** Menebaknya dari pola keluaran berarti menebak apa yang dilakukan yt-dlp terhadap nama yang mengandung karakter aneh.
- **Baris yang tidak dikenali masuk log dan tidak menggagalkan apa pun** (SPEC §7.2). yt-dlp berbicara banyak; hanya dua bentuk baris yang berarti bagi Onsa.
- **Penghentian memakai Job Object di Windows dan process group di Linux**, dan keduanya dibuat sejak proses dijalankan, bukan dicari saat hendak dihentikan. Job Object-nya diberi `KILL_ON_JOB_CLOSE`, jadi unduhan tidak hidup lebih lama daripada Onsa yang memulainya. Tidak ada dependensi baru: `windows-sys` dan `libc` sudah ada di pohon dependensi lewat Tauri dan cpal.
- **Hasil unduhan ditaruh di dalam folder library**, di subfolder `Unduhan`, supaya ia muncul di library dengan sendirinya. Kalau belum ada folder library sama sekali, Onsa menolak mengunduh alih-alih mengarang tempat: berkas yang tidak diminta, di tempat yang tidak dipilih, lebih buruk daripada penolakan yang jelas.
- **Satu unduhan pada satu waktu untuk sekarang.** Antrean paralel dan antrean yang disimpan di database adalah M10b.

## 2026-09-18 · Lirik: apa yang diingat, dan siapa yang memegang apa

- **Yang dekat dibaca ulang, yang jauh diingat.** `.lrc` di sebelah lagu dan lirik di dalam tag dibaca tiap kali lagu berganti; hanya jawaban dari internet yang masuk cache. Membaca berkas teks kecil lebih murah daripada memutuskan apakah yang diingat tentangnya masih benar.
- **"Tidak ada liriknya" juga diingat**, supaya lagu yang memang tak berlirik tidak ditanyakan tiap kali diputar. Tapi **layanan yang tidak bisa dihubungi bukan jawaban**: tidak ada yang diingat, dan lagunya ditanyakan lagi lain kali. Itu sebabnya `net::ask_json` kini mengembalikan status bersama isinya — menelan status membuat layanan yang tumbang tak bisa dibedakan dari layanan yang berkata tidak tahu.
- **Cache dan offset dalam satu baris, tapi umurnya berbeda.** Offset yang disetel dengan telinga milik lagunya, bukan milik jawabannya: mengganti atau melupakan lirik tidak menghapusnya.
- **`[offset:]` milik berkas diterapkan saat parsing**, jadi yang tersisa hanya satu offset — milik pendengar, dengan satu arah: positif berarti lirik datang lebih lambat. Dua offset dengan tanda berlawanan adalah jebakan bagi siapa pun yang membaca kodenya nanti.
- **Lirik adalah field tag biasa** (`overrides`), bukan penyimpanan tersendiri. Dengan begitu menyuntingnya lewat jalan yang sama dengan judul: disimpan di Onsa dulu, masuk riwayat yang sama, bisa dibatalkan, dan "tulis ke berkas" tetap tombol terpisah.
- **Dua nama untuk satu field.** ID3 menyimpan lirik di `USLT` (lofty menyebutnya `UnsyncLyrics`); Vorbis dan MP4 memakai nama biasa. Keduanya dibersihkan lalu yang diterima tag itulah yang ditulis, jadi tidak ada berkas yang menyimpan lirik baru di satu tempat dan lirik lama di tempat lain. Format yang tidak bisa menampung keduanya (RIFF INFO di WAV) membuat penulisan **ditolak** — pemeriksaan baca-ulang yang sudah ada menangkapnya, dan berkas aslinya utuh.
- **Editor tag satuan dijepit ke folder lagunya sendiri**, dengan batas satu lagu. Editor yang terbuka atas satu lagu tidak punya cara menyentuh yang lain, bahkan karena salah ketik.
- **Parser LRC memaafkan, bukan menolak.** Berkas lirik ditulis tangan dan oleh seratus program berbeda. Yang tidak terbaca tetap menjadi baris teks; tidak ada lagu yang kehilangan liriknya karena satu tanda kurung.
- **Batas ukuran `.lrc` 512 KiB.** Lirik lagu terpanjang beberapa kilobyte; yang seratus kali lipat itu berkas lain yang kebetulan berakhiran `.lrc`.

## 2026-09-19 · Penyiapan v1: janji yang ditepati atau dicabut

Sisiran seluruh antarmuka sebelum rilis, mencari hal yang mengarah ke fitur v1.1. Keputusan pemilik proyek atas temuannya:

- **Unduhan jalan tanpa ffmpeg.** `-x`, `--embed-metadata`, dan `--embed-thumbnail` semuanya pekerjaan ffmpeg, dan mengirimnya ke mesin yang tidak punya ffmpeg **menggagalkan seluruh unduhan** — termasuk format "Asli" yang tidak mengonversi apa pun. Sekarang ketiganya hanya dikirim bila ada ffmpeg. Tanpa ffmpeg audionya diambil apa adanya: tanpa tag tertanam, tanpa sampul, tanpa konversi — dan halamannya mengatakan itu di tempat pilihannya dibuat.
- **Yang ditanya adalah PATH yang akan dilihat yt-dlp, bukan pencarian Onsa** (`Programs::reachable`). yt-dlp mencari PATH atas namanya sendiri, jadi mematikan "pakai program sistem" tidak membuat ffmpeg hilang dari pandangannya. Menanyakan pertanyaan Onsa di tempat pertanyaan yt-dlp yang berlaku adalah cara sebuah unduhan gagal dengan alasan yang tidak bisa dijelaskan apa pun di jendela. Diuji dengan menjalankan aplikasi ber-PATH bersih; itu satu-satunya bukti yang berlaku.
- **MP3 dan FLAC dinonaktifkan, bukan disembunyikan.** Orang perlu tahu fitur itu ada dan apa syaratnya.
- **Prioritas format tidak lagi punya penadah `bestaudio`.** Yang diminta hanya format yang bisa diputar Onsa (`m4a`, `mp3`, `aac`). Dengan penadah, sumber yang hanya punya Opus akan mengirim berkas yang **mendarat di disk lalu tidak pernah muncul di library** — gagal tanpa sepatah kata. Tanpa penadah, yt-dlp menolak di muka dan Onsa mengatakannya. Opus menyusul di v1.1 bersama decoder-nya (M11).
- **ffprobe tidak lagi jadi baris sendiri** di halaman Unduhan: ia datang bersama ffmpeg, dari folder yang sama. Dua baris untuk satu hal adalah dua pekerjaan yang sebenarnya satu.
- **Deno dihapus dari daftar v1.** Tidak ada pengambilnya, perilakunya belum diuji, dan `--js-runtimes` tidak lagi dikirim — yt-dlp mencari sendiri kalau memang ada. Menampilkan program yang perilakunya belum dipahami lebih buruk daripada tidak menampilkannya.
- **Alasan gagal dibawa sampai ke jendela.** Dua sebab yang bisa ditindaklanjuti pendengar diterjemahkan (tidak ada ffmpeg; tidak ada format yang bisa diputar); sisanya menampilkan kalimat yt-dlp sendiri. Onsa sudah memegang informasinya — membuangnya dan menulis "gagal" adalah kerugian gratis.
- **Aplikasi tidak menunjuk berkas repo.** Catatan di editor tag tidak lagi menyebut `docs/PROGRESS.md`.
- **Versi dinaikkan ke 1.0.0.**
- **Menghapus folder library tetap v1.1**, dicatat di README sebagai batasan yang diketahui. Menambah fitur di minggu rilis lebih berisiko daripada menyebutnya dengan jujur.

## 2026-09-19 · v1.0.1: apa yang dikatakan Onsa saat gagal

- **Kegagalan membuka database tidak boleh mematikan proses.** `setup()` Tauri yang mengembalikan `Err` menghentikan aplikasi **sebelum jendela pertama dibuat**, jadi satu-satunya gejala adalah ikon yang diklik dan tidak terjadi apa-apa. Sekarang kegagalannya ditangkap, disimpan sebagai state (`startup::Startup`), jendelanya tetap dibuka, dan halaman pertama menanyakannya sebelum yang lain. Aplikasi yang tidak bisa bekerja tetap harus bisa menjelaskan kenapa.
- **Onsa tidak menyentuh database yang rusak.** Tidak menghapus, tidak mengganti nama, tidak "memperbaiki". Itu satu-satunya salinan riwayat dengar, playlist, dan suntingan tag seseorang, dan alat pemulihan SQLite yang baik ada di luar Onsa. Jendelanya menunjukkan letaknya, membuka foldernya bila diminta, dan mengatakan dengan jelas bahwa tidak ada yang diubah.
- **Dua jenis kegagalan, dua kalimat.** Database dari Onsa yang lebih baru (`Error::UnsupportedSchema`, atau "schema version" di rantai sebabnya) bukan database rusak: yang perlu diperbarui adalah aplikasinya, dan datanya baik-baik saja. Menyatukan keduanya di bawah satu kata "rusak" akan membuat orang membuang data yang sebenarnya utuh.
- **Penandaan "hilang" hanya bila berkasnya memang tidak ada.** Pemutaran bisa gagal karena berkasnya cacat, karena izinnya, atau karena formatnya; hanya `!path.exists()` yang berarti hilang. Menandai baris karena sebab lain membuat tanda itu tidak bisa dipercaya.
- **Pesan URL tinggal di bagian URL.** Tempat pesan bersama di halaman Unduhan dibersihkan tiap kali status program dibaca ulang beberapa detik sekali, jadi kalimat yang ditaruh di sana berkedip lalu hilang sebelum sempat dibaca. Pesan milik satu bagian disimpan oleh bagian itu.
- **`notAUrl` dan `urlRefused` dipisah.** Teks yang bukan alamat sama sekali adalah salah ketik yang bisa diperbaiki pengetiknya; alamat sungguhan yang tidak bisa dibaca yt-dlp adalah hal lain. Satu kalimat untuk keduanya membuat yang pertama terdengar seperti kesalahan Onsa dan yang kedua seperti kesalahan pengguna.
- **Keluhan tema dibawa bersama temanya**, bukan dikirim lewat jalur lain: `read_user_dir` mengembalikan `(Vec<Theme>, Vec<Trouble>)`. Yang memuat tema dan yang tahu tema mana yang gagal adalah satu pembacaan folder yang sama; memisahkannya berarti membacanya dua kali dan bisa berbeda.
- **Tombol salin tema menyalin tema yang sedang dipakai**, bukan membuat tema kosong. Contoh yang bisa diubah sedikit demi sedikit lebih berguna daripada berkas kosong yang skemanya harus dicari sendiri. Nama berkasnya dibersihkan (`safe_stem`) dan **tidak pernah menimpa**: yang sudah ada mendapat akhiran angka.
- **Batasan ditulis di tempat batasan itu ditemui.** Kalimat "folder belum bisa dihapus" ada di halaman Library, bukan hanya di README, karena di sanalah orang mencarinya.

## 2026-09-20 · v1.1: bagaimana Onsa menjelaskan dirinya

- **Bantuan menempel pada halaman, bukan satu dokumen panjang.** Yang dibaca orang adalah halaman yang sedang membingungkannya, bukan daftar isi. Karena itu tombol tanda tanya ada di sudut yang sama di tiap halaman dan panelnya hanya berisi halaman itu. Halaman Bantuan tetap ada untuk hal yang tidak menempel di satu halaman mana pun.
- **Nama kontrol di bantuan adalah kunci kamus milik kontrol itu sendiri.** Menuliskannya ulang berarti dua tempat yang harus diubah bersama dan, cepat atau lambat, bantuan yang menyebut kata yang tidak ada di layar. Dengan memakai kunci yang sama, bantuannya ikut berganti bahasa dan ikut berganti kata setiap kali kontrolnya berganti nama.
- **Panel bantuan tidak pernah menghalangi halaman.** Tidak ada scrim, tidak ada jebakan fokus, dan di jendela sempit pun halamannya tetap menerima klik. Bantuan yang menghentikan pekerjaan bukan bantuan; dan yang membaca bantuan biasanya sedang ingin mencoba hal yang dijelaskannya.
- **Kontrol yang namanya data diwakili satu entri umum.** "Baris lagu", "Kartu album", "Baris artis": judul lagu seseorang tidak mungkin ada di kamus. Aturan perwakilan itu ditulis di pemeriksanya dan ikut dicetak per kontrol, supaya bisa dinilai dari luar, bukan disembunyikan di dalam alat ujinya.
- **Contoh berkas tema tidak masuk kamus i18n.** Ia isi berkas, bukan kalimat, dan sama persis di kedua bahasa. Menaruhnya di kamus berarti dua salinan yang harus dijaga sama.
- **Kalimat asli yt-dlp disimpan, tidak dibuang.** Kata yang diterjemahkan untuk yang memakai; kalimat aslinya untuk yang memperbaiki — dan keduanya orang yang sama ketika ia diminta melaporkan masalah. Karena itu pemeriksaan URL tidak lagi memakai `ErrorCode` biasa: ia mengembalikan bentuknya sendiri (`code` + `said`), karena dua hal harus sampai ke jendela, bukan satu.
- **Sebab gagal dipisah sejauh yang bisa ditindaklanjuti, tidak lebih.** 404 dan 500 dua kalimat berbeda karena yang bisa dilakukan berbeda: satu berarti alamatnya salah, satu berarti menunggu. Sebaliknya "tidak bisa dihubungi" cukup satu kalimat untuk semua bentuk kegagalan sambungan.
- **Baris ffmpeg menyebut siapa yang memakainya.** Yang membingungkan bukan jawabannya melainkan pertanyaannya: saklar itu mengatur program yang dijalankan **Onsa**, sedangkan yt-dlp mencari ffmpeg atas namanya sendiri. Sekali keduanya disebut di baris yang sama, tidak ada lagi yang terbaca seperti saklar rusak.
- **Folder yang hilang ditanyakan ke disk, bukan ditunggu kabarnya.** Pengintai folder tidak mengabarkan apa-apa ketika drive-nya dicabut — tidak ada lagi yang bisa diintai. Menanyakan keberadaan beberapa jalur tiap empat detik, hanya selama halamannya terbuka, adalah harga yang jauh lebih murah daripada tidak bisa mengatakannya sama sekali.
- **Folder yang hilang tidak menghapus apa pun.** Tidak dihapus dari daftar, dan lagunya tidak ditandai hilang hanya karena foldernya tidak terlihat: drive yang dicabut satu menit bukan alasan untuk mengubah catatan tentang ratusan lagu. Yang ditandai hilang tetap yang benar-benar gagal dibuka atau yang tidak ditemukan saat scan ulang.

## 2026-09-21 · v1.2: membuktikan yang selama ini hanya ditiru

- **Layanan tiruan tetap untuk CI, layanan sungguhan sekali sebelum diserahkan.** CI tidak boleh bergantung pada internet, jadi tiruan di 127.0.0.1 tidak diganti. Tapi tiruan hanya menjawab apa yang sudah diduga; yang tidak pernah diduga — status HTTP yang tidak dibaca, kalimat yang salah untuk layanan tumbang — baru muncul ketika layanan aslinya yang menjawab.
- **Sumber uji unduhan dipilih menurut asal-usul lisensinya, bukan menurut mudahnya.** Yang dipakai adalah rilis netlabel yang labelnya sendiri menerbitkannya di bawah Creative Commons, bukan album orang yang diunggah ulang seseorang sambil mengaku CC0. Klaim lisensi dari pihak ketiga atas rilis orang lain tidak dipercaya.
- **Status HTTP dibaca sebelum isinya, di mana pun jawaban layanan dinilai.** "Coba kunci" dulu hanya membaca badan jawaban dan menyimpulkan kunci lolos ketika keluhannya bukan soal kunci; layanan yang tumbang kebetulan juga begitu. Pola yang sama sudah pernah salah di M8 (`ask_json` menelan status 503); kali ini di tempat lain. Aturannya sekarang: jawaban bukan jawaban sampai statusnya diperiksa.
- **Kegagalan yang tidak bisa dijelaskan tidak pernah berhenti di log.** Tiga tempat kini seragam: sebab dalam kalimat manusia, kalimat asli di balik "Lihat detail". Yang membedakan bukan jenis kegagalannya melainkan siapa yang membacanya — orang yang sama yang nanti diminta melaporkan.
- **Sebab kegagalan disimpan terpisah dari hal yang gagal.** `progress` dihapus begitu pengambilan berhenti, jadi alasan yang menumpang di sana ikut hilang sebelum sempat dibaca. Alasan hidup lebih lama daripada prosesnya.

## 2026-09-21 · Kunci dan siapa yang membangun apa

- **Build lokal boleh membawa kunci; yang diserahkan ke orang lain hanya dari CI.** `option_env!` tetap seperti semula, karena memang berguna: salinan yang dibangun pemilik proyek untuk dirinya sendiri langsung punya kuncinya. Yang berubah adalah dari mana berkas pemasang berasal — dari CI, yang tidak punya kunci siapa pun.
- **Buktinya build itu sendiri, bukan janji tentang lingkungannya.** `ONSA_KEYLESS` menyalakan penjaga di `keys.rs` yang membuat kompilasi **gagal** kalau ada kunci yang akan ikut tertanam. Dengan begitu, berkas pemasang yang ada berarti berkas pemasang tanpa kunci — bukan sekadar berkas yang kebetulan dibangun di mesin yang variabelnya tidak disetel. Diuji dua arah: dengan kunci di lingkungan, `cargo build` berhenti dengan pesan; tanpa kunci, build jalan seperti biasa.
- **Dua lapis, karena satu lapis hanya berlaku saat kompilasi.** Penjaga itu dibaca compiler; satu tes biasa mengatakan hal yang sama saat dijalankan, dan workflow-nya memeriksa lingkungannya sendiri sebelum mulai. Ketiganya memeriksa hal yang sama dari sudut berbeda.
- **Berkas pemasang diunggah oleh CI, bukan oleh siapa pun.** Yang sampai ke tester adalah berkas yang dibangun workflow itu, tidak melewati mesin siapa pun di tengah jalan.
- **Tidak ada penandatanganan kode.** Sertifikatnya berbayar dan belum dibeli; yang dilakukan adalah menjelaskan peringatan Windows itu apa adanya kepada tester, bukan menyembunyikannya.

## 2026-09-25 · v1.3-a: memilih fpcalc

- **Berkas yang ditunjuk dijalankan dulu, baru dipercaya.** Jendela pemilih berkas hanya bisa menyaring menurut ekstensi, saringannya bisa diganti di jendela itu juga, dan di Linux sebuah program tidak punya ekstensi sama sekali. Jadi yang menjadi penjaga sebenarnya bukan saringan itu melainkan pertanyaan langsung kepada berkasnya: ia dijalankan sekali dengan tanda versinya, dan harus menjawab sebagai dirinya sendiri. Berlaku sama di Windows dan di Linux.
- **Nama program harus mengawali baris versinya, bukan sekadar ada di dalamnya.** fpcalc menyebut ffmpeg di baris versinya sendiri (`fpcalc version 1.5.1 (FFmpeg ...)`), jadi pemeriksaan yang hanya mencari kata akan menerima fpcalc sebagai ffmpeg. Yang diperiksa adalah pembuka baris pertama.
- **yt-dlp tidak menyebut namanya, dan itu ditulis apa adanya.** Jawabannya hanya tanggal versi, jadi untuk yt-dlp yang bisa diminta cuma "berjalan dan menjawab". Memaksakan pemeriksaan nama untuk semua program berarti menolak yt-dlp yang sungguhan.
- **Pilihan yang ditolak tidak disimpan.** Pilihan yang tidak bisa dipakai lebih buruk daripada tidak ada pilihan, karena ia menutupi salinan yang tadinya akan ditemukan sendiri oleh Onsa.
- **Tiga sebab kegagalan yang selama ini satu.** "fpcalc belum ada", "ada berkasnya tapi bukan fpcalc", dan "berkas lagu ini yang gagal dibaca" adalah tiga hal berbeda dengan tiga tindakan berbeda. Sebelumnya semua kegagalan menjalankan fpcalc jatuh ke kalimat tentang berkas lagunya — yang mengirim orang memeriksa berkas yang tidak apa-apa.
- **Hanya fpcalc yang sudah terbukti fpcalc yang boleh menyalahkan sebuah lagu.** Karena itu fpcalc ditanya lebih dulu — sekali untuk seluruh putaran, bukan sekali per lagu — dan kalau yang menjawab bukan fpcalc, semua lagu dalam putaran itu mendapat kalimat tentang fpcalc dan tidak ada satu pun yang disalahkan.
- **Halaman Metadata memisahkan "ada berkasnya" dari "berkasnya benar".** Sebelumnya status hanya menanyakan apakah berkasnya ada, sehingga berkas yang salah — termasuk pilihan lama dari sebelum pemeriksaan ini ada — tetap terbaca "terpasang". Sekarang berkas yang ditemukan ikut ditanya siapa dirinya, dengan ongkos yang sama seperti menanyakan versinya seperti dulu.
- **Nama tombol menyebut yang dicari.** "Pilih berkasnya" tidak mengatakan berkas apa; "Cari fpcalc…" mengatakan. Di bawahnya ada letak yang biasa di Windows dan di Linux, supaya orang tidak menebak-nebak lalu menunjuk berkas yang salah.
