# Onsa: Spesifikasi

Onsa (音叉, garpu tala) adalah music player desktop untuk Windows dan Linux. Tampilannya simpel dan mudah dipakai, sistemnya efisien, dan hampir semua aspek bisa dikustomisasi: tampilan maupun kualitas/spek pemutaran. Identitas visualnya adalah **panel instrumen**: aplikasi tampil seperti alat ukur yang jujur dan menunjukkan apa yang sebenarnya terjadi pada sinyal audio.

Dokumen ini adalah sumber kebenaran proyek. Aturan kerja ada di `CLAUDE.md`. Referensi visual ada di `docs/design/palet-preview.html` (buka di browser).

---

## 1. Keputusan terkunci

Keputusan berikut sudah final. Jangan diubah tanpa persetujuan pemilik proyek.

| Area | Keputusan |
|---|---|
| Nama | Onsa. Nama program/paket `onsa`. Identifier aplikasi `io.github.mufuyumoku.onsa`. |
| Platform | Windows 10/11 dan Linux (x86_64). macOS tidak ditargetkan, tapi jangan sengaja memakai hal yang mustahil di-port. |
| Shell aplikasi | Tauri 2 |
| Backend | Rust stable, Cargo workspace dengan crate terpisah per modul (lihat §2) |
| Frontend | SvelteKit (Svelte 5, TypeScript strict), `adapter-static`, SPA (`ssr = false`) |
| Styling | CSS biasa + CSS variables dari sistem tema. Tanpa Tailwind dan tanpa library komponen. |
| Database | SQLite via `rusqlite` (fitur `bundled`), migrasi berversi |
| Audio | `symphonia` (decode), `rubato` (resample), `cpal` (output), `rtrb` (ring buffer lock-free), `realfft` (analisis) |
| Tag | `lofty` (baca dan tulis) |
| HTTP | `reqwest` dengan TLS bawaannya (rustls), klien blocking. Satu klien untuk seluruh proyek: M7 sampai M10 memakai yang sama. Wajib: User-Agent `Onsa/<versi> (<kontak>)`, timeout koneksi dan timeout total, batas ukuran jawaban dan unduhan, serta kegagalan jaringan yang dikembalikan sebagai nilai — tidak boleh membuat aplikasi macet atau berhenti. |
| Tema bawaan | Enam: tiga gelap (Kaca asap, Kokpit kaca, Deck malam) dan tiga terang (Kubikel biru, Kilau milenium, Musim dingin utara). Lihat §9. Onsa tetap membuka dengan tema gelap. |
| Bahasa UI | Indonesia dan Inggris, default mengikuti locale OS |

---

## 2. Arsitektur

Prinsip utamanya: **mesin audio terpisah total dari tampilan.** Mesin berjalan di thread sendiri dan tidak tahu apa-apa soal UI. UI hanyalah remote control yang bicara lewat command dan event Tauri. Seberat apa pun UI, pemutaran tidak boleh tersendat.

```
onsa/
├─ Cargo.toml                 (workspace)
├─ crates/
│  ├─ onsa-audio/             mesin audio: sumber, resample, crossfade, DSP, output, tap analisis
│  ├─ onsa-library/           SQLite, scanner, watcher, tag, cover, playlist, smart playlist, pencarian
│  ├─ onsa-downloader/        pengelola binary (yt-dlp, ffmpeg, deno) + runner unduhan
│  ├─ onsa-lyrics/            parser LRC, sumber lirik, cache
│  ├─ onsa-scrobble/          Last.fm (dan ListenBrainz nanti), antrean offline
│  └─ onsa-cli/               CLI untuk menguji mesin audio tanpa UI
├─ src-tauri/                 lem aplikasi: state, command, event, integrasi OS, tray
├─ ui/                        SvelteKit
├─ themes/                    tiga tema bawaan (JSON), dibundel ke aplikasi
└─ docs/
```

Aturan dependensi antar-crate: `onsa-audio` tidak bergantung pada crate Onsa lain. `onsa-library`, `onsa-downloader`, `onsa-lyrics`, dan `onsa-scrobble` tidak saling bergantung. Semuanya disatukan hanya di `src-tauri`. Contoh: downloader tidak tahu soal library, ia cukup mengembalikan path file hasil, lalu `src-tauri` yang menyerahkannya ke library.

Komunikasi UI dan backend:

- **Command** (UI → backend): `invoke`, contohnya `play`, `seek`, `set_eq`, `library_query`.
- **Event** (backend → UI): posisi putar (≤ 10 Hz), ganti lagu, perubahan status, progres scan/unduhan, frame analisis (spektrum/meter, ≤ 60 Hz), dan error.
- Cover art dilayani lewat custom protocol (misal `onsa://cover/<id>/<ukuran>`), bukan base64 di event.

---

## 3. Mesin audio (`onsa-audio`)

### 3.1 Model thread

```
[thread kontrol] ←perintah (channel)── src-tauri
      │
[thread decode]  sumber A/B → decode → resample → ReplayGain per sumber → mix crossfade
      │ ring buffer (rtrb, f32 interleaved)
[callback output cpal]  EQ → preamp → limiter → volume → dither (bila perlu) → perangkat
      │ tap (rtrb, bila analisis aktif)
[thread analisis]  FFT, peak/RMS, spectral centroid → event ke UI
```

- Format internal `f32` interleaved.
- **Callback output wajib real-time safe**: tanpa alokasi, tanpa lock, tanpa I/O, tanpa logging. Parameter DSP diterima lewat atomics atau `rtrb`, lalu diterapkan dengan smoothing (ramp) supaya tidak ada zipper noise saat slider digeser.
- Tahap yang peka latensi (EQ, volume) ada di callback agar perubahan langsung terdengar. Tahap yang terikat per lagu (ReplayGain, crossfade) ada di thread decode.
- Mesin punya **sink offline** yang me-render ke buffer/WAV alih-alih ke perangkat. Sink ini dipakai untuk semua tes otomatis (§12).

### 3.2 Format

