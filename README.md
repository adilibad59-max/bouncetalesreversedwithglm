# Bounce Tales RS — Rust + macroquad Native Port

Bounce Tales (Nokia J2ME, v2.0.4) oyununun tersine mühendislik analizine dayanan,
**sıfırdan yazılmış özgün Rust implementasyonu**. Java bytecode çevirisi DEĞİLDİR:
analizden çıkarılan oyun *mekanikleri* (yuvarlanan top fiziği, vektör tabanlı arazi,
momentum hissi) modern Rust ile yeniden tasarlanmıştır.

## Özellikler

- **Yuvarlanan top fiziği**: segment (vektör) tabanlı arazi üzerinde
  kaymaksızın yuvarlanma (ω = v_t / r), sert düşüşte sekme, coyote time,
  zıplama tamponu, momentum korunumlu atış
- **Sanal 240×320 çözünürlük** (orijinal J2ME canvas boyutu) — her ekranda
  letterbox ile ölçeklenir
- **3 demo seviyesi** + insan-okur metin tabanlı seviye formatı
- **Durum makinesi**: Menü → Oyun → Seviye Tamam → Bitiş
- **Girdi**: klavye (masaüstü) + çoklu dokunmatik (Android üç-bölge şeması)
- Tamamen **prosedürel grafik** — derleme bağımlılığı yok
- 5 birim + davranış testi (`cargo test`)

## Masaüstünde çalıştırma

```bash
cargo run --release        # Linux/macOS/Windows
cargo test                 # fizik ve seviye formatı testleri
```

Kontroller: `←/→` veya `A/D` hareket, `↑/W/Space` zıpla, `R` yeniden başla.

## Android — native derleme (EMÜLATÖRSÜZ)

APK, ARM için doğrudan Rust'tan derlenir (KVM/Java/emülatör katmanı yoktur);
oyun OpenGLES üzerinden miniquad ile native çalışır.

**Ön koşullar** (Android SDK + NDK kurulu olmalı, `ANDROID_NDK_HOME` tanımlı):

```bash
# 1) cargo-apk'yi kur
cargo install cargo-apk

# 2) Android hedeflerini ekle
rustup target add aarch64-linux-android armv7-linux-androideabi

# 3) APK üret (release)
cargo apk build --release
# → target/android/apk/release/bounce-rs.apk

# 4) Cihaza yükle
adb install -r target/android/apk/release/bounce-rs.apk
```

`Cargo.toml` içindeki `[package.metadata.android]` bölümü APK etiketini,
paket adını (`com.bouncetales.port`), tam ekran + portre yönelimini ve
`aarch64-linux-android` hedefini yapılandırır.

**Alternatif (Gradle entegrasyonlu)**: [`cargo-ndk`](https://github.com/bbqsrc/cargo-ndk)
ile `.so` üretip bir Android Studio projesine gömebilirsin — Play Store
imzalaması için bu yol daha uygundur:

```bash
cargo install cargo-ndk
rustup target add aarch64-linux-android
cargo ndk -t arm64-v8a -o app/src/main/jniLibs build --release
```

## Dokunmatik kontrol şeması

```
┌─────────────┐
│   ZIPLA     │  üst yarı: dokun = zıpla
│             │
├──────┬──────┤
│  ◄   │  ►   │  alt yarı: sol/sağ hareket
└──────┴──────┘
```

Çoklu dokunma desteklidir: yön basılı tutulurken üst yarıya ikinci parmakla
zıplanabilir.

## Seviye formatı

`src/level.rs` içindeki ayrıştırıcı basit metin komutları okur:

```text
name   Seviye Adim
size   2400 320
spawn  60 80
death  600
ground 0 320 200 240 500 180 1200 300   # katı arazi kırık hattı (x y çiftleri)
orb    300 140
spike  700 260
goal   2300 250
```

Yeni seviye eklemek için `Level::demo()` içindeki eşleyiciye bir işlev ekleyip
`demo_count()` değerini güncelle.

## Proje yapısı

| Dosya | Sorumluluk |
|---|---|
| `src/main.rs` | giriş, oyun döngüsü, durum makinesi, kamera takibi |
| `src/ball.rs` | top fiziği: yuvarlanma, çarpışma çözümü, zıplama |
| `src/level.rs` | seviye formatı, segment ızgara uzamsal indeksi, demo seviyeleri |
| `src/entities.rs` | küreler, dikenler, bitiş halkası |
| `src/input.rs` | klavye + dokunmatik birleşik girdi |
| `src/render.rs` | prosedürel çizim, sanal çözünürlük kamerası |
| `assets_original/` | orijinal J2ME varlıkları (SADECE KİŞİSEL KULLANIM) |

## `assets_original/` hakkında — önemli

Bu klasör, kendi JAR'ından çıkarılmış orijinal sprite (PNG) ve müzik (MIDI)
dosyalarını içerir. Bunlar Nokia/Disco Media'nın telifli varlıklarıdır ve
**yalnızca kişisel kullanım/araştırma** amaçlı saklanmıştır. Oyun bu klasörü
**kullanmaz** (grafikler prosedürel çizilir). Varlıkları kendi yapılarında
kullanmak isteyenler sprite atlas formatını `src/render.rs`'teki `draw_texture`
çağrılarıyla değiştirmelidir.

## Tersine mühendislik notu

Ayrıntılı analiz (sınıf envanteri, sabit nokta matematiği, RLEF seviye formatı,
kaynak paketi yapısı) teslimatla birlikte gelen PDF raporundadır.
Orijinal Java kaynak kodu, J2ME CLDC/MIDP API'lerine bağımlı olduğundan
doğrudan çeviri pratik değildir; bu proje mekaniği modern araçlarla yeniden
kurar.
