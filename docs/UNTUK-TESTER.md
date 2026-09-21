# Onsa — untuk kamu yang mau mencobanya

Onsa adalah pemutar musik untuk Windows. Ia memutar berkas musik yang sudah ada di komputermu — bukan layanan langganan, dan tidak ada katalog lagu di dalamnya. Yang kuminta darimu bukan menilai bagus atau tidaknya, melainkan memakainya seperti biasa dan memberitahuku di mana kamu tersendat: apa yang membingungkan, apa yang tidak kamu temukan, apa yang kamu kira akan terjadi tapi ternyata tidak.

## Kalau kamu berhenti dan tidak tahu harus apa

**Itu temuan paling berharga, dan itu yang paling kuinginkan.** Catat di mana kamu berhenti, lalu lanjut ke hal lain. Jangan cari tahu sendiri, dan jangan tanya aku dulu — begitu kamu bertanya, aku tidak bisa lagi tahu apakah Onsa bisa menjelaskan dirinya sendiri.

## Memasang

1. Jalankan berkas pemasangnya (`Onsa_1.2.0_x64-setup.exe`).
2. Windows kemungkinan besar menampilkan layar biru bertuliskan **"Windows melindungi PC Anda"** (*Windows protected your PC*). Itu bukan tanda ada yang salah dengan berkasnya: Windows menampilkan itu untuk setiap program yang belum ditandatangani secara digital, dan menandatanganinya butuh sertifikat berbayar yang belum kubeli. Klik **"Info selengkapnya"** (*More info*), lalu **"Tetap jalankan"** (*Run anyway*).
3. Setelah terpasang, Onsa ada di Start Menu dengan nama **Onsa**.

## Sebelum mulai

- **Onsa butuh satu folder musik.** Saat pertama dibuka, ia menanyakan folder tempat lagu-lagumu disimpan. Ia membaca isinya, tidak memindahkan dan tidak mengubah apa pun di dalamnya.
- **Onsa tidak mengubah berkasmu kecuali kamu menekan tombol tertentu.** Hanya empat hal yang bisa menyentuh berkas: **"Tulis ke berkas"** di jendela sunting satu lagu, **"Tulis ke file"** dan **"Ganti nama berpola"** di halaman Rapikan, dan tombol **"Simpan .lrc"** di panel lirik. Selain itu — memutar, playlist, pencarian, semua pengaturan suara, tema — tidak menyentuh berkas sama sekali. Perubahan yang kamu buat tersimpan di dalam Onsa dulu, dan bisa dibatalkan lewat daftar Riwayat.
- **Semua yang memakai internet mati sejak awal.** Tidak ada yang dikirim ke mana pun sampai kamu sendiri yang menyalakannya.
- **Di tiap halaman ada tombol tanda tanya** di kanan atas. Itu membuka penjelasan halaman itu. Silakan dipakai — tapi kalau kamu terpaksa membukanya untuk hal yang menurutmu seharusnya jelas sendiri, catat juga.

## Yang kuminta kamu coba

Kerjakan sesukamu, tidak harus berurutan, tidak harus semuanya.

1. Pasang Onsa dan bawa ia sampai lagu pertamamu terdengar.
2. Putar satu lagu sampai habis, lalu biarkan lagu berikutnya jalan tanpa kamu menyentuh daftarnya.
3. Cari satu lagu yang kamu tahu ada di komputermu, dari kotak pencarian.
4. Susun satu daftar berisi lima lagu untuk didengar besok pagi, lalu ubah urutannya.
5. Putar satu lagu yang kamu hafal liriknya, dan coba tampilkan liriknya di layar.
6. Ganti tampilan Onsa jadi terang, lalu kembalikan ke gelap.
7. Cari satu lagu yang judul atau artisnya salah tulis di komputermu, dan perbaiki namanya.
8. Rapikan sekaligus beberapa lagu dari satu album yang keterangannya berantakan.
9. Ambil satu lagu dari sebuah alamat web. Untuk ini Onsa perlu mengunduh dulu sebuah program bernama yt-dlp — **itu wajar dan memang bagian dari caranya bekerja**; ia menanyakannya dulu dan tidak mengunduh apa pun sebelum kamu menekan tombolnya.
10. Tutup Onsa di tengah lagu, buka lagi, dan lihat apakah ia kembali seperti yang kamu tinggalkan.

## Yang perlu kamu tahu kalau sampai ke situ

Onsa bisa mengenali lagu **dari bunyinya sendiri**, berguna untuk berkas yang keterangannya kosong sama sekali. Itu **tidak wajib** dan butuh dua hal yang tidak datang otomatis: satu kunci gratis dari acoustid.org, dan satu program kecil bernama fpcalc. **Tanpa keduanya, pencarian lewat keterangan lagu tetap jalan** — dan itu yang dipakai untuk hampir semua lagu yang sudah punya judul dan artis. Jangan repot memasangnya kecuali kamu memang ingin mencobanya; kalau tidak, lewati saja bagian itu.

## Cara melapor

Untuk tiap masalah, tiga pertanyaan ini yang paling kubutuhkan:

1. **Apa yang kamu lakukan?**
2. **Apa yang kamu harapkan terjadi?**
3. **Apa yang sebenarnya terjadi?**

Kalau ada yang benar-benar salah — Onsa berhenti, tidak mau jalan, atau melakukan hal yang tidak masuk akal — catatannya membantu:

1. Buka **Pengaturan → Tentang**, nyalakan **"Log debug"**.
2. Tutup Onsa, buka lagi, lalu kerjakan sekali lagi hal yang bermasalah itu.
3. Tekan **"Buka folder log"**, dan kirimkan berkas hari itu.
4. Matikan lagi "Log debug" sesudahnya, karena catatannya cepat membesar.

**Sebelum mengirim, lihat dulu isinya.** Catatan itu memuat jalur folder dan nama lagu di komputermu. Kalau ada yang tidak ingin kamu kirimkan, katakan saja dan kita cari cara lain.

## Yang memang belum bisa, jadi tidak perlu dilaporkan

- Alamat web yang audionya hanya tersedia dalam format Opus ditolak dengan pesan — Onsa belum bisa memutar format itu.
- Folder musik belum bisa dihapus dari dalam Onsa; yang ada baru menambah dan memeriksa ulang.
- Mengubah hasil unduhan jadi MP3 atau FLAC butuh program bernama ffmpeg, yang tidak dipasang Onsa sendiri. Tanpa ffmpeg unduhan tetap jalan, hanya tanpa perubahan bentuk dan tanpa sampul tertanam.
- Onsa belum pernah dicoba saat disk penuh. Kalau kamu kebetulan mengalaminya, itu justru menarik — ceritakan apa yang kamu lihat.