- Lewat symphonia: FLAC, MP3, AAC (M4A/MP4), ALAC, OGG Vorbis, WAV/AIFF.
- **Opus**: symphonia belum punya decoder Opus bawaan. Tambahkan decoder Opus (binding libopus, misalnya crate `opus`/`audiopus`) sebagai `Decoder` kustom yang disambungkan ke demuxer OGG/WebM symphonia. Ini dikerjakan di M11. Sampai saat itu, downloader memprioritaskan M4A (§7).
- File yang gagal di-decode ditandai di library, tidak membuat aplikasi crash.

### 3.3 Antrean, gapless, crossfade

- Mesin memegang antrean yang dikirim dari `src-tauri`. Lagu berikutnya dibuka lebih awal (pre-roll) supaya perpindahan mulus.
- **Gapless**: aktifkan `enable_gapless` di opsi format symphonia agar encoder delay/padding (LAME, iTunSMPB) dihormati. Tidak boleh ada jeda atau klik di batas lagu.
- **Crossfade**: mesin mampu memutar dua sumber bersamaan dan mencampurnya.
  - Durasi 0–12 detik, dengan 0 berarti mati. Kurva *equal-power* (default) atau linear.
  - Otomatis dimatikan untuk lagu berurutan dari album yang sama (album sama dan nomor trek berurutan). Perilaku ini bisa diubah pengguna.
  - Durasi crossfade untuk skip manual diatur terpisah (default 0,3 detik).
- **Micro-fade** 50–100 ms saat pause, resume, seek, dan stop supaya tidak ada klik.
- Seek akurat (sample-accurate bila format mendukung).

### 3.4 Output dan kualitas

| Pengaturan | Nilai |
|---|---|
| Perangkat output | Daftar perangkat dari cpal. Default sistem ikut berpindah otomatis. |
| Sample rate output | Ikuti perangkat (default) / samakan dengan sumber bila perangkat mendukung / nilai tetap |
| Kualitas resampler | Cepat / Seimbang (default) / Terbaik (preset parameter sinc rubato). Bila rasio tetap, boleh memakai resampler FFT. |
| Ukuran buffer | Rendah / Normal / Besar (latensi vs ketahanan) |
| Dither | TPDF otomatis saat output berformat integer 16-bit |
| Mode hemat daya | Buffer besar, resampler Cepat, frame analisis diturunkan atau dimatikan, event posisi lebih jarang |
| Exclusive mode (Windows) | M11, lewat WASAPI langsung karena cpal tidak menyediakannya |

- Bila perangkat dicabut atau berubah (misalnya headphone), stream dibangun ulang ke perangkat default tanpa crash, dan posisi putar dipertahankan.
- Linux: cpal memakai ALSA. Pastikan jalan di sistem PipeWire/PulseAudio lewat lapisan kompatibilitas ALSA.

### 3.5 Sleep timer

Pilihannya: setelah N menit, di akhir lagu ini, atau setelah N lagu. Sebelum berhenti, volume diturunkan perlahan (durasi fade bisa diatur). Aksi akhirnya bisa pause, stop, atau tutup aplikasi.

---

## 4. Rantai DSP

Rantai DSP modular. Setiap tahap bisa dinyalakan dan dimatikan. Tahap di sisi callback bisa diurutkan ulang, kecuali volume dan dither yang selalu paling akhir.

Urutan default: `ReplayGain → EQ → preamp → limiter → volume → dither`

### 4.1 ReplayGain

- Mode: mati / track / album / otomatis (album bila antrean berisi satu album berurutan, selain itu track).
- Membaca tag `REPLAYGAIN_*` (ID3 TXXX, Vorbis comment, atom MP4). Ada preamp ReplayGain dan nilai cadangan untuk lagu tanpa tag.
- Pencegahan clipping memakai nilai peak dari tag.
- (Nanti, bukan milestone wajib) analisis loudness EBU R128 di latar untuk lagu tanpa tag.

### 4.2 EQ

- Satu mesin EQ berupa tumpukan filter biquad (rumus RBJ Audio EQ Cookbook, state dalam f64), maksimal 16 band. Koefisien dihitung ulang saat sample rate berubah.
- Jenis filter: peaking, low shelf, high shelf, low pass, high pass, notch.
- **Mode grafis**: 10 band peaking tetap di 31, 62, 125, 250, 500, 1k, 2k, 4k, 8k, 16k Hz dengan Q ≈ 1,41.
- **Mode parametrik**: frekuensi, gain, Q, dan jenis filter bebas per band.
- **Preamp otomatis**: bila aktif, preamp = −(puncak positif kurva respons gabungan). Kurva dihitung pada grid frekuensi logaritmik.
- **Import preset** berformat Equalizer APO / AutoEQ (`ParametricEQ.txt`). Baris `Preamp: X dB` dan `Filter N: ON <PK|LSC|HSC|LS|HS|LP|HP|NO> Fc <f> Hz Gain <g> dB Q <q>`. Baris yang tidak dikenal dilewati dengan peringatan, tidak membuat import gagal total.
- Preset tersimpan (bawaan dan buatan pengguna) bisa diekspor ke format yang sama.
- UI menampilkan kurva respons dan deretan fader (mode grafis) atau editor titik (mode parametrik).

### 4.3 Limiter

Peak limiter dengan lookahead pendek (~5 ms), ceiling −0,1 dBFS, release bisa diatur. Saat limiter bekerja, indikator clip di UI menyala.

### 4.4 Tap analisis dan meter

- Callback menyalin sampel pasca-DSP ke ring buffer tap **hanya bila analisis aktif** (flag atomik). Bila visualizer dan meter tidak tampil, tap mati sehingga tidak ada beban sama sekali.
- Thread analisis menghitung:
  - Spektrum: FFT (window Hann, ukuran 2048 default), dipetakan ke band logaritmik (default 64), dengan peak-hold.
  - Meter: peak dan RMS per kanal, beserta status clip.
  - **Spectral centroid** untuk fitur warna nada (§9.5).
