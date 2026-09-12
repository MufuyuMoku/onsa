# Uji coba Onsa (M4)

Ini versi coba-coba yang sudah lengkap untuk dipakai sehari-hari: pilih folder, scan, jelajahi, putar, atur suara, atur antrean, ganti tampilan, dan kendalikan lewat tombol media atau tray. Yang belum dikerjakan sengaja tidak ditampilkan, jadi kalau sebuah tombol ada, artinya tombol itu memang berfungsi.

Bagian 1 sampai 13 sama seperti sebelumnya dan sudah kamu lalui; kalau mau, langsung saja ke bagian 14 dan seterusnya, yang isinya baru.

Yang dibutuhkan: file `Onsa-M4.exe`, folder musik, dan headphone atau speaker yang biasa dipakai.

## Cara menjalankan

1. Klik dua kali `Onsa-M4.exe`. Tidak perlu dipasang.
2. Kalau Windows menampilkan peringatan "Windows protected your PC", klik **More info**, lalu **Run anyway**. Peringatan itu muncul karena file ini belum ditandatangani secara resmi, dan itu memang urusan menjelang rilis.
3. Menutup jendela berarti menutup aplikasi.

Onsa hanya membaca folder musik. File lagu tidak pernah diubah, dipindah, atau dihapus.

---

## Daftar yang perlu dicoba

Tidak harus berurutan, tapi nomor 1 memang paling awal.

### 1. Layar pertama

Pilih folder musik, pilih salah satu dari tiga tema, lalu mulai scan.

**Yang seharusnya terjadi:** tampilan langsung berubah begitu tema dipilih. Saat scan berjalan, jumlah file yang ditemukan terus bertambah, dan aplikasi tetap bisa digerakkan, tidak membeku. Setelah selesai, muncul tombol untuk membuka library.

### 2. Kecepatan scan dan pencarian

Setelah scan pertama selesai, tutup Onsa lalu buka lagi. Ketik beberapa huruf di kotak pencarian di kanan atas.

**Yang seharusnya terjadi:** saat dibuka lagi, Onsa memeriksa foldernya dengan sangat cepat, jauh lebih cepat daripada scan pertama. Hasil pencarian muncul seketika sambil mengetik, bukan setelah menunggu. Potongan kata sudah cukup: "lamp" menemukan "Lampu Kota". Huruf beraksen juga tidak jadi masalah: "cafe" menemukan "Café".

**Tolong perhatikan:** ada lagu yang seharusnya ketemu tapi tidak muncul, atau ada yang hilang dari daftar.

### 3. Putar album dan antrean

Buka **Album** di kiri, pilih satu album, lalu tekan **Putar album**. Setelah itu coba juga klik dua kali sebuah lagu di daftar **Lagu**.

**Yang seharusnya terjadi:** panel **Antrean** di kanan terisi. Lagu yang sedang berjalan ditandai dengan warna berbeda. Klik dua kali lagu di antrean langsung melompat ke lagu itu.

### 4. Gapless: lagu yang seharusnya menyambung

Ini yang paling penting. Putar album yang lagunya memang menyambung tanpa jeda, misalnya album live atau album konsep, lalu dengarkan perpindahan antar-lagunya.

Ulangi dengan album MP3 dan album M4A, karena dua format itu yang paling sering bermasalah di aplikasi lain.

**Yang seharusnya terjadi:** perpindahan antar-lagu benar-benar mulus. Tidak ada jeda sunyi sekejap, tidak ada bunyi "klik", dan tidak ada potongan suara yang hilang atau terdengar dua kali.

**Tolong perhatikan:** kalau terdengar jeda atau klik, catat albumnya dan perpindahan dari lagu keberapa ke keberapa.

### 5. Crossfade

Buka **Pengaturan → Output & Kualitas**, atur **Crossfade** ke sekitar 4 detik. Putar beberapa lagu dari artis berbeda dan biarkan berpindah sendiri. Coba juga tekan tombol berikutnya di tengah lagu.

Setelah itu, putar album yang lagunya menyambung (nomor 4) dengan crossfade tetap menyala.

**Yang seharusnya terjadi:** antar-lagu biasa, suara lagu lama meredup sementara lagu baru menguat, halus dan tanpa terdengar melemah di tengah persilangan. Untuk album yang lagunya berurutan, crossfade sengaja dilewati supaya albumnya tetap menyambung, selama pilihannya masih dicentang.

### 6. Klik saat jeda dan saat memindahkan posisi

