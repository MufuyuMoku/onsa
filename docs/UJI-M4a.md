# Uji coba Onsa (M4a)

Ini versi coba-coba. Isinya sudah cukup untuk dipakai mendengarkan musik sungguhan: pilih folder, scan, jelajahi, putar, atur suara, dan ganti tampilan. Yang belum dikerjakan sengaja tidak ditampilkan, jadi kalau sebuah tombol ada, artinya tombol itu memang berfungsi.

Yang dibutuhkan: file `Onsa-M4a.exe`, folder musik, dan headphone atau speaker yang biasa dipakai.

## Cara menjalankan

1. Klik dua kali `Onsa-M4a.exe`. Tidak perlu dipasang.
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

Supaya tidak bingung mencari: visualizer spektrum, lirik, playlist, pengeditan tag, unduhan, scrobble Last.fm, mini player, kontrol lewat tombol media di keyboard, ikon di tray, dan pemulihan antrean saat aplikasi dibuka lagi. Semuanya ada di tahap berikutnya.
