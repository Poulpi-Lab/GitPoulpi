[English](../image-view.md) · [Čeština](../cs/image-view.md) · [Deutsch](../de/image-view.md) · [Español](../es/image-view.md) · [Français](../fr/image-view.md) · [Italiano](../it/image-view.md) · [日本語](../ja/image-view.md) · [한국어](../ko/image-view.md) · [Nederlands](../nl/image-view.md) · [Polski](../pl/image-view.md) · [Português (Brasil)](../pt-BR/image-view.md) · [Português (Portugal)](../pt-PT/image-view.md) · [Русский](../ru/image-view.md) · [Svenska](../sv/image-view.md) · **Türkçe** · [Українська](../uk/image-view.md) · [简体中文](../zh-CN/image-view.md)

# Diff Görünümü ve Dosya Görünümü'ndeki görseller

Görsel olan bir ikili dosya, "(no textual changes - binary content...)" uyarısı
yerine bir resim olarak gösterilir. Diff Görünümü **önceyi** ve **sonrayı**
gösterir; Dosya Görünümü dosyayı olduğu gibi gösterir; merge düzenleyicisi
çakışmalı bir görselin taraflarını gösterir. Buradaki her şey salt okunurdur:
bir görsele bakmak depoyu asla değiştirmez, dolayısıyla geri alınacak bir şey
yoktur.

**Ayarlar > Görseller > Görselleri önizle** (`ui.image_view`) ile kapatın:
sade ikili dosya uyarısı geri gelir. `image-view` özelliği olmayan özel bir
derlemede hiç görüntüleyici yoktur ve Görseller sayfasında yalnızca düzenleyici
listesi kalır (bkz. [Özel derlemeler](custom-builds.md)).

## Hangi dosyalar

PNG ve JPEG her zaman; derlemede `image-view` varsa GIF, WebP (ilk karesi), BMP
ve ICO/CUR. TIFF, AVIF, HEIC, PSD ve RAW tanınır ama çözülmez: bir bilgi kartı
biçimi, boyutu ve ölçüleri verir. SVG metindir ve metin diff'ini korur.
**Maks. önizleme boyutunu** (varsayılan olarak 32 MiB) aşan ya da 64
megapikselden büyük bir görselin yerine bir bilgi kartı gösterilir. Git LFS ile
saklanan bir dosya, nesnesi yerel depodaysa gösterilir; değilse kart önce
`LFS > Pull` yapmanızı söyler. Saydamlık bir dama tahtası üzerinde çizilir.

## Nasıl çizilir

Görsel protokolü olan bir terminalde (Kitty ve Ghostty, WezTerm ve iTerm2, Rio,
mintty ve sixel terminalleri Windows Terminal, foot ve Tabby) resim gerçek bir görsel olarak çizilir. Başka her yerde (sade Windows
konsolu, zellij, GitPoulpi'nin tanımadığı bir terminal) renkli yarım blok
karakterlerle çizilir; dikey çözünürlüğün yarısıdır ama her yerde okunur;
**Görsel protokolü yokken** seçeneği *Yalnızca bilgi kartı* olarak ayarlanabilir.
**Log ve grafik** sayfasındaki **Görüntü protokolü** birini zorlar.

## Diff Görünümü

Bir görsel dosyanın diff'ini açın (bir commit'inki ya da WIP'in unstaged veya
staged yarısı). Başlık her iki dosyayı adlandırır
(`PNG 512x512 . 34 KiB -> PNG 640x480 . 41 KiB`), ardından yakınlaştırma
düğmeleri, **Sığdır**, **1:1** ve **Şununla düzenle** gelir.

- Başlığın sağındaki **Yan yana | Satır içi** ya da `m`: yan yana ya da her
  seferinde tek görsel. **Tab** ya da *Önce | Sonra* üzerine bir tıklama, satır
  içi görünümde diğerini gösterir. İki bölme için fazla dar olan bir bölünmüş
  görünüm, ayarınızı değiştirmeden satır içi çizilir (`ui.diff_layout`).
- Bölünmüş görünümün **ayırıcısı** sürüklenerek, `Alt+Right` /
  `Alt+Left` ile (önce bölmesi yer alır / verir) hareket eder ve çift tıklama ya
  da `Alt+0` ile ortaya döner. Pay oturum boyunca hatırlanır.
- Yeni bir dosya *Yeni dosya* başlıklı tek bir bölme, silinmiş bir dosya *Silinen
  dosya* başlıklı tek bir bölme gösterir; ikisi de geçiş düğmesini gizler.
  Çalışma değişikliklerinde seçilen izlenmeyen bir görsel yeni bir dosya olarak
  önizlenir.