- Frame dikirim ke UI dalam bentuk ringkas (misalnya `u8` per band), maksimal 60 fps. Mode hemat daya menurunkannya ke 20 fps atau mematikannya.

---

## 5. Library (`onsa-library`)

### 5.1 Skema database (arahan)

Nama tabel dan kolom di bawah adalah arahan, boleh disempurnakan selama semua kebutuhan terpenuhi. Setiap perubahan skema lewat migrasi berversi.

| Tabel | Isi |
|---|---|
| `folders` | folder musik yang dipantau |
| `tracks` | path (unik), folder, mtime, ukuran, durasi, codec, sample rate, bit depth, kanal, bitrate, tag dasar (judul, artis, album, album artist, nomor trek/disk, tahun, genre, komposer), nilai ReplayGain, `album_id`, `cover_id`, status (ok / hilang / gagal decode), waktu ditambahkan |
| `albums` | judul, album artist, tahun, cover |
| `covers` | hash isi gambar, path thumbnail di cache |
| `overrides` | editan metadata per lagu per field (§8), beserta flag "belum ditulis ke file" |
| `stats` | jumlah putar, jumlah skip, terakhir diputar, rating (0–5) |
| `plays` | riwayat putar (lagu, waktu mulai, durasi didengar) |
| `playlists`, `playlist_items` | playlist manual dan smart (§6) |
| `lyrics` | cache lirik (§10) |
| `scrobble_queue` | antrean scrobble offline (§11) |
| `downloads` | riwayat dan status unduhan (§7) |
| `settings` | pengaturan (key → JSON) |
| `tracks_fts` | tabel FTS5 untuk pencarian (judul, artis, album, album artist, genre) |

Nilai tag yang tampil = nilai `overrides` bila ada, selain itu nilai dari file.

### 5.2 Scanner dan pemantau

- Pemindaian pertama: telusuri semua folder, baca tag dengan lofty, lalu tulis ke DB dalam batch transaksi. Progres dikirim ke UI sebagai event.
- Pemindaian ulang bersifat bertahap: file yang mtime dan ukurannya tidak berubah dilewati.
- Pemantau folder memakai `notify` + debouncer. File baru, berubah, dihapus, atau dipindah langsung tercermin di library.
- File yang hilang tidak langsung dihapus dari DB. Statusnya ditandai "hilang" supaya statistik dan playlist tidak rusak bila drive sedang dilepas. Pengguna bisa membersihkannya secara manual.
- Cover diambil dari gambar tertanam, lalu dari `cover.*`, `folder.*`, atau `front.*` di folder yang sama. Thumbnail 128 px dan 512 px dibuat dan disimpan di cache. Daftar lagu tidak pernah memuat gambar resolusi penuh.
- Format yang didukung scanner mengikuti §3.2.

### 5.3 Penjelajahan dan pencarian

- Tampilan: Lagu, Album, Artis, Genre, Folder.
- Sorting per kolom. Setiap daftar panjang wajib memakai **virtual list** (hanya baris yang terlihat yang di-render).
- Pencarian cepat memakai FTS5, dengan hasil yang dikelompokkan (lagu, album, artis).

---

## 6. Antrean dan playlist

### 6.1 Antrean

- Antrean terpisah dari playlist, dan disimpan sehingga pulih saat aplikasi dibuka lagi.
- Operasi: putar sekarang, putar berikutnya, tambah ke akhir, drag untuk mengurutkan, hapus, kosongkan, dan simpan antrean sebagai playlist.
- Shuffle tidak merusak urutan: urutan asli disimpan, jadi mematikan shuffle mengembalikan urutan semula.
- Mode repeat: mati, semua, satu.

### 6.2 Playlist manual

Playlist manual punya urutan (`position`), bisa berisi lagu yang sama lebih dari sekali, dan bisa diganti nama, diduplikasi, serta dihapus.

### 6.3 Smart playlist

Aturan disimpan sebagai JSON, lalu dikompilasi menjadi **SQL berparameter**. Nilai dari pengguna tidak pernah digabung langsung ke string SQL.

```json
{
  "match": "all",
  "rules": [
    { "field": "rating", "op": ">=", "value": 4 },
    { "field": "last_played", "op": "not_in_last", "value": { "days": 30 } }
  ],
  "sort": { "field": "random" },
  "limit": 100
}
```

- Field: judul, artis, album, album artist, genre, tahun, rating, jumlah putar, terakhir diputar, waktu ditambahkan, codec, durasi, folder.
- Operator menyesuaikan tipe field: teks (`contains`, `is`, `starts_with`, dan negasinya), angka (`=`, `!=`, `<`, `<=`, `>`, `>=`, `between`), dan tanggal (`in_last`, `not_in_last`, `before`, `after`).
- Isi smart playlist dihitung ulang saat dibuka dan saat library berubah.

### 6.4 Import dan export

Mendukung M3U8 dengan `#EXTINF`, dengan pilihan path relatif atau absolut. Saat import, path dicocokkan ke library. Entri yang tidak ditemukan dilaporkan ke pengguna, bukan dibuang diam-diam.

---

## 7. Downloader (`onsa-downloader`)

Downloader punya bagian tersendiri di UI. yt-dlp dijalankan sebagai proses terpisah yang dikendalikan Onsa, bukan dilebur ke kode.

### 7.1 Pengelolaan binary

