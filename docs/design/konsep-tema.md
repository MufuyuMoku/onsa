# Onsa — Konsep tema untuk M16 (v1.5 "Tampilan")

Disetujui pemilik proyek pada 2026-10-10. Dokumen ini adalah "dokumen konsep yang disetujui" yang diminta SPEC §15 M16, untuk keenam tema sekaligus.

Referensi visualnya ada di folder `referensi/`: satu berkas HTML per tema, berupa tiruan layar Sedang diputar di layout khasnya. Berkas itu acuan rupa (warna, bentuk, susunan), bukan kode yang disalin. Angka yang tertulis di dokumen ini menang atas yang terlihat di berkas referensi.

---

## 1. Keputusan umum

Semua keputusan di bawah membuka atau mengubah aturan yang selama ini terkunci. Catat di DECISIONS dan perbarui SPEC sesuai bagian 9.

### 1.1 Enam tema baru menggantikan enam yang lama

| # | Nama | id | Gelap/terang | Menggantikan |
|---|---|---|---|---|
| 1 | **Kokpit Tua** | `kokpit-tua` | gelap, **tema pembuka** | Kokpit kaca, Deck malam |
| 2 | Laut Dalam | `laut-dalam` | gelap | Kaca asap |
| 3 | Jendela Biru | `jendela-biru` | terang | Kubikel biru |
| 4 | Lorong Kuning | `lorong-kuning` | terang | — |
| 5 | Siang Mimpi | `siang-mimpi` | terang | — |
| 6 | Krom Saku | `krom-saku` | terang | Kilau milenium |

Musim dingin utara dihapus tanpa pengganti langsung.

- Tetap enam tema bawaan. Onsa tetap membuka dengan tema gelap, yaitu **Kokpit Tua**.
- **Pilihan tema lama yang tersimpan dipetakan**: `kaca-asap` → `laut-dalam`; `kokpit-kaca` dan `deck-malam` → `kokpit-tua`; `kubikel-biru` → `jendela-biru`; `kilau-milenium` → `krom-saku`; `musim-dingin` → `kokpit-tua`. Tidak ada orang yang membuka Onsa lalu mendapati temanya hilang tanpa penjelasan.
- Tema buatan pengguna dari skema lama tetap mengikuti aturan M16 yang sudah ada: terbaca, atau ditolak dengan alasan yang jelas di halaman Tampilan.

### 1.2 Batas meniru

- **Boleh**: meniru rasa dan susunan sebuah era atau gaya — warna yang mirip, bilah judul biru membulat, kertas dinding kuning bergaris, krom dan plastik bening. Semuanya digambar ulang dengan CSS atau gambar buatan sendiri.
- **Tidak boleh**: aset asli (wallpaper, ikon, logo, suara, foto), nama produk atau merek sebagai nama tema atau teks di layar, dan font milik pihak lain.
- Tiap tema punya daftar "Dilarang"-nya sendiri di bawah. Daftar itu tambahan, bukan pengganti aturan umum ini.

### 1.3 Font per tema

- Tiap tema boleh membawa font sendiri. Syaratnya: lisensinya OFL atau lebih bebas, dibundel di `ui/static/fonts/` beserta berkas lisensinya, dan tidak pernah dimuat dari internet (SPEC §14 tetap berlaku).
- Ambil hanya bobot yang benar-benar dipakai.
- Noto Sans JP tetap jadi fallback CJK untuk semua tema.
- Font lama yang tidak dipakai tema mana pun lagi (Chakra Petch, Share Tech Mono, Barlow, Barlow Condensed, B612 biasa bila tidak terpakai) dihapus dari bundel beserta lisensinya.
- Daftar font baru ada di bagian 8.

### 1.4 Dekorasi boleh, dengan tiga syarat

Aturan lama "tidak ada dekorasi tanpa fungsi" (SPEC §9.1) dilonggarkan. Dekorasi boleh, asal:

1. **Tidak pernah mengganggu keterbacaan** meter, waktu, judul, lirik, dan semua teks kontrol.
2. **Dekorasi yang bergerak mati** di mode hemat daya dan saat `prefers-reduced-motion`. Dekorasi diam boleh tetap ada.
3. **Tidak ada kedip cepat**: tidak lebih dari satu kedipan per detik, dan tidak ada kilatan terang besar (aturan keselamatan fotosensitif).