- Önce tarafı üst commit'tir (commit diff'i), `HEAD`'dir (staged) ya da indekstir
  (unstaged); sonra tarafı commit, indeks ya da diskteki dosyadır. Diff açıkken
  diskteki dosya değişirse (bir düzenleyiciden kaydettiniz, bir araç yeniden
  yazdı) görsel yaklaşık yarım saniye içinde buna uyar.

## Dosya Görünümü

Bir dosyanın menüsünden ya da diff çubuğundan **Dosya Görünümü** ile bir görseli
açın: tek bir bölme; dosya o revizyondaki hâliyle (çalışma değişiklikleri için
diskteki hâliyle) gösterilir.

## Yakınlaştırma ve kaydırma

| | |
|---|---|
| `+` / `-` (veya `=`) | Bölmenin merkezi etrafında yakınlaştır / uzaklaştır |
| `0` | **Sığdır** (varsayılan): görselin tamamı; pencere veya ayırıcı yeniden boyutlandırıldığında da sığdırılmış kalır |
| `1` | Gerçek boyut (1:1) |
| Oklar, `h j k l` | Bölmenin onda biri kadar kaydır; `Shift` ile bir bölme kadar; `PageUp`/`PageDown`, `Home`/`End` |
| Tekerlek | Dikey kaydır; `Shift`+tekerlek yatay; `Ctrl`+tekerlek imlecin etrafında yakınlaştırır |
| Sürükleme | Görseli tutup taşıyın; yanlardaki kaydırma çubukları her yerdeki gibi çalışır |

Yakınlaştırılmış bir görünüm, bölme yeniden boyutlandırıldığında ölçeğini ve
merkezindeki noktayı korur; Sığdır, Sığdır olarak kalır. Diff Görünümü'nde iki
bölme tek bir yakınlaştırmayı ve tek bir kaydırma konumunu paylaşır
(**Önce/sonra yakınlaştırma ve kaydırmayı eşitle**, varsayılan olarak açık); böylece
iki görselin aynı bölümü görünür kalır; boyutları farklıysa yakınlaştırma her
birine göreli tutulur. **Şu yakınlaştırmadan itibaren pikselli** değerinden
(varsayılan 2x) itibaren görsel yumuşatılmak yerine kare pikseller olarak
büyütülür; bu simgelere ve piksel sanatına uygundur.

## Şununla düzenle

Başlıktaki **Şununla düzenle** düğmesi, bu makinede bulunan görsel düzenleyicileri
listeler (GIMP, Krita, Inkscape, Photoshop, Paint.NET, Preview, ... ve sistemin
varsayılan uygulaması); `o` varsayılanı çalıştırır, `Shift+O` listeyi açar. Ayrıca
dosya listelerindeki bir görselin sağ tık menüsünde ve komut paletinde
(*Görsel: Şununla Düzenle...*) bulunur. Liste ve varsayılan, [Şurada aç](open-in.md)
özelliğinin çalışma biçimiyle aynı şekilde **Ayarlar > Görseller** altında
ayarlanır.

Diskteki dosyayı düzenlemek (çalışma değişiklikleri tarafı) doğrudandır:
düzenleyici yerinde kaydeder ve görsel yenilenir. Diğer her sürüm (bir commit'inki,
indeks, silinmiş bir dosya) durum çubuğunda bir uyarıyla birlikte **salt okunur bir
geçici kopyaya** yazılır; böylece bir düzenleyici depoyu değiştiremez.

Sistemin görüntünün türü için kaydettiği uygulamalar (onun *Birlikte aç* listesi) listede düzenleyicilerinizden sonra gelir ve zaten orada olan bir programı tekrarlamaz. Diğer ikili dosyalar aynı listeye sahiptir, bkz. [Şununla aç](open-in.md).

## Merge düzenleyicisinde

Çakışmalı bir görsel kartını gösterir; **A**, **B** ve (varsa) **Temel** görselleri
kutularda yan yana durur, her biri kutusuna sığdırılmıştır ve salt okunurdur (orada
yakınlaştırma yoktur; görseller yalnızca seçmenize yardımcı olur). Düğmeler ve
tuşlar değişmez: **A'yı koru** (`a`), **B'yi al** (`b`), **Temeli al** (`o`),
**Merge aracında aç** (`m`). Önizlemeler yeterince geniş bir pencere (yaklaşık 70
sütun) ve birkaç boş satır ister; daha darsa kart, düz metin olandır. Bkz.
[Merge çakışmalarını çözmek](merge-conflicts.md).

## Tuşlar ve ayarlar

Tuşlar [Klavye kısayolları](keybindings.md#görsel-görünümü) sayfasındadır; her biri
yeniden atanabilir ve paletten ulaşılabilir (`Ctrl+P`, *Görsel* yazın).
Seçenekler [Ayarlar](settings.md#görseller)'ın **Görseller** sayfasında ve
[yapılandırma başvurusunda](config-reference.md) (`ui.image_view*`,
`ui.image_editors`) yer alır.