- Onsa membutuhkan tiga program untuk pengunduh: **yt-dlp**, **ffmpeg** (beserta ffprobe), dan **Deno**. Deno dibutuhkan yt-dlp untuk dukungan YouTube yang lengkap. Metadata (§8) menambah satu lagi: **fpcalc** dari Chromaprint, untuk sidik suara AcoustID.
- Pengelola program luar ini **satu untuk semuanya** (`onsa-downloader::programs`), dipakai M7 maupun M10: pencarian (pilihan pengguna → folder `bin` Onsa → PATH sistem), menjalankan, dan menanyakan versi.
- Lokasi: `<app_data>/bin/`.
- Saat bagian downloader pertama kali dibuka, tampilkan daftar yang akan diunduh (nama, sumber, perkiraan ukuran). Unduhan baru dimulai setelah pengguna menekan tombol setuju.
- Sumber hanya dari rilis resmi di GitHub:
  - yt-dlp: rilis resmi (`yt-dlp.exe` untuk Windows, `yt-dlp_linux` untuk Linux). Verifikasi dengan `SHA2-256SUMS` yang disertakan di rilis.
  - Deno: rilis resmi (`deno-x86_64-pc-windows-msvc.zip` / `deno-x86_64-unknown-linux-gnu.zip`).
  - ffmpeg: build BtbN FFmpeg-Builds (win64 / linux64).
- Opsi **"pakai program sistem"**: bila yt-dlp, ffmpeg, atau Deno sudah ada di PATH (umum di Linux), pengguna bisa memakai itu.
- Ada tombol update untuk yt-dlp (`yt-dlp -U` atau unduh ulang) dan tampilan versi setiap binary.

### 7.2 Alur

1. Pengguna menempelkan URL.
2. Onsa menjalankan `yt-dlp -J --flat-playlist <url>` lalu menampilkan pratinjau: judul, durasi, thumbnail, dan isi playlist (dengan checkbox per item).
3. Pengguna memilih format:
   - **Asli (disarankan)**: `-x` tanpa konversi, dengan prioritas `bestaudio[ext=m4a]/bestaudio` sampai decoder Opus ada (M11). Setelah itu prioritasnya `bestaudio`.
   - Konversi (MP3, FLAC, dsb.) tetap tersedia, tapi UI memberi catatan singkat bahwa konversi tidak menambah kualitas: FLAC hanya memperbesar ukuran, MP3 menurunkan kualitas.
4. Unduhan berjalan dengan argumen kira-kira seperti ini (sesuaikan bila perlu):
   ```
   --ffmpeg-location <bin> --js-runtimes deno:<bin>/deno
   --embed-metadata --embed-thumbnail
   --newline --progress-template "download:%(progress)j"
   --print after_move:filepath
   -o "<folder_output>/%(artist,uploader)s/%(title)s [%(id)s].%(ext)s"
   ```
5. Progres diparse per baris dan dikirim ke UI sebagai progress bar (persen, kecepatan, ETA). Parser harus toleran: baris yang tidak dikenali disimpan ke log unduhan, tidak membuat proses gagal.
6. Path hasil akhir diserahkan `src-tauri` ke library untuk diimpor. Lagu langsung muncul di library.

### 7.3 Aturan proses

- Proses dijalankan dengan **array argumen, tidak pernah lewat shell**, supaya URL tidak bisa menyuntikkan perintah.
- Windows: gunakan flag `CREATE_NO_WINDOW` supaya tidak ada jendela konsol yang muncul.
- Batal = hentikan seluruh pohon proses (process group di Linux, Job Object di Windows), supaya ffmpeg yang dijalankan yt-dlp ikut berhenti.
- Jumlah unduhan paralel bisa diatur (default 2), dan antrean unduhan disimpan di DB.
- Folder output default ada di dalam salah satu folder library. Bila folder output di luar library, tanyakan apakah folder itu mau ditambahkan ke library.
- UI menampilkan catatan singkat dan netral bahwa pengguna bertanggung jawab atas hak konten yang diunduh.

---

## 8. Metadata

- Editor tag mendukung semua format lewat lofty (ID3v2, Vorbis comment, atom MP4, APE).
- Field: judul, artis, album, album artist, nomor trek/total, nomor disk/total, tahun, genre, komposer, komentar, lirik, dan ReplayGain (baca saja).
- **Edit banyak lagu sekaligus**: field yang nilainya berbeda antar-lagu ditampilkan sebagai "(beragam)" dan tidak disentuh kecuali diubah.
- Cover art bisa diganti, dihapus, atau diekspor. Gambar besar diperkecil dulu (maksimal 1200 px, dengan opsi mempertahankan asli).
- **Editan disimpan di database dulu** (tabel `overrides`). "Tulis ke file" adalah aksi terpisah, per lagu atau massal. Dengan begitu, pengguna bisa punya koreksi yang hanya berlaku di Onsa tanpa mengubah file asli.
- **Penulisan ke file yang aman**: salin file ke file sementara di folder yang sama, tulis tag ke file sementara, fsync, lalu rename atomik menimpa file asli. Ada opsi menyimpan cadangan. Bila gagal, file asli tetap utuh.
- **Rename/pindah file berdasarkan pola**, misalnya `{album_artist}/{album}/{disc}-{track:02} {title}`. Selalu tampilkan pratinjau (dry-run) dan daftar bentrokan sebelum dijalankan. Karakter ilegal di Windows diganti otomatis.
- **Perapihan otomatis** dari apa yang sudah ada, tanpa jaringan: sisa nama unduhan yang menempel di tag, huruf besar-kecil, penulisan `feat.` yang diseragamkan, nama artis yang tertulis beda-beda diseragamkan ke ejaan yang paling banyak dipakai di library itu sendiri, dan artis album yang kosong diisi dari artis utama. Semuanya berupa usulan yang melewati ringkasan dan bisa dibatalkan; yang menyangkut huruf besar-kecil tidak tercentang otomatis.
- **Mencari tahu lewat internet** (akhir M7). Semuanya mati secara bawaan dan bisa dimatikan total kapan saja, termasuk menghentikan pencarian yang sedang berjalan.
  - **AcoustID lewat sidik suara** untuk lagu yang tagnya kosong, dengan `fpcalc` sebagai program luar (lihat §7.1 dan §7.3). Yang dikirim hanya ringkasan sidik suara, panjang lagu, dan API key — bukan audio.
  - **Nama file sebagai jalur cadangan**: sisa nama unduhan (`[j9RGt9Z_UeE]`, `(Official Video)`) dibersihkan lebih dulu. Bila sidik suara dan nama file memberi hasil berbeda, keduanya ditampilkan dan pengguna yang memilih.
  - **MusicBrainz** untuk lagu yang sudah bertag, dan untuk melengkapi tahun rilis. Wajib memakai User-Agent `Onsa/<versi> (<kontak>)` dan batas 1 request per detik.
  - **Cover Art Archive** untuk sampul, diambil sekali per rilis, ditampilkan sebagai gambar sebelum disetujui.
  - Hasilnya **selalu berupa usulan**. Tiap usulan menyebut tingkat keyakinannya; yang rendah tidak tercentang otomatis. Menerapkannya melewati jalur yang sama dengan editan manual: cakupan folder, ringkasan sebelum tombol, dan riwayat yang bisa dibatalkan.
  - Jaringan yang mati atau lambat tidak boleh membekukan aplikasi, dan hasil yang sudah terkumpul tidak boleh hilang.