Dan satu aturan tambahan: **tidak ada tombol palsu**. Apa pun yang terlihat seperti tombol harus benar-benar berfungsi. Kalau tidak berfungsi, jangan dibuat terlihat seperti tombol.

### 1.5 Teks hiasan

Beberapa tema membawa kalimat hiasan ("kamu pernah ke sini sebelumnya", "TIDAK ADA ORANG LAIN DI SINI", "ANTI-SKIP 40s").

- Teks hiasan masuk kamus i18n (`id` dan `en`) seperti teks lain, di bawah kunci milik temanya.
- Teks hiasan tidak pernah menggantikan nama fungsi. Nama menu, tombol, dan pengaturan tetap kata yang sama di semua tema. (Di papan referensi Laut Dalam, menu tertulis "MENYELAM"; di Onsa tetap "Sedang diputar".)
- Teks hiasan disembunyikan dari pembaca layar (`aria-hidden`).

### 1.6 Layout khas per tema

Preset layout tetap sumbu terpisah dari tema (SPEC M16). Yang ditambahkan:

- **Tiap tema punya satu layout khas.** Saat tema dipilih, layout khasnya ikut menjadi bawaan. Pengguna tetap bisa memilih preset lain sesudahnya.
- **Ada tiga preset standar** yang wajib bisa dipakai bersama **setiap** tema:
  - **Klasik**: susunan Onsa sekarang (sidebar, konten, panel kanan, transport, strip jalur sinyal).
  - **Galeri**: sidebar, grid sampul untuk library, bar pemutar di bawah.
  - **Layar penuh**: sampul atau visual besar dengan lirik, kontrol minimum.
- Layout khas hanya tersedia untuk temanya sendiri.
- Saat tema dipakai di preset standar, ia tetap tampil dengan rupanya (warna, font, material), tapi tanpa susunan khasnya. Contoh: Laut Dalam di Klasik memakai progress bar horizontal biasa dengan warna laut, bukan skala kedalaman.
- Layout khas, seperti preset lain, adalah data (lanjutan dari `setPanels` di M14), bukan cabang kode yang menyebut nama tema.

---

## 2. Kokpit Tua (`kokpit-tua`) — tema pembuka

**Watak**: panel instrumen pesawat tua. Logam berpaku keling, pelat label hitam, jarum di atas muka meter krem, lampu indikator pijar.

**Material**: pelat logam olive-hitam dengan tekstur sikat sangat halus, paku keling di sudut panel, bingkai instrumen hitam tebal.

**Palet**

| Peran | Warna | Catatan |
|---|---|---|
| latar | `#1C1F1A` | |
| panel | `#2A2E26` | bagian dalam panel `#24271F` |
| bingkai instrumen | `#0E0F0C` | |
| garis | `#3A3D33` | |
| teks utama | `#F2E6C2` | |
| `label` | `#C9C2AE` | pelat label: latar `#111111`, teks `#EDE6D2` |
| muka meter | `#E9E0C6` | angka dan tanda di atasnya `#1A1A16` |
| `adjustable` | `#EDE6D2` | |
| `active` | `#7BE07A` | lampu hijau menyala |
| `position` | `#F2E6C2` | |
| `caution` | `#E8A23A` | garis peringatan `#E2B53A` / hitam |
| `clip` | `#C8401F` | juga warna jarum VU |
| tepi terang / gelap | `#5A5D50` / `#0B0C09` | |
| warna nada | hangat `#D9603A`, dingin `#8FC7D8` | yang bergeser: spektrum |

**Font**: Saira Stencil One untuk pelat label, judul, dan menu. B612 Mono untuk angka, waktu, format, dan lirik.

**Komponen**
- Kontrol: kotak bertepi timbul (`bevel`), sudut kecil.
- Meter: **jarum VU bulat** L/R dengan muka krem.
- Spektrum: batang tipis putih krem di atas muka instrumen hitam, dengan garis bantu horizontal samar.
- Waktu: angka B612 Mono, dan di layout khas juga sebagai jam jarum.
- Indikator tahap: lampu bulat yang menyala hijau, padam gelap.
- Bingkai panel: timbul (`raised`) dengan paku keling.