Tekan jeda lalu lanjut berkali-kali, di bagian lagu yang sedang keras. Geser posisi putar ke sana kemari beberapa kali dengan cepat. Tekan tombol lagu berikutnya dan sebelumnya beberapa kali berturut-turut.

**Yang seharusnya terjadi:** setiap kali, suara meredup dan muncul kembali dalam waktu sangat singkat, sehingga tidak terdengar bunyi "tik", "pop", atau "krek".

### 7. Cabut dan colok headphone

Sambil lagu berjalan, cabut headphone. Tunggu beberapa detik, lalu colok lagi. Kalau punya perangkat lain, misalnya speaker bluetooth atau HDMI, coba juga pindah perangkat lewat **Pengaturan → Output & Kualitas → Perangkat output** sambil lagu berjalan.

**Yang seharusnya terjadi:** Onsa tidak menutup diri dan tidak diam selamanya. Suara pindah ke perangkat yang tersedia, dan lagu berlanjut dari posisi yang kira-kira sama. Saat perangkat dipilih sendiri, suara pindah ke perangkat itu tanpa perlu memutar ulang lagunya.

### 8. Menggeser EQ saat lagu berjalan

Buka **Pengaturan → DSP & EQ**. Pastikan **EQ aktif** dicentang. Sambil lagu berjalan, geser-geser fader, terutama yang paling kiri (nada rendah) dan yang di tengah, naik turun agak ekstrem.

**Yang seharusnya terjadi:** perubahan suara langsung terdengar mengikuti geseran, dan kurva di atas fader ikut berubah. Tidak ada bunyi klik, retak, atau suara yang putus saat menggeser.

### 9. Preset AutoEQ untuk headphone

Kalau punya file preset AutoEQ untuk headphone-mu, biasanya bernama `ParametricEQ.txt`, tekan **Import AutoEQ…** lalu pilih file itu. Kalau belum punya, lewati saja nomor ini.

**Yang seharusnya terjadi:** daftar filter terisi, EQ berpindah ke mode parametrik, dan muncul keterangan berapa filter yang dimuat. Suara headphone terasa lebih rata sesuai presetnya. Tombol **Export AutoEQ…** menyimpan kembali pengaturan itu sebagai file yang bisa dipakai lagi.

### 10. Preamp otomatis dan limiter

Masih di **DSP & EQ**: naikkan beberapa fader EQ cukup tinggi, lalu nyalakan **Preamp otomatis**. Setelah itu coba matikan dan nyalakan **Limiter aktif** sambil memutar lagu yang keras.

**Yang seharusnya terjadi:** dengan preamp otomatis menyala, suara jadi sedikit lebih pelan tapi tetap bersih, tidak pecah. Tanpa preamp otomatis dan tanpa limiter, lagu keras dengan EQ tinggi bisa terdengar pecah, dan tulisan **CLIP** di meter kanan bawah menyala merah. Saat limiter bekerja, tulisan **LIM** menyala.

### 11. ReplayGain

Di **Pengaturan → Output & Kualitas**, bagian **ReplayGain**, pilih **Otomatis**. Putar beberapa lagu dari album yang berbeda-beda, terutama yang biasanya terdengar jauh lebih keras atau jauh lebih pelan dari yang lain.

**Yang seharusnya terjadi:** perbedaan keras-pelan antar-lagu jadi lebih kecil, tanpa suara jadi pecah. Di strip paling bawah, tahap ReplayGain menyala dan menampilkan berapa desibel yang sedang diterapkan untuk lagu itu.

### 12. Ganti tema

Buka **Pengaturan → Tampilan** dan cobalah ketiga tema: Kaca asap, Kokpit kaca, dan Deck malam.

**Yang seharusnya terjadi:** seluruh jendela langsung berganti, termasuk bentuk meter di kanan bawah, yang berubah antara batang, segmen, dan jarum. Tema yang dipilih tetap terpakai saat Onsa dibuka lagi.

### 13. Strip jalur sinyal (bagian paling bawah)

Perhatikan deretan kotak kecil di bawah tombol pemutar saat lagu berjalan. Klik salah satunya, misalnya **EQ** atau **Limiter**, lalu coba juga klik kanan sebuah kotak.

**Yang seharusnya terjadi:** kotak-kotak itu menunjukkan apa yang sebenarnya terjadi pada suara: format file, perubahan sample rate, ReplayGain, EQ, limiter, dan perangkat yang dipakai. Klik menyalakan atau mematikan tahap itu dan suaranya langsung berubah. Klik kanan membuka halaman pengaturannya.