---

## 9. Tampilan, struktur UI, dan tema

### 9.1 Prinsip identitas

- Onsa tampil seperti alat ukur yang jujur: tenang, rapi, dan menunjukkan apa yang terjadi pada sinyal. Tidak ada dekorasi tanpa fungsi.
- Kekayaan visual datang dari **material** (kaca, bezel, logam) dan **warna yang punya arti**, bukan dari banyaknya warna.
- Animasi cepat (100–150 ms) dan tanpa efek memantul. Yang bergerak terus-menerus hanya meter dan visualizer.
- Hormati `prefers-reduced-motion`: meter tetap berjalan, tapi transisi dekoratif dimatikan.
- Semua angka memakai tabular numbers agar digit tidak bergoyang.
- Teks UI pendek, teknis, dan tepat.

### 9.2 Struktur jendela

```
┌────────┬──────────────────────────────────────┬──────────────┐
│Sidebar │ Konten utama                         │ Panel kanan  │
│        │ (Library / Playlist / Downloader /   │ (Antrean /   │
│        │  Pengaturan / Now Playing)           │  Lirik)      │
├────────┴──────────────────────────────────────┴──────────────┤
│ Transport: cover · judul · kontrol · posisi · volume · meter │
├──────────────────────────────────────────────────────────────┤
│ Strip jalur sinyal: FLAC 44.1 kHz 16-bit → Resample → RG →   │
│ EQ → Limiter → WASAPI  (klik tahap = toggle / buka setelan)  │
└──────────────────────────────────────────────────────────────┘
```

- **Sidebar**: Library (Lagu, Album, Artis, Genre, Folder), Playlist, Downloader, Pengaturan.
- **Now Playing** (layar penuh di area konten): cover besar, visualizer, lirik sinkron, info teknis.
- **Strip jalur sinyal** adalah elemen khas Onsa. Isinya rantai DSP yang sebenarnya, dengan format sumber, setiap tahap aktif atau nonaktif, dan backend output. Klik tahap untuk menyalakan atau mematikannya, klik kanan atau tahan untuk membuka pengaturannya.
- **Peak meter L/R** di dekat volume, dengan indikator clip.
- **Mini player**: jendela ringkas satu baris seperti satu unit rak alat audio (cover kecil, judul, waktu, meter, kontrol), dengan opsi always-on-top.
- **Pengaturan**: Output & Kualitas, DSP & EQ, Library, Tampilan, Lirik, Integrasi (Last.fm), Downloader, Pintasan, Tentang.
- **Layar pertama** (first-run): pilih folder musik, lalu pilih salah satu dari tiga tema (dengan pratinjau langsung), lalu scan dimulai.

### 9.3 Skema tema

Tema adalah file JSON. Tema bawaan ada di `themes/` dan bersifat baca saja. Pengguna bisa menduplikasi lalu mengeditnya. Tema buatan pengguna disimpan di `<app_config>/themes/*.json`. Tema divalidasi saat dimuat: field yang hilang atau salah diganti nilai default, dan tema tidak boleh membuat aplikasi rusak.

Tema terdiri dari lima lapis:

1. **Token warna**, termasuk **peran warna berbasis arti** yang wajib diisi setiap tema:

   | Peran | Arti |
   |---|---|
   | `label` | teks label tetap |
   | `adjustable` | nilai yang bisa diatur pengguna |
   | `active` | tahap/fitur yang sedang aktif |
   | `position` | posisi putar, playhead |
   | `caution` | level mendekati batas |
   | `clip` | clipping / error |

   Peran boleh berbagi warna yang sama (misalnya di tema Kaca asap), tapi komponen selalu memakai peran, bukan warna mentah.

   Selain peran, ada **warna tepi** (`color.edge.light` dan `color.edge.dark`): sisi yang kena cahaya dan sisi yang tidak. Itulah yang membuat sebuah kontrol bisa tampak dicetak atau dicetak-timbul tanpa satu baris kode pun yang tahu tema mana yang sedang dipakai.

2. **Bentuk**: radius sudut, kerapatan baris, tebal hairline, **bentuk kontrol** (`soft`, `square`, `pill`, `bevel`), dan **bingkai panel** (`hairline`, `inset`, `raised`, `none`).

3. **Varian komponen**: gaya meter (`bar`, `segment`, `needle`), gaya spektrum (`segment`, `bar`, `soft`), tekstur panel (`glass`, `bezel`, `brushed`), indikator tahap (`glow-chip`, `outline-chip`, `led`), tampilan waktu (`ghost-segment`, `plain`), **baris daftar** (`plain`, `lines`, `stripes`), **scrollbar** (`thin`, `classic`, `hidden`), **tooltip** (`plain`, `panel`), dan **dialog** (`flat`, `raised`, `titled`).

4. **Gerak** (`motion`): `fast` dan `slow` dalam milidetik, dan `ease` (`standard`, `linear`, `snap`, `soft`). Tema boleh mengatur temponya, tidak boleh mengatur perangainya: kedua durasi dijepit ke rentang yang tetap cepat, dan semua kurva yang tersedia berhenti tanpa memantul (§9.1).