**Gerak**: cepat dan mekanis (`snap`). Jarum VU bergerak dengan balistik jarum sungguhan, tidak melompat.

**Dekorasi**: paku keling, garis peringatan kuning-hitam di bawah sidebar, tekstur sikat. Semuanya diam.

**Layout khas "Panel Instrumen"**: sampul dalam bingkai instrumen bergaris silang. Di sebelahnya judul dan lirik. Posisi putar ditampilkan sebagai **jam jarum berlabel MENIT**, di samping spektrum. VU jarum L/R ada di transport. Tiap bagian diberi pelat label hitam (SAMPUL, POSISI, SPEKTRUM).

**Dilarang**: logo maskapai, pabrikan pesawat, atau angkatan udara; insignia militer sungguhan; foto kokpit asli; nama tipe pesawat sungguhan.

---

## 3. Laut Dalam (`laut-dalam`)

**Watak**: menyelam ke laut dalam. Makin ke bawah makin gelap; satu-satunya cahaya berasal dari makhluk yang bercahaya sendiri.

**Material**: tidak ada panel. Antarmuka melayang di air gelap. Garis tipis, tanpa bingkai.

**Palet**

| Peran | Warna | Catatan |
|---|---|---|
| latar | gradien vertikal `#0E2C42` → `#071826` → `#030B12` → `#010306` | makin gelap ke bawah |
| garis | `#0E2230`, `#1C3A4C` | |
| teks utama | `#E8F4F8` | |
| teks kedua | `#8FB0C0` | |
| `label` | `#5C7A8C` | |
| `adjustable` | `#9FD8EA` | |
| `active` | `#7FE3FF` | |
| `position` | `#DFFBFF` | dengan cahaya `#7FE3FF` |
| aksen kedua | `#D59BFF` | sebagian plankton |
| `caution` | `#FFC56B` | |
| `clip` | `#FF5A4A` | |
| tepi terang / gelap | `#1C3A4C` / `#010306` | |
| warna nada | hangat `#D59BFF`, dingin `#A8FFE0` | yang bergeser: spektrum |

**Font**: Cormorant Garamond untuk judul, nama artis, dan lirik. Martian Mono untuk label, menu, angka, dan format.

**Komponen**
- Kontrol: lingkaran bergaris tipis tanpa isi. Tombol putar sedikit bercahaya.
- Meter: garis horizontal tipis yang bercahaya.
- Spektrum: **titik-titik plankton** yang menyala, bertumpuk vertikal, sebagian ungu.
- Waktu: Martian Mono tipis.
- Indikator tahap: teks yang menyala `#7FE3FF` bila aktif.
- Bingkai panel: tidak ada (`none`).

**Gerak**: pelan dan lembut (`soft`), tetap dalam batas jepit skema.

**Dekorasi**
- Partikel "salju laut" melayang sangat pelan di layar Sedang diputar (bergerak; mati di hemat daya dan gerak dikurangi).
- **Umpan cahaya** ikan pemancing menggantung dari atas dan menerangi sudut sampul. Sampul tetap harus jelas terlihat: bagian terjauh dari umpan paling banyak digelapkan 50%.

**Layout khas "Menyelam"**: di kanan ada **skala kedalaman vertikal** yang menggantikan progress bar. Awal lagu 0 m, akhir lagu adalah dasar (4000 m). Penanda menunjukkan kedalaman dan waktu sekarang. Skala itu bisa diklik dan diseret untuk seek, persis progress bar. Zona ditandai (SENJA 200 m, MALAM 1000 m, DASAR 4000 m). Judul serif besar di sebelah sampul, lirik di bawahnya, spektrum plankton di dasar.

**Dilarang**: aset dari game atau film laut dalam tertentu; makhluk yang sengaja menakutkan atau jumpscare; lambang angkatan laut sungguhan.

---

## 4. Jendela Biru (`jendela-biru`)

**Watak**: rasa sistem operasi desktop awal 2000-an. Bilah judul biru mengilap, tombol krem timbul, tombol putar hijau bulat, panel tugas biru di kiri, bilah tugas di bawah.

**Material**: plastik mengilap pada bilah judul dan tombol, kotak krem dengan tepi timbul dan cekung 3D.

**Palet**

