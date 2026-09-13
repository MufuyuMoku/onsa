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
| Kokpit kaca | batang penuh yang naik turun |
| Deck malam | satu kurva terisi yang meliuk |

Coba ganti tema di **Pengaturan → Tampilan** sambil lagu berjalan, lalu kembali ke Sedang diputar.

## 2. Warna nada

Masih di Sedang diputar, dengarkan lagu yang pindah dari bagian tenang dan berat ke bagian terang dan ramai.

**Yang seharusnya terjadi:** rona spektrum bergeser pelan-pelan. Suara terang bergeser ke rona yang lebih dingin, suara berat ke rona yang lebih hangat. Pergeserannya **tenang**, bukan berkedip mengikuti ketukan. Kalau terasa terlalu ramai atau terlalu samar, bilang saja — rentangnya bisa diatur per tema.

Matikan lewat **Pengaturan → Tampilan → Warna nada**. Warnanya harus kembali ke warna tema apa adanya, dan spektrumnya tetap bergerak seperti biasa.

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
