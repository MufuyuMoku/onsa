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