| Peran | Warna | Catatan |
|---|---|---|
| latar luar | `#3A6EA5` | |
| bingkai jendela | `#0831D9` | |
| bilah judul | gradien `#3D95FF` → `#0055E5` → `#0638B0` | teks putih bayangan `#0A1F63` |
| badan | `#ECE9D8` | |
| panel tugas | gradien `#7BA2E7` → `#6375D6` | kotak `#D6DFF7`, kepala kotak teks `#215DC6` |
| teks utama | `#000000` | |
| `label` | `#555555` | |
| sorotan pilihan | `#316AC5` | teks putih |
| `adjustable` | `#0046D5` | |
| `active` | `#3EBE3F` | |
| `position` | `#3EBE3F` | progress bersegmen |
| `caution` | `#E0A21F` | |
| `clip` | `#C33A1C` | |
| tombol | gradien `#FFFFFF` → `#ECEBE6` → `#D6D0C5`, tepi `#003C74` | |
| tepi terang / gelap | `#FFFFFF` / `#808080` | |
| warna nada | hangat `#F0A030`, dingin `#5FE0C0` | yang bergeser: spektrum (dasar `#57A0FF`) |

**Font**: **DejaVu Sans Condensed**. Bentuknya sekeluarga dengan font sistem era itu, dan lisensinya bebas dibundel. Bold untuk bilah judul. Teks di bilah tugas bold miring.

**Komponen**
- Kontrol: kotak krem membulat 3 px, bertepi timbul. Tombol putar bulat hijau mengilap.
- Meter: bar horizontal hijau dalam kotak putih bertepi.
- Spektrum: batang gradien biru di dalam kotak hitam-biru cekung.
- Progress: **blok hijau bersegmen**.
- Indikator tahap: centang hijau dalam kotak cekung.
- Bingkai panel: kotak bergrup (`fieldset`) dan tepi cekung.
- Dialog: berbilah judul biru (`titled`).

**Gerak**: patah dan cepat (`snap`).

**Layout khas "Jendela"**: aplikasi digambar sebagai sebuah jendela, dengan bilah judul, baris menu, panel tugas berkelompok di kiri (Musik, Alat), dan bilah tugas di bawah berisi lagu yang sedang diputar dan jam.

Dua hal yang harus diputuskan Claude Code dan dilaporkan sebelum dikerjakan:
1. **Bilah judul**: Onsa sudah punya bilah judul dari sistem operasi. Pilihannya: jendela tanpa bingkai sistem dengan bilah judul buatan sendiri yang **berfungsi penuh** (seret, perkecil, perbesar, tutup), atau bilah judul hiasan di dalam aplikasi **tanpa tombol perkecil, perbesar, dan tutup**. Tombol yang tidak berfungsi tidak boleh ada (1.4).
2. **Baris menu** (Berkas, Putar, Tampilan, Alat, Bantuan): harus benar-benar membuka menu ke aksi yang sudah ada, atau tidak ditampilkan.

Jam di bilah tugas adalah jam sungguhan.

**Dilarang**: logo sistem operasi, bendera empat warna, wallpaper bukit hijau, ikon asli, suara sistem, kata "Start", nama produk sistem operasi, dan font Tahoma, Trebuchet, Verdana, atau Franklin Gothic.

---

## 5. Lorong Kuning (`lorong-kuning`)

**Watak**: ruang kantor kosong yang terlalu tenang. Kertas dinding kuning bergaris, lampu neon, karpet lembap, dan kertas memo yang tertinggal.

**Material**: kertas dinding bergaris, kertas memo dengan bayangan keras, karpet berbintik.

**Palet**

| Peran | Warna | Catatan |
|---|---|---|
| dinding | `#C9B458` | garis vertikal `rgba(120,98,30,0.10)` tiap 22 px |
| plafon | `#DCD6B4` | lampu `#FBFAEA` dengan cahaya lembut |
| kartu memo | `#E9DFA4` | tepi `#B5A55A`, bayangan `2px 3px 0 rgba(90,74,20,0.18)` |
| teks utama | `#3A3218` | |
| `label` | `#6B5E2A` | |
| teks redup | `#8A7C42` | |
| karpet | `#8A7A44` berbintik | garis atas `#5E5228` |
| strip bawah | `#5E5228` | teks `#E9DFA4` |
| sorotan | `#FBFAEA` | baris lirik aktif |
| `adjustable` | `#3A3218` | |
| `active` | `#2E7D3A` | hijau papan KELUAR |
| `position` | `#3A3218` | |
| `caution` | `#C27A1A` | |
| `clip` | `#B0301C` | |
| tepi terang / gelap | `#FBFAEA` / `#6B5E2A` | |
| warna nada | hangat `#8A3A1A`, dingin `#1F5A52` | yang bergeser: spektrum (dasar `#4A4122`) |

