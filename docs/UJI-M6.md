# Uji M6: playlist biasa, playlist pintar, dan M3U8

Lanjutan dari `UJI-M5.md`. Yang di sana masih berlaku; ini tambahannya saja.

File: `dist-test/Onsa-M6.exe`. Tinggal klik dua kali, tidak perlu dipasang. Semua setelan dan library-mu tetap seperti sebelumnya — playlist masuk lewat migrasi database, jadi tidak ada yang hilang.

---

## 1. Playlist biasa

Di sidebar sekarang ada **Playlist**, di bawah Folder.

1. Klik **Playlist baru**, beri nama, lalu **Buat**. Onsa langsung membuka playlist kosongnya.
2. Kembali ke **Lagu**, klik kanan sebuah lagu. Di bagian bawah menunya ada **Tambah ke playlist** dengan daftar playlist biasa milikmu. Pilih salah satu.
3. Buka playlist itu lagi.

**Yang seharusnya terjadi:** lagunya ada di sana, dalam urutan yang kamu buat. Setiap baris punya tiga tombol di kanan: naikkan, turunkan, dan hapus dari playlist. Baris juga bisa ditarik-lepas ke tempat lain. Klik dua kali sebuah baris untuk memutar playlist mulai dari situ.

Yang perlu diperiksa:

- Lagu yang sama boleh dimasukkan dua kali; keduanya berdiri sendiri, dan menghapus yang satu tidak menghapus yang lain.
- Menghapus baris tidak meninggalkan lubang di penomorannya.
- Tombol **⋯** di kanan atas: ganti nama, gandakan, ekspor, hapus. Menghapus playlist **tidak** menghapus lagunya dari library.

## 2. Playlist pintar

Klik **Playlist pintar baru**.

1. **Tambah aturan** — misalnya *Genre* · *mengandung* · `rock`.
2. Perhatikan baris **Pratinjau** di bawah: jumlahnya ikut berubah sambil kamu mengetik, sebelum apa pun disimpan.
3. Atur **Urutkan** dan **Batas** kalau mau, lalu **Buat**.

**Yang seharusnya terjadi:** playlist itu berisi lagu yang cocok, dan tidak menyimpan daftarnya sendiri. Buktinya:

- Tambahkan lagu baru ke folder library-mu yang cocok dengan aturannya, tunggu scan selesai, lalu buka playlist pintarnya lagi. Lagu barunya sudah ada di sana tanpa kamu sentuh.
- **Cocokkan: semua aturan / salah satu aturan** mengubah artinya; coba dua aturan dengan keduanya.
- Kolom yang dipilih menentukan perbandingan yang ditawarkan. Kolom teks memberi *mengandung / sama dengan / diawali*, kolom angka memberi *lebih dari / antara*, dan kolom tanggal memberi *dalam kurun … hari*. Itu disengaja: aturan yang tidak masuk akal tidak bisa dibuat.
- Playlist pintar hanya bisa diubah lewat **⋯ → Ubah aturan**; barisnya tidak bisa ditarik atau dihapus satu-satu, karena yang menentukan isinya adalah aturannya.

Aturan yang layak dicoba: *Rating* · *paling sedikit* · `4`; *Terakhir diputar* · *tidak dalam kurun* · `30` hari; *Folder* · *mengandung* · nama folder.

## 3. Simpan antrean sebagai playlist

Di panel antrean, tombol keempat (ikon playlist) menyimpan antrean apa adanya jadi playlist baru. Namanya sudah diisi tanggal hari ini dan tinggal diganti.

**Yang seharusnya terjadi:** playlist barunya berisi lagu antrean dalam urutan yang sama, termasuk lagu yang muncul dua kali.

## 4. M3U8 keluar dan masuk

**Ekspor**: buka sebuah playlist, **⋯ → Ekspor M3U8**, simpan di mana saja. Kalau kamu menyimpannya **di sebelah musiknya**, path di dalam file ditulis relatif, jadi playlist itu ikut pindah kalau foldernya dipindah. Kalau disimpan jauh dari musiknya, path-nya ditulis lengkap.

File-nya bisa dibuka dengan Notepad: baris `#EXTM3U`, lalu sepasang baris `#EXTINF` dan path untuk tiap lagu.

**Impor**: **Playlist → Impor M3U8**, pilih file .m3u8 (punya Onsa atau punya pemutar lain). Onsa membuat playlist baru bernama sama dengan nama filenya.

**Yang seharusnya terjadi:** kalau ada baris yang lagunya tidak ada di library-mu, Onsa **mengatakan berapa banyak** yang tidak ditemukan, bukan membuangnya diam-diam. Sisanya tetap masuk.

Layak dicoba: ekspor sebuah playlist, pindahkan file .m3u8-nya ke folder lain, lalu impor dari sana.

## 5. Perbaikan yang ikut masuk

Saat memverifikasi M6 ketahuan satu bug lama: **jendela yang diminimalkan menyimpan ukuran dan tempat yang omong kosong**. Kalau Onsa diminimalkan lalu ditutup (atau sekadar kehilangan fokus saat kecil), sesi menyimpan ukuran 0×0 di posisi jauh di luar layar, dan Onsa berikutnya terbuka sekecil mungkin di pojok — bukan seperti yang kamu tinggalkan.

Cara memeriksanya: atur jendelanya ke ukuran yang kamu suka, minimalkan, lalu tutup lewat klik kanan di taskbar. Buka lagi: ukurannya harus persis yang tadi.

## 6. Yang paling perlu kuketahui

- Apakah editor aturan terasa masuk akal tanpa penjelasan? Kolom mana yang kamu cari tapi tidak ada?
- Apakah pratinjaunya cukup membantu, atau kamu ingin melihat daftar lagunya sekalian, bukan hanya jumlahnya?
- Apakah klik kanan → Tambah ke playlist sudah cukup, atau kamu ingin menambahkan satu album/artis sekaligus?
- Apakah playlist perlu muncul di sidebar sebagai daftar, bukan cuma satu tombol?

## 7. Kalau ada yang aneh

Sama seperti sebelumnya: **Pengaturan → Tentang → Buka folder log**, lalu kirimkan log-nya beserta apa yang kamu lakukan sebelum itu.
