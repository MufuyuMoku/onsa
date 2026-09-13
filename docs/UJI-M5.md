# Uji M5: visualizer, warna nada, dan mode hemat daya

Lanjutan dari `UJI-M4.md`. Yang di sana masih berlaku; ini tambahannya saja.

File: `dist-test/Onsa-M5.exe`. Tinggal klik dua kali, tidak perlu dipasang. Semua setelan dan library-mu tetap seperti sebelumnya.

---

## 1. Visualizer

Putar sebuah lagu, lalu buka **Sedang diputar** (ikon di transport, atau Ctrl+N).

**Yang seharusnya terjadi:** di bawah cover dan info lagu ada spektrum yang bergerak mengikuti musik, lengkap dengan garis tipis yang menahan puncak sebentar lalu turun perlahan. Bentuknya berbeda-beda menurut tema:

| Tema | Bentuk spektrum |
|---|---|
| Kaca asap | blok-blok bertumpuk; blok yang mati tetap terlihat samar di belakangnya |
| Kokpit kaca | batang hijau polos yang naik turun, penanda puncak putih |
| Deck malam | kolom hangat yang memudar dari bawah, dengan penanda puncak merah |

Coba ganti tema di **Pengaturan → Tampilan** sambil lagu berjalan, lalu kembali ke Sedang diputar.

Saat lagu dijeda atau belum ada yang diputar, spektrumnya tidak membeku di gambar terakhir melainkan berganti tulisan **Diam**. Itu disengaja: kalau tidak ada sinyal, Onsa tidak berpura-pura ada. Bilang kalau menurutmu lebih enak kalau batangnya turun perlahan ke nol.

## 2. Warna nada

Masih di Sedang diputar, dengarkan lagu yang pindah dari bagian tenang dan berat ke bagian terang dan ramai.

**Yang seharusnya terjadi:** rona spektrum bergeser pelan-pelan. Suara terang bergeser ke rona yang lebih dingin, suara berat ke rona yang lebih hangat. Pergeserannya **tenang**, bukan berkedip mengikuti ketukan. Kalau terasa terlalu ramai atau terlalu samar, bilang saja — rentangnya bisa diatur per tema.

Matikan lewat **Pengaturan → Tampilan → Warna nada**. Warnanya harus kembali ke warna tema apa adanya, dan spektrumnya tetap bergerak seperti biasa.

## 2b. Kekuatan warna nada dan mini player (perbaikan terbaru)

Dua hal yang kamu laporkan sudah diperbaiki; ini cara memeriksanya.

**Kekuatan warna nada.** Di **Pengaturan → Tampilan → Warna nada** sekarang ada empat pilihan: mati, halus, sedang, kuat. Bawaannya **sedang**, dan pergeserannya jauh lebih terlihat daripada versi sebelumnya karena warnanya benar-benar bergeser ke warna lain milik tema, bukan sekadar diputar ronanya:

| Tema | Suara berat | Suara terang |
|---|---|---|
| Kaca asap | amber | biru fosfor |
| Kokpit kaca | amber hati-hati | cyan |
| Deck malam | merah jarum VU | baja dingin |

Di Deck malam, bar posisi ikut bergeser bersama spektrum. Meter sengaja **tidak** ikut: hijau, kuning, dan merahnya berarti "mendekati batas", dan itu tidak boleh berubah arti. Kalau "sedang" masih terasa kurang, coba "kuat"; kalau kejauhan, "halus".

**Mini player.** Maksimalkan jendela, masuk mini player, lalu kembali. Jendelanya harus kembali **maximized persis seperti semula**. Coba juga dari jendela biasa yang kamu atur sendiri ukurannya, dan lewat Ctrl+M, dan setelah Onsa ditutup lalu dibuka lagi dalam mode mini. Setelah kembali dari mini player, tombol restore di judul jendela juga harus memberi jendela yang wajar, bukan strip.

## 2c. Ukuran jendela

Onsa sekarang menyesuaikan diri saat jendelanya dikecilkan, dan tidak bisa lagi dikecilkan sampai rusak: **batas bawahnya 820×600**.

Coba tarik sisi jendela pelan-pelan dari lebar sampai sempit dan perhatikan urutannya:

| Lebar jendela | Yang berubah |
|---|---|
| di bawah 1160 | panel antrean menyingkir; buka lewat tombol daftar di kanan kotak pencarian, dan ia menimpa daftar lagu |
| di bawah 1120 | tombol Sedang diputar, sleep timer, dan mini player masuk ke tombol **…** di transport |
| di bawah 1040 | sidebar menyempit jadi ikon saja |
| di bawah 1000 | meter L/R menyingkir |
| di bawah 880 | sidebar jadi laci; buka lewat tombol garis tiga di kiri atas |
| di bawah 780 | ReplayGain dan Limiter di strip bawah masuk ke tombol **…** di ujung kanan strip |

Kolom daftar lagu juga menyusut berurutan: tahun lebih dulu, lalu album, lalu artis. **Judul dan durasi tidak pernah hilang.**

**Yang perlu kamu pastikan:** di lebar mana pun, tidak ada teks yang menindih atau terpotong tanpa titik-titik, tidak ada scrollbar mendatar yang aneh, dan tombol putar, bar posisi, serta volume selalu terlihat. Kalau ada yang masih berantakan, sebutkan ukuran jendelanya kira-kira berapa dan di halaman apa.

Ukuran yang sudah kuperiksa otomatis pada tiap halaman (lebar area gambar dalam piksel CSS): **1546, 1266, 1086, 946, 806, dan 686** — dua yang terakhir tepat di dan di bawah batas bawah.

## 3. Hemat daya

Di **Pengaturan → Output & Kualitas**, nyalakan **Mode hemat daya** sambil lagu berjalan.

**Yang seharusnya terjadi:**

- Suaranya tidak terputus saat mode ini dinyalakan atau dimatikan.
- Spektrum dan meter bergerak lebih patah-patah (20 gambar per detik, bukan 60). Ini memang disengaja.
- Penunjuk waktu dan bar posisi bergerak tiap setengah detik, bukan tiap sepersepuluh detik.
- Warna nada berhenti bergeser.
- Setelah dimatikan lagi, pilihan buffer dan kualitas resampler yang kamu set sendiri kembali seperti semula — mode ini hanya menimpanya sementara.

Kalau kamu memakai laptop tanpa charger, coba juga dengarkan satu album penuh dengan mode ini menyala dan perhatikan apakah kipasnya lebih tenang.

## 4. Yang paling perlu kuketahui

Onsa sengaja **berhenti menganalisis** saat tidak ada yang menampilkannya, supaya tidak membuang tenaga:

- Keluar dari Sedang diputar → spektrum berhenti dihitung, meter tetap jalan.
- Jendela di-minimize, atau ditutup ke tray → semuanya berhenti.
- Buka lagi → semuanya kembali jalan.

**Yang perlu kamu pastikan:** setelah minimize lalu dibuka lagi, atau setelah keluar-masuk Sedang diputar berkali-kali, meter dan spektrum benar-benar **hidup lagi** dan tidak ada yang membeku. Kalau ada yang membeku, itu bug, dan tolong laporkan.

## 5. Kalau ada yang aneh

Sama seperti sebelumnya: ceritakan apa yang terjadi, di lagu apa dan detik ke berapa, lalu nyalakan **Log debug** di Pengaturan → Tentang, ulangi kejadiannya, tekan **Buka folder log**, dan kirim file yang paling baru.

Untuk masalah visualizer, sebutkan juga tema yang sedang kamu pakai dan apakah mode hemat daya sedang menyala.