**Font**: Courier Prime untuk semua teks, seperti memo yang diketik. VT323 untuk cap waktu ala handycam.

**Komponen**
- Kontrol: kotak datar bertepi tegas, tanpa sudut membulat.
- Meter: bar horizontal hijau di atas kartu.
- Spektrum: batang cokelat gelap padat di atas kartu memo.
- Waktu: **cap waktu handycam** (`▶ PLAY SP 00:02:14`) di pojok, putih dengan bayangan. Angkanya posisi putar sungguhan.
- Indikator tahap: `[rg]` `[eq]` dalam kurung, hijau bila aktif.
- Bingkai panel: kartu dengan bayangan keras.

**Gerak**: biasa (`standard`).

**Dekorasi**
- Lampu neon di plafon. Satu lampu boleh **meredup pelan sesekali** (paling sering sekali tiap 20 detik, tanpa kilatan). Mati di hemat daya dan gerak dikurangi.
- Papan "KELUAR" hijau di bawah sidebar adalah **papan, bukan tombol**: tidak terlihat bisa diklik dan disembunyikan dari pembaca layar.
- Teks hiasan "TIDAK ADA ORANG LAIN DI SINI" (1.5).

**Layout khas "Lorong"**: plafon dengan lampu neon di atas, dinding sebagai latar, sidebar dan informasi lagu di atas kertas memo, sampul sebagai foto tertempel, transport di atas karpet, dan cap waktu handycam di dua pojok atas.

**Dilarang**: foto asli yang beredar dengan nama itu (fotonya ada pemiliknya), entitas atau monster, kata "Backrooms" di mana pun di layar, dan istilah fandom ("Level 0"). Ganti teks "level 0 · ruang 3" di papan referensi dengan teks hiasan lain.

---

## 6. Siang Mimpi (`siang-mimpi`)

**Watak**: mimpi siang hari yang terasa akrab tapi janggal. Langit yang terlalu terang, padang rumput, dan sebuah pintu yang berdiri sendirian.

**Material**: langit bergradien dengan cahaya berlebih, awan buram, rumput, kotak putih bertepi gelap dengan bayangan keras seperti web lama.

**Palet**

| Peran | Warna | Catatan |
|---|---|---|
| langit | gradien `#7FB2EC` → `#A9CFF5` → `#E4F1FF` → `#F7FBFF` | cahaya putih dari atas |
| rumput | gradien `#B9E08E` → `#8CCB5E` → `#6CB04A` | |
| teks utama | `#2B2B5A` | |
| `label` | `#4A5A8A` | |
| lirik tidak aktif | `#2B2B5A` transparan | minimal 3:1 terhadap langit di belakangnya |
| kotak dan tombol | `#FFFFFF`, tepi `#2B2B5A`, bayangan `3px 3px 0 rgba(43,43,90,0.25)` | sorotan `#FFF6C9` |
| pintu | `#F4EEDF`, bingkai `#C8BCA0` | |
| teks hiasan | `#A8326E` | sengaja beda dari `clip` |
| `adjustable` | `#2B2B5A` | |
| `active` | `#2E8A2E` | |
| `position` | `#5A7FD8` | progress bergaris `#8FC0F0` / `#B9D8F8` |
| `caution` | `#E0A21F` | |
| `clip` | `#D0312D` | |
| tepi terang / gelap | `#FFFFFF` / `#2B2B5A` | |
| warna nada | hangat `#E89A3A`, dingin `#3FB8A8` | yang bergeser: spektrum rumput (dasar `#7CC45A`) |

**Font**: Gloock untuk judul, nama aplikasi, dan lirik. Pixelify Sans untuk menu, label, dan angka.