5. **Efek dan aset**: glow, pantulan kaca, grain, dan gambar tekstur opsional.

Token diterjemahkan menjadi CSS variables berprefiks `--onsa-`. Varian dipasang sebagai atribut `data-*` pada root, dan komponen membaca varian tersebut. Contoh lengkapnya ada di file `themes/*.json`.

### 9.4 Enam tema bawaan

Referensi visual: `docs/design/palet-preview.html`.

- **Kaca asap** (`kaca-asap`): layar fosfor teal di balik kaca gelap, seperti deck kaset dan receiver era 80-an. Segmen yang mati tetap samar terlihat ("88:88" di belakang angka waktu, blok spektrum dan meter yang redup). Setiap layar punya pantulan kaca tipis, glow halus di elemen yang menyala, dan amber sebagai filter kedua. Font: Chakra Petch + Share Tech Mono.
- **Kokpit kaca** (`kokpit-kaca`): bezel berlapis di latar navy. Warna sepenuhnya mengikuti peran (putih label, cyan bisa diatur, hijau aktif, magenta posisi putar, amber hati-hati, merah clip). Font: B612 + B612 Mono, font terbuka yang dirancang untuk layar kokpit.
- **Deck malam** (`deck-malam`): pelat logam hitam bertekstur sikat, meter jarum VU dengan muka krem yang disinari lampu hangat, jarum merah, dan LED biru untuk tahap aktif. Label bergaya sablon huruf kapital kecil. Font: Barlow + Barlow Condensed.

Tiga tema terang, dibuat sebagai **terinspirasi, bukan tiruan**: tidak ada logo, ikon, wallpaper, atau aset asli dari mana pun, dan tidak ada kode yang tahu nama tema tertentu — semuanya lewat skema §9.3.

- **Kubikel biru** (`kubikel-biru`): panel krem-abu dengan kontrol bersudut siku dan tepi timbul, bilah judul dialog berwarna biru dengan tulisan terang, scrollbar lebar, dan garis tipis antar baris. Rasa kantor awal 2000-an. Font: Barlow + Barlow Condensed + B612 Mono.
- **Kilau milenium** (`kilau-milenium`): plastik bening di atas putih kebiruan — kontrol berbentuk kapsul, kaca, glow, playhead merah muda, baris berselang-seling. Font: Chakra Petch + Share Tech Mono.
- **Musim dingin utara** (`musim-dingin`): putih kebiruan seperti pagi bersalju, meter jarum bermuka krem, aksen tembaga, panel yang sedikit cekung, label huruf kapital kecil, dan sedikit grain. Font: B612 + B612 Mono + Barlow Condensed.

Semua tema memakai **Noto Sans JP** sebagai fallback untuk teks Jepang/CJK. Semua font berlisensi OFL dan **dibundel di dalam aplikasi** (tidak dimuat dari internet), di `ui/static/fonts/`, lengkap dengan file lisensinya. Tema tambahan tidak menambah font baru: yang sudah dibundel dipilih ulang untuk masing-masing watak.

### 9.5 Warna nada

Fitur khas Onsa: rona spektrum (dan elemen lain yang dipilih tema) bergeser mengikuti karakter suara yang sedang diputar. Suara terang bergeser ke rona dingin, suara berat ke rona hangat.

- Sumber data: spectral centroid dari thread analisis (§4.4), dihaluskan (misalnya EMA sekitar 1 detik) supaya pergeseran warna terasa tenang, tidak berkedip.
- Implementasi di UI: `hue-rotate` atau interpolasi warna dalam rentang yang ditentukan tema (`toneColor.range`, dalam derajat).
- Setiap tema menentukan elemen mana yang ikut bergeser. Pengguna bisa mematikan fitur ini. Mode hemat daya ikut mematikannya.

### 9.6 Aksesibilitas dan internasionalisasi

- Kontras teks terhadap latar mengikuti WCAG AA untuk teks utama.
- Semua kontrol bisa dipakai lewat keyboard dan punya fokus yang terlihat.
- String UI disimpan di kamus `id` dan `en`. Tidak boleh ada teks UI yang ditulis langsung di komponen.
- Judul, artis, dan lirik dengan aksara Jepang, Korea, atau Cina harus tampil benar.

---

## 10. Lirik (`onsa-lyrics`)

- Urutan sumber:
  1. File `.lrc` di sebelah file lagu dengan nama dasar yang sama.
  2. Lirik tertanam di file (USLT/SYLT di ID3, `LYRICS` di Vorbis, `©lyr` di MP4).
  3. **LRCLIB** (`https://lrclib.net/api/get` dengan artis, judul, album, dan durasi; bila tidak ada, pakai `/api/search`). Gratis, tanpa API key. Kirim User-Agent Onsa.
- Hasil dari internet disimpan di cache DB. Ada opsi menulis `.lrc` ke sebelah file lagu.
- Parser LRC mendukung beberapa timestamp dalam satu baris, tag `[offset:]`, dan (opsional) timing per kata `<mm:ss.xx>`.
- Tampilan: baris aktif disorot, klik baris untuk seek ke posisi itu, dan baris otomatis bergulir. Lirik tanpa timing ditampilkan sebagai teks biasa.
- Offset waktu bisa diatur per lagu (tombol ±), dan nilainya disimpan.
- Pengambilan lirik dari internet bisa dimatikan total.

---

## 11. Scrobble (`onsa-scrobble`)

- **Last.fm API 2.0**:
  - Autentikasi desktop: `auth.getToken`, lalu buka browser ke halaman otorisasi, lalu `auth.getSession`. Session key disimpan di **credential store OS** (crate `keyring`), bukan di DB atau file teks.
  - `track.updateNowPlaying` dikirim saat lagu mulai.
  - `track.scrobble` dikirim bila lagu berdurasi > 30 detik dan sudah diputar ≥ 50% durasinya atau ≥ 4 menit, mana yang lebih dulu. Timestamp yang dipakai adalah waktu mulai lagu dalam UTC.
  - Bila offline atau gagal, scrobble masuk `scrobble_queue` dan dikirim ulang dalam batch (maksimal 50 per request) saat koneksi kembali.
  - Signature `api_sig` = MD5 dari parameter yang diurutkan ditambah secret.
