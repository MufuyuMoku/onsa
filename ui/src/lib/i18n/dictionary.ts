/**
 * Interface text (SPEC section 9.6).
 *
 * No user visible string is ever written inside a component. Indonesian is
 * the reference dictionary; English has to answer every key it holds.
 * `{name}` marks a value filled in by `t(key, { name })`.
 */

export const LOCALES = ['id', 'en'] as const;

export type Locale = (typeof LOCALES)[number];

const id = {
	'shell.loading': 'Memuat…',

	'startup.title': 'Onsa tidak bisa membuka library-nya',
	'startup.newer':
		'Berkas library ini ditulis oleh Onsa versi yang lebih baru. Versi ini tidak bisa membacanya, dan memaksanya bisa merusak isinya.',
	'startup.newerWhat':
		'Pakai Onsa versi terbaru untuk membukanya. Kalau kamu memang ingin memakai versi ini, pindahkan dulu berkas itu ke tempat lain — Onsa akan membuat library baru yang kosong, dan yang lama tetap utuh untuk dibuka nanti.',
	'startup.unreadable':
		'Berkas library ini tidak bisa dibaca. Biasanya karena komputer mati saat Onsa sedang menulis, atau berkasnya rusak di disk.',
	'startup.unreadableWhat':
		'Pindahkan berkas itu ke tempat lain, lalu buka Onsa lagi: library baru akan dibuat dan folder musikmu dipindai ulang. Berkas musikmu sendiri tidak tersentuh oleh ini. Simpan yang lama kalau kamu ingin mencoba menyelamatkannya.',
	'startup.nothingTouched': 'Onsa tidak menghapus dan tidak memperbaiki berkas itu sendiri. Keputusannya ada padamu.',
	'startup.where': 'Berkasnya ada di',
	'startup.openFolder': 'Buka foldernya',
	'startup.said': 'Yang dilaporkan',

	'nav.library': 'Library',
	'nav.tracks': 'Lagu',
	'nav.albums': 'Album',
	'nav.artists': 'Artis',
	'nav.genres': 'Genre',
	'nav.folders': 'Folder',
	'nav.settings': 'Pengaturan',
	'settings.output': 'Output & Kualitas',
	'settings.dsp': 'DSP & EQ',
	'settings.library': 'Library',
	'settings.lyrics': 'Lirik',
	'settings.appearance': 'Tampilan',
	'settings.about': 'Tentang',

	'search.placeholder': 'Cari lagu, album, artis',
	'search.clear': 'Hapus pencarian',
	'search.tracks': 'Lagu',
	'search.albums': 'Album',
	'search.artists': 'Artis',
	'search.none': 'Tidak ada yang cocok.',

	'column.number': '#',
	'column.title': 'Judul',
	'column.artist': 'Artis',
	'column.album': 'Album',
	'column.year': 'Tahun',
	'column.duration': 'Durasi',
	'column.resize': 'Atur lebar kolom {name}',
	'column.resizeHint': 'Tarik, atau panah kiri/kanan',
	'column.choose': 'Pilih kolom',
	'column.reset': 'Kembalikan ke bawaan',
	'column.sortHint': 'Urutkan',

	'track.unknownArtist': 'Artis tak dikenal',
	'track.missing': 'hilang',
	'track.failed': 'tak terbaca',
	'track.playHint': 'Klik dua kali atau tekan Enter untuk memutar',

	'library.empty': 'Library masih kosong.',
	'library.tracks': '{n} lagu',

	'album.play': 'Putar album',
	'album.back': 'Semua album',
	'album.tracks': '{n} lagu',

	'transport.play': 'Putar',
	'transport.pause': 'Jeda',
	'transport.next': 'Berikutnya',
	'transport.previous': 'Sebelumnya',
	'transport.seek': 'Posisi putar',
	'transport.volume': 'Volume',
	'transport.nothing': 'Belum ada yang diputar',
	'transport.more': 'Lainnya',
	'transport.noOutput': 'Tidak ada output audio. Periksa perangkat di Output & Kualitas.',
	'transport.failed': 'Lagu tidak bisa diputar dan dilewati: {path}',

	'meter.label': 'Peak meter',
	'meter.left': 'L',
	'meter.right': 'R',
	'meter.vu': 'VU',
	'meter.clip': 'CLIP',
	'meter.limit': 'LIM',

	'spectrum.label': 'Spektrum',
	'spectrum.rest': 'Diam',

	'signal.label': 'Jalur sinyal',
	'signal.idle': 'Belum ada sinyal',
	'signal.hint': 'Klik: nyalakan atau matikan. Klik kanan: pengaturan.',
	'signal.resample': 'Resample',
	'signal.resampleShort': 'RS',
	'signal.replaygain': 'ReplayGain',
	'signal.replaygainShort': 'RG',
	'signal.more': 'Tahap lainnya',
	'signal.eq': 'EQ',
	'signal.limiter': 'Limiter',
	'signal.bands': '{n} band',
	'signal.filters': '{n} filter',
	'signal.album': 'album',
	'signal.track': 'lagu',

	'queue.title': 'Antrean',
	'queue.count': '{n} lagu',
	'queue.empty': 'Antrean kosong. Klik dua kali sebuah lagu untuk memutarnya.',
	'queue.shuffle': 'Acak',
	'queue.repeat': 'Ulang',
	'queue.repeatOff': 'Ulang: mati',
	'queue.repeatAll': 'Ulang: seluruh antrean',
	'queue.repeatOne': 'Ulang: satu lagu',
	'queue.clear': 'Kosongkan antrean',
	'queue.remove': 'Hapus dari antrean',
	'queue.playNext': 'Putar berikutnya',
	'queue.addToEnd': 'Tambah ke antrean',
	'queue.dragHint': 'Tarik untuk mengurutkan, atau Alt+panah',

	'lyrics.title': 'Lirik',
	'lyrics.nothingPlaying': 'Tidak ada yang diputar.',
	'lyrics.none': 'Tidak ada lirik untuk lagu ini.',
	'lyrics.looking': 'Mencari lirik…',
	'lyrics.offline': 'Pencarian lirik lewat internet sedang mati.',
	'lyrics.unsynced': 'Tanpa waktu',
	'lyrics.source.edited': 'Suntinganmu',
	'lyrics.source.file': 'Berkas .lrc',
	'lyrics.source.tag': 'Tag berkas',
	'lyrics.source.lrclib': 'LRCLIB',
	'lyrics.earlier': 'Lirik lebih awal',
	'lyrics.later': 'Lirik lebih lambat',
	'lyrics.offsetValue': 'Geser {seconds} d',
	'lyrics.resetOffset': 'Kembalikan geseran',
	'lyrics.lookAgain': 'Cari lagi',
	'lyrics.saveBeside': 'Simpan .lrc',
	'lyrics.seekHint': 'Klik baris untuk melompat ke sana',
	'lyrics.settingsTitle': 'Lirik',
	'lyrics.sourcesHint':
		'Onsa membaca berkas .lrc di sebelah lagu dan lirik yang tertanam di tag lagu, tanpa jaringan dan tanpa izin apa pun.',
	'lyrics.onlineLabel': 'Cari lirik di internet (LRCLIB)',
	'lyrics.onlineHint':
		'Yang dikirim hanya nama artis, judul, album, dan durasi lagu. Mematikannya juga menghentikan pencarian yang sedang berjalan.',
	'lyrics.writeBesideLabel': 'Simpan lirik dari internet sebagai .lrc di sebelah lagu',
	'lyrics.writeBesideHint': 'Dengan begitu liriknya jadi milikmu, dan tetap ada walau tanpa internet.',
	'nowPlaying.open': 'Sedang diputar',
	'nowPlaying.close': 'Kembali ke library',
	'nowPlaying.format': 'Format',
	'nowPlaying.output': 'Output',
	'mini.open': 'Mini player',
	'mini.close': 'Jendela penuh',
	'sleep.title': 'Sleep timer',
	'sleep.when': 'Berhenti',
	'sleep.minutes': 'Setelah {n} menit',
	'sleep.endOfTrack': 'Di akhir lagu ini',
	'sleep.customMinutes': 'Menit',
	'sleep.customTracks': 'Jumlah lagu',
	'sleep.action': 'Lalu',
	'sleep.pause': 'Jeda',
	'sleep.stop': 'Berhenti',
	'sleep.quit': 'Tutup Onsa',
	'sleep.fade': 'Volume turun selama',
	'sleep.start': 'Mulai',
	'sleep.cancel': 'Batalkan',
	'sleep.left': 'Sisa {time}',
	'sleep.tracksLeft': 'Sisa {n} lagu',
	'sleep.fading': 'Volume sedang turun',
	'tray.show': 'Tampilkan Onsa',
	'tray.play': 'Putar atau jeda',
	'tray.quit': 'Keluar',
	'tray.closeToTray': 'Tutup jendela hanya menyembunyikan Onsa ke tray',

	'firstRun.welcome': 'Selamat datang di Onsa',
	'firstRun.intro': 'Tiga langkah singkat, lalu musikmu siap diputar.',
	'firstRun.folderStep': '1 · Folder musik',
	'firstRun.folderHint': 'Pilih folder tempat lagu-lagumu disimpan. Onsa hanya membaca isinya.',
	'firstRun.pickFolder': 'Pilih folder…',
	'firstRun.changeFolder': 'Ganti folder…',
	'firstRun.dialogTitle': 'Pilih folder musik',
	'firstRun.themeStep': '2 · Tema',
	'firstRun.themeHint': 'Tampilan langsung berganti saat tema dipilih. Bisa diubah lagi kapan saja.',
	'firstRun.scanStep': '3 · Scan',
	'firstRun.start': 'Mulai scan',
	'firstRun.open': 'Buka library',
	'firstRun.moreStep': 'Selebihnya',
	'firstRun.moreHint':
		'Tiga hal yang mudah terlewat. Klik namanya untuk langsung ke sana; semuanya juga ada di navigasi kiri.',
	'firstRun.moreTidy':
		'Merapikan tag: ganti nama berpola dengan pratinjau, perbaikan otomatis tanpa jaringan, dan — bila kamu nyalakan — usulan dari internet yang tetap harus kamu setujui.',
	'firstRun.moreDownloads':
		'Mengambil audio dari sebuah URL lewat yt-dlp, satu per satu, setelah kamu menyetujui program yang diunduh Onsa untuk itu.',
	'firstRun.moreLyrics':
		'Lirik dari berkas .lrc di sebelah lagu, dari tag di dalamnya, atau dari LRCLIB bila kamu menyalakannya; baris yang sedang dinyanyikan menyala di panel kanan.',
	'firstRun.again': 'Layar pertama, dibuka lagi. Tidak ada yang harus kamu lakukan di sini.',
	'firstRun.againScan':
		'Library-mu sudah ada. Pilih folder di atas hanya kalau kamu ingin menambah satu lagi.',
	'firstRun.close': 'Tutup',

	'scan.running': 'Memindai… {seen} file, {read} dibaca',
	'scan.done': '{seen} file dalam {seconds} detik',
	'scan.failed': '{n} file tak terbaca',
	'scan.idle': 'Belum ada scan',

	'output.title': 'Output',
	'output.device': 'Perangkat output',
	'output.systemDefault': 'Default sistem (ikut berpindah)',
	'output.sampleRate': 'Sample rate',
	'output.followDevice': 'Ikuti perangkat',
	'output.current': 'Sekarang: {name}, {rate}',
	'output.none': 'Tidak ada output yang terbuka',
	'output.quality': 'Kualitas resampler',
	'output.buffer': 'Buffer',
	'output.bufferNote': 'Buffer baru berlaku saat output dibuka ulang.',
	'output.dither': 'Dither TPDF untuk output 16-bit',
	'output.powerSave': 'Mode hemat daya',
	'output.powerSaveHint':
		'Buffer besar, resampler cepat, meter dan visualizer lebih jarang digambar, dan warna nada dimatikan. Pengaturan buffer dan resampler di atas kembali berlaku saat mode ini dimatikan.',
	'output.matchSource': 'Samakan dengan sumber',
	'output.matchSourceHint':
		'Perangkat dibuka ulang mengikuti sample rate tiap lagu, kalau perangkatnya mendukung.',
	'quality.fast': 'Cepat',
	'quality.balanced': 'Seimbang',
	'quality.best': 'Terbaik',
	'buffer.low': 'Rendah',
	'buffer.normal': 'Normal',
	'buffer.large': 'Besar',

	'crossfade.title': 'Crossfade',
	'crossfade.duration': 'Durasi',
	'crossfade.off': 'Mati',
	'crossfade.curve': 'Kurva',
	'crossfade.albumGapless': 'Tanpa crossfade untuk lagu berurutan dalam satu album',
	'crossfade.skip': 'Crossfade saat skip',
	'curve.equalPower': 'Equal power',
	'curve.linear': 'Linear',

	'replaygain.title': 'ReplayGain',
	'replaygain.mode': 'Mode',
	'replaygain.preamp': 'Preamp ReplayGain',
	'replaygain.fallback': 'Lagu tanpa tag',
	'replaygain.preventClipping': 'Cegah clipping memakai nilai peak dari tag',
	'rg.off': 'Mati',
	'rg.track': 'Lagu',
	'rg.album': 'Album',
	'rg.auto': 'Otomatis',

	'eq.title': 'Equalizer',
	'eq.enabled': 'EQ aktif',
	'eq.graphic': 'Grafis',
	'eq.parametric': 'Parametrik',
	'eq.flat': 'Datarkan',
	'eq.curve': 'Kurva respons',
	'eq.addBand': 'Tambah filter',
	'eq.remove': 'Hapus filter',
	'eq.bandOn': 'Aktif',
	'eq.type': 'Jenis',
	'eq.freq': 'Frekuensi (Hz)',
	'eq.gain': 'Gain (dB)',
	'eq.q': 'Q',
	'eq.noBands': 'Belum ada filter. Tambah filter atau import preset AutoEQ.',
	'eq.maxBands': 'Maksimal {n} filter.',
	'eq.import': 'Import AutoEQ…',
	'eq.export': 'Export AutoEQ…',
	'eq.importTitle': 'Pilih preset EQ',
	'eq.exportTitle': 'Simpan preset EQ',
	'eq.fileFilter': 'Preset Equalizer APO / AutoEQ',
	'eq.imported': '{n} filter dimuat.',
	'eq.importedSkipped': '{n} filter dimuat, {skipped} baris dilewati (lihat log).',
	'eq.exported': 'Preset tersimpan.',
	'filter.peaking': 'Peaking',
	'filter.lowShelf': 'Low shelf',
	'filter.highShelf': 'High shelf',
	'filter.lowPass': 'Low pass',
	'filter.highPass': 'High pass',
	'filter.notch': 'Notch',

	'preamp.title': 'Preamp',
	'preamp.auto': 'Preamp otomatis, supaya EQ tidak membuat clipping',
	'preamp.manual': 'Preamp manual',
	'preamp.effective': 'Yang berlaku: {value}',

	'limiter.title': 'Limiter',
	'limiter.enabled': 'Limiter aktif (batas −0,1 dBFS)',
	'limiter.release': 'Release',

	'librarySettings.folders': 'Folder library',
	'librarySettings.add': 'Tambah folder…',
	'librarySettings.rescan': 'Scan ulang',
	'librarySettings.cannotRemove':
		'Folder belum bisa dihapus dari sini di versi ini. Folder yang berkasnya tidak ada lagi ditandai hilang saat scan ulang.',

	'about.version': 'Versi {version}',
	'about.logs': 'Log',
	'about.logDir': 'Folder log',
	'about.openLogs': 'Buka folder log',
	'about.debug': 'Log debug',
	'about.debugHint':
		'Mencatat lebih rinci. Nyalakan, ulangi masalahnya, lalu kirim file log terbaru dari folder log.',

	'theme.label': 'Tema',
	'toneColor.label': 'Warna nada',
	'toneColor.off': 'Mati',
	'toneColor.subtle': 'Halus',
	'toneColor.medium': 'Sedang',
	'toneColor.strong': 'Kuat',
	'toneColor.hint':
		'Warna bergeser mengikuti karakter suara: suara terang ke arah warna dingin tema, suara berat ke arah warna hangatnya. Setiap tema menentukan warna kedua ujungnya dan bagian mana yang ikut bergeser.',

	'theme.userFolder': 'Tema buatan sendiri',
	'theme.openFolder': 'Buka folder tema',
	'theme.userHint':
		'Taruh file tema .json di folder itu, lalu buka Onsa lagi supaya temanya muncul di daftar.',
	'theme.copy': 'Salin tema yang dipakai sekarang',
	'theme.copyHint':
		'Menyalin tema yang sedang dipakai ke folder itu, sebagai contoh yang bisa kamu ubah. Berkas yang sudah ada tidak ditimpa.',
	'theme.copyName': 'Tema saya',
	'theme.copied': 'Disalin ke {file}. Buka Onsa lagi supaya muncul di daftar.',
	'theme.troubles': 'Berkas tema yang tidak terpakai',
	'theme.troublesHint': 'Berkas ini ada di folder tema tapi tidak bisa dibaca, jadi tidak muncul di daftar.',
	'language.label': 'Bahasa',
	'language.id': 'Indonesia',
	'language.en': 'Inggris',

	'nav.playlists': 'Playlist',
	'nav.allPlaylists': 'Lihat semua',
	'playlist.manual': 'Playlist biasa',
	'playlist.smart': 'Playlist pintar',
	'playlist.count': '{n} lagu',
	'playlist.none': 'Belum ada playlist.',
	'playlist.new': 'Playlist baru',
	'playlist.newSmart': 'Playlist pintar baru',
	'playlist.import': 'Impor M3U8',
	'playlist.export': 'Ekspor M3U8',
	'playlist.more': 'Tindakan playlist',
	'playlist.dragHint': 'Tarik untuk mengurutkan, atau Alt+panah',
	'playlist.rename': 'Ganti nama',
	'playlist.duplicate': 'Gandakan',
	'playlist.delete': 'Hapus playlist',
	'playlist.deleteHint': 'Hapus "{name}"? Lagunya tetap ada di library.',
	'playlist.namePrompt': 'Nama playlist',
	'playlist.copySuffix': '{name} (salinan)',
	'playlist.untitled': 'Playlist',
	'playlist.empty': 'Playlist ini masih kosong.',
	'playlist.emptySmart': 'Belum ada lagu yang cocok dengan aturannya.',
	'playlist.play': 'Putar playlist',
	'playlist.addTo': 'Tambah ke playlist',
	'playlist.carried': '{n} lagu',
	'playlist.addedTo': '{n} lagu ditambahkan ke "{name}".',
	'playlist.removeTrack': 'Hapus dari playlist',
	'playlist.moveUp': 'Naikkan',
	'playlist.moveDown': 'Turunkan',
	'playlist.saveQueue': 'Simpan antrean sebagai playlist',
	'playlist.queueName': 'Antrean {date}',
	'playlist.exported': 'Tersimpan di {path}.',
	'playlist.imported': '{n} lagu masuk ke "{name}".',
	'playlist.importedNone': 'Tidak ada lagu dari file itu yang ada di library.',
	'playlist.importMissing': '{n} entri tidak ditemukan di library.',
	'playlist.fileFilter': 'Playlist M3U8',
	'playlist.editRules': 'Ubah aturan',
	'playlist.cancel': 'Batal',
	'playlist.save': 'Simpan',
	'playlist.create': 'Buat',

	'rules.title': 'Aturan playlist pintar',
	'rules.match': 'Cocokkan',
	'rules.matchAll': 'semua aturan',
	'rules.matchAny': 'salah satu aturan',
	'rules.add': 'Tambah aturan',
	'rules.remove': 'Hapus aturan',
	'rules.none': 'Tanpa aturan, seluruh library masuk.',
	'rules.sort': 'Urutkan',
	'rules.descending': 'Menurun',
	'rules.limit': 'Batas',
	'rules.noLimit': 'Tanpa batas',
	'rules.preview': 'Pratinjau: {n} lagu',
	'rules.previewMore': 'Pratinjau: {n} lagu pertama',
	'rules.invalid': 'Ada aturan yang nilainya belum benar.',
	'rules.days': 'hari',
	'rules.and': 'dan',
	'rules.field': 'Kolom',
	'rules.op': 'Perbandingan',
	'rules.value': 'Nilai',
	'rules.nothingToFill': '—',
	'rules.previewNone': 'Belum ada lagu yang cocok.',
	'rules.previewRest': '…dan {n} lagi',

	'field.title': 'Judul',
	'field.artist': 'Artis',
	'field.album': 'Album',
	'field.album_artist': 'Artis album',
	'field.track_number': 'Nomor trek',
	'field.disc_number': 'Nomor disk',
	'field.composer': 'Komposer',
	'field.genre': 'Genre',
	'field.year': 'Tahun',
	'field.rating': 'Rating',
	'field.play_count': 'Jumlah putar',
	'field.last_played': 'Terakhir diputar',
	'field.added_at': 'Ditambahkan',
	'field.codec': 'Codec',
	'field.duration': 'Durasi (detik)',
	'field.folder': 'Folder',
	'field.sample_rate': 'Sample rate (Hz)',
	'field.bit_depth': 'Kedalaman bit',
	'field.bitrate': 'Bitrate (kbit/s)',
	'field.has_cover': 'Cover',
	'field.lyrics': 'Lirik',

	'editor.open': 'Sunting tag…',
	'editor.title': 'Sunting tag',
	'editor.save': 'Simpan di Onsa',
	'editor.saved': 'Tersimpan di Onsa. Berkasnya belum diubah.',
	'editor.write': 'Tulis ke berkas',
	'editor.written': 'Sudah ditulis ke berkas.',
	'editor.writeFailed': 'Tidak bisa menulis ke berkas.',
	'editor.revert': 'Kembalikan ke isi berkas',
	'editor.unwritten': 'Belum ditulis ke berkas',
	'editor.editedHere': 'Diubah di Onsa',
	'editor.close': 'Tutup',
	'editor.gain': 'ReplayGain (dari berkas, tidak bisa diubah)',
	'editor.gainNone': 'Berkas ini tidak menyebutkan ReplayGain.',
	'editor.gainTrack': 'Lagu',
	'editor.gainAlbum': 'Album',
	'editor.peak': 'puncak {value}',
	'editor.laterFields': 'Total trek, total disk, dan komentar belum bisa diubah di versi ini.',
	'field.has_artist': 'Tag artis',

	'op.contains': 'mengandung',
	'op.not_contains': 'tidak mengandung',
	'op.is': 'sama dengan',
	'op.is_not': 'bukan',
	'op.starts_with': 'diawali',
	'op.not_starts_with': 'tidak diawali',
	'op.eq': 'sama dengan',
	'op.ne': 'tidak sama dengan',
	'op.lt': 'kurang dari',
	'op.le': 'paling banyak',
	'op.gt': 'lebih dari',
	'op.ge': 'paling sedikit',
	'op.between': 'antara',
	'op.in_last': 'dalam kurun',
	'op.not_in_last': 'tidak dalam kurun',
	'op.before': 'sebelum',
	'op.after': 'sesudah',
	'op.yes': 'ada',
	'op.no': 'tidak ada',

	'sortField.title': 'Judul',
	'sortField.artist': 'Artis',
	'sortField.album': 'Album',
	'sortField.year': 'Tahun',
	'sortField.duration': 'Durasi',
	'sortField.rating': 'Rating',
	'sortField.play_count': 'Jumlah putar',
	'sortField.last_played': 'Terakhir diputar',
	'sortField.added_at': 'Ditambahkan',
	'sortField.random': 'Acak',

	'settings.metadata': 'Metadata',
	'metadata.internet': 'Akses internet',
	'metadata.online': 'Ambil metadata dari internet',
	'metadata.onlineHint':
		'Selama ini mati, Onsa tidak pernah menghubungi layanan mana pun untuk metadata. Setiap usulan dari internet tetap harus kamu setujui sebelum diterapkan.',
	'metadata.acoustid': 'Kunci AcoustID',
	'metadata.acoustidHint':
		'AcoustID mengenali lagu dari suaranya sendiri, berguna untuk file tanpa tag. Kuncinya gratis: daftarkan aplikasi di acoustid.org/new-application, lalu salin "application API key" dari acoustid.org/my-applications. Bukan kunci pribadi di halaman akun — itu hanya untuk mengirim fingerprint.',
	'metadata.key': 'Kunci aplikasi',
	'metadata.keyPlaceholder': 'tempel kuncimu di sini',
	'metadata.keySave': 'Simpan kunci',
	'metadata.keyClear': 'Hapus kunci',
	'metadata.keySaved': 'Kunci tersimpan. Onsa tidak pernah menampilkannya lagi.',
	'metadata.keyCleared': 'Kunci dihapus.',
	'metadata.keyTry': 'Coba kunci',
	'metadata.keyTrying': 'Mencoba…',
	'metadata.keyWorks': 'AcoustID menerima kunci ini.',
	'metadata.keyRefused': 'AcoustID menolak kunci ini. Pastikan itu application key, bukan kunci akun.',
	'metadata.keyOffline': 'Nyalakan dulu "Ambil metadata dari internet" di atas.',
	'metadata.keyUnreachable': 'AcoustID tidak bisa dihubungi. Periksa koneksimu, lalu coba lagi.',
	'metadata.keyFromSettings': 'Memakai kunci yang kamu simpan di sini.',
	'metadata.keyFromEnvironment': 'Memakai kunci dari variabel lingkungan ONSA_ACOUSTID_API_KEY.',
	'metadata.keyFromBuild': 'Memakai kunci yang dipasang saat build.',
	'metadata.keyMissing': 'Belum ada kunci. Pengenalan lewat AcoustID tidak akan berjalan.',

	'nav.tidy': 'Rapikan',
	'tidy.scopeHeld': 'Cakupan: hanya folder ini',
	'tidy.scopeWhole': 'Cakupan: SELURUH LIBRARY',
	'tidy.scopeWholeHint': 'Tidak ada folder pembatas. Semua yang kamu jalankan bisa menyentuh library aslimu.',
	'tidy.inScope': '{n} lagu di cakupan',
	'tidy.limit': 'Batas per jalan',
	'tidy.pickFolder': 'Pilih folder',
	'tidy.pickFolderTitle': 'Pilih folder untuk dirapikan',
	'tidy.clearFolder': 'Lepas batas folder',
	'tidy.jobEdit': 'Edit massal',
	'tidy.jobWrite': 'Tulis ke file',
	'tidy.jobRename': 'Ganti nama berpola',
	'tidy.editWhat': 'Menyetel satu field untuk semua lagu di cakupan. Ini baru tersimpan di Onsa, belum menyentuh file.',
	'tidy.writeWhat': 'Menulis editan yang masih tersimpan di Onsa ke dalam file itu sendiri.',
	'tidy.renameWhat': 'Menyusun ulang nama dan folder file menurut pola. Selalu ditampilkan dulu sebelum dijalankan.',
	'tidy.valueEmpty': 'kosongkan untuk menghapus editan',
	'tidy.pattern': 'Pola',
	'tidy.renameRoot': 'Folder tujuan',
	'tidy.look': 'Lihat yang akan terjadi',
	'tidy.summary': 'Yang akan terjadi',
	'tidy.countTracks': 'lagu terkena',
	'tidy.countFields': 'nilai field berubah',
	'tidy.countWritten': 'file ditulis ulang',
	'tidy.countMoved': 'file dipindah atau diganti nama',
	'tidy.skipped': 'Yang dilewati',
	'tidy.skipOutside': 'di luar folder cakupan',
	'tidy.skipOverLimit': 'melewati batas per jalan',
	'tidy.skipNoChange': 'sudah sama, tidak ada yang berubah',
	'tidy.skipNameTaken': 'namanya akan bentrok',
	'tidy.nothingToDo': 'Tidak ada yang perlu dikerjakan.',
	'tidy.apply': 'Terapkan',
	'tidy.working': 'Sedang berjalan…',
	'tidy.done': 'Selesai: {n} perubahan.',
	'tidy.someFailed': '{n} gagal, filenya tidak tersentuh.',
	'tidy.history': 'Riwayat',
	'tidy.noHistory': 'Belum ada yang dijalankan.',
	'tidy.steps': '{n} langkah',
	'tidy.undo': 'Batalkan',
	'tidy.undone': 'sudah dibatalkan',
	'tidy.noteEdit': 'Edit massal: {field}',
	'tidy.noteWrite': 'Tulis ke file',
	'tidy.noteRename': 'Ganti nama berpola',

	'tidy.jobMatch': 'Cari data online',
	'tidy.matchWhat': 'Mencari tahu lagu apa ini lewat AcoustID dan MusicBrainz. Hasilnya usulan, bukan perubahan: yang kamu centang tetap lewat ringkasan, tombol terapkan, dan riwayat yang bisa dibatalkan.',
	'tidy.countCovers': 'sampul diganti',
	'tidy.noteMatch': 'Dari internet: isi field',
	'tidy.noteCover': 'Dari internet: sampul',

	'nav.downloads': 'Unduhan',
	'downloads.intro': 'Mengunduh audio dari URL lewat yt-dlp, yang dijalankan Onsa sebagai program terpisah.',
	'downloads.rights': 'Kamu yang bertanggung jawab atas hak konten yang kamu unduh.',
	'downloads.needs': 'Program yang dibutuhkan',
	'downloads.needsWhat': 'Onsa tidak mengunduh apa pun sebelum kamu menekan tombolnya. Berkasnya diambil dari rilis resmi di GitHub, dan dicocokkan dengan checksum yang diterbitkan rilis itu sebelum ditulis ke mana pun.',
	'downloads.fromRelease': 'dari rilis resmi',
	'downloads.installYourself': 'dipasang sendiri',
	'downloads.ffmpegWhere':
		'ffmpeg dipasang sendiri karena rilisnya berbentuk arsip. Di Linux biasanya sudah ada, atau lewat paket ffmpeg. Di Windows: unduh dari ffmpeg.org, lalu taruh foldernya di PATH — atau centang "pakai program yang sudah ada di sistem" di bawah.',
	'downloads.withoutFfmpeg':
		'Tanpa ffmpeg, unduhan tetap jalan: berkasnya diambil apa adanya, tanpa tag dan tanpa sampul tertanam, dan konversi tidak tersedia.',
	'downloads.noFfmpeg': 'Butuh ffmpeg, dan tidak ada yang ditemukan.',
	'downloads.noPlayableFormat':
		'Di sana tidak ada format yang bisa diputar Onsa. Opus menyusul di versi berikutnya.',
	'downloads.cannotRun': 'yt-dlp tidak bisa dijalankan.',
	'downloads.agree': 'Setuju dan unduh',
	'downloads.update': 'Perbarui',
	'downloads.stop': 'Hentikan',
	'downloads.useSystem': 'Pakai program yang sudah ada di sistem',
	'downloads.useSystemWhat': 'Kalau yt-dlp atau ffmpeg sudah terpasang di sistem (biasa di Linux), Onsa memakainya. Salinan milik Onsa sendiri tetap didahulukan.',
	'downloads.fromUrl': 'Unduh dari URL',
	'downloads.needYtDlp': 'yt-dlp harus ada dulu sebelum ada yang bisa diunduh.',
	'downloads.reading': 'Memeriksa program yang ada…',
	'downloads.urlPlaceholder': 'Tempel URL di sini',
	'downloads.look': 'Lihat isinya',
	'downloads.looking': 'Menanyakan…',
	'downloads.format': 'Format',
	'downloads.formatOriginal': 'Asli (disarankan)',
	'downloads.conversionNote': 'Konversi tidak menambah kualitas: FLAC hanya memperbesar ukuran, MP3 menurunkan kualitas.',
	'downloads.fetchCount': 'Unduh {n} lagu',
	'downloads.queue': 'Antrean',
	'downloads.waiting': 'menunggu',
	'downloads.running': 'sedang diunduh',
	'downloads.done': 'selesai',
	'downloads.failed': 'gagal',
	'downloads.wasStopped': 'dihentikan',
	'downloads.stopAll': 'Hentikan semua',
	'downloads.notAUrl': 'Itu bukan alamat web.',
	'downloads.refused': 'yt-dlp tidak bisa membaca URL itu.',
	'error.no_folder': 'Belum ada folder library, jadi belum ada tempat menaruh hasil unduhan.',
	'downloads.unreachable': 'Rilisnya tidak terjangkau. Coba lagi nanti.',
	'downloads.tooLarge': 'Berkasnya jauh lebih besar daripada yang seharusnya, jadi tidak diambil.',
	'downloads.stopped': 'Dihentikan. Tidak ada yang ditulis.',
	'downloads.noChecksum': 'Rilisnya tidak menyebutkan checksum untuk berkas ini, jadi tidak dipasang.',
	'downloads.checksumFailed': 'Checksum-nya tidak cocok. Berkasnya dibuang, tidak ada yang dipasang.',
	'downloads.cannotWrite': 'Berkasnya tidak bisa ditulis ke folder program Onsa.',
	'downloads.unreadable': 'Daftar checksum-nya tidak bisa dibaca.',
	'error.busy': 'Masih ada yang berjalan. Tunggu sampai selesai.',
	'error.download': 'Program itu tidak jadi dipasang. Detailnya ada di log.',

	'tidy.jobAuto': 'Rapikan otomatis',
	'tidy.autoWhat': 'Merapikan tag yang sudah ada, tanpa internet sama sekali: sisa nama unduhan, huruf besar-kecil, cara menulis "feat.", dan nama artis yang tertulis beda-beda di library yang sama.',
	'tidy.autoLook': 'Cari yang bisa dirapikan',
	'tidy.autoNone': 'Tidak ada yang perlu dirapikan di cakupan ini.',
	'tidy.autoFound': '{n} hal bisa dirapikan',
	'tidy.noteAuto': 'Perapihan otomatis',
	'auto.residue': 'sisa unduhan',
	'auto.capitals': 'huruf besar-kecil',
	'auto.credit': 'penulisan feat.',
	'auto.spelling': 'penulisan nama',
	'auto.albumArtist': 'artis album kosong',
	'auto.capitalsNote': 'Usulan huruf besar-kecil tidak tercentang otomatis: sering kali itu memang ditulis begitu dengan sengaja.',

	'match.start': 'Mulai cari',
	'match.stop': 'Hentikan',
	'match.stopped': 'Dihentikan. Yang sudah ditemukan tetap ada di bawah.',
	'match.progress': '{done} dari {total}',
	'match.looked': '{n} lagu sudah dilihat',
	'match.tickSure': 'Centang semua yang diusulkan',
	'match.tickNone': 'Hapus semua centang',
	'match.internetOff': 'Fitur internet sedang mati. Nyalakan di Pengaturan → Metadata kalau mau memakainya.',
	'match.keyMissing': 'Belum ada API key AcoustID. Tanpa itu, pencocokan lewat suara tidak bisa jalan.',
	'match.fpcalcMissing': 'fpcalc belum terpasang. Tanpa itu, hanya lagu yang sudah bertag yang bisa dicari.',
	'match.fromSound': 'Dari suara',
	'match.fromTags': 'Dari tag',
	'match.fromName': 'Dari nama file',
	'match.disagree': 'Dua sumber memberi hasil berbeda. Pilih sendiri yang mana; tidak ada yang tercentang otomatis.',
	'match.useNone': 'Jangan pakai keduanya',
	'match.sureTitle': 'Tingkat keyakinan',
	'match.nothing': '(kosong)',
	'match.takeCover': 'Pakai sampul ini',
	'match.coverAlt': 'Pratinjau sampul yang diusulkan',
	'match.noFingerprinter': 'Tidak bisa dicari lewat suara: fpcalc belum terpasang.',
	'match.unreadable': 'Berkasnya tidak bisa dibaca untuk diambil sidik suaranya.',
	'match.noKey': 'Tidak bisa dicari lewat suara: belum ada API key AcoustID.',
	'match.offline': 'Layanannya tidak terjangkau saat itu. Coba lagi nanti.',
	'match.refused': 'API key AcoustID ditolak.',
	'match.nothingFound': 'Tidak ada yang cocok.',

	'programs.title': 'Program luar',
	'programs.hint': 'Onsa menjalankan beberapa program luar. fpcalc dipakai di halaman ini, untuk mengenali lagu lewat suaranya; yt-dlp dan ffmpeg dipakai di halaman Unduhan. Onsa memakai yang ada di sistem kalau ada.',
	'programs.installed': 'terpasang',
	'programs.missing': 'belum ada',
	'programs.fromManaged': 'dari folder Onsa',
	'programs.fromChosen': 'dari pilihanmu',
	'programs.fromSystem': 'dari sistem',
	'programs.choose': 'Pilih berkasnya',
	'programs.chooseTitle': 'Pilih berkas program',
	'programs.forget': 'Lupakan pilihan',
	'programs.refresh': 'Periksa lagi',
	'programs.fpcalcWhere': 'fpcalc ada di paket Chromaprint. Di Linux biasanya bernama libchromaprint-tools; di Windows unduh dari halaman rilis Chromaprint, lalu tunjukkan berkasnya di sini.',

	'error.offline': 'Fitur internet sedang mati.',

	'error.backend': 'Backend tidak terjangkau. Jalankan Onsa lewat jendela aplikasinya.',
	'error.theme_not_found': 'Tema itu tidak ada.',
	'error.library': 'Library gagal diakses. Detailnya ada di log.',
	'error.engine': 'Mesin audio menolak perintah. Detailnya ada di log.',
	'error.no_output': 'Tidak ada output audio yang bisa dibuka.',
	'error.dialog': 'Jendela pemilih file tidak bisa dibuka.',
	'error.io': 'File tidak bisa dibaca atau ditulis.',
	'error.auto_eq_empty': 'File itu tidak berisi filter EQ yang bisa dipakai.',

	'nav.help': 'Bantuan',
	'help.open': 'Bantuan halaman ini',
	'help.close': 'Tutup bantuan',
	'help.here': 'Di halaman ini',
	'help.controls': 'Kontrol di halaman ini',
	'help.filesHead': 'Berkas musikmu sendiri',
	'help.limitsHead': 'Batasan di halaman ini',
	'help.elsewhere':
		'Kontrol yang selalu ada di jendela — baris pemutar dan jalur sinyal di bawah, navigasi di kiri, panel antrean dan lirik di kanan — dijelaskan di halaman Bantuan.',
	'help.toHelpPage': 'Buka halaman Bantuan',
	'help.firstRun': 'Buka layar pertama lagi',
	'help.title.album': 'Satu album',
	'help.title.artist': 'Satu artis',
	'help.title.genre': 'Satu genre',
	'help.title.folder': 'Satu folder',
	'help.title.playlist': 'Satu playlist',
	'help.title.search': 'Hasil pencarian',
	'help.title.header': 'Baris atas jendela',
	'help.title.transport': 'Baris pemutar dan jalur sinyal',
	'help.title.panel': 'Panel kanan: antrean dan lirik',
	'help.tracks.what':
		'Daftar semua lagu yang ditemukan Onsa di folder musikmu. Onsa memutar berkas yang sudah ada di komputermu; tidak ada katalog lagu di internet di sini, dan tidak ada yang dikirim ke mana pun. Klik dua kali sebuah baris untuk memutarnya.',
	'help.tracks.limit':
		'Yang terlihat di sini adalah hasil pemeriksaan folder yang terakhir. Berkas yang kamu tambahkan lewat program lain biasanya muncul sendiri dalam beberapa detik; kalau tidak muncul, ada tombol "Scan ulang" — yaitu memeriksa isi foldermu sekali lagi dari awal — di Pengaturan → Library.',
	'help.albums.what':
		'Semua album di library-mu, digambar dengan sampulnya. Sampul diambil dari dalam berkas lagunya atau dari gambar yang ada di folder yang sama. Onsa tidak mengambil gambar dari internet kecuali kamu menyalakannya sendiri di Pengaturan → Metadata, halaman yang mengatur apa yang boleh ditanyakan ke internet tentang lagumu.',
	'help.album.what': 'Satu album: sampulnya, artisnya, dan lagunya menurut urutan di album itu.',
	'help.artists.what':
		'Semua artis di library-mu, dengan jumlah lagu masing-masing. Klik satu untuk melihat lagunya.',
	'help.artist.what': 'Semua lagu dari satu artis, dikumpulkan dari seluruh album.',
	'help.genres.what':
		'Genre adalah jenis musik yang tertulis di dalam berkas lagunya sendiri — misalnya "Rock" atau "Jazz". Halaman ini mengumpulkan lagu-lagumu menurut jenis itu.',
	'help.genres.limit':
		'Lagu yang tidak punya keterangan genre di dalam berkasnya tidak muncul di sini. Kamu bisa mengisinya sendiri lewat "Sunting tag…" pada klik kanan sebuah lagu — jendela untuk mengubah keterangan yang tersimpan di dalam berkas itu — atau lewat halaman Rapikan.',
	'help.genre.what': 'Semua lagu dari satu genre.',
	'help.folders.what':
		'Lagumu menurut folder tempat berkasnya benar-benar berada di komputermu, bukan menurut keterangan di dalam berkasnya.',
	'help.folder.what': 'Semua lagu di dalam satu folder, seperti yang ada di komputermu.',
	'help.playlists.what':
		'Playlist adalah daftar lagu pilihanmu sendiri. Ada dua macam: yang kamu isi sendiri lagu demi lagu, dan yang pintar — yang isinya mengikuti aturan yang kamu buat, misalnya "semua lagu tahun 1990-an yang pernah kuputar", dan berubah sendiri ketika library-mu berubah.',
	'help.playlists.files':
		'Playlist hidup di dalam Onsa saja. Membuat, mengubah, atau menghapus playlist tidak menyentuh berkas musikmu sama sekali. "Impor M3U8" pun hanya membaca berkas daftarnya, tidak memindahkan apa pun.',
	'help.playlist.what':
		'Isi satu playlist, menurut urutan yang kamu susun. Lagu bisa diseret naik-turun untuk mengubah urutannya.',
	'help.playlist.files':
		'Menghapus lagu dari playlist hanya mengeluarkannya dari daftar ini. Berkasnya tetap ada di komputermu dan tetap ada di library.',
	'help.search.what':
		'Hasil pencarian dari kotak cari di atas: artis, album, dan lagu yang namanya cocok dengan yang kamu ketik. Pencarian ini hanya melihat library-mu sendiri.',
	'help.search.limit':
		'Yang dicari adalah judul, artis, album, dan genre. Lirik dan nama berkas tidak ikut dicari.',
	'help.tidy.what':
		'Halaman untuk merapikan keterangan lagu dalam jumlah banyak sekaligus. Keterangan ini — judul, artis, album, tahun, genre — tersimpan di dalam berkas lagunya sendiri; di mana-mana orang menyebutnya "tag". Setiap pekerjaan di halaman ini selalu ditampilkan dulu sebagai ringkasan sebelum dijalankan, dan setiap yang sudah dijalankan bisa dibatalkan lewat daftar Riwayat di bawah.',
	'help.tidy.files':
		'Dua pekerjaan di sini menyentuh berkas aslimu: "Tulis ke file" menulis keterangan ke dalam berkasnya, dan "Ganti nama berpola" mengganti nama berkas dan memindahkannya antar folder. Yang lain hanya disimpan di dalam Onsa — berkasmu tidak berubah sampai kamu menekan "Tulis ke file". Keduanya tetap lewat ringkasan dulu dan tetap ada di Riwayat, dan pembatalan mengembalikan nama berkas maupun isi keterangannya.',
	'help.tidy.limit':
		'Satu jalan mengerjakan paling banyak sebanyak angka di "Batas per jalan" — sisanya menunggu jalan berikutnya, supaya kesalahan tidak terlanjur mengenai seluruh library.',
	'help.downloads.what':
		'Mengambil audio dari sebuah alamat web (URL) dan menaruhnya di folder musikmu, satu alamat pada satu waktu. Yang mengambil bukan Onsa sendiri melainkan yt-dlp, sebuah program terpisah yang diunduh Onsa hanya setelah kamu menekan tombol persetujuannya. Kamu yang bertanggung jawab atas hak isi yang kamu unduh.',
	'help.downloads.files':
		'Hasil unduhan ditulis sebagai berkas baru di dalam folder library-mu, di subfolder "Unduhan", lalu muncul sendiri di library. Berkas musik yang sudah ada tidak disentuh.',
	'help.downloads.limitOpus':
		'Sumber yang hanya menyediakan audio format Opus ditolak di muka, dengan pesan, karena Onsa belum bisa memutar format itu.',
	'help.downloads.limitOne':
		'Satu unduhan berjalan pada satu waktu. Antrean yang berjalan bersamaan menyusul di versi berikutnya.',
	'help.nowPlaying.what':
		'Layar penuh untuk lagu yang sedang diputar: sampul besar, liriknya kalau ada, dan keterangan teknis tentang apa yang sedang terjadi pada suaranya.',
	'help.output.what':
		'Ke mana suara dikirim, dan seberapa teliti Onsa mengolahnya sebelum dikirim. Semua yang di halaman ini hanya mengubah suara yang keluar sekarang; tidak ada satu pun yang mengubah berkas musikmu.',
	'help.dsp.what':
		'Pengaturan suara yang diolah sebelum keluar: EQ (pengatur nada, menaikkan atau menurunkan bagian tinggi dan rendah), preamp (pengatur volume sebelum pengolahan), dan limiter (penjaga supaya suara tidak melewati batas dan pecah). Tidak ada satu pun di halaman ini yang mengubah berkas musikmu.',
	'help.library.what':
		'Folder mana saja yang dianggap Onsa sebagai tempat musikmu. Onsa membaca isinya dan mengingat apa yang ada di sana; isinya sendiri tidak dipindahkan, diganti nama, atau diubah.',
	'help.library.files':
		'Menambah folder dan memeriksanya ulang hanya membaca. Tidak ada berkas yang ditulis, dipindah, atau dihapus dari halaman ini.',
	'help.metadata.what':
		'Apakah Onsa boleh bertanya ke internet tentang lagumu, dan dengan apa. Semua ini mati secara bawaan: dengan keadaan mati, tidak ada satu pun permintaan yang dikirim. Kalau dinyalakan, yang dikirim adalah sidik jari suara lagu (angka yang dihitung dari bunyinya, bukan berkasnya) dan judul-artis-album, untuk ditanyakan ke AcoustID dan MusicBrainz. Hasilnya selalu berupa usulan yang harus kamu setujui.',
	'help.lyricsSettings.what':
		'Dari mana lirik boleh diambil, dan apakah lirik dari internet disimpan di sebelah lagunya. Keduanya mati secara bawaan.',
	'help.lyricsSettings.files':
		'Dengan saklar kedua dinyalakan, Onsa menulis satu berkas teks kecil berakhiran .lrc di sebelah berkas lagunya. Berkas lagunya sendiri tidak disentuh.',
	'help.appearance.what':
		'Rupa jendela: tema (kumpulan warna), warna yang ikut bergerak mengikuti bunyi, dan bahasa. Tidak ada yang di halaman ini menyentuh musik atau berkasmu.',
	'help.about.what':
		'Versi Onsa yang sedang berjalan, catatan kejadian (log) untuk dikirim kalau ada yang salah, dan apa yang terjadi ketika jendela ditutup.',
	'help.header.what': 'Baris di atas isi halaman, dan selalu ada di halaman mana pun.',
	'help.transport.what':
		'Dua baris paling bawah jendela, selalu ada di halaman mana pun: kontrol pemutaran, dan di bawahnya jalur sinyal — rantai yang menunjukkan apa saja yang dilewati suara dari berkas sampai ke perangkat output.',
	'help.panel.what':
		'Panel di kanan jendela dengan dua tab: Antrean, yaitu daftar lagu yang akan diputar sesudah yang sekarang, dan Lirik. Di jendela yang sempit panel ini tersembunyi dan dibuka lewat tombol di baris atas.',
	'help.panel.files':
		'Tombol "Simpan .lrc" di tab Lirik menulis satu berkas teks kecil di sebelah berkas lagunya. Selain itu tidak ada yang menyentuh berkas.',
	'help.help.what':
		'Halaman ini: daftar seluruh halaman Onsa, ditambah hal-hal yang tidak menempel di satu halaman saja.',

	'help.name.columns': 'Judul · Artis · Album · Tahun · Durasi',
	'help.says.columns':
		'Nama kolom, di atas daftar. Klik satu untuk mengurutkan daftar menurut kolom itu; klik sekali lagi untuk membalik urutannya. Panah kecil menunjukkan kolom mana yang sedang dipakai mengurutkan.',
	'help.name.resize': 'Garis di antara dua nama kolom',
	'help.says.resize':
		'Tarik untuk melebarkan atau menyempitkan kolom di sebelah kirinya; lebarnya diingat. Dengan papan ketik: tekan Tab sampai garisnya terpilih, lalu pakai panah kiri dan kanan.',
	'help.says.chooseColumns':
		'Membuka daftar pilihan: kolom mana yang ditampilkan dan mana yang disembunyikan, dengan satu pilihan untuk mengembalikan semuanya ke bawaan.',
	'help.name.row': 'Baris lagu',
	'help.says.row':
		'Satu baris satu lagu. Klik dua kali untuk memutarnya sekarang. Klik sekali untuk memilih; Ctrl+klik menambah satu lagi ke pilihan dan Shift+klik memilih sederet. Baris yang berkasnya tidak ada lagi di komputermu ditandai "hilang". Baris bisa diseret ke playlist di navigasi kiri.',
	'help.name.rowMenu': 'Klik kanan pada baris lagu',
	'help.says.rowMenu':
		'Membuka menu kecil: "Putar berikutnya" menaruh lagu itu tepat sesudah yang sedang diputar, "Tambah ke antrean" menaruhnya di ujung daftar tunggu, "Tambah ke playlist" memasukkannya ke salah satu daftarmu, dan "Sunting tag…" membuka jendela untuk mengubah keterangan lagu itu — judul, artis, album, dan seterusnya — yang tersimpan di dalam berkasnya.',
	'help.name.albumCard': 'Kartu album',
	'help.says.albumCard': 'Sampul dengan nama album dan artisnya. Klik untuk membuka album itu.',
	'help.says.albumBack': 'Kembali ke halaman berisi semua album.',
	'help.says.albumPlay':
		'Memutar album ini dari lagu pertamanya, menggantikan daftar tunggu yang ada.',
	'help.says.addTo':
		'Menaruh semua lagu yang terlihat di halaman ini ke salah satu playlist-mu. Berkasnya tidak berpindah ke mana-mana; yang bertambah hanya daftarnya.',
	'help.name.artistRow': 'Baris artis',
	'help.says.artistRow':
		'Nama artis dan berapa lagunya yang kamu punya. Klik untuk melihat lagunya.',
	'help.name.genreRow': 'Baris genre',
	'help.says.genreRow':
		'Nama genre dan berapa lagu yang punya keterangan genre itu di dalam berkasnya. Klik untuk melihat lagunya.',
	'help.name.folderRow': 'Baris folder',
	'help.says.folderRow':
		'Nama folder beserta jalur lengkapnya di komputermu. Klik untuk melihat lagu di dalamnya.',
	'help.name.browseBack': 'Tombol kembali di kiri atas',
	'help.says.browseBack':
		'Kembali ke daftar sebelumnya: semua artis, semua genre, atau semua folder.',
	'help.says.browsePlay': 'Memutar semua lagu yang terlihat di halaman ini, dari yang pertama.',
	'help.says.playlistNew':
		'Membuat playlist kosong yang kamu isi sendiri. Onsa menanyakan namanya dulu.',
	'help.says.playlistNewSmart':
		'Membuat playlist yang isinya mengikuti aturan, bukan pilihan satu per satu. Kamu menyusun aturannya — misalnya tahun, artis, genre, berapa kali diputar — dan isinya menyesuaikan sendiri setiap kali library-mu berubah.',
	'help.says.playlistImport':
		'Membaca berkas daftar lagu buatan program lain (berakhiran .m3u8) dan membuat playlist dari isinya. Lagu yang tidak ada di library-mu dilewati dan jumlahnya disebutkan.',
	'help.name.playlistCard': 'Kartu playlist',
	'help.says.playlistCard':
		'Nama playlist, jenisnya (biasa atau pintar), dan berapa lagu isinya. Klik untuk membukanya.',
	'help.name.playlistBack': 'Tombol kembali di kiri atas',
	'help.says.playlistBack': 'Kembali ke halaman semua playlist.',
	'help.says.playlistPlay': 'Memutar playlist ini dari lagu pertamanya.',
	'help.says.playlistQueue':
		'Menaruh seluruh isi playlist ini di ujung daftar tunggu, tanpa menghentikan lagu yang sedang diputar.',
	'help.says.playlistMore':
		'Membuka menu: ganti nama, gandakan, ekspor jadi berkas .m3u8, dan hapus playlist. Menghapus playlist tidak menghapus satu pun berkas musik.',
	'help.says.playlistRules':
		'Hanya ada pada playlist pintar: membuka aturannya, supaya kamu bisa mengubah apa yang masuk ke daftar ini.',
	'help.name.playlistRow': 'Baris lagu di playlist',
	'help.says.playlistRow':
		'Sama seperti baris lagu di halaman lain, ditambah dua hal: lagunya bisa diseret naik-turun untuk mengubah urutan, dan ada tombol untuk mengeluarkannya dari playlist ini. Mengeluarkan lagu tidak menghapus berkasnya.',
	'help.name.searchArtist': 'Hasil berupa artis',
	'help.says.searchArtist':
		'Artis yang namanya cocok dengan yang kamu ketik. Klik untuk membuka halamannya.',
	'help.name.searchAlbum': 'Hasil berupa album',
	'help.says.searchAlbum':
		'Album yang namanya cocok dengan yang kamu ketik. Klik untuk membukanya.',

	'help.name.tidyScope': 'Cakupan',
	'help.says.tidyScope':
		'Tulisan di atas halaman yang menyebut lagu mana saja yang akan terkena pekerjaan di sini. Tanpa folder pembatas, cakupannya seluruh library-mu; dengan folder pembatas, hanya lagu di dalam folder itu.',
	'help.says.tidyPick':
		'Memilih satu folder sebagai pembatas, supaya pekerjaan di halaman ini tidak bisa menyentuh lagu di luar folder itu. Cara paling aman untuk mencoba sesuatu pertama kali.',
	'help.says.tidyClear': 'Melepas folder pembatas, sehingga cakupannya kembali seluruh library.',
	'help.says.tidyLimit':
		'Berapa lagu paling banyak yang boleh dikerjakan dalam satu jalan. Sisanya menunggu jalan berikutnya. Gunanya supaya satu kesalahan tidak langsung mengenai seluruh library.',
	'help.says.tidyEdit':
		'Menyetel satu keterangan — misalnya artis album — untuk semua lagu yang masuk cakupan, yaitu yang disebut di baris paling atas halaman ini. Hasilnya baru tersimpan di dalam Onsa; berkas lagumu belum berubah sampai kamu menjalankan "Tulis ke file".',
	'help.says.tidyWrite':
		'Menulis keterangan yang tersimpan di dalam Onsa ke dalam berkas lagunya sendiri. Inilah satu-satunya tombol di halaman ini yang mengubah isi berkasmu.',
	'help.says.tidyRename':
		'Menyusun ulang nama berkas dan folder menurut pola yang kamu tulis — misalnya "artis/album/nomor judul". Berkasnya benar-benar diganti nama dan dipindah di komputermu, selalu setelah kamu melihat ringkasannya dulu.',
	'help.says.tidyAuto':
		'Memperbaiki hal-hal kecil tanpa internet: spasi berlebih, huruf besar yang kacau, nomor trek yang ditulis "3/12". Yang diusulkannya tetap lewat ringkasan sebelum dijalankan.',
	'help.says.tidyMatch':
		'Bertanya ke internet lagu apa ini, dengan menghitung sidik jari dari bunyinya lalu menanyakannya ke AcoustID dan MusicBrainz. Hanya jalan kalau kamu menyalakannya di Pengaturan → Metadata, halaman yang mengatur apa yang boleh ditanyakan ke internet. Hasilnya usulan yang kamu centang sendiri, bukan perubahan.',
	'help.says.tidyField':
		'Keterangan mana yang akan disetel oleh "Edit massal": artis, album, artis album, genre, atau tahun.',
	'help.says.tidyValue':
		'Nilai yang akan diisikan ke keterangan itu. Dikosongkan berarti menghapus editan yang tersimpan di Onsa dan mengembalikan nilai yang ada di dalam berkas.',
	'help.says.tidyPattern':
		'Pola nama berkas untuk "Ganti nama berpola". Bagian dalam kurung kurawal diganti isi lagunya, misalnya {artist} dan {title}; garis miring berarti folder baru.',
	'help.says.tidyRoot':
		'Folder tempat hasil penggantian nama diletakkan. Kosong berarti tetap di tempatnya sekarang.',
	'help.says.tidyLook':
		'Menghitung apa yang akan terjadi dan menampilkannya — berapa lagu terkena, berapa berkas ditulis, apa yang dilewati dan kenapa — tanpa mengubah apa pun.',
	'help.says.tidyApply':
		'Menjalankan apa yang baru saja ditampilkan di ringkasan. Baru sesudah tombol ini ditekan ada yang benar-benar berubah.',
	'help.says.tidyUndo':
		'Membatalkan satu jalan yang ada di daftar Riwayat, termasuk yang sudah ditulis ke berkas dan nama berkas yang sudah diganti.',
	'help.says.useSystem':
		'Kalau yt-dlp atau ffmpeg sudah ada di komputermu, Onsa memakai yang itu dan tidak perlu mengunduh apa pun. Kalau dimatikan, Onsa hanya menjalankan program yang ada di foldernya sendiri — tapi yt-dlp tetap mencari ffmpeg sendiri di komputermu, karena itu urusannya, bukan urusan Onsa.',
	'help.name.programRow': 'Baris program',
	'help.says.programRow':
		'Satu baris untuk tiap program yang dibutuhkan halaman ini, dengan keterangan apakah ia sudah ada, dari mana asalnya, dan versinya. yt-dlp wajib: tanpa dia tidak ada yang bisa diunduh. ffmpeg tidak wajib: tanpa dia unduhan tetap jalan, tapi tanpa keterangan lagu dan sampul yang tertanam, dan tanpa mengubah format. Baris ffmpeg bisa berkata "ada di sistem, dipakai yt-dlp sendiri": itu berarti ffmpeg-nya ada di komputermu dan yt-dlp memakainya atas namanya sendiri, di luar saklar di bawah.',
	'help.says.agree':
		'Mengunduh program itu dari halaman rilis resminya, dan mencocokkannya dengan sidik berkas yang diterbitkan rilis itu sebelum menyimpannya. Tidak ada yang diunduh sebelum tombol ini ditekan.',
	'help.says.update': 'Mengambil versi terbaru program itu, dengan pemeriksaan yang sama.',
	'help.says.urlField': 'Tempat menempelkan alamat web yang audionya ingin kamu ambil.',
	'help.says.look':
		'Menanyakan ke yt-dlp apa isi alamat itu — satu lagu atau sebuah daftar — dan menampilkan judulnya. Belum ada yang diunduh pada tahap ini.',
	'help.says.format':
		'Bentuk berkas yang kamu inginkan. "Asli" berarti berkasnya diambil apa adanya tanpa diubah. Pilihan MP3 dan FLAC mengubah bentuknya, dan itu pekerjaan ffmpeg — tanpa ffmpeg keduanya tidak bisa dipilih.',
	'help.says.fetch': 'Mulai mengunduh apa yang ditemukan tadi ke folder musikmu.',
	'help.says.stop':
		'Menghentikan satu unduhan yang sedang berjalan. Berkas setengah jadi dibuang.',
	'help.says.stopAll': 'Menghentikan semua yang sedang berjalan dan yang masih menunggu.',
	'help.says.nowPlayingClose': 'Menutup layar ini dan kembali ke daftar lagu.',
	'help.says.seek':
		'Batang panjang berisi posisi lagu. Klik atau tarik untuk melompat ke bagian lain. Panah kiri dan kanan di papan ketik memundurkan atau memajukan lima detik.',
	'help.name.nowPlayingLyrics': 'Lirik di layar ini',
	'help.says.nowPlayingLyrics':
		'Kalau lagunya punya lirik, barisnya muncul besar di sini dan baris yang sedang dinyanyikan menyala. Klik sebuah baris untuk melompat ke bagian itu.',

	'help.says.device':
		'Lewat mana suaranya keluar: speaker, headphone, atau perangkat lain yang dikenali komputermu. "Default sistem" berarti Onsa ikut ke mana pun Windows atau Linux mengirim suara, termasuk ketika kamu mencolok headphone.',
	'help.says.sampleRate':
		'Berapa kali per detik suara dikirim ke perangkat itu. "Ikuti perangkat" memakai angka yang disukai perangkatnya dan hampir selalu jawaban yang benar.',
	'help.says.matchSource':
		'Kalau dinyalakan, Onsa berusaha memakai angka yang sama dengan berkas lagunya, supaya suaranya tidak perlu dihitung ulang. Kalau perangkatnya tidak mendukung, Onsa tetap menghitung ulang.',
	'help.says.quality':
		'Seberapa teliti perhitungan ulang itu dilakukan ketika angka lagu dan angka perangkat berbeda. Tiga pilihannya: "Cepat", "Seimbang", dan "Terbaik". Makin teliti makin berat untuk prosesor; "Seimbang" cukup untuk hampir semua orang.',
	'help.says.buffer':
		'Seberapa banyak suara disiapkan lebih dulu sebelum dikirim, dengan tiga pilihan: "Rendah", "Normal", dan "Besar". Rendah berarti Onsa menanggapi lebih cepat; besar berarti lebih tahan terhadap komputer yang sedang sibuk. Kalau suaranya putus-putus, besarkan.',
	'help.says.dither':
		'Menambahkan desis yang sangat halus ketika suara dikecilkan ke 16-bit, supaya bagian yang paling pelan tidak terdengar kasar. Hanya berlaku untuk perangkat yang memang 16-bit.',
	'help.says.powerSave':
		'Mengurangi pekerjaan gambar bergerak seperti meter dan spektrum ketika jendela tidak terlihat, supaya baterai lebih awet.',
	'help.says.crossfade':
		'Berapa lama suara lagu lama dan lagu baru bertumpang tindih saat berganti. Nol berarti tidak ada tumpang tindih.',
	'help.says.curve':
		'Bentuk perpindahan antara dua lagu. "Equal power" menjaga kerasnya terdengar tetap di tengah perpindahan; "Linear" turun dan naik lurus.',
	'help.says.albumGapless':
		'Banyak album dibuat supaya satu lagu menyambung langsung ke lagu berikutnya tanpa jeda sedetik pun. Dengan pilihan ini dinyalakan, dua lagu berurutan dari album yang sama disambung persis seperti itu, tanpa tumpang tindih, dan crossfade hanya dipakai di luar album.',
	'help.says.skipFade':
		'Perpindahan singkat ketika kamu sendiri menekan tombol lagu berikutnya, supaya tidak terdengar terpotong mendadak.',
	'help.says.rgMode':
		'ReplayGain adalah angka kenyaringan yang sudah tersimpan di dalam banyak berkas lagu oleh program lain. Empat pilihannya: "Mati" tidak memakainya sama sekali, "Lagu" memakai angka per lagu, "Album" memakai angka per album — supaya perbedaan keras-pelan di dalam satu album tetap terjaga — dan "Otomatis" memakai angka album ketika yang diputar memang satu album.',
	'help.says.rgPreamp':
		'Menaikkan atau menurunkan seluruh hasil perataan kenyaringan itu, kalau menurutmu hasilnya terlalu pelan atau terlalu keras.',
	'help.says.rgFallback':
		'Berapa keras lagu yang tidak punya angka kenyaringan tersimpan di dalamnya, supaya tidak terdengar melonjak di antara lagu-lagu yang punya.',
	'help.says.rgClip':
		'Memakai angka puncak yang tersimpan bersama angka kenyaringan untuk menurunkan volume sedikit bila perlu, supaya suaranya tidak melewati batas dan pecah.',
	'help.says.eqOn':
		'Menyalakan atau mematikan pengatur nada. Dimatikan berarti suara lewat tanpa diubah sama sekali.',
	'help.says.eqGraphic':
		'Bentuk sederhana: sepuluh tuas, masing-masing untuk satu daerah nada dari yang paling rendah sampai yang paling tinggi.',
	'help.says.eqParametric':
		'Bentuk lanjut: kamu sendiri yang menentukan letak, lebar, dan jenis tiap filter. Untuk orang yang mengikuti setelan siap pakai dari internet.',
	'help.name.eqBand':
		'Tuas nada: 31 Hz, 62 Hz, 125 Hz, 250 Hz, 500 Hz, 1 kHz, 2 kHz, 4 kHz, 8 kHz, 16 kHz',
	'help.says.eqBand':
		'Sepuluh tuas, satu untuk tiap daerah nada. Angka kecil di kiri adalah suara rendah seperti bas; angka besar di kanan adalah suara tinggi seperti desis simbal. Menaikkan tuas membuat daerah itu lebih keras, menurunkannya membuat lebih pelan.',
	'help.says.eqFlat': 'Mengembalikan semua tuas ke nol, yaitu tanpa perubahan nada sama sekali.',
	'help.says.eqImport':
		'Membaca berkas setelan EQ buatan AutoEQ — setelan siap pakai untuk model headphone tertentu — dan memakainya.',
	'help.says.eqExport':
		'Menyimpan setelan EQ-mu sekarang sebagai berkas teks dalam bentuk yang sama, supaya bisa dipakai di tempat lain.',
	'help.says.eqAdd': 'Hanya pada bentuk parametrik: menambah satu filter baru.',
	'help.says.eqRemove': 'Hanya pada bentuk parametrik: membuang filter itu.',
	'help.says.preampAuto':
		'Menurunkan volume sebelum pengatur nada, secukupnya, supaya tuas yang dinaikkan tidak membuat suara melewati batas dan pecah.',
	'help.says.preampManual':
		'Mengatur sendiri penurunan itu, kalau kamu tidak ingin Onsa menghitungnya.',
	'help.says.limiterOn':
		'Penjaga terakhir sebelum suara keluar: menahan puncak yang masih terlalu tinggi supaya tidak pecah.',
	'help.says.limiterRelease':
		'Berapa cepat penjaga itu melepaskan tahanannya setelah puncaknya lewat. Terlalu cepat terdengar bergetar, terlalu lambat terdengar menekan.',
	'help.says.addFolder':
		'Menambah satu folder lagi sebagai tempat musikmu. Onsa langsung membaca isinya.',
	'help.says.rescan':
		'Menyuruh Onsa melihat lagi seluruh folder itu dari awal. Berguna kalau ada berkas yang tidak muncul sendiri, misalnya setelah folder dipindah dari komputer lain.',
	'help.name.folderList': 'Daftar folder',
	'help.says.folderList':
		'Folder yang sudah kamu tambahkan, dengan jumlah lagu di masing-masing. Folder yang tidak ada di tempatnya lagi — drive-nya dicabut, atau foldernya dipindah lewat program lain — ditandai di daftar ini beserta keterangannya. Onsa tidak menghapus apa pun karena itu.',
	'help.says.metaOnline':
		'Saklar utama untuk semua pertanyaan ke internet di halaman Rapikan. Selama mati, Onsa tidak mengirim apa pun ke mana pun.',
	'help.says.metaKey':
		'AcoustID meminta kunci aplikasi untuk setiap program yang bertanya kepadanya. Kunci itu gratis dan diambil sendiri dari situs AcoustID; Onsa tidak membawa kunci apa pun.',
	'help.says.metaKeySave':
		'Menyimpan kunci itu di komputermu saja, untuk dipakai pada pertanyaan berikutnya.',
	'help.says.metaKeyClear': 'Menghapus kunci yang tersimpan.',
	'help.says.metaKeyTry': 'Mengirim satu pertanyaan kecil untuk memastikan kuncinya diterima.',
	'help.says.metaChoose':
		'Menunjukkan sendiri letak berkas fpcalc di komputermu. fpcalc adalah program kecil yang menghitung sidik jari dari bunyi lagu; tanpa dia, "Cari data online" tidak bisa bertanya apa-apa.',
	'help.says.metaForget': 'Melupakan letak yang tadi kamu tunjukkan.',
	'help.says.metaRefresh':
		'Memeriksa lagi apakah programnya sekarang ada, misalnya sesudah kamu memasangnya.',
	'help.says.lyricsOnline':
		'Boleh atau tidaknya Onsa bertanya ke LRCLIB — sebuah layanan lirik terbuka — untuk lagu yang liriknya tidak ada di komputermu. Yang dikirim hanya judul, artis, album, dan durasi lagunya.',
	'help.says.lyricsBeside':
		'Kalau dinyalakan, lirik yang datang dari internet disimpan sebagai berkas teks kecil berakhiran .lrc di sebelah berkas lagunya, sehingga tetap ada nanti tanpa internet.',
	'help.says.themePick':
		'Kumpulan warna untuk seluruh jendela. Enam tema bawaan: "Kaca asap", "Kokpit kaca", "Deck malam", "Kubikel biru", "Kilau milenium", dan "Musim dingin utara"; tema buatanmu sendiri muncul di baris yang sama. Tampilan langsung berganti saat kamu memilih; tidak ada yang perlu disimpan.',
	'help.says.tone':
		'Seberapa kuat warna jendela ikut bergerak mengikuti bunyi yang sedang diputar, dengan empat pilihan: "Mati", "Halus", "Sedang", dan "Kuat". Mati berarti warnanya diam saja.',
	'help.says.themeFolder':
		'Membuka folder tempat Onsa membaca tema buatan sendiri, di penjelajah berkas komputermu.',
	'help.says.themeCopy':
		'Menyalin tema yang sedang kamu pakai ke folder itu sebagai berkas yang bisa kamu ubah warnanya. Berkas yang sudah ada tidak pernah ditimpa.',
	'help.says.language':
		'Bahasa seluruh tulisan di jendela, termasuk bantuan ini: "Indonesia" atau "Inggris".',
	'help.says.debug':
		'Menyalakan catatan kejadian yang jauh lebih rinci. Dipakai ketika ada yang salah dan kamu ingin melaporkannya; matikan lagi sesudahnya, karena catatannya cepat membesar.',
	'help.says.openLogs':
		'Membuka folder tempat catatan itu disimpan, supaya berkasnya bisa kamu kirim.',
	'help.says.closeToTray':
		'Kalau dinyalakan, menutup jendela tidak menghentikan Onsa: ia menyembunyikan diri di dekat jam dan musiknya terus berjalan.',

	'help.says.search':
		'Mencari di library-mu sambil kamu mengetik. Ctrl+F melompat ke sini dari mana pun.',
	'help.says.searchClear': 'Mengosongkan kotak cari dan mengembalikan halaman yang tadi terbuka.',
	'help.says.queueButton':
		'Hanya muncul kalau jendelanya sempit: membuka dan menutup panel antrean dan lirik.',
	'help.name.menuButton': 'Tombol tiga garis',
	'help.says.menuButton':
		'Hanya muncul kalau jendelanya terlalu sempit untuk navigasi kiri: membuka daftar halaman sebagai laci.',
	'help.says.helpButton':
		'Tombol ini: membuka penjelasan halaman yang sedang kamu buka. Tempatnya sama di semua halaman, dan panelnya bisa ditutup kapan saja tanpa mengganggu apa yang sedang kamu kerjakan.',
	'help.says.previous':
		'Kembali ke lagu sebelumnya. Kalau lagu sudah berjalan lebih dari beberapa detik, tombol ini mengulang lagu itu dari awal dulu.',
	'help.says.playPause':
		'Memutar, dan menjeda kalau sedang berjalan. Bilah spasi di papan ketik melakukan hal yang sama.',
	'help.says.next': 'Lompat ke lagu berikutnya di antrean.',
	'help.says.volume':
		'Kerasnya suara Onsa sendiri, terpisah dari volume sistem. Ctrl+panah atas dan bawah menaikkan dan menurunkannya.',
	'help.says.openNowPlaying':
		'Membuka layar penuh untuk lagu yang sedang diputar. Tombolnya mati kalau belum ada yang diputar.',
	'help.name.sleep': 'Sleep timer',
	'help.says.sleep':
		'Menyuruh Onsa berhenti sendiri setelah waktu yang kamu pilih, atau setelah lagu yang sedang diputar selesai.',
	'help.says.mini':
		'Mengecilkan jendela jadi satu baris tipis yang muat di pojok layar. Tombol yang sama mengembalikannya.',
	'help.name.signal': 'Jalur sinyal',
	'help.says.signal':
		'Baris paling bawah: rantai yang menunjukkan apa saja yang dilewati suara dari berkas sampai keluar. Tahapnya, dari kiri ke kanan: bentuk berkasnya, "Resample" (menghitung ulang kecepatan suara supaya cocok dengan perangkatnya), "ReplayGain" (meratakan kenyaringan antar lagu), "EQ" (pengatur nada), "Limiter" (penjaga supaya suara tidak pecah), dan di ujung kanan nama perangkat output beserta caranya tersambung. Tahap yang menyala sedang bekerja, yang redup sedang mati. Klik sebuah tahap untuk mematikan atau menyalakannya; klik kanan untuk membuka pengaturannya.',
	'help.says.queueTab':
		'Daftar lagu yang akan diputar sesudah yang sekarang. Lagu bisa diseret naik-turun untuk mengubah urutannya.',
	'help.says.lyricsTab':
		'Lirik lagu yang sedang diputar, kalau ada. Baris yang sedang dinyanyikan menyala, dan klik sebuah baris melompat ke bagian itu.',
	'help.says.shuffle':
		'Mengacak urutan antrean. Lagu yang sedang diputar tetap yang sedang diputar.',
	'help.says.repeat':
		'Berputar-putar: mati, mengulang seluruh antrean, atau mengulang satu lagu saja.',
	'help.says.clearQueue': 'Mengosongkan daftar tunggu. Tidak ada berkas yang dihapus.',
	'help.says.saveQueue': 'Menyimpan isi antrean sekarang sebagai playlist baru.',
	'help.says.removeFromQueue': 'Mengeluarkan satu lagu dari daftar tunggu.',
	'help.says.lyricsEarlier':
		'Menggeser lirik supaya muncul seperempat detik lebih awal, kalau kata-katanya terasa ketinggalan dari suaranya. Geserannya disimpan untuk lagu itu saja dan tetap ada lain kali.',
	'help.says.lyricsLater':
		'Menggeser lirik seperempat detik lebih lambat, kalau kata-katanya mendahului suaranya. Juga disimpan per lagu.',
	'help.says.lyricsReset': 'Mengembalikan geseran lagu itu ke nol.',
	'help.says.lyricsSave':
		'Menyimpan lirik yang sedang terlihat sebagai berkas teks kecil di sebelah berkas lagunya, supaya jadi milikmu sendiri dan tetap ada tanpa internet.',
	'help.says.lyricsAgain':
		'Bertanya sekali lagi ke layanan lirik untuk lagu ini. Hanya bisa kalau pencarian lirik lewat internet kamu nyalakan.',
	'help.name.contents': 'Daftar halaman',
	'help.says.contents':
		'Setiap halaman Onsa. Klik satu untuk membukanya sekaligus membuka penjelasannya di panel kanan.',
	'help.says.firstRunAgain':
		'Menampilkan lagi layar pertama yang muncul saat Onsa baru dipasang, termasuk daftar hal yang mudah terlewat. Tidak ada yang berubah karena membukanya.',

	'help.page.intro':
		'Onsa memutar berkas musik yang sudah ada di komputermu. Halaman ini menjelaskan tiap bagiannya. Di tiap halaman ada tombol tanda tanya di kanan atas yang membuka penjelasan halaman itu saja.',
	'help.page.contents': 'Halaman di Onsa',
	'help.page.things': 'Hal yang perlu dijelaskan sendiri',
	'help.lyrics.head': 'Lirik: dari mana datangnya',
	'help.lyrics.body':
		'Onsa mencari lirik di empat tempat, berurutan, dan yang ketemu lebih dulu yang dipakai. Pertama, lirik yang kamu sunting sendiri di dalam Onsa. Kedua, berkas teks kecil berakhiran .lrc yang ada di sebelah berkas lagunya, dengan nama yang sama. Ketiga, lirik yang tersimpan di dalam berkas lagunya sendiri. Keempat, layanan lirik LRCLIB di internet — dan itu hanya kalau kamu menyalakannya di Pengaturan → Lirik.',
	'help.lyrics.offset':
		'Kalau liriknya benar tapi waktunya meleset, dua tombol di panel lirik menggesernya seperempat detik lebih awal atau lebih lambat. Geseran itu disimpan untuk lagu itu sendiri, bukan untuk semua lagu, dan masih ada ketika lagunya kamu putar lagi besok. Mengganti atau melupakan liriknya tidak menghapus geseran itu.',
	'help.theme.head': 'Tema buatan sendiri',
	'help.theme.body':
		'Tema adalah satu berkas teks berakhiran .json di folder tema Onsa. Cara paling mudah membuatnya: buka Pengaturan → Tampilan, tekan "Salin tema yang dipakai sekarang", lalu buka berkas hasilnya dengan editor teks apa pun dan ganti warnanya. Buka Onsa lagi dan temamu muncul di daftar. Warna ditulis sebagai kode enam angka seperti #3fb950. Bagian "role" adalah warna menurut artinya: label untuk tulisan tetap, adjustable untuk nilai yang bisa kamu atur, active untuk yang sedang aktif, position untuk posisi putar, caution untuk yang mendekati batas, clip untuk yang melewati batas. Nilai yang salah atau hilang diganti nilai bawaan, jadi tema yang keliru tidak merusak apa pun; berkas yang sama sekali tidak terbaca disebutkan di halaman Tampilan beserta alasannya.',
	'help.theme.example': 'Contoh isi berkas tema',
	'help.theme.copy': 'Salin contohnya',
	'help.theme.copied': 'Tersalin.',
	'help.write.head': '"Belum ditulis ke berkas": kapan Onsa menyentuh berkasmu',
	'help.write.body':
		'Keterangan lagu — judul, artis, album, tahun, genre, lirik — tersimpan di dalam berkas lagunya sendiri. Ketika kamu mengubahnya di Onsa, perubahan itu disimpan dulu di dalam Onsa saja, dan berkasnya sama sekali tidak disentuh. Itulah arti tanda "Belum ditulis ke berkas". Yang kamu lihat di seluruh Onsa adalah nilai yang sudah kamu ubah, sementara berkasnya masih berisi yang lama. Berkasnya baru berubah kalau kamu menekan "Tulis ke berkas" di editor satu lagu, atau menjalankan "Tulis ke file" di halaman Rapikan. Keduanya bisa dibatalkan lewat Riwayat.',
	'help.write.never':
		'Yang tidak pernah menyentuh berkasmu: memutar, playlist, antrean, pencarian, semua pengaturan suara, dan tema. Yang bisa menyentuh berkasmu hanya empat: menulis keterangan ke dalam berkas, mengganti nama berkas berpola, menyimpan lirik sebagai berkas teks kecil berakhiran .lrc di sebelah lagu, dan hasil unduhan yang ditulis sebagai berkas baru.',
	'help.programs.head': 'yt-dlp dan ffmpeg',
	'help.programs.body':
		'Keduanya program terpisah yang bukan bagian dari Onsa, dan hanya dipakai di halaman Unduhan. yt-dlp wajib: dialah yang mengambil audio dari alamat web. ffmpeg tidak wajib: tanpa dia unduhan tetap berjalan, tapi berkas hasilnya datang apa adanya — tanpa judul dan artis yang tertanam di dalamnya, tanpa gambar sampul, dan tanpa bisa diubah ke MP3 atau FLAC. Onsa tidak mengunduh keduanya diam-diam: ada tombol persetujuan untuk masing-masing, berkasnya diambil dari halaman rilis resminya, dan dicocokkan dengan sidik berkas yang diterbitkan rilis itu sebelum disimpan. Kalau keduanya sudah ada di komputermu, nyalakan "pakai program yang sudah ada di sistem" dan Onsa tidak mengunduh apa pun.',
	'help.limits.head': 'Yang belum bisa di versi ini',
	'help.limits.opus':
		'Alamat web yang audionya hanya tersedia dalam format Opus ditolak di muka dengan pesan, karena Onsa belum punya cara memutar format itu. Lebih baik ditolak sejak awal daripada berkasnya terunduh lalu tidak pernah bisa diputar.',
	'help.limits.folder':
		'Folder library belum bisa dihapus dari dalam Onsa; yang ada baru menambah dan memeriksa ulang. Lagu yang berkasnya hilang ditandai "hilang" dan tidak ikut diputar.',
	'help.limits.lastfm': 'Belum ada pencatatan lagu ke Last.fm.',
	'help.log.head': 'Kalau ada yang salah',
	'help.log.body':
		'Onsa menulis catatan kejadian di komputermu. Untuk melaporkan sesuatu: buka Pengaturan → Tentang, nyalakan "Log debug", tutup Onsa lalu buka lagi, kerjakan hal yang bermasalah itu sekali lagi, lalu tekan "Buka folder log". Kirim berkas hari itu. Catatannya berisi jalur berkas dan nama lagu di library-mu, jadi lihat dulu isinya sebelum dikirim ke orang lain. Matikan lagi "Log debug" sesudahnya, karena catatannya cepat membesar.',
	'help.firstRun.head': 'Layar pertama',
	'help.firstRun.body':
		'Layar yang muncul saat Onsa pertama kali dibuka, berisi pemilihan folder musik, tema, dan daftar singkat hal yang mudah terlewat. Membukanya lagi tidak mengubah apa pun.',

	'help.says.themeExample':
		'Menyalin contoh berkas tema di atas ke papan klip, supaya bisa ditempel ke editor teks.',
	'help.name.contentsLinks': 'Nama halaman di daftar',
	'help.says.contentsLinks':
		'Tiap nama di daftar itu bisa diklik: halamannya terbuka dan penjelasannya ikut terbuka di panel kanan. Yang bukan halaman — baris atas jendela, baris pemutar, panel kanan, hasil pencarian, dan halaman rincian seperti satu album — hanya membuka penjelasannya.',

	'help.name.queueRow': 'Baris lagu di antrean',
	'help.says.queueRow':
		'Satu lagu yang menunggu giliran. Klik dua kali untuk melompat ke lagu itu sekarang; seret naik-turun untuk mengubah urutannya. Yang sedang diputar ditandai.',

	'downloads.seeDetail': 'Lihat detail',
	'downloads.otherTrouble':
		'yt-dlp berhenti dengan keluhan yang belum dikenali Onsa. Kalimat aslinya ada di bawah.',
	'downloads.unsupportedSite':
		'yt-dlp tidak mengenali situs itu, jadi tidak ada yang bisa diambil dari sana.',
	'downloads.needsSignIn':
		'Isi itu hanya bisa diambil oleh orang yang sudah masuk ke akunnya di situs itu. Onsa tidak masuk ke akun mana pun.',
	'downloads.geoBlocked': 'Situsnya tidak mengizinkan isi itu diambil dari negara ini.',
	'downloads.gone':
		'Isinya sudah tidak ada di alamat itu — dihapus, dipindah, atau alamatnya salah ketik.',
	'downloads.tooManyAsks':
		'Situsnya menolak karena terlalu banyak permintaan dalam waktu singkat. Coba lagi beberapa menit kemudian.',
	'downloads.siteRefused':
		'Situsnya menolak permintaan Onsa. Biasanya karena isinya memang tidak dibuka untuk umum.',
	'downloads.cannotReach':
		'Situsnya tidak bisa dihubungi. Periksa sambungan internetmu, lalu coba lagi.',
	'programs.fromSystemAnyway': 'ada di sistem, dipakai yt-dlp sendiri',
	'programs.systemAnywayWhat':
		'Saklar di bawah hanya mengatur program mana yang dijalankan Onsa. yt-dlp mencari ffmpeg di sistem atas namanya sendiri, jadi ffmpeg ini tetap dipakainya meski saklar itu mati — dan karena itu konversi tetap tersedia.',

	'downloads.siteBroken':
		'Situsnya menjawab, tapi sedang bermasalah di sisi mereka. Coba lagi nanti.',

	'librarySettings.folderGone': 'tidak ada di tempatnya',
	'librarySettings.goneWhat':
		'{n} folder tidak ada di tempatnya lagi. Biasanya karena drive-nya dicabut, atau foldernya dipindah atau diganti nama lewat program lain.',
	'librarySettings.goneNothingLost':
		'Onsa tidak menghapus apa pun karena ini. Lagu dari folder itu tetap ada di library; yang berkasnya tidak bisa dibuka ditandai hilang. Colokkan lagi drive-nya lalu tekan "Scan ulang", atau tambahkan tempat barunya kalau foldernya kamu pindah.'
} as const;