**Komponen**
- Kontrol: kotak putih bertepi gelap dengan bayangan keras.
- Meter: bar hijau dalam kotak putih bertepi.
- Spektrum: **rumput** — helai segitiga yang tumbuh dari tanah mengikuti suara.
- Progress: bar bergaris dengan **bintang kecil** sebagai penanda posisi.
- Indikator tahap: teks hijau bila aktif.
- Bingkai panel: kotak putih bertepi (`hairline`) dengan bayangan keras.

**Gerak**: lembut (`soft`).

**Dekorasi** (disetujui pemilik proyek)
- Awan buram yang bergerak sangat pelan, dan kilau bintang. Keduanya mati di hemat daya dan gerak dikurangi.
- **Satu mata kecil di langit**, digambar sederhana dan samar, diam, tidak berkedip, tidak mengikuti kursor.
- **Teks hiasan merah anggur** "kamu pernah ke sini sebelumnya." di atas judul (1.5). Kalimatnya tenang, tidak mengancam.

**Layout khas "Padang"**: seluruh aplikasi berada di dalam pemandangan. Menu berupa kotak-kotak putih di kiri atas langit. Judul dan lirik melayang di langit. **Sampul terlihat di balik pintu yang terbuka**, berdiri di atas rumput dengan bayangannya. Spektrum adalah rumput di sepanjang padang. Transport berupa strip bawah bergaya web lama.

**Dilarang**: foto orang atau anak-anak; mata realistis, banyak mata, atau mata yang mengikuti kursor; aset dari karya dreamcore tertentu; kalimat hiasan yang mengancam atau menakutkan.

---

## 7. Krom Saku (`krom-saku`)

**Watak**: pemutar CD saku sekitar tahun 2000. Krom perak, plastik biru bening, layar LCD hijau-abu, dan stiker hologram.

**Material**: krom bergradien dengan garis sikat halus, plastik bening berwarna yang menangkap cahaya, layar LCD cekung.

**Palet**

| Peran | Warna | Catatan |
|---|---|---|
| latar | perak `#E7EAF0` / `#CFD5DE` | garis sikat halus |
| krom | gradien `#FDFDFE` → `#C9CED6` → `#9AA1AC` → `#E6E9EE`, tepi `#7E8794` | |
| plastik biru | gradien `rgba(120,190,255,0.55)` → `rgba(40,120,230,0.35)` | kilau putih di atas |
| plastik merah muda | gradien `rgba(255,190,230,0.55)` → `rgba(255,120,190,0.25)` | kotak lirik |
| LCD | latar `#B4C49B`, tepi `#6E7B5C`, piksel `#1F2A14` | |
| stiker hologram | `#FFD6F5`, `#C9F2FF`, `#E9FFC9`, `#FFE3B8`, `#E3C9FF` | |
| teks utama | `#18305A` | |
| `label` | `#3E5B8C` | |
| `adjustable` | `#1E5FD0` | |
| `active` | `#1A8A4A` | |
| `position` | `#FF5FA8` | |
| `caution` | `#E08A1A` | |
| `clip` | `#D8232A` | |
| tepi terang / gelap | `#FFFFFF` / `#7E8794` | |
| warna nada | hangat `#E8C9A0`, dingin `#9CCFD8` | yang bergeser: **lampu latar LCD** (dasar `#B4C49B`), bukan pikselnya |

**Font**: Orbitron untuk LCD, logo, dan angka. Exo 2 untuk judul, menu, dan lirik.

**Komponen**
- Kontrol: tombol krom bulat. Tombol putar plastik biru bening, lebih besar.
- Meter: LCD bersegmen.
- Spektrum: LCD bertitik bersegmen.
- Waktu: angka LCD besar di atas judul.
- Indikator tahap: teks hijau di strip bawah.
- Bingkai panel: plastik bening membulat besar.

**Gerak**: biasa (`standard`).

**Dekorasi**
- **Cakram yang berputar pelan** selama lagu diputar, berhenti saat jeda. Mati di hemat daya dan gerak dikurangi.
- Stiker hologram berisi nama aplikasi.
- Teks hiasan "ANTI-SKIP 40s ✓" (1.5).