### 14. Jelajah lewat Artis, Genre, dan Folder

Di kiri sekarang ada **Artis**, **Genre**, dan **Folder**. Buka salah satunya, klik satu nama, lalu tekan **Putar**.

**Yang seharusnya terjadi:** daftarnya muncul dengan jumlah lagu di kanan, dan membukanya menampilkan lagu-lagunya. Untuk folder, nama folder ditampilkan besar dengan jalur lengkapnya di bawahnya.

### 15. Mengatur antrean

Di panel **Antrean** di kanan: tarik sebuah lagu untuk memindahkannya, tekan tanda silang untuk menghapusnya, lalu coba **Acak**, **Ulang**, dan **Kosongkan antrean**. Klik kanan sebuah lagu di daftar untuk **Putar berikutnya** atau **Tambah ke antrean**.

**Yang seharusnya terjadi:** semua itu terjadi **tanpa memotong lagu yang sedang berbunyi**. Menyalakan Acak memindahkan lagu yang sedang diputar ke urutan pertama dan mengacak sisanya; mematikannya mengembalikan urutan semula. **Ulang** berpindah antara mati, seluruh antrean, dan satu lagu. Saat mengulang satu lagu, sambungannya harus tetap mulus, tanpa jeda.

**Tolong perhatikan:** kalau ada bunyi terpotong saat mengubah antrean, catat apa yang sedang kamu lakukan.

### 16. Sedang diputar

Tekan tombol bulat di sebelah kanan tombol pemutar, atau **Ctrl+N**.

**Yang seharusnya terjadi:** cover tampil besar, lengkap dengan judul, artis, album, bar posisi, serta keterangan format file dan perangkat output yang sedang dipakai.

### 17. Mini player

Tekan tombol panah mengecil di kanan, atau **Ctrl+M**.

**Yang seharusnya terjadi:** jendela menyusut jadi satu baris dan selalu berada di atas jendela lain, tapi tetap bisa memutar, menggeser posisi, dan menampilkan meter. Tekan tombol panah membesar untuk kembali. Kalau Onsa ditutup dalam mode mini, ia akan terbuka lagi dalam mode mini.

### 18. Sleep timer

Tekan tombol jam di dekat kanan bawah. Pilih misalnya **Setelah 15 menit**, atau **Di akhir lagu ini**, lalu pilih apa yang terjadi sesudahnya: jeda, berhenti, atau tutup Onsa. Untuk mencoba cepat, isi **Menit** dengan 1 lalu tekan **Mulai**.

**Yang seharusnya terjadi:** sisa waktunya terlihat di sebelah tombol. Menjelang waktunya, volume turun perlahan (lamanya bisa diatur), lalu Onsa melakukan yang kamu pilih. Setelah itu **volume kembali ke angka semula**, jadi pemutaran berikutnya tidak senyap. Tombol **Batalkan** menghentikan timer kapan saja.

### 19. Tombol media di keyboard dan panel Windows

Tekan tombol play/pause, berikutnya, dan sebelumnya di keyboard atau headset. Perhatikan juga panel media Windows yang muncul saat volume diubah.

**Yang seharusnya terjadi:** Onsa menanggapi tombol-tombol itu, dan panel media Windows menampilkan judul, artis, album, serta covernya.

### 20. Ikon tray

Cari ikon Onsa di tray (dekat jam). Klik kanan untuk menu, klik kiri untuk memunculkan jendela. Di **Pengaturan → Tentang**, nyalakan **Tutup jendela hanya menyembunyikan Onsa ke tray**, lalu tutup jendelanya.

**Yang seharusnya terjadi:** menu tray berisi tampilkan, putar atau jeda, sebelumnya, berikutnya, dan keluar — dalam bahasa yang sedang dipakai. Dengan pilihan tadi menyala, menutup jendela tidak menutup Onsa: musiknya jalan terus dan jendelanya bisa dipanggil lagi dari tray.

### 21. Buka file lewat Onsa dan seret ke jendela

Seret satu atau beberapa file lagu ke jendela Onsa. Seret juga sebuah folder. Coba pula klik kanan sebuah file lagu di Explorer, lalu **Open with → Onsa** (kalau belum ada di daftar, pilih lewat "Choose another app" dan arahkan ke `Onsa-M4.exe`).

**Yang seharusnya terjadi:** file langsung diputar, folder ditambahkan ke library lalu di-scan. Kalau Onsa sudah berjalan, tidak ada jendela kedua yang terbuka: jendela yang ada maju ke depan dan memutar file itu.