- **API key dan secret**: dimasukkan saat build lewat environment variable (`ONSA_LASTFM_API_KEY`, `ONSA_LASTFM_API_SECRET`) dan tidak pernah di-commit. Di pengaturan, pengguna bisa memakai key miliknya sendiri. Catat di dokumentasi bahwa secret di aplikasi open-source tidak bisa benar-benar dirahasiakan.
- Modul ini hanya mendengarkan event mesin audio (mulai, pause, resume, seek, selesai) untuk menghitung durasi yang benar-benar didengar.
- (Nanti) ListenBrainz: cukup memakai token pengguna dan endpoint submit-listens.

---

## 12. Pengujian

- **Jangan pernah meng-commit file musik berhak cipta.** Fixture audio dibuat saat tes berjalan: sinus, sweep, noise, dan hening dalam format WAV/FLAC, beserta tag hasil tulisan tes sendiri.
- Mesin audio diuji lewat **sink offline** (§3.1):
  - Gapless: dua file hasil potongan satu sinus kontinu diputar berurutan, lalu output-nya dibandingkan dengan sinus utuh. Tidak boleh ada diskontinuitas.
  - Crossfade: level di titik tengah sesuai kurva equal-power.
  - Micro-fade: tidak ada lompatan sampel saat pause atau seek.
- DSP: respons biquad di beberapa frekuensi uji sesuai perhitungan (toleransi ±0,1 dB), preamp otomatis mencegah clipping, parser AutoEQ diuji dengan contoh valid dan rusak.
- Library: scan folder fixture, scan ulang bertahap, pemantau folder (tambah, hapus, pindah), kompilasi smart playlist ke SQL, dan round-trip M3U8.
- Metadata: round-trip tulis dan baca tag per format, serta simulasi kegagalan penulisan yang harus menyisakan file asli utuh.
- Parser: LRC, progres yt-dlp (dari contoh output yang direkam), dan respons Last.fm/LRCLIB (dari contoh JSON, tanpa jaringan).
- Frontend: `svelte-check` bersih. Uji komponen utama seperlunya.
- CI (GitHub Actions): matriks `windows-latest` dan `ubuntu-latest`, menjalankan `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`, `svelte-check`, dan build aplikasi.

---

## 13. Integrasi OS dan perilaku aplikasi

- **Kontrol media sistem** memakai `souvlaki`: SMTC di Windows (butuh HWND jendela Tauri) dan MPRIS di Linux (pilih backend zbus bila tersedia supaya tidak butuh libdbus). Tombol media keyboard, overlay volume/media Windows, dan widget media desktop Linux harus terhubung.
- **Tray icon**: play/pause, next, prev, tampilkan jendela, keluar. Ada opsi "tutup ke tray".
- **Single instance**: membuka file saat Onsa sudah berjalan akan mengirim file itu ke instance yang ada (putar atau tambah ke antrean).
- **Asosiasi file** "Buka dengan Onsa" dan drag-and-drop file/folder ke jendela.
- **Pulihkan keadaan** saat dibuka: antrean, lagu dan posisi terakhir (dalam keadaan pause), volume, tema, EQ, ukuran dan posisi jendela, serta mode jendela (penuh atau mini).
- **Pintasan keyboard default** (bisa diubah di M4 atau nanti):

  | Aksi | Tombol |
  |---|---|
  | Play/pause | Space |
  | Seek ±5 detik | ← / → |
  | Lagu sebelumnya/berikutnya | Ctrl+← / Ctrl+→ |
  | Volume ± | Ctrl+↑ / Ctrl+↓ |
  | Cari | Ctrl+F |
  | Mini player | Ctrl+M |
  | Now Playing | Ctrl+N |
  | Pengaturan | Ctrl+, |

- **Log** memakai `tracing`, ditulis ke file bergulir di folder log aplikasi. Ada tombol "buka folder log" di Tentang.

### 13.1 Target efisiensi

- Saat pause atau idle, penggunaan CPU mendekati nol: tidak ada polling, dan analisis serta event posisi berhenti.
- Library berisi 10.000 lagu bisa di-scroll mulus, dan pencarian terasa instan.
- Scan pertama berjalan di latar tanpa membuat UI tersendat. Lagu bisa diputar selama scan berlangsung.
- Frontend tidak menyimpan seluruh library di memori. Data diambil per halaman lewat command.

---

## 14. Keamanan dan privasi

- Semua proses eksternal dijalankan dengan array argumen, tanpa shell.
- Binary yang diunduh hanya berasal dari URL rilis resmi yang tercantum di §7.1 dan diverifikasi checksum-nya bila tersedia.
- Secret (session Last.fm) disimpan di credential store OS.
- Tidak ada telemetri. Akses jaringan hanya terjadi untuk fitur yang pengguna nyalakan: LRCLIB, Last.fm, MusicBrainz, dan unduhan binary/yt-dlp. Setiap fitur jaringan bisa dimatikan.
- CSP Tauri dibuat ketat. Frontend tidak memuat sumber daya dari internet, termasuk font.

---

## 15. Milestone

Kerjakan berurutan. Milestone berikutnya dimulai setelah **kriteria selesai** milestone sebelumnya terpenuhi dan dilaporkan di `docs/PROGRESS.md`.

Urutannya pernah diubah sekali, atas keputusan pemilik proyek (2026-09-18): sesudah M7a, yang dikerjakan adalah sebagian M10 (pengambil binary dan yt-dlp) lalu M8, dan M9 dilewati. Alasannya beserta utang yang timbul ada di `docs/DECISIONS.md`.

### M0. Kerangka proyek
Workspace Cargo sesuai §2, Tauri 2 + SvelteKit berjalan di Windows dan Linux, CI sesuai §12, dan pemuat tema (stub) yang sudah membaca satu tema bawaan.
**Selesai bila:** jendela kosong bertema terbuka di kedua OS, dan CI hijau.