**Layout khas "Pemutar Saku"**: cakram besar dalam cincin krom di kiri. **Sampul lagu sungguhan tampil sebagai label di tengah cakram**, dengan lubang di tengahnya. Di permukaan cakram ada pantulan pelangi. Di kanan ada LCD (nomor trek, waktu, format), lalu judul, artis, dan lirik di kotak plastik merah muda. Spektrum di LCD lebar. Transport krom di bawah.

**Dilarang**: logo atau nama merek pemutar CD dan gadget sungguhan; bentuk produk tertentu yang ditiru persis; logo cakram resmi ("Compact Disc").

---

## 8. Font yang dibundel

| Font | Dipakai di | Lisensi |
|---|---|---|
| Saira Stencil One | Kokpit Tua | OFL |
| B612 Mono | Kokpit Tua | OFL (sudah dibundel) |
| Cormorant Garamond | Laut Dalam | OFL |
| Martian Mono | Laut Dalam | OFL |
| DejaVu Sans Condensed | Jendela Biru | lisensi Bitstream Vera / DejaVu (bebas, bukan OFL — periksa dan catat) |
| Courier Prime | Lorong Kuning | OFL |
| VT323 | Lorong Kuning | OFL |
| Gloock | Siang Mimpi | OFL |
| Pixelify Sans | Siang Mimpi | OFL |
| Orbitron | Krom Saku | OFL |
| Exo 2 | Krom Saku | OFL |
| Noto Sans JP | semua (fallback CJK) | OFL (sudah dibundel) |

Sumber, versi yang dipatok, dan ukuran total sesudahnya dicatat di DECISIONS, seperti saat font dibundel pertama kali. Pastikan tiap font bisa menampilkan huruf Indonesia dan tanda baca yang dipakai Onsa. Font yang tidak punya huruf tertentu jatuh ke fallback, dan itu harus terlihat wajar.

---

## 9. Yang perlu diubah di SPEC

- **§1 Tema bawaan**: enam tema baru (tabel 1.1), dua gelap dan empat terang. Onsa membuka dengan Kokpit Tua.
- **§9.1**: aturan dekorasi diganti dengan 1.4, ditambah aturan "tidak ada tombol palsu".
- **§9.3**: skema tema bertambah: font per tema (1.3), teks hiasan (1.5), dan layout khas (1.6).
- **§9.4**: diganti dengan ringkasan bagian 2–7 dokumen ini, dengan rujukan ke dokumen ini sebagai rinciannya.
- **M16**: kalimat "jumlah dan nama keenam tema bawaan tidak berubah" dihapus. Tambahkan tiga preset standar dan layout khas.

Simpan dokumen ini di repo sebagai `docs/design/konsep-tema.md`, dan folder `referensi/` di `docs/design/konsep-tema/`. `palet-preview.html` lama boleh dihapus sesudah tema baru selesai.

---

## 10. Kriteria selesai M16 (pengganti yang lama)

Di kedua OS:

1. Keenam tema tampil sesuai dokumen ini dan berkas referensinya, di layout khasnya masing-masing. Tangkapan layar tiap tema dikirim ke pemilik proyek untuk dinilai.
2. Tiap tema bisa dipakai di ketiga preset standar, dan tiap preset bisa dipakai dengan keenam tema: 18 susunan preset standar ditambah 6 layout khas, semuanya diperiksa pemeriksa tata letak yang sudah ada (tidak ada tumpang tindih, tidak ada gulir ke samping, tidak ada teks terpotong tanpa elipsis).
3. Teks utama lolos WCAG AA di keenam tema; lirik tidak aktif dan label minimal 3:1. Warna yang gagal digelapkan atau diterangkan, lalu angkanya dilaporkan.
4. Warna nada lolos kriteria yang sudah dijaga tes (CIEDE2000 dan beda rona), dengan ujung-ujung usulan dokumen ini. Ujung yang gagal diganti dan dilaporkan.
5. Semua dekorasi yang bergerak berhenti di mode hemat daya dan saat gerak dikurangi, dan tidak ada yang berkedip lebih dari sekali per detik.
6. Tidak ada font yang dimuat dari internet; semuanya dibundel dengan lisensinya.
7. Pilihan tema lama yang tersimpan berpindah sesuai pemetaan 1.1.
8. Tidak ada tombol palsu di layout mana pun.