export type MessageKey = keyof typeof id;

const en: Record<MessageKey, string> = {
	'shell.loading': 'Loading…',

	'startup.title': 'Onsa cannot open its library',
	'startup.newer':
		'This library file was written by a newer version of Onsa. This version cannot read it, and forcing it could damage what is inside.',
	'startup.newerWhat':
		'Open it with the newest Onsa. If you would rather use this version, move that file somewhere else first — Onsa will start a new, empty library, and the old file stays whole for later.',
	'startup.unreadable':
		'This library file cannot be read. Usually that means the computer stopped while Onsa was writing, or the file was damaged on the disk.',
	'startup.unreadableWhat':
		'Move that file somewhere else and open Onsa again: a new library is made and your music folders are scanned afresh. Your music files themselves are untouched by any of this. Keep the old file if you want to try to rescue it.',
	'startup.nothingTouched': 'Onsa neither deletes nor repairs that file by itself. That decision is yours.',
	'startup.where': 'The file is at',
	'startup.openFolder': 'Open the folder',
	'startup.said': 'What it reported',

	'nav.library': 'Library',
	'nav.tracks': 'Tracks',
	'nav.albums': 'Albums',
	'nav.artists': 'Artists',
	'nav.genres': 'Genres',
	'nav.folders': 'Folders',
	'nav.settings': 'Settings',
	'settings.output': 'Output & Quality',
	'settings.dsp': 'DSP & EQ',
	'settings.library': 'Library',
	'settings.lyrics': 'Lyrics',
	'settings.appearance': 'Appearance',
	'settings.about': 'About',

	'search.placeholder': 'Search tracks, albums, artists',
	'search.clear': 'Clear search',
	'search.tracks': 'Tracks',
	'search.albums': 'Albums',
	'search.artists': 'Artists',
	'search.none': 'Nothing matches.',

	'column.number': '#',
	'column.title': 'Title',
	'column.artist': 'Artist',
	'column.album': 'Album',
	'column.year': 'Year',
	'column.duration': 'Length',
	'column.resize': 'Resize the {name} column',
	'column.resizeHint': 'Drag, or the left and right arrows',
	'column.choose': 'Choose columns',
	'column.reset': 'Back to the defaults',
	'column.sortHint': 'Sort',

	'track.unknownArtist': 'Unknown artist',
	'track.missing': 'missing',
	'track.failed': 'unreadable',
	'track.playHint': 'Double-click or press Enter to play',

	'library.empty': 'The library is empty.',
	'library.tracks': '{n} tracks',

	'album.play': 'Play album',
	'album.back': 'All albums',
	'album.tracks': '{n} tracks',

	'transport.play': 'Play',
	'transport.pause': 'Pause',
	'transport.next': 'Next',
	'transport.previous': 'Previous',
	'transport.seek': 'Playback position',
	'transport.volume': 'Volume',
	'transport.nothing': 'Nothing playing',
	'transport.more': 'More',
	'transport.noOutput': 'No audio output. Check the device in Output & Quality.',
	'transport.failed': 'A track could not be played and was skipped: {path}',

	'meter.label': 'Peak meter',
	'meter.left': 'L',
	'meter.right': 'R',
	'meter.vu': 'VU',
	'meter.clip': 'CLIP',
	'meter.limit': 'LIM',

	'spectrum.label': 'Spectrum',
	'spectrum.rest': 'At rest',

	'signal.label': 'Signal path',
	'signal.idle': 'No signal',
	'signal.hint': 'Click: switch on or off. Right-click: settings.',
	'signal.resample': 'Resample',
	'signal.resampleShort': 'RS',
	'signal.replaygain': 'ReplayGain',
	'signal.replaygainShort': 'RG',
	'signal.more': 'The other stages',
	'signal.eq': 'EQ',
	'signal.limiter': 'Limiter',
	'signal.bands': '{n} bands',
	'signal.filters': '{n} filters',
	'signal.album': 'album',
	'signal.track': 'track',

	'queue.title': 'Queue',
	'queue.count': '{n} tracks',
	'queue.empty': 'The queue is empty. Double-click a track to play it.',
	'queue.shuffle': 'Shuffle',
	'queue.repeat': 'Repeat',
	'queue.repeatOff': 'Repeat: off',
	'queue.repeatAll': 'Repeat: the whole queue',
	'queue.repeatOne': 'Repeat: this track',
	'queue.clear': 'Clear the queue',
	'queue.remove': 'Remove from the queue',
	'queue.playNext': 'Play next',
	'queue.addToEnd': 'Add to the queue',
	'queue.dragHint': 'Drag to reorder, or Alt+arrow',

	'lyrics.title': 'Lyrics',
	'lyrics.nothingPlaying': 'Nothing is playing.',
	'lyrics.none': 'No lyrics for this song.',
	'lyrics.looking': 'Looking for lyrics…',
	'lyrics.offline': 'Looking lyrics up on the internet is off.',
	'lyrics.unsynced': 'Without timing',
	'lyrics.source.edited': 'Your own edit',
	'lyrics.source.file': '.lrc file',
	'lyrics.source.tag': "The file's tags",
	'lyrics.source.lrclib': 'LRCLIB',
	'lyrics.earlier': 'Words earlier',
	'lyrics.later': 'Words later',
	'lyrics.offsetValue': 'Shifted {seconds} s',
	'lyrics.resetOffset': 'Put the shift back',
	'lyrics.lookAgain': 'Look again',
	'lyrics.saveBeside': 'Save .lrc',
	'lyrics.seekHint': 'Click a line to jump to it',
	'lyrics.settingsTitle': 'Lyrics',
	'lyrics.sourcesHint':
		'Onsa reads a .lrc file beside the song and the words kept in the song’s own tags, with no network and no permission needed.',
	'lyrics.onlineLabel': 'Look lyrics up on the internet (LRCLIB)',
	'lyrics.onlineHint':
		'What is sent is the artist, the title, the album and the length of the song. Turning it off also stops the lookups already under way.',
	'lyrics.writeBesideLabel': 'Keep words from the internet as a .lrc beside the song',
	'lyrics.writeBesideHint': 'Then they are yours, and they are still there without the internet.',
	'nowPlaying.open': 'Now Playing',
	'nowPlaying.close': 'Back to the library',
	'nowPlaying.format': 'Format',
	'nowPlaying.output': 'Output',
	'mini.open': 'Mini player',
	'mini.close': 'Full window',
	'sleep.title': 'Sleep timer',
	'sleep.when': 'Stop',
	'sleep.minutes': 'After {n} minutes',
	'sleep.endOfTrack': 'At the end of this track',
	'sleep.customMinutes': 'Minutes',
	'sleep.customTracks': 'Tracks',
	'sleep.action': 'Then',
	'sleep.pause': 'Pause',
	'sleep.stop': 'Stop',
	'sleep.quit': 'Close Onsa',
	'sleep.fade': 'Fade out over',
	'sleep.start': 'Start',
	'sleep.cancel': 'Cancel',
	'sleep.left': '{time} left',
	'sleep.tracksLeft': '{n} tracks left',
	'sleep.fading': 'Fading out',
	'tray.show': 'Show Onsa',
	'tray.play': 'Play or pause',
	'tray.quit': 'Quit',
	'tray.closeToTray': 'Closing the window only hides Onsa in the tray',

	'firstRun.welcome': 'Welcome to Onsa',
	'firstRun.intro': 'Three short steps, then your music is ready to play.',
	'firstRun.folderStep': '1 · Music folder',
	'firstRun.folderHint': 'Choose the folder your music lives in. Onsa only reads it.',
	'firstRun.pickFolder': 'Choose folder…',
	'firstRun.changeFolder': 'Change folder…',
	'firstRun.dialogTitle': 'Choose a music folder',
	'firstRun.themeStep': '2 · Theme',
	'firstRun.themeHint': 'The window changes as soon as you pick a theme. You can change it any time.',
	'firstRun.scanStep': '3 · Scan',
	'firstRun.start': 'Start scan',
	'firstRun.open': 'Open library',
	'firstRun.moreStep': 'What else is here',
	'firstRun.moreHint':
		'Three things that are easy to miss. Click a name to go straight there; they are all in the navigation on the left as well.',
	'firstRun.moreTidy':
		'Tidying tags: renaming by pattern with a preview, automatic fixes that need no network, and — if you turn it on — suggestions from the internet that still wait for your yes.',
	'firstRun.moreDownloads':
		'Fetching audio from a URL with yt-dlp, one at a time, once you have approved the programs Onsa downloads for it.',
	'firstRun.moreLyrics':
		'Lyrics from an .lrc file beside the song, from the tags inside it, or from LRCLIB if you turn that on; the line being sung lights up in the right-hand panel.',
	'firstRun.again': 'The first screen, opened again. There is nothing you have to do here.',
	'firstRun.againScan':
		'You already have a library. Choose a folder above only if you want to add another one.',
	'firstRun.close': 'Close',

	'scan.running': 'Scanning… {seen} files, {read} read',
	'scan.done': '{seen} files in {seconds} s',
	'scan.failed': '{n} files unreadable',
	'scan.idle': 'No scan yet',

	'output.title': 'Output',
	'output.device': 'Output device',
	'output.systemDefault': 'System default (follows changes)',
	'output.sampleRate': 'Sample rate',
	'output.followDevice': 'Follow device',
	'output.current': 'Now: {name}, {rate}',
	'output.none': 'No output is open',
	'output.quality': 'Resampler quality',
	'output.buffer': 'Buffer',
	'output.bufferNote': 'A new buffer size applies when the output reopens.',
	'output.dither': 'TPDF dither for 16-bit output',
	'output.powerSave': 'Power saving',
	'output.powerSaveHint':
		'The largest buffer, the cheapest resampler, meters and visualizer drawn less often, and no tone colour. The buffer and resampler above apply again once this is off.',
	'output.matchSource': 'Match the source',
	'output.matchSourceHint':
		'The device reopens at each track\'s own sample rate, when it supports it.',
	'quality.fast': 'Fast',
	'quality.balanced': 'Balanced',
	'quality.best': 'Best',
	'buffer.low': 'Low',
	'buffer.normal': 'Normal',
	'buffer.large': 'Large',

	'crossfade.title': 'Crossfade',
	'crossfade.duration': 'Length',
	'crossfade.off': 'Off',
	'crossfade.curve': 'Curve',
	'crossfade.albumGapless': 'No crossfade between consecutive tracks of one album',
	'crossfade.skip': 'Crossfade when skipping',
	'curve.equalPower': 'Equal power',
	'curve.linear': 'Linear',

	'replaygain.title': 'ReplayGain',
	'replaygain.mode': 'Mode',
	'replaygain.preamp': 'ReplayGain preamp',
	'replaygain.fallback': 'Untagged tracks',
	'replaygain.preventClipping': 'Prevent clipping using the tagged peak',
	'rg.off': 'Off',
	'rg.track': 'Track',
	'rg.album': 'Album',
	'rg.auto': 'Auto',

	'eq.title': 'Equaliser',
	'eq.enabled': 'EQ on',
	'eq.graphic': 'Graphic',
	'eq.parametric': 'Parametric',
	'eq.flat': 'Flatten',
	'eq.curve': 'Response curve',
	'eq.addBand': 'Add filter',
	'eq.remove': 'Remove filter',
	'eq.bandOn': 'On',
	'eq.type': 'Type',
	'eq.freq': 'Frequency (Hz)',
	'eq.gain': 'Gain (dB)',
	'eq.q': 'Q',
	'eq.noBands': 'No filters yet. Add one or import an AutoEQ preset.',
	'eq.maxBands': 'At most {n} filters.',
	'eq.import': 'Import AutoEQ…',
	'eq.export': 'Export AutoEQ…',
	'eq.importTitle': 'Choose an EQ preset',
	'eq.exportTitle': 'Save the EQ preset',
	'eq.fileFilter': 'Equalizer APO / AutoEQ preset',
	'eq.imported': '{n} filters loaded.',
	'eq.importedSkipped': '{n} filters loaded, {skipped} lines skipped (see the log).',
	'eq.exported': 'Preset saved.',
	'filter.peaking': 'Peaking',
	'filter.lowShelf': 'Low shelf',
	'filter.highShelf': 'High shelf',
	'filter.lowPass': 'Low pass',
	'filter.highPass': 'High pass',
	'filter.notch': 'Notch',

	'preamp.title': 'Preamp',
	'preamp.auto': 'Auto preamp, so the EQ cannot clip',
	'preamp.manual': 'Manual preamp',
	'preamp.effective': 'In effect: {value}',

	'limiter.title': 'Limiter',
	'limiter.enabled': 'Limiter on (ceiling −0.1 dBFS)',
	'limiter.release': 'Release',

	'librarySettings.folders': 'Library folders',
	'librarySettings.add': 'Add folder…',
	'librarySettings.rescan': 'Rescan',
	'librarySettings.cannotRemove':
		'A folder cannot be removed here in this version. A folder whose files are gone is marked missing on the next rescan.',

	'about.version': 'Version {version}',
	'about.logs': 'Logs',
	'about.logDir': 'Log folder',
	'about.openLogs': 'Open log folder',
	'about.debug': 'Debug log',
	'about.debugHint':
		'Records more detail. Switch it on, repeat the problem, then send the newest file from the log folder.',

	'theme.label': 'Theme',
	'toneColor.label': 'Tone colour',
	'toneColor.off': 'Off',
	'toneColor.subtle': 'Subtle',
	'toneColor.medium': 'Medium',
	'toneColor.strong': 'Strong',
	'toneColor.hint':
		'The colour follows the character of the sound: bright sound leans towards the theme’s cool colour, heavy sound towards its warm one. Each theme names both ends and which parts follow them.',

	'theme.userFolder': 'Your own themes',
	'theme.openFolder': 'Open the theme folder',
	'theme.userHint':
		'Put .json theme files there, then open Onsa again to see them in the list.',
	'theme.copy': 'Copy the theme in use',
	'theme.copyHint':
		'Copies the theme you are using into that folder, as an example to change. An existing file is never overwritten.',
	'theme.copyName': 'My theme',
	'theme.copied': 'Copied to {file}. Open Onsa again for it to appear in the list.',
	'theme.troubles': 'Theme files that could not be used',
	'theme.troublesHint': 'These files are in the theme folder but cannot be read, so they are not in the list.',
	'language.label': 'Language',
	'language.id': 'Indonesian',
	'language.en': 'English',

	'nav.playlists': 'Playlists',
	'nav.allPlaylists': 'See them all',
	'playlist.manual': 'Playlist',
	'playlist.smart': 'Smart playlist',
	'playlist.count': '{n} tracks',
	'playlist.none': 'No playlists yet.',
	'playlist.new': 'New playlist',
	'playlist.newSmart': 'New smart playlist',
	'playlist.import': 'Import M3U8',
	'playlist.export': 'Export M3U8',
	'playlist.more': 'Playlist actions',
	'playlist.dragHint': 'Drag to reorder, or Alt with an arrow',
	'playlist.rename': 'Rename',
	'playlist.duplicate': 'Duplicate',
	'playlist.delete': 'Delete playlist',
	'playlist.deleteHint': 'Delete "{name}"? The tracks stay in the library.',
	'playlist.namePrompt': 'Playlist name',
	'playlist.copySuffix': '{name} (copy)',
	'playlist.untitled': 'Playlist',
	'playlist.empty': 'This playlist is empty.',
	'playlist.emptySmart': 'Nothing matches these rules yet.',
	'playlist.play': 'Play the playlist',
	'playlist.addTo': 'Add to playlist',
	'playlist.carried': '{n} tracks',
	'playlist.addedTo': '{n} tracks added to "{name}".',
	'playlist.removeTrack': 'Remove from the playlist',
	'playlist.moveUp': 'Move up',
	'playlist.moveDown': 'Move down',
	'playlist.saveQueue': 'Save the queue as a playlist',
	'playlist.queueName': 'Queue {date}',
	'playlist.exported': 'Saved to {path}.',
	'playlist.imported': '{n} tracks went into "{name}".',
	'playlist.importedNone': 'The library holds none of the tracks in that file.',
	'playlist.importMissing': '{n} entries are not in the library.',
	'playlist.fileFilter': 'M3U8 playlist',
	'playlist.editRules': 'Edit the rules',
	'playlist.cancel': 'Cancel',
	'playlist.save': 'Save',
	'playlist.create': 'Create',

	'rules.title': 'Smart playlist rules',
	'rules.match': 'Match',
	'rules.matchAll': 'all rules',
	'rules.matchAny': 'any rule',
	'rules.add': 'Add a rule',
	'rules.remove': 'Remove the rule',
	'rules.none': 'With no rules, the whole library matches.',
	'rules.sort': 'Order by',
	'rules.descending': 'Descending',
	'rules.limit': 'Limit',
	'rules.noLimit': 'No limit',
	'rules.preview': 'Preview: {n} tracks',
	'rules.previewMore': 'Preview: the first {n} tracks',
	'rules.invalid': 'One of the rules has no usable value yet.',
	'rules.days': 'days',
	'rules.and': 'and',
	'rules.field': 'Field',
	'rules.op': 'Comparison',
	'rules.value': 'Value',
	'rules.nothingToFill': '—',
	'rules.previewNone': 'Nothing matches yet.',
	'rules.previewRest': '…and {n} more',

	'field.title': 'Title',
	'field.artist': 'Artist',
	'field.album': 'Album',
	'field.album_artist': 'Album artist',
	'field.track_number': 'Track number',
	'field.disc_number': 'Disc number',
	'field.composer': 'Composer',
	'field.genre': 'Genre',
	'field.year': 'Year',
	'field.rating': 'Rating',
	'field.play_count': 'Play count',
	'field.last_played': 'Last played',
	'field.added_at': 'Date added',
	'field.codec': 'Codec',
	'field.duration': 'Length (seconds)',
	'field.folder': 'Folder',
	'field.sample_rate': 'Sample rate (Hz)',
	'field.bit_depth': 'Bit depth',
	'field.bitrate': 'Bitrate (kbit/s)',
	'field.has_cover': 'Cover',
	'field.lyrics': 'Lyrics',

	'editor.open': 'Edit tags…',
	'editor.title': 'Edit tags',
	'editor.save': 'Save in Onsa',
	'editor.saved': 'Saved in Onsa. The file itself is unchanged.',
	'editor.write': 'Write to the file',
	'editor.written': 'Written to the file.',
	'editor.writeFailed': 'The file could not be written.',
	'editor.revert': 'Back to what the file says',
	'editor.unwritten': 'Not written to the file yet',
	'editor.editedHere': 'Changed in Onsa',
	'editor.close': 'Close',
	'editor.gain': 'ReplayGain (from the file, not editable)',
	'editor.gainNone': 'This file says nothing about ReplayGain.',
	'editor.gainTrack': 'Track',
	'editor.gainAlbum': 'Album',
	'editor.peak': 'peak {value}',
	'editor.laterFields': 'Track total, disc total and comment cannot be changed in this version.',
	'field.has_artist': 'Artist tag',

	'op.contains': 'contains',
	'op.not_contains': 'does not contain',
	'op.is': 'is',
	'op.is_not': 'is not',
	'op.starts_with': 'starts with',
	'op.not_starts_with': 'does not start with',
	'op.eq': 'is',
	'op.ne': 'is not',
	'op.lt': 'is less than',
	'op.le': 'is at most',
	'op.gt': 'is more than',
	'op.ge': 'is at least',
	'op.between': 'is between',
	'op.in_last': 'in the last',
	'op.not_in_last': 'not in the last',
	'op.before': 'before',
	'op.after': 'after',
	'op.yes': 'is there',
	'op.no': 'is missing',

	'sortField.title': 'Title',
	'sortField.artist': 'Artist',
	'sortField.album': 'Album',
	'sortField.year': 'Year',
	'sortField.duration': 'Length',
	'sortField.rating': 'Rating',
	'sortField.play_count': 'Play count',
	'sortField.last_played': 'Last played',
	'sortField.added_at': 'Date added',
	'sortField.random': 'Random',

	'settings.metadata': 'Metadata',
	'metadata.internet': 'Internet access',
	'metadata.online': 'Look metadata up on the internet',
	'metadata.onlineHint':
		'While this is off, Onsa never contacts any service for metadata. Anything the internet suggests still has to be approved before it is applied.',
	'metadata.acoustid': 'AcoustID key',
	'metadata.acoustidHint':
		'AcoustID recognises a track from the sound itself, which is what untagged files need. The key is free: register an application at acoustid.org/new-application, then copy the application API key from acoustid.org/my-applications. Not the personal key on the account page — that one only signs fingerprint submissions.',
	'metadata.key': 'Application key',
	'metadata.keyPlaceholder': 'paste your key here',
	'metadata.keySave': 'Save the key',
	'metadata.keyClear': 'Remove the key',
	'metadata.keySaved': 'The key is stored. Onsa never shows it again.',
	'metadata.keyCleared': 'The key is gone.',
	'metadata.keyTry': 'Try the key',
	'metadata.keyTrying': 'Trying…',
	'metadata.keyWorks': 'AcoustID accepts this key.',
	'metadata.keyRefused': 'AcoustID refused this key. Check it is the application key, not the account one.',
	'metadata.keyOffline': 'Turn on "Look metadata up on the internet" above first.',
	'metadata.keyUnreachable': 'AcoustID could not be reached. Check the connection and try again.',
	'metadata.keyFromSettings': 'Using the key you stored here.',
	'metadata.keyFromEnvironment': 'Using the key from the ONSA_ACOUSTID_API_KEY environment variable.',
	'metadata.keyFromBuild': 'Using the key built into this copy of Onsa.',
	'metadata.keyMissing': 'There is no key yet. AcoustID recognition will not run.',

	'nav.tidy': 'Tidy up',
	'tidy.scopeHeld': 'Held to this folder',
	'tidy.scopeWhole': 'THE WHOLE LIBRARY',
	'tidy.scopeWholeHint': 'No folder is holding this back. Anything you run can reach your real library.',
	'tidy.inScope': '{n} tracks in scope',
	'tidy.limit': 'Per run',
	'tidy.pickFolder': 'Choose a folder',
	'tidy.pickFolderTitle': 'Choose the folder to tidy',
	'tidy.clearFolder': 'Remove the folder',
	'tidy.jobEdit': 'Edit in bulk',
	'tidy.jobWrite': 'Write to the files',
	'tidy.jobRename': 'Rename by pattern',
	'tidy.editWhat': 'Sets one field on every track in scope. This stays inside Onsa; the files are not touched.',
	'tidy.writeWhat': 'Writes the edits that are still only in Onsa into the files themselves.',
	'tidy.renameWhat': 'Lays the files out by a pattern. Always listed before anything moves.',
	'tidy.valueEmpty': 'leave empty to clear the edit',
	'tidy.pattern': 'Pattern',
	'tidy.renameRoot': 'Into this folder',
	'tidy.look': 'See what would happen',
	'tidy.summary': 'What would happen',
	'tidy.countTracks': 'tracks touched',
	'tidy.countFields': 'field values changed',
	'tidy.countWritten': 'files rewritten',
	'tidy.countMoved': 'files moved or renamed',
	'tidy.skipped': 'Left alone',
	'tidy.skipOutside': 'outside the folder',
	'tidy.skipOverLimit': 'past the cap for one run',
	'tidy.skipNoChange': 'already say that',
	'tidy.skipNameTaken': 'the name would clash',
	'tidy.nothingToDo': 'There is nothing to do.',
	'tidy.apply': 'Apply',
	'tidy.working': 'Working…',
	'tidy.done': 'Done: {n} changes.',
	'tidy.someFailed': '{n} failed; those files are untouched.',
	'tidy.history': 'What has been run',
	'tidy.noHistory': 'Nothing has been run yet.',
	'tidy.steps': '{n} steps',
	'tidy.undo': 'Undo',
	'tidy.undone': 'taken back',
	'tidy.noteEdit': 'Bulk edit: {field}',
	'tidy.noteWrite': 'Write to the files',
	'tidy.noteRename': 'Rename by pattern',

	'tidy.jobMatch': 'Look it up online',
	'tidy.matchWhat': 'Finds out what these tracks are, through AcoustID and MusicBrainz. What comes back is a suggestion, not a change: whatever you tick still goes through the summary, the apply button, and a run you can take back.',
	'tidy.countCovers': 'covers replaced',
	'tidy.noteMatch': 'From the internet: fields',
	'tidy.noteCover': 'From the internet: covers',

	'nav.downloads': 'Downloads',
	'downloads.intro': 'Downloads audio from a URL through yt-dlp, which Onsa runs as a separate program.',
	'downloads.rights': 'What you download, and the rights to it, are yours to answer for.',
	'downloads.needs': 'The programs this needs',
	'downloads.needsWhat': 'Onsa fetches nothing until you press the button. Files come from the official GitHub releases, and are checked against the checksum that release published before anything is written.',
	'downloads.fromRelease': 'from the official release',
	'downloads.installYourself': 'installed by you',
	'downloads.ffmpegWhere':
		'ffmpeg is installed by you, because its release comes as an archive. On Linux it is usually there already, or in the ffmpeg package. On Windows: download it from ffmpeg.org and put its folder on PATH \u2014 or tick "use the programs already on this system" below.',
	'downloads.withoutFfmpeg':
		'Without ffmpeg a download still works: the audio comes as it is, with no tags and no cover written into it, and converting is unavailable.',
	'downloads.noFfmpeg': 'This needs ffmpeg, and none was found.',
	'downloads.noPlayableFormat':
		'Nothing there is in a format Onsa can play. Opus arrives in the next version.',
	'downloads.cannotRun': 'yt-dlp could not be run.',
	'downloads.agree': 'Agree and fetch',
	'downloads.update': 'Update',
	'downloads.stop': 'Stop',
	'downloads.useSystem': "Use the programs already on this system",
	'downloads.useSystemWhat': 'If yt-dlp or ffmpeg is already installed (usual on Linux), Onsa uses that. A copy of its own still comes first.',
	'downloads.fromUrl': 'Download from a URL',
	'downloads.needYtDlp': 'yt-dlp has to be here before anything can be downloaded.',
	'downloads.reading': 'Looking at what is installed…',
	'downloads.urlPlaceholder': 'Paste a URL here',
	'downloads.look': 'See what is there',
	'downloads.looking': 'Asking…',
	'downloads.format': 'Format',
	'downloads.formatOriginal': 'As it is (suggested)',
	'downloads.conversionNote': 'Converting adds no quality: FLAC only makes it larger, MP3 makes it worse.',
	'downloads.fetchCount': 'Download {n} tracks',
	'downloads.queue': 'Queue',
	'downloads.waiting': 'waiting',
	'downloads.running': 'downloading',
	'downloads.done': 'done',
	'downloads.failed': 'failed',
	'downloads.wasStopped': 'stopped',
	'downloads.stopAll': 'Stop everything',
	'downloads.notAUrl': 'That is not a web address.',
	'downloads.refused': 'yt-dlp could not read that URL.',
	'error.no_folder': 'There is no library folder yet, so there is nowhere to put a download.',
	'downloads.unreachable': 'The release could not be reached. Try again later.',
	'downloads.tooLarge': 'The file is far larger than it should be, so it was not taken.',
	'downloads.stopped': 'Stopped. Nothing was written.',
	'downloads.noChecksum': 'The release lists no checksum for this file, so it was not installed.',
	'downloads.checksumFailed': 'The checksum did not match. The file was thrown away and nothing was installed.',
	'downloads.cannotWrite': "The file could not be written into Onsa's program folder.",
	'downloads.unreadable': 'The list of checksums could not be read.',
	'error.busy': 'Something is already running. Wait for it to finish.',
	'error.download': 'That program was not installed. The log says more.',

	'tidy.jobAuto': 'Tidy up automatically',
	'tidy.autoWhat': 'Tidies the tags that are already there, with no internet at all: what a download left behind, capitals, the way "feat." is written, and a name spelled more than one way in the same library.',
	'tidy.autoLook': 'Find what could be tidied',
	'tidy.autoNone': 'There is nothing to tidy in this folder.',
	'tidy.autoFound': '{n} things could be tidied',
	'tidy.noteAuto': 'Tidied automatically',
	'auto.residue': "a download's leftovers",
	'auto.capitals': 'capitals',
	'auto.credit': 'how feat. is written',
	'auto.spelling': 'how the name is spelled',
	'auto.albumArtist': 'no album artist',
	'auto.capitalsNote': 'Suggestions about capitals do not tick themselves: often enough it was written that way on purpose.',

	'match.start': 'Start looking',
	'match.stop': 'Stop',
	'match.stopped': 'Stopped. What was found is still below.',
	'match.progress': '{done} of {total}',
	'match.looked': '{n} tracks looked at',
	'match.tickSure': 'Tick everything suggested',
	'match.tickNone': 'Clear every tick',
	'match.internetOff': 'Looking things up is switched off. Turn it on in Settings → Metadata to use this.',
	'match.keyMissing': 'There is no AcoustID key. Without one, matching by sound cannot run.',
	'match.fpcalcMissing': 'fpcalc is not installed. Without it, only tracks that already have tags can be looked up.',
	'match.fromSound': 'From the sound',
	'match.fromTags': 'From the tags',
	'match.fromName': 'From the file name',
	'match.disagree': 'Two sources say different things. Choose which one yourself; nothing is ticked on its own.',
	'match.useNone': 'Use neither',
	'match.sureTitle': 'How sure this is',
	'match.nothing': '(empty)',
	'match.takeCover': 'Take this cover',
	'match.coverAlt': 'The suggested cover',
	'match.noFingerprinter': 'Cannot be matched by sound: fpcalc is not installed.',
	'match.unreadable': 'The file could not be read for a fingerprint.',
	'match.offline': 'The service could not be reached. Try again later.',
	'match.noKey': 'Cannot be matched by sound: there is no AcoustID key.',
	'match.refused': 'The AcoustID key was refused.',
	'match.nothingFound': 'Nothing matched.',

	'programs.title': 'Outside programs',
	'programs.hint': 'Onsa runs a few outside programs. fpcalc is the one this page uses, to recognise a track by its sound; yt-dlp and ffmpeg belong to the Downloads page. A copy already on the system is used when there is one.',
	'programs.installed': 'installed',
	'programs.missing': 'not there',
	'programs.fromManaged': "from Onsa's own folder",
	'programs.fromChosen': 'the one you chose',
	'programs.fromSystem': 'from the system',
	'programs.choose': 'Choose the file',
	'programs.chooseTitle': 'Choose a program file',
	'programs.forget': 'Forget that choice',
	'programs.refresh': 'Look again',
	'programs.fpcalcWhere': 'fpcalc comes with Chromaprint. On Linux it is usually the libchromaprint-tools package; on Windows, download it from the Chromaprint releases page and point Onsa at the file here.',

	'error.offline': 'Looking things up on the internet is switched off.',

	'error.backend': 'The backend cannot be reached. Start Onsa from its own window.',
	'error.theme_not_found': 'There is no such theme.',
	'error.library': 'The library failed. The log has the details.',
	'error.engine': 'The audio engine refused. The log has the details.',
	'error.no_output': 'No audio output can be opened.',
	'error.dialog': 'The file dialog cannot be opened.',
	'error.io': 'The file cannot be read or written.',
	'error.auto_eq_empty': 'That file holds no usable EQ filter.',

	'nav.help': 'Help',
	'help.open': 'Help for this page',
	'help.close': 'Close the help',
	'help.here': 'On this page',
	'help.controls': 'The controls on this page',
	'help.filesHead': 'Your own music files',
	'help.limitsHead': 'Limits on this page',
	'help.elsewhere':
		'The controls that are always in the window — the player bar and the signal path below, the navigation on the left, the queue and lyrics panel on the right — are explained on the Help page.',
	'help.toHelpPage': 'Open the Help page',
	'help.firstRun': 'Show the first screen again',
	'help.title.album': 'One album',
	'help.title.artist': 'One artist',
	'help.title.genre': 'One genre',
	'help.title.folder': 'One folder',
	'help.title.playlist': 'One playlist',
	'help.title.search': 'Search results',
	'help.title.header': 'The strip along the top',
	'help.title.transport': 'The player bar and the signal path',
	'help.title.panel': 'The right-hand panel: queue and lyrics',
	'help.tracks.what':
		'A list of every song Onsa found in your music folders. Onsa plays files that are already on your computer; there is no online catalogue here, and nothing is sent anywhere. Double-click a row to play it.',
	'help.tracks.limit':
		'What you see here is the result of the last look through your folders. A file you add with another program usually appears by itself within seconds; if it does not, there is a "Rescan" button — which looks through your folders again from the start — in Settings → Library.',
	'help.albums.what':
		'Every album in your library, drawn with its cover. Covers come from inside the song files or from an image in the same folder. Onsa fetches nothing from the internet unless you switch that on yourself in Settings → Metadata, the page that decides what may be asked about your songs online.',
	'help.album.what': 'One album: its cover, its artist, and its songs in album order.',
	'help.artists.what':
		'Every artist in your library, with how many songs each has. Click one to see those songs.',
	'help.artist.what': 'Every song by one artist, gathered from all their albums.',
	'help.genres.what':
		'A genre is the kind of music written inside the song file itself — "Rock" or "Jazz", say. This page gathers your songs by that.',
	'help.genres.limit':
		'A song with no genre written inside it does not appear here. You can fill that in yourself with "Edit tags…" from a right-click on a song — the window for changing the details stored inside that file — or on the Tidy up page.',
	'help.genre.what': 'Every song of one genre.',
	'help.folders.what':
		'Your songs by the folder the files actually live in on your computer, rather than by what is written inside them.',
	'help.folder.what': 'Every song inside one folder, as it is on your computer.',
	'help.playlists.what':
		'A playlist is a list of songs you put together. There are two kinds: the one you fill song by song, and the smart one, whose contents follow rules you set — "everything from the 1990s I have played", say — and change by themselves as your library changes.',
	'help.playlists.files':
		'Playlists live inside Onsa only. Making, changing or deleting one does not touch your music files at all. "Import M3U8" only reads the list file; it moves nothing.',
	'help.playlist.what':
		'What is in one playlist, in the order you put it. Songs can be dragged up and down to change that order.',
	'help.playlist.files':
		'Removing a song from a playlist only takes it off this list. The file stays on your computer and stays in the library.',
	'help.search.what':
		'The results from the search box above: artists, albums and songs whose names match what you typed. This search looks only inside your own library.',
	'help.search.limit':
		'It searches titles, artists, albums and genres. Lyrics and file names are not searched.',
	'help.tidy.what':
		'The page for cleaning up song details in bulk. Those details — title, artist, album, year, genre — are stored inside the song files themselves; everybody calls them "tags". Every job on this page is shown as a summary before it runs, and everything that has run can be undone from the History list below.',
	'help.tidy.files':
		'Two jobs here touch your own files: "Write to the files" writes the details into them, and "Rename by pattern" renames files and moves them between folders. The rest is kept inside Onsa only — your files do not change until you press "Write to the files". Both still go through the summary first and both are in the History, and undoing puts back the file names as well as the details.',
	'help.tidy.limit':
		'One run does at most the number in "Per run" — the rest waits for the next run, so a mistake cannot reach the whole library at once.',
	'help.downloads.what':
		'Fetches audio from a web address (a URL) and puts it in your music folder, one address at a time. The fetching is not done by Onsa itself but by yt-dlp, a separate program Onsa downloads only after you press the button that agrees to it. The rights to whatever you download are your responsibility.',
	'help.downloads.files':
		'What is fetched is written as a new file inside your library folder, in a "Downloads" subfolder, and then appears in the library by itself. The music files you already have are not touched.',
	'help.downloads.limitOpus':
		'A source that offers audio only as Opus is refused up front, with a message, because Onsa cannot play that format yet.',
	'help.downloads.limitOne':
		'One download runs at a time. Several at once comes in a later version.',
	'help.nowPlaying.what':
		'A full screen for the song that is playing: a large cover, the words if there are any, and the technical detail of what is happening to the sound.',
	'help.output.what':
		'Where the sound goes, and how carefully Onsa works on it before sending it there. Everything on this page changes only the sound coming out now; none of it changes your music files.',
	'help.dsp.what':
		'The sound shaping that happens before the sound leaves: the EQ (tone control, lifting or lowering the high and low parts), the preamp (a volume control before the shaping), and the limiter (a guard that keeps the sound from going over the edge and breaking up). Nothing on this page changes your music files.',
	'help.library.what':
		'Which folders Onsa treats as where your music lives. Onsa reads what is in them and remembers it; it does not move, rename or change anything in them.',
	'help.library.files':
		'Adding a folder and looking through it again only read. No file is written, moved or deleted from this page.',
	'help.metadata.what':
		'Whether Onsa may ask the internet about your songs, and with what. All of it is off to begin with: while it is off, not one request is sent. Switched on, what is sent is an acoustic fingerprint (a number worked out from the sound, not the file) along with title, artist and album, to ask AcoustID and MusicBrainz. What comes back is always a suggestion you have to accept.',
	'help.lyricsSettings.what':
		'Where the words to your songs may come from, and whether words from the internet are kept beside the song. Both are off to begin with.',
	'help.lyricsSettings.files':
		'With the second switch on, Onsa writes a small text file ending in .lrc beside the song file. The song file itself is not touched.',
	'help.appearance.what':
		'How the window looks: the theme (a set of colours), the colour that moves with the sound, and the language. Nothing on this page touches your music or your files.',
	'help.about.what':
		'Which version of Onsa is running, the record of what happened (the log) to send if something goes wrong, and what happens when the window is closed.',
	'help.header.what': 'The strip above the page, present on every page.',
	'help.transport.what':
		'The two strips along the bottom of the window, present on every page: the playback controls, and under them the signal path — the chain showing everything the sound passes through on its way from the file to the output device.',
	'help.panel.what':
		'The panel on the right of the window with two tabs: the Queue, which is what will play after the song that is playing, and the Lyrics. In a narrow window it is hidden and opens from a button in the top strip.',
	'help.panel.files':
		'The "Save .lrc" button on the Lyrics tab writes a small text file beside the song file. Nothing else here touches a file.',
	'help.help.what':
		'This page: a list of every page in Onsa, plus the things that do not belong to any one page.',

	'help.name.columns': 'Title · Artist · Album · Year · Length',
	'help.says.columns':
		'The column names, above the list. Click one to sort the list by it; click it again to turn the order round. The small arrow shows which column the order follows.',
	'help.name.resize': 'The line between two column names',
	'help.says.resize':
		'Drag it to make the column on its left wider or narrower; the width is remembered. With the keyboard: press Tab until the line is selected, then use the left and right arrows.',
	'help.says.chooseColumns':
		'Opens a list to tick: which columns are shown and which are hidden, with one entry to put everything back the way it came.',
	'help.name.row': 'A song row',
	'help.says.row':
		'One row is one song. Double-click it to play it now. Click once to select it; Ctrl+click adds another to the selection and Shift+click takes a run of them. A row whose file is no longer on your computer is marked "missing". Rows can be dragged onto a playlist in the navigation on the left.',
	'help.name.rowMenu': 'Right-click on a song row',
	'help.says.rowMenu':
		"Opens a small menu: \"Play next\" puts that song straight after the one playing, \"Add to the queue\" puts it at the end of what is waiting, \"Add to playlist\" puts it on one of your lists, and \"Edit tags…\" opens a window for changing that song's details — title, artist, album and so on — which are stored inside the file.",
	'help.name.albumCard': 'An album card',
	'help.says.albumCard':
		'A cover with the album name and its artist. Click it to open that album.',
	'help.says.albumBack': 'Back to the page with all the albums.',
	'help.says.albumPlay': 'Plays this album from its first song, replacing whatever was waiting.',
	'help.says.addTo':
		'Puts every song on this page onto one of your playlists. No file moves anywhere; only the list grows.',
	'help.name.artistRow': 'An artist row',
	'help.says.artistRow':
		"An artist's name and how many of their songs you have. Click it to see them.",
	'help.name.genreRow': 'A genre row',
	'help.says.genreRow':
		'A genre and how many songs carry it inside their files. Click it to see them.',
	'help.name.folderRow': 'A folder row',
	'help.says.folderRow':
		'A folder name with its full path on your computer. Click it to see the songs inside.',
	'help.name.browseBack': 'The back button, top left',
	'help.says.browseBack':
		'Back to the list you came from: all artists, all genres or all folders.',
	'help.says.browsePlay': 'Plays every song on this page, starting at the first.',
	'help.says.playlistNew':
		'Makes an empty playlist for you to fill yourself. Onsa asks for a name first.',
	'help.says.playlistNewSmart':
		'Makes a playlist whose contents follow rules rather than one-by-one choices. You set the rules — year, artist, genre, how often it was played — and the contents keep up with your library by themselves.',
	'help.says.playlistImport':
		'Reads a song-list file made by another program (ending in .m3u8) and makes a playlist from it. Songs you do not have are skipped and their number is named.',
	'help.name.playlistCard': 'A playlist card',
	'help.says.playlistCard':
		"A playlist's name, which kind it is (ordinary or smart), and how many songs are on it. Click it to open it.",
	'help.name.playlistBack': 'The back button, top left',
	'help.says.playlistBack': 'Back to the page with all your playlists.',
	'help.says.playlistPlay': 'Plays this playlist from its first song.',
	'help.says.playlistQueue':
		'Puts everything on this playlist at the end of what is waiting, without stopping what is playing.',
	'help.says.playlistMore':
		'Opens a menu: rename, duplicate, export as an .m3u8 file, and delete the playlist. Deleting a playlist deletes no music file.',
	'help.says.playlistRules':
		'Only on a smart playlist: opens its rules, so you can change what lands on this list.',
	'help.name.playlistRow': 'A song row on a playlist',
	'help.says.playlistRow':
		'The same as a song row anywhere else, plus two things: it can be dragged up and down to change the order, and it has a button to take it off this playlist. Taking a song off deletes nothing.',
	'help.name.searchArtist': 'An artist among the results',
	'help.says.searchArtist':
		'An artist whose name matches what you typed. Click to open their page.',
	'help.name.searchAlbum': 'An album among the results',
	'help.says.searchAlbum': 'An album whose name matches what you typed. Click to open it.',

	'help.name.tidyScope': 'What is in range',
	'help.says.tidyScope':
		'The line at the top of the page saying which songs a job here would reach. With no folder set, that is your whole library; with one set, only the songs inside that folder.',
	'help.says.tidyPick':
		'Picks one folder as a boundary, so nothing on this page can reach a song outside it. The safest way to try something for the first time.',
	'help.says.tidyClear': 'Takes the boundary folder away, so the whole library is in range again.',
	'help.says.tidyLimit':
		'How many songs one run may touch at most. The rest waits for the next run. It is there so one mistake cannot reach the whole library at once.',
	'help.says.tidyEdit':
		'Sets one detail — the album artist, say — for every song in range, meaning the ones named in the line at the top of this page. That is kept inside Onsa; your song files do not change until you run "Write to the files".',
	'help.says.tidyWrite':
		'Writes the details kept inside Onsa into the song files themselves. This is the one button here that changes what is inside your files.',
	'help.says.tidyRename':
		'Rebuilds file and folder names from a pattern you write — "artist/album/number title", say. The files really are renamed and moved on your computer, always after you have seen the summary first.',
	'help.says.tidyAuto':
		'Fixes small things without the internet: extra spaces, capitals gone wrong, track numbers written "3/12". What it suggests still goes through the summary before it runs.',
	'help.says.tidyMatch':
		'Asks the internet what a song is, by working out a fingerprint from the sound and asking AcoustID and MusicBrainz. It only runs if you switched that on in Settings → Metadata, the page that decides what may be asked online. What comes back is a suggestion you tick yourself, not a change.',
	'help.says.tidyField':
		'Which detail "Edit in bulk" will set: artist, album, album artist, genre or year.',
	'help.says.tidyValue':
		'The value that detail will be given. Left empty, it clears the edit kept in Onsa and lets the value inside the file stand again.',
	'help.says.tidyPattern':
		"The name pattern for \"Rename by pattern\". Anything in curly brackets is replaced by the song's own details, such as {artist} and {title}; a slash means a folder.",
	'help.says.tidyRoot':
		'The folder the renamed files are put in. Left empty, they stay where they are.',
	'help.says.tidyLook':
		'Works out what would happen and shows it — how many songs are touched, how many files written, what is skipped and why — without changing anything.',
	'help.says.tidyApply':
		'Runs what the summary just showed. Only after this button is pressed does anything actually change.',
	'help.says.tidyUndo':
		'Undoes one run from the History list, including what was written into files and file names that were changed.',
	'help.says.useSystem':
		"If yt-dlp or ffmpeg are already on your computer, Onsa uses those and downloads nothing. Switched off, Onsa runs only the programs in its own folder — but yt-dlp still looks for ffmpeg itself, because that is its business and not Onsa's.",
	'help.name.programRow': 'A program row',
	'help.says.programRow':
		'One row for each program this page needs, saying whether it is there, where it came from and which version it is. yt-dlp is required: without it nothing can be fetched. ffmpeg is not: without it downloads still work, but with no song details or cover put inside, and with no change of format. The ffmpeg row can say "on the system, used by yt-dlp itself": that means the copy is on your computer and yt-dlp uses it on its own account, outside the switch below.',
	'help.says.agree':
		'Downloads that program from its official release page and checks it against the fingerprint that release published, before keeping it. Nothing is downloaded until this button is pressed.',
	'help.says.update': 'Fetches the newest version of that program, with the same check.',
	'help.says.urlField': 'Where you paste the web address whose audio you want.',
	'help.says.look':
		'Asks yt-dlp what is at that address — one track or a list — and shows the titles. Nothing is downloaded at this stage.',
	'help.says.format':
		"The shape of file you want. \"As it comes\" means the file is taken as it is. MP3 and FLAC change its shape, and that is ffmpeg's work — without ffmpeg neither can be chosen.",
	'help.says.fetch': 'Starts fetching what was found into your music folder.',
	'help.says.stop': 'Stops one download that is running. A half-finished file is thrown away.',
	'help.says.stopAll': 'Stops everything that is running and everything still waiting.',
	'help.says.nowPlayingClose': 'Closes this screen and goes back to the list of songs.',
	'help.says.seek':
		'The long bar holding the position in the song. Click or drag it to jump elsewhere. The left and right arrow keys step back and forward five seconds.',
	'help.name.nowPlayingLyrics': 'The words on this screen',
	'help.says.nowPlayingLyrics':
		'If the song has words, the lines appear here in large type and the line being sung lights up. Click a line to jump to that part of the song.',

	'help.says.device':
		'Where the sound goes out: speakers, headphones, or another device your computer knows. "System default" means Onsa follows wherever Windows or Linux sends sound, including when you plug headphones in.',
	'help.says.sampleRate':
		'How many times a second the sound is handed to that device. "Follow the device" uses the number the device itself prefers and is nearly always the right answer.',
	'help.says.matchSource':
		'Switched on, Onsa tries to use the same number as the song file, so the sound needs no re-reckoning. If the device will not have it, Onsa re-reckons anyway.',
	'help.says.quality':
		"How carefully that re-reckoning is done when the song's number and the device's number differ. The three choices are \"Fast\", \"Balanced\" and \"Best\". More careful means more work for the processor; \"Balanced\" is enough for nearly everyone.",
	'help.says.buffer':
		'How much sound is made ready before it is handed over, with three choices: "Low", "Normal" and "Large". Low means Onsa answers faster; large means it copes better with a busy computer. If the sound breaks up, make it larger.',
	'help.says.dither':
		'Adds a very fine hiss when the sound is brought down to 16 bits, so the quietest parts do not turn coarse. It only applies to devices that really are 16-bit.',
	'help.says.powerSave':
		'Does less drawing work — the meters and the spectrum — while the window is out of sight, so a battery lasts longer.',
	'help.says.crossfade':
		'How long the old song and the new one overlap when one follows the other. Zero means no overlap at all.',
	'help.says.curve':
		'The shape of that crossing. "Equal power" keeps the loudness steady through the middle of it; "Linear" simply falls and rises in a straight line.',
	'help.says.albumGapless':
		'Many albums are made so that one track runs straight into the next with no gap at all. With this on, two neighbouring tracks from the same album are joined exactly that way, with no overlap, and the crossfade is used only elsewhere.',
	'help.says.skipFade':
		'A short crossing when you press the next-track button yourself, so it does not sound cut off.',
	'help.says.rgMode':
		'ReplayGain is a loudness figure already stored inside many song files by other programs. The four choices: "Off" ignores it, "Track" uses the per-song figure, "Album" uses the per-album one — which keeps the loud and quiet parts of one album in proportion — and "Automatic" uses the album figure when an album is what is playing.',
	'help.says.rgPreamp':
		'Lifts or lowers the whole result of that levelling, if you find it too quiet or too loud.',
	'help.says.rgFallback':
		'How loud a song with no loudness figure inside it is played, so it does not jump out among the songs that have one.',
	'help.says.rgClip':
		'Uses the peak figure stored alongside the loudness one to bring the volume down a little when needed, so the sound cannot go over the edge and break up.',
	'help.says.eqOn':
		'Turns the tone control on or off. Off means the sound passes through untouched.',
	'help.says.eqGraphic':
		'The simple shape: ten sliders, each for one band of the range from the lowest sounds to the highest.',
	'help.says.eqParametric':
		'The advanced shape: you set where each filter sits, how wide it is and what kind it is. For people following a ready-made setting from the internet.',
	'help.name.eqBand':
		'The tone sliders: 31 Hz, 62 Hz, 125 Hz, 250 Hz, 500 Hz, 1 kHz, 2 kHz, 4 kHz, 8 kHz, 16 kHz',
	'help.says.eqBand':
		'Ten sliders, one for each band. The small numbers on the left are low sounds like bass; the large ones on the right are high sounds like cymbals. Raising a slider makes that band louder, lowering it makes it quieter.',
	'help.says.eqFlat': 'Puts every slider back to zero, which is no tone change at all.',
	'help.says.eqImport':
		'Reads an EQ settings file made by AutoEQ — a ready-made setting for a particular model of headphone — and uses it.',
	'help.says.eqExport':
		'Saves your current EQ as a text file in that same shape, so it can be used elsewhere.',
	'help.says.eqAdd': 'On the parametric shape only: adds one more filter.',
	'help.says.eqRemove': 'On the parametric shape only: takes that filter away.',
	'help.says.preampAuto':
		'Brings the volume down before the tone control, by just enough that a raised slider cannot push the sound over the edge.',
	'help.says.preampManual':
		'Sets that reduction yourself, if you would rather Onsa did not work it out.',
	'help.says.limiterOn':
		'The last guard before the sound leaves: it holds back peaks that are still too high, so they cannot break up.',
	'help.says.limiterRelease':
		'How quickly that guard lets go again after a peak has passed. Too fast sounds wobbly, too slow sounds squashed.',
	'help.says.addFolder':
		'Adds another folder as a place your music lives. Onsa reads what is in it straight away.',
	'help.says.rescan':
		'Tells Onsa to look through those folders again from the start. Useful when a file does not appear by itself, for instance after a folder was moved from another computer.',
	'help.name.folderList': 'The list of folders',
	'help.says.folderList':
		'The folders you have added, with how many songs are in each. A folder that is no longer where it was — an unplugged drive, or one moved with another program — is marked in this list, with a word about what it means. Onsa deletes nothing over it.',
	'help.says.metaOnline':
		'The main switch for every question to the internet on the Tidy up page. While it is off, Onsa sends nothing anywhere.',
	'help.says.metaKey':
		'AcoustID asks every program that questions it for an application key. The key is free and you fetch it from the AcoustID site yourself; Onsa carries no key of its own.',
	'help.says.metaKeySave': 'Keeps that key on your computer only, to use for the next question.',
	'help.says.metaKeyClear': 'Deletes the key that was kept.',
	'help.says.metaKeyTry': 'Sends one small question to be sure the key is accepted.',
	'help.says.metaChoose':
		'Points Onsa at the fpcalc file on your computer yourself. fpcalc is a small program that works out a fingerprint from the sound of a song; without it, "Look it up online" has nothing to ask with.',
	'help.says.metaForget': 'Forgets the place you pointed at.',
	'help.says.metaRefresh':
		'Looks again for whether the program is there now, after you have installed it for instance.',
	'help.says.lyricsOnline':
		'Whether Onsa may ask LRCLIB — an open lyrics service — about a song whose words are not on your computer. What is sent is the title, artist, album and length, and nothing else.',
	'help.says.lyricsBeside':
		'Switched on, words that came from the internet are kept as a small text file ending in .lrc beside the song file, so they are still there later without the internet.',
	'help.says.themePick':
		'A set of colours for the whole window. Six come with Onsa: "Smoked Glass", "Glass Cockpit", "Night Deck", "Blue Cubicle", "Millennium Shine" and "Northern Winter"; themes you make yourself appear in the same row. The window changes as you choose; there is nothing to save.',
	'help.says.tone':
		"How strongly the window's colour moves with the sound that is playing, with four choices: \"Off\", \"Gentle\", \"Middling\" and \"Strong\". Off means the colour stays still.",
	'help.says.themeFolder':
		"Opens the folder Onsa reads home-made themes from, in your computer's file browser.",
	'help.says.themeCopy':
		'Copies the theme you are using into that folder as a file whose colours you can change. An existing file is never written over.',
	'help.says.language':
		'The language of every word in the window, including this help: "Indonesian" or "English".',
	'help.says.debug':
		'Turns on a far more detailed record of what happened. Use it when something is wrong and you want to report it; turn it off afterwards, because the record grows quickly.',
	'help.says.openLogs': 'Opens the folder those records are kept in, so you can send the file.',
	'help.says.closeToTray':
		'Switched on, closing the window does not stop Onsa: it hides itself near the clock and the music keeps going.',

	'help.says.search': 'Searches your library as you type. Ctrl+F jumps here from anywhere.',
	'help.says.searchClear': 'Empties the search box and brings back the page you were on.',
	'help.says.queueButton':
		'Only there when the window is narrow: opens and closes the queue and lyrics panel.',
	'help.name.menuButton': 'The three-line button',
	'help.says.menuButton':
		'Only there when the window is too narrow for the navigation on the left: opens the list of pages as a drawer.',
	'help.says.helpButton':
		'This button: it opens the explanation of the page you are on. It sits in the same place on every page, and the panel can be closed at any time without getting in the way of what you are doing.',
	'help.says.previous':
		'Back to the song before. If the song is more than a few seconds in, this starts it again from the beginning first.',
	'help.says.playPause':
		'Plays, and pauses if it is already playing. The space bar does the same.',
	'help.says.next': 'Jumps to the next song in the queue.',
	'help.says.volume':
		'How loud Onsa itself is, apart from the system volume. Ctrl and the up and down arrows raise and lower it.',
	'help.says.openNowPlaying':
		'Opens the full screen for the song that is playing. The button is dead while nothing is playing.',
	'help.name.sleep': 'Sleep timer',
	'help.says.sleep':
		'Tells Onsa to stop by itself after a time you choose, or when the song that is playing ends.',
	'help.says.mini':
		'Shrinks the window to one thin strip that fits in a corner of the screen. The same button brings it back.',
	'help.name.signal': 'The signal path',
	'help.says.signal':
		'The bottom strip: the chain showing everything the sound passes through on its way out. The stages, left to right: the shape of the file, "Resample" (re-reckoning the rate so it fits the device), "ReplayGain" (levelling the loudness between songs), "EQ" (the tone control), "Limiter" (the guard that keeps the sound from breaking up), and at the right end the name of the output device and how it is connected. A lit stage is working, a dim one is off. Click a stage to switch it off or on; right-click to open its settings.',
	'help.says.queueTab':
		'What will play after the song that is playing. Songs can be dragged up and down to change that order.',
	'help.says.lyricsTab':
		'The words of the song that is playing, if there are any. The line being sung lights up, and clicking a line jumps to that part of the song.',
	'help.says.shuffle': 'Shuffles the order of the queue. Whatever is playing keeps playing.',
	'help.says.repeat': 'Steps round: off, repeat the whole queue, or repeat one song.',
	'help.says.clearQueue': 'Empties what is waiting. No file is deleted.',
	'help.says.saveQueue': 'Keeps what is in the queue now as a new playlist.',
	'help.says.removeFromQueue': 'Takes one song off what is waiting.',
	'help.says.lyricsEarlier':
		'Shifts the words a quarter of a second earlier, when they feel late against the singing. The shift is kept for that song alone and is still there next time.',
	'help.says.lyricsLater':
		'Shifts the words a quarter of a second later, when they run ahead of the singing. Also kept per song.',
	'help.says.lyricsReset': "Puts that song's shift back to nothing.",
	'help.says.lyricsSave':
		'Keeps the words you can see as a small text file beside the song file, so they are yours and stay there without the internet.',
	'help.says.lyricsAgain':
		'Asks the lyrics service about this song once more. Only possible if you switched the internet search for lyrics on.',
	'help.name.contents': 'The list of pages',
	'help.says.contents': 'Every page in Onsa. Click one to open it with its explanation beside it.',
	'help.says.firstRunAgain':
		'Shows the first screen again, the one that appears when Onsa is newly installed, including the list of things that are easy to miss. Opening it changes nothing.',

	'help.page.intro':
		'Onsa plays music files that are already on your computer. This page explains each part of it. Every page has a question-mark button in its top right corner that opens the explanation of that page alone.',
	'help.page.contents': 'The pages in Onsa',
	'help.page.things': 'Things that need explaining on their own',
	'help.lyrics.head': 'Words: where they come from',
	'help.lyrics.body':
		'Onsa looks for words in four places, in order, and the first one that has them wins. First, words you edited yourself inside Onsa. Second, a small text file ending in .lrc sitting beside the song file, with the same name. Third, words stored inside the song file itself. Fourth, the LRCLIB lyrics service on the internet — and only if you switched that on in Settings → Lyrics.',
	'help.lyrics.offset':
		'If the words are right but the timing is off, two buttons in the lyrics panel shift them a quarter of a second earlier or later. That shift is kept for that one song, not for all of them, and it is still there when you play the song again tomorrow. Changing or forgetting the words does not clear it.',
	'help.theme.head': 'Making your own theme',
	'help.theme.body':
		"A theme is one text file ending in .json in Onsa's theme folder. The easiest way to make one: open Settings → Appearance, press \"Copy the theme in use\", then open the resulting file in any text editor and change the colours. Open Onsa again and your theme is in the list. Colours are written as six-figure codes like #3fb950. The \"role\" part is colour by meaning: label for fixed wording, adjustable for values you can set, active for what is switched on, position for the playing position, caution for nearing a limit, clip for going over it. A wrong or missing value is replaced by the default, so a mistaken theme breaks nothing; a file that cannot be read at all is named on the Appearance page, with the reason.",
	'help.theme.example': 'What a theme file looks like',
	'help.theme.copy': 'Copy the example',
	'help.theme.copied': 'Copied.',
	'help.write.head': '"Not written to the file yet": when Onsa touches your files',
	'help.write.body':
		"A song's details — title, artist, album, year, genre, words — are stored inside the song file itself. When you change one in Onsa, the change is kept inside Onsa first, and the file is not touched at all. That is what \"Not written to the file yet\" means. What you see everywhere in Onsa is your changed value, while the file still holds the old one. The file changes only when you press \"Write to the file\" in the single-song editor, or run \"Write to the files\" on the Tidy up page. Both can be undone from the History.",
	'help.write.never':
		'What never touches your files: playing, playlists, the queue, searching, every sound setting, and themes. Only four things can: writing details into a file, renaming files by pattern, keeping words as a small text file ending in .lrc beside a song, and a download written as a new file.',
	'help.programs.head': 'yt-dlp and ffmpeg',
	'help.programs.body':
		'Both are separate programs, not part of Onsa, and are used only on the Downloads page. yt-dlp is required: it is what fetches audio from a web address. ffmpeg is not: without it downloads still work, but what arrives is the file as it comes — with no title and artist inside it, no cover picture, and no way to turn it into MP3 or FLAC. Onsa does not download either behind your back: each has a button that agrees to it, the file is taken from its official release page, and it is checked against the fingerprint that release published before it is kept. If you already have them, switch on "use the programs already on this system" and Onsa downloads nothing.',
	'help.limits.head': 'What this version cannot do',
	'help.limits.opus':
		'A web address whose audio comes only as Opus is refused up front, with a message, because Onsa has no way to play that format yet. Better refused at the start than downloaded and never playable.',
	'help.limits.folder':
		'A library folder cannot be removed from inside Onsa yet; there is only adding and looking again. A song whose file is gone is marked "missing" and is not played.',
	'help.limits.lastfm': 'There is no scrobbling to Last.fm yet.',
	'help.log.head': 'If something goes wrong',
	'help.log.body':
		'Onsa keeps a record of what happens on your computer. To report something: open Settings → About, switch on "Debug log", close Onsa and open it again, do the thing that went wrong once more, then press "Open log folder". Send the file for that day. The record holds file paths and song names from your library, so read it before sending it to anybody. Switch "Debug log" off again afterwards, because the record grows quickly.',
	'help.firstRun.head': 'The first screen',
	'help.firstRun.body':
		'The screen that appears the first time Onsa is opened, with the choice of music folder, the theme, and a short list of things that are easy to miss. Opening it again changes nothing.',

	'help.says.themeExample':
		'Copies the example theme file above to the clipboard, ready to paste into a text editor.',
	'help.name.contentsLinks': 'A page name in the list',
	'help.says.contentsLinks':
		'Every name on that list can be clicked: the page opens and its explanation opens beside it. The ones that are not pages — the top strip, the player bar, the right-hand panel, the search results, and detail pages such as one album — only open their explanation.',

	'help.name.queueRow': 'A song row in the queue',
	'help.says.queueRow':
		'One song waiting its turn. Double-click it to jump to it now; drag it up and down to change the order. The one playing is marked.',

	'downloads.seeDetail': 'See the detail',
	'downloads.otherTrouble':
		'yt-dlp stopped with a complaint Onsa does not recognise. Its own sentence is below.',
	'downloads.unsupportedSite':
		'yt-dlp does not know that site, so there is nothing it can take from it.',
	'downloads.needsSignIn':
		'That is only available to somebody signed in to that site. Onsa signs in to nothing.',
	'downloads.geoBlocked': 'The site will not hand that over from this country.',
	'downloads.gone':
		'There is nothing at that address any more — taken down, moved, or the address was mistyped.',
	'downloads.tooManyAsks':
		'The site refused because it was asked too often in a short time. Try again in a few minutes.',
	'downloads.siteRefused':
		'The site refused the request. Usually that means the content is not open to everybody.',
	'downloads.cannotReach':
		'The site could not be reached. Check your internet connection and try again.',
	'programs.fromSystemAnyway': 'on the system, used by yt-dlp itself',
	'programs.systemAnywayWhat':
		'The switch below only decides which programs Onsa runs. yt-dlp looks for ffmpeg on the system on its own account, so it will use this one even with that switch off — which is why converting is still available.',

	'downloads.siteBroken':
		'The site answered, but something is wrong at their end. Try again later.',

	'librarySettings.folderGone': 'not where it was',
	'librarySettings.goneWhat':
		'{n} of these folders is not where it was. Usually that means a drive was unplugged, or the folder was moved or renamed with another program.',
	'librarySettings.goneNothingLost':
		'Onsa deletes nothing over this. The songs from that folder stay in the library; the ones whose files cannot be opened are marked missing. Plug the drive back in and press "Rescan", or add the new place if you moved the folder.'
};

export const dictionaries: Record<Locale, Record<MessageKey, string>> = { id, en };

/** Whether a string names a locale Onsa speaks. */
export function isLocale(value: string): value is Locale {
	return (LOCALES as readonly string[]).includes(value);
}