### M1. Mesin audio inti (tanpa UI)
Decode (§3.2 kecuali Opus), resample, output cpal, sink offline, antrean, gapless, crossfade, micro-fade, seek, pemulihan saat perangkat berubah, dan `onsa-cli` (`play`, `queue`, `devices`, `render-wav`).
**Selesai bila:** `onsa-cli` memutar daftar file secara gapless di kedua OS, dan tes gapless, crossfade, serta micro-fade (§12) lulus.

### M2. Rantai DSP
ReplayGain, EQ (grafis dan parametrik), preamp otomatis, import/export AutoEQ, limiter, volume, dither, parameter lock-free dengan smoothing, dan tap analisis (spektrum, meter, centroid). Semua bisa dikendalikan dari `onsa-cli`.
**Selesai bila:** tes DSP (§12) lulus, dan mengubah EQ saat lagu berjalan tidak menimbulkan glitch atau klik.

### M3. Library
Skema DB dan migrasi, scanner, pemantau folder, cache cover, FTS, statistik dan riwayat putar.
**Selesai bila:** tes library lulus, dan scan ulang folder yang tidak berubah selesai jauh lebih cepat daripada scan pertama.

### M4. UI dasar dan integrasi OS
Command dan event Tauri, layar pertama, sidebar dan tampilan library (virtual list), transport, strip jalur sinyal, panel antrean, Now Playing, pengaturan (Output & Kualitas, DSP & EQ, Library, Tampilan), sistem tema lengkap dengan **tiga tema bawaan**, mini player, sleep timer, kontrol media OS, tray, single instance, pintasan, pemulihan keadaan, dan i18n id/en.
**Selesai bila:** seluruh alur dari pilih folder, scan, jelajah, putar, atur EQ, sampai ganti tema bisa dilakukan tanpa CLI di kedua OS, dan ketiga tema sesuai `palet-preview.html`.

### M5. Visualizer, meter, warna nada
Visualizer spektrum dan meter dengan varian per tema (segmen, batang, jarum VU), warna nada, dan perilaku di mode hemat daya.
**Selesai bila:** saat visualizer disembunyikan atau aplikasi di-minimize, tap analisis berhenti (terverifikasi lewat log atau tes), dan warna nada bisa dinyalakan dan dimatikan.

### M6. Playlist
Playlist manual, smart playlist beserta editor aturannya, simpan antrean sebagai playlist, dan import/export M3U8.
**Selesai bila:** tes smart playlist dan M3U8 lulus, dan smart playlist ikut berubah saat library berubah.

### M7. Metadata
Editor tag satuan dan massal, cover, `overrides` dengan aksi "tulis ke file", penulisan aman, rename berdasarkan pola dengan pratinjau, lalu MusicBrainz (opsional).
**Selesai bila:** tes metadata lulus, termasuk simulasi kegagalan penulisan.

Dibelah atas keputusan pemilik proyek (2026-09-18): **M7a** (edit massal, penulisan aman, rename, pengaman, dan pencarian online) selesai; **M7b** (hapus/ekspor sampul, menanam sampul ke berkas beserta aturan 1200 px, "(beragam)", dan tiga field yang belum bisa di-override) dikerjakan setelah M10 penuh; **editor tag satuan pindah ke M8**, karena lirik salah satu field di dalamnya.

### M8. Lirik
Tiga sumber lirik, cache, parser LRC, tampilan sinkron, klik untuk seek, offset per lagu, dan **editor tag satuan** (pindahan dari M7).
**Selesai bila:** lirik `.lrc` lokal dan lirik dari LRCLIB tampil sinkron, dan offset tersimpan.

### M9. Scrobble
Autentikasi Last.fm, now playing, aturan scrobble, antrean offline, dan keyring.
**Selesai bila:** scrobble tercatat di akun Last.fm penguji, dan scrobble saat offline terkirim setelah koneksi kembali.

### M10. Downloader
Pengelolaan binary (persetujuan, checksum, update, opsi program sistem), pratinjau URL/playlist, pilihan format, antrean unduhan paralel, progres, batal, dan impor otomatis ke library.
**Selesai bila:** URL satu video dan satu playlist bisa diunduh sampai muncul di library di kedua OS, tidak ada jendela konsol di Windows, dan tombol batal menghentikan semua proses turunan.

Dikerjakan lebih awal dan dibelah atas keputusan pemilik proyek (2026-09-18): **M10a** = pengambil binary berkas tunggal (unduh + checksum) dengan yt-dlp sebagai konsumen pertama, integrasi yt-dlp, dan halaman Unduhan beserta persetujuan §7.1. **M10b** = Deno dan ffmpeg (yang butuh pembongkar arsip), antrean paralel, dan sisanya. Urutan barunya: M7a → M10a → M8 → M10b.

### M11. Kualitas lanjutan
Decoder Opus (lalu ubah prioritas format downloader), exclusive mode WASAPI, dan penyempurnaan mode hemat daya.
**Selesai bila:** file Opus hasil yt-dlp bisa diputar dengan gapless, dan exclusive mode bisa dinyalakan dan dimatikan tanpa crash.

### M12. Rilis
Bundling installer (Windows: NSIS/MSI; Linux: AppImage dan deb), ikon final, halaman Tentang beserta daftar lisensi pihak ketiga, dan README.
**Selesai bila:** installer terpasang dan berjalan di mesin bersih untuk kedua OS.

---

## 16. Hal yang belum diputuskan

Hal-hal di bawah tidak menghalangi milestone M0–M11. Tanyakan ke pemilik proyek saat sampai di titik yang membutuhkannya.

- Lisensi proyek (dibutuhkan paling lambat di M12).
- Ikon final. Sampai saat itu, pakai placeholder `assets/icon.svg` (garpu tala sederhana).
- Distribusi tambahan: Flatpak/Flathub, AUR, winget.
- Analisis loudness EBU R128 dan ListenBrainz.
