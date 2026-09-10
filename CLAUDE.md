# CLAUDE.md: Onsa

Onsa adalah music player desktop (Windows + Linux) dengan Tauri 2, Rust, dan SvelteKit. Identitas visualnya "panel instrumen". Sumber kebenaran proyek ada di **`docs/SPEC.md`**. Baca seluruhnya sebelum mulai, dan baca ulang bagian yang relevan di awal setiap milestone.

## Cara bekerja

- Kerjakan milestone di `docs/SPEC.md` §15 **secara berurutan**. Jangan memulai milestone berikutnya sebelum kriteria selesai milestone sekarang terpenuhi.
- Di akhir setiap milestone, perbarui `docs/PROGRESS.md`: apa yang selesai, bagaimana kriteria selesai diverifikasi, dan apa yang tertunda. Buat file ini di M0.
- Bila spesifikasi kurang jelas, pilih opsi paling sederhana yang tetap konsisten dengan spesifikasi, lalu catat di `docs/DECISIONS.md` (tanggal, konteks, keputusan, alasan). Buat file ini di M0.
- **Jangan mengubah "Keputusan terkunci" (SPEC §1)** atau menambah dependensi besar di luar SPEC tanpa bertanya ke pemilik proyek.
- Hal di SPEC §16 ditanyakan ke pemilik proyek saat benar-benar dibutuhkan, bukan diputuskan sendiri.
- Berkomunikasi dengan pemilik proyek dalam **bahasa Indonesia**.

## Aturan real-time audio (wajib)

Di callback output cpal dan jalur panas DSP:
- tidak ada alokasi memori, lock/mutex, I/O, logging, atau panic;
- parameter masuk lewat atomics atau `rtrb`, lalu diterapkan dengan smoothing;
- semua buffer dialokasikan di muka.

Bila ragu apakah sesuatu aman untuk real-time, anggap tidak aman dan pindahkan ke thread lain.

## Konvensi kode

- **Bahasa**: kode, identifier, komentar, pesan commit, dan README dalam bahasa Inggris. Semua teks UI lewat kamus i18n (`id`, `en`), tidak pernah ditulis langsung di komponen.
- **Rust**: stable, `rustfmt`, `cargo clippy -- -D warnings` bersih. Crate library memakai `thiserror`, lapisan aplikasi (`src-tauri`, `onsa-cli`) memakai `anyhow`. Tidak ada `unwrap()`/`expect()` di kode non-tes kecuali ada komentar alasan. Log memakai `tracing`.
- **Batas crate** mengikuti SPEC §2. `onsa-audio` tidak bergantung pada crate Onsa lain. Modul fitur tidak saling bergantung dan disatukan hanya di `src-tauri`.
- **Frontend**: Svelte 5 (runes), TypeScript strict, `svelte-check` bersih. Styling hanya CSS + CSS variables `--onsa-*` dari sistem tema. Komponen memakai **peran warna** (`label`, `adjustable`, `active`, `position`, `caution`, `clip`), bukan hex mentah. Tanpa Tailwind dan tanpa library komponen.
- **Database**: setiap perubahan skema lewat migrasi berversi. Query dari input pengguna selalu berparameter.
- **Commit**: kecil dan terfokus, dengan pesan berformat Conventional Commits (`feat(audio): ...`, `fix(library): ...`).

## Keamanan

- Proses eksternal (yt-dlp, ffmpeg, deno) **selalu dengan array argumen, tanpa shell**. Di Windows pakai `CREATE_NO_WINDOW`.
- Binary hanya diunduh dari URL rilis resmi di SPEC §7.1, dengan verifikasi checksum bila tersedia.
- Jangan pernah meng-commit API key, secret, atau token. Key Last.fm masuk lewat env `ONSA_LASTFM_API_KEY` / `ONSA_LASTFM_API_SECRET` saat build.
- Frontend tidak memuat apa pun dari internet. Font dibundel di `ui/static/fonts/` beserta lisensinya.

## Pengujian

- **Jangan meng-commit file musik berhak cipta.** Fixture audio dibuat saat tes berjalan (sinus, sweep, noise, hening).
- Tes mesin audio memakai sink offline, bukan perangkat audio sungguhan.
- Tes yang butuh jaringan memakai contoh respons yang direkam. CI tidak boleh bergantung pada internet selain untuk mengunduh dependensi.
- Sebelum menyatakan milestone selesai, jalankan:
  ```
  cargo fmt --check
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cd ui && npm run check
  ```

## Lintas platform

- Setiap fitur harus jalan di Windows **dan** Linux. Hal yang khusus OS diletakkan di modul `platform/` dengan `cfg`.
- Path memakai `PathBuf`/`Path`, tidak digabung sebagai string. Uji juga path berisi aksara Jepang dan spasi.
- Dependensi sistem Linux untuk build (webkit2gtk-4.1, ALSA dev, dan lain-lain) didokumentasikan di README.

## Referensi di repo

- `docs/SPEC.md`: spesifikasi lengkap
- `docs/design/palet-preview.html`: referensi visual tiga tema bawaan (buka di browser)
- `themes/*.json`: tema bawaan, juga contoh skema tema (SPEC §9.3)
- `assets/icon.svg`: ikon placeholder