### 22. Samakan sample rate dengan sumber

Di **Pengaturan → Output & Kualitas**, nyalakan **Samakan dengan sumber**. Putar lagu 44,1 kHz lalu lagu 48 kHz secara berurutan.

**Yang seharusnya terjadi:** di strip bawah, tahap **Resample** hilang, dan perangkat output mengikuti sample rate tiap lagu. Perpindahan antar-lagu dengan sample rate berbeda mungkin terdengar jeda singkat, karena perangkat memang dibuka ulang.

### 23. Antrean kembali saat dibuka lagi

Putar beberapa lagu, lalu tutup Onsa di tengah lagu dan buka lagi.

**Yang seharusnya terjadi:** antrean yang sama kembali, berhenti di lagu dan posisi terakhir, dalam keadaan **jeda**. Ukuran dan posisi jendela juga kembali seperti semula.

### 24. Tema buatan sendiri

Di **Pengaturan → Tampilan**, tekan **Buka folder tema**. Taruh file `.json` tema di situ (paling mudah: salin salah satu tema bawaan dari folder `themes/` di repositori, lalu ubah `id`, `name`, dan beberapa warnanya). Tutup dan buka Onsa lagi.

**Yang seharusnya terjadi:** tema itu ikut muncul di daftar tema dan bisa dipilih. Tema yang rusak dilewati begitu saja, tidak membuat Onsa gagal terbuka.

### 25. Mengedit antrean cepat-cepat (perbaikan terbaru)

Ini pengujian ulang untuk bug yang kamu temukan. Klik dua kali sebuah lagu supaya seluruh daftar masuk antrean, lalu selagi lagu berjalan:

1. Hapus baris antrean secepat mungkin dengan menekan tombol x berkali-kali, termasuk baris lagu yang sedang berbunyi.
2. Hapus semua baris sesudah lagu yang sedang berbunyi.
3. Tekan **Kosongkan** saat lagu berjalan.
4. Tarik-lepas beberapa baris untuk mengubah urutan, lalu nyalakan dan matikan **Acak**.

**Yang seharusnya terjadi:** lagu yang disorot di antrean selalu sama dengan lagu yang terdengar, dan sama dengan judul di bawah. Menghapus lagu yang sedang berbunyi langsung memutar lagu berikutnya, tanpa bunyi klik; kalau tidak ada lagi lagu sesudahnya, pemutaran berhenti. Menghapus lagu-lagu sesudahnya tidak mengganggu lagu yang sedang berjalan sama sekali. Mengosongkan antrean menghentikan suara. Mengubah urutan dan mengacak tidak pernah menukar lagu yang sedang berbunyi.

### 26. Pintasan keyboard

| Tombol | Fungsi |
|---|---|
| Spasi | putar atau jeda |
| ← / → | mundur atau maju 5 detik |
| Ctrl+← / Ctrl+→ | lagu sebelumnya atau berikutnya |
| Ctrl+↑ / Ctrl+↓ | volume naik atau turun |
| Ctrl+F | ke kotak pencarian |
| Ctrl+N | Sedang diputar |
| Ctrl+M | mini player |
| Ctrl+, | pengaturan |

---

## Kalau ada yang aneh

Tidak perlu menebak penyebabnya. Cukup ceritakan apa yang kamu alami, misalnya "ada bunyi klik tiap pindah lagu di album ini" atau "suaranya hilang setelah headphone dicolok lagi".

Supaya cepat ketemu, tolong sertakan:

1. **Apa yang terjadi** dan apa yang kamu harapkan terjadi.
2. **Lagu apa dan detik ke berapa**, misalnya "lagu ketiga album X, sekitar 2:15, tepat saat pindah ke lagu keempat".
3. **File log.**

Cara mengambil log:

1. Buka **Pengaturan → Tentang**, nyalakan **Log debug**.
2. Ulangi lagi kejadian yang bermasalah, supaya tercatat.
3. Tekan **Buka folder log**, lalu ambil file yang paling baru.
4. Kirim file itu.

Log hanya berisi catatan teknis tentang jalannya aplikasi, misalnya nama file lagu dan nama perangkat audio. Tidak ada isi lagu di dalamnya.

---

## Yang belum ada di versi ini

Supaya tidak bingung mencari: visualizer spektrum dan warna nada, lirik, playlist (termasuk menyimpan antrean sebagai playlist), pengeditan tag, unduhan, scrobble Last.fm, mode hemat daya, dan pengaturan pintasan keyboard. Semuanya ada di tahap berikutnya.
