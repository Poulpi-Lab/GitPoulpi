[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · **Türkçe** · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Terminal seçmek

GitPoulpi fare desteği olan her terminalde çalışır, ancak resim çizen, gerçek renk
gösteren ve klavyenin tamamını gönderen bir terminalde en iyi görünür ve en iyi
hissettirir. Bu sayfa her terminalin her işletim sisteminde size ne sunduğunu ve hangi
ayarı değiştirmeniz gerektiğini söyler.

**Nasıl okunur.** Tablolar **Kasım 2026**'daki bir test turundan gelir: her terminal
ham kaçış dizileriyle ve çalıştırabildiğimiz yerde GitPoulpi'nin kendisiyle; Linux,
Windows 11 ve macOS 15 üzerinde denendi. Sürümler [aşağıda](#denenen-sürümler)
listelenir. Her hücre şunlardan biridir:

- **çalışır**;
- **kısmen** (ve neyin eksik olduğu);
- **çalışmaz** (ve nedeni: *terminal*, *işletim sistemi katmanı* veya *GitPoulpi*);
- **yok** (terminal o işletim sisteminde yoktur);
- **ölçülmedi** (ve nedeni).

Çoklayıcıların (tmux, zellij, screen, herdr, psmux, paneller) kendi sayfası vardır:
[Çoklayıcılar](multiplexers.md).

## En iyi seçimler

| İşletim sistemi | En iyi seçimler, sırayla | Neden |
|---|---|---|
| **Windows** | **Windows Terminal**, sonra **WezTerm** ve **Rio** | Windows Terminal grafiği ve avatarları sağa hizalı sixel resimler olarak çizer ve başlatıcının varsayılanıdır. WezTerm ve Rio onları Kasım 2026 ConPTY düzeltmesinden beri sağa hizalı iTerm2 resimleri olarak çizer ([Windows](#windows)'a bakın). |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio** | kitty ve Ghostty resimleri, işaretçi şekillerini ve klavyenin tamamını sunar. foot sixel resimler çizer, klavyenin tamamına ve işaretçi şekillerine sahiptir. |
| **macOS** | **iTerm2**, sonra **kitty** ve **Ghostty** | iTerm2 3.7.7 resimleri kitty protokolüyle çizer. Terminal.app çalışır ama ne resim ne de gerçek renk sunar. kitty ve Ghostty Linux'taki gibi davranır (macOS'ta ölçülmedi). |

Hangi işletim sisteminde olursa olsun **screen** ve **zellij**, terminalin resim
protokolünü GitPoulpi'den gizler; bu yüzden karakter çizer. tmux sixel'i geçirir. Bkz.
[Çoklayıcılar](multiplexers.md).

## GitPoulpi'nin kullandıkları

- **Resimler** (grafik sütunu, yazar avatarları, görsel diff'i ve Dosya Görünümü):
  Unicode yer tutucularıyla **kitty grafik protokolü**, **iTerm2 satır içi
  görselleri** (`OSC 1337`) ya da **sixel**. Algılama varsayılan olarak reddeder;
  `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`, `GHOSTTY_RESOURCES_DIR`,
  `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` ve çoklayıcı değişkenlerine
  bakar. Ortam kimseyi adlandırmadığında (`-ti vt340` ile başlatılmış bir xterm,
  görselleri açık bir VS Code terminali, `TERM_PROGRAM`'ın iletilmediği SSH üzerinden
  erişilen her terminal) terminalin kendi yanıtları karar verir: GitPoulpi başlangıçta
  bir kez sorar (aşağıdaki "Başlangıç sorgusu"na bakın) ve terminalin XTVERSION adını
  (kitty, Ghostty, WezTerm, iTerm2, mintty, foot) ya da aygıt özniteliklerindeki sixel'i
  kullanır. Bir çoklayıcının içinde yanıtlar yok sayılır, çünkü tmux arkasında ne olursa
  olsun sixel olduğunu söyler.
- **Hangi protokol nerede.** kitty, Ghostty ve iTerm2 3.7.7 veya daha yenisi: kitty
  protokolü. WezTerm ve Rio: iTerm2 protokolü, PNG'nin saydamlığını karıştırarak. mintty
  ve 3.7.7'den eski iTerm2: opak zeminli iTerm2 protokolü. Windows Terminal, foot, Tabby,
  Konsole ve mlterm: **sixel**. Başka her yer: karakterler ve avatar baş harf rozeti olur.
  `ui.image_protocol` birini zorlar.
- **iTerm2 3.7.7.** Her `OSC 1337` satır içi görseli, "Remember my choice" işaretlenene
  kadar pencereyi engelleyen bir "Allow Terminal-Initiated Display?" kipli iletişim
  kutusu açar. Aynı resimler kitty protokolüyle, Unicode yer tutucular dahil, soru
  sormadan çizilir; bu yüzden GitPoulpi orada kitty protokolünü kullanır.
  `ui.image_protocol = "iterm2"` yine de satır içi görselleri verir.
- **Sixel**, **grafik sütunu** için (şeritler, düğümler ve `ui.avatars = "graph"` ile
  commit dairelerinin içindeki avatarlar) ve başka yerlerdeki **avatarlar** için (Log,
  commit ayrıntısı, Dosya Geçmişi, Blame, görsel görünümü) çizilir. Sixel'de alfa, kimlik
  ve ölçekleme yoktur; bu yüzden resim tam olarak `sütun x hücre genişliği` çarpı
  `satır x hücre yüksekliği` pikseldir, yalnızca bir şey olan yerde, arka planını koruyan
  boş hücrelerin üzerine çizilir; yalnızca şeritler, kaydırma konumu veya yakınlaştırma
  değişince yeniden kodlanır. Sixel'i silmenin yolu yoktur: GitPoulpi kaldırmak için
  hücrelerinin üzerine metin yazar. Sixel hücrenin gerçek piksel boyutunu gerektirir:
  Windows Terminal, Tabby ve mintty `CSI 16 t` sorusunu başlangıçta bir kez ve
  yakınlaştırmadan sonra yanıtlar (sessiz kalırlarsa hiçbir şeyi engellemez); başka
  yerlerde terminalin pencere boyutundan veya `GITPOULPI_TERMINAL_CELL_PX=10x20`
  değerinden gelir. Sixel çizdiğini söylemeyen bir terminal `ui.image_protocol = "sixel"`
  ister. 1.22 öncesi Windows Terminal sixel çizmez; orada `ui.image_protocol = "none"`
  yapın.
- **Gerçek renk**: temalar RGB'dir. `ui.color_depth = "auto"`, `COLORTERM=truecolor`
  olmayan bir `*-256color` `TERM` üzerinde (ve gerçek rengi olmayan Terminal.app'te)
  256 renge, Linux konsolunda 16 renge düşer.
- **Fare**: düğmeler, sürükleme ve hareketle SGR fare raporlaması (üzerine gelme
  geri bildirimi, sürükle ve bırak, sağ tık, çift tık).
- **Başlangıç sorgusu.** Dört soru tek bir yazmada çıkar: kitty klavye bayrakları
  (`CSI ? u`), arka plan rengi (`OSC 11`, yalnızca `ui.theme = system` için kullanılır),
  terminalin adı (XTVERSION, `CSI > 0 q`) ve aygıt öznitelikleri (`CSI c`, sonda: her
  terminal yanıtlar, bu yüzden okumayı kapatır). Okuma, 100 ms'lik bir son tarihi olan
  tek bir `poll` döngüsüdür ve klavye okuyucusu başlamadan biter. Hiçbir şey yanıtlamayan
  bir terminal 100 ms'ye bir kez mal olur ve her şey ortamın söylediği gibi kalır. Yalnızca
  Unix: Windows'ta konsol tuşları kendisi bildirir ve ortam karar verir. Yanıtlar Etkinlik
  Günlüğü'nün ortam raporunda (`start-up query:` satırı) bulunur.
- **Klavye**: **kitty klavye protokolü** (yalnızca disambiguate bayrağı) başlangıçta
  istenir. `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` ve `Ctrl+Shift+Z` tuşlarının
  ulaşmasını sağlar. Onsuz eski tuşlar kalır (`Ctrl+Y` yineler, `Alt` varyantları bazı
  akorların yerini alır). Bkz.
  [Klavye kısayolları](keybindings.md#terminalin-gönderemediği-tuşlar-ve-kitty-klavye-protokolü).
- **Pano**: her kopyalama eylemi tek bir ilkeyle gider. Yerelde GitPoulpi'nin çalıştığı
  makinenin **sistem panosu**, artı terminalin bunu yerine getirdiği biliniyorsa (kitty,
  WezTerm, Ghostty, foot, Alacritty, Windows Terminal, Rio, Tabby, iTerm2) terminale
  `OSC 52`; sistem panosu başarısız olursa da `OSC 52` yedektir. **SSH üzerinden veya
  ekran yokken** (bir Linux konsolu, bir kapsayıcı) yalnızca `OSC 52` gönderilir ve tmux
  içinde geçiş (passthrough) için sarılmış olarak da çıplak olarak da gönderilir. Durum
  çubuğu "kopyalandı" yalnızca sistem panosu metni aldığında der; terminale giden için
  "... terminalin panosuna gönderildi (OSC 52, doğrulanmadı)" der, çünkü hiçbir şey
  terminalin uyduğunu doğrulamaz. Etkinlik Günlüğü'nün ortam raporu yolu `clipboard:`
  satırında adlandırır.
- **Fare işaretçisi şekli** (ayırıcılar üzerinde yeniden boyutlandırma işaretçisi,
  tıklanabilir öğeler üzerinde el, metin alanları üzerinde I-beam): `OSC 22`; terminalin
  uyguladığı yerde bir ek özelliktir. GitPoulpi onu her zaman gönderir (Linux konsolu
  ile onu yutan veya yazdıran screen, zellij ve herdr arkası hariç; tmux arkasında geçiş
  için sarılır). `TERM_PROGRAM=mintty` altında mintty'nin kendi imleç adlarını (`hand`,
  `ibeam`, `sizewe`, ...) gönderir; mintty yalnızca onları okur. **Rio hiçbir işletim
  sisteminde `OSC 22`'yi uygulamaz**: kaynağı diziyi ayrıştırır ve atar, bu yüzden iki
  taraftaki hiçbir ayar bunu değiştirmez.
- **Metin imleci**: `ui.text_cursor = "terminal"` ile (varsayılan) terminalin kendi imleci
  odaktaki metin alanının düzenleme noktasına yanıp sönen bir çubuk olarak konur
  (`DECSCUSR`) ve başka yerde gizlenir; çıkışta kendi imleç biçiminiz geri gelir.
  `drawn` bunun yerine GitPoulpi'nin kendi imlecini çizer.
- **Odak olayları** etkindir (pencere odağı kaybedince ipuçları kapanır). **Bracketed
  paste** başlangıçta istenir ve her çıkış yolunda bırakılır (Unix); böylece çok satırlı
  bir yapıştırma tek yapıştırma olarak gelir: satır sonları bir alanı asla göndermez ve
  harfleri kısayolları asla tetiklemez. Windows'ta konsolun yapıştırma ayraçları yoktur;
  GitPoulpi'nin bunun yerine ne yaptığı için [Windows](#windows)'a bakın. GitPoulpi OSC 8
  köprülerini veya undercurl'ü istemez. Yalnızca resim yolunda her kare eşzamanlı bir
  güncellemeye sarılır (mode 2026; bilmeyen terminaller yok sayar); böylece bir resim, bir
  açılır pencerenin üzerinde asla yanıp sönmez.
- **Emoji ve semboller**: GitPoulpi yalnızca varsayılan emoji sunumlu ya da tek
  genişlikli BMP şekilli glifleri kullanır; çünkü terminaller ve yazı tipleri
  varyasyon seçicileri konusunda anlaşamaz.

## Matris

Satırlar terminaller, sütunlar işletim sistemleridir. Bir hücrede "GitPoulpi", GitPoulpi'nin
kendisinin orada çalıştırıldığı anlamına gelir; "yoklama" yalnızca ham kaçış dizilerinin
çalıştırıldığı anlamına gelir. **yok** hücreleri: kitty'nin yerel bir Windows sürümü yoktur
(sürümler Linux ve macOS için çıkar); Ghostty'nin Windows sürümü yoktur (yalnızca
libghostty kitaplığı platformlar arasıdır); foot bir Wayland terminalidir (yalnızca Linux);
xterm bir X11 istemcisidir (Linux ve XQuartz üzerinden macOS); iTerm2 ve Terminal.app
yalnızca macOS'tur; Windows Terminal, conhost ve mintty yalnızca Windows'tur (mintty
Cygwin ve MSYS2 altında da).

**macOS'ta ölçülmedi: kitty, Ghostty, WezTerm, Rio ve Warp.** Bir GPU gerektirirler (Metal
veya OpenGL 3.3). Aynı programın ne yaptığı için Linux sonuçları referanstır; bir macOS
hücresine güvenmeden önce onları GPU'lu bir Mac'te yeniden çalıştırın.

### Resimler: GitPoulpi hangi protokolü seçer ve görünür mü

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **çalışır**: kitty protokolü (GitPoulpi) | yok | ölçülmedi (GPU gerekir) |
| **Ghostty** | **çalışır**: kitty protokolü (GitPoulpi) | yok | ölçülmedi (GPU gerekir) |
| **WezTerm** | **çalışır**: saydamlıklı iTerm2 protokolü (GitPoulpi). Yer tutucular ezik çizilir, bu yüzden kitty protokolü kullanılmaz | **çalışır**: saydamlıklı iTerm2 protokolü, grafik ve avatarlar kendi satırlarında (resmî 20240203 ve upstream PR #8092 derlemesi). Resmî 20240203, GitPoulpi imleci resimlerin çevresinde göstermeden önce resimleri yanlış satırlara koyuyordu; bkz. "ConPTY sınırı". | ölçülmedi (GPU gerekir) |
| **foot** | **çalışır**: saydamlıklı sixel (GitPoulpi) | yok | yok |
| **Rio** | **çalışır**: saydamlıklı iTerm2 protokolü (GitPoulpi) | **çalışır**: saydamlıklı iTerm2 protokolü, denenen her yazı tipi boyutunda hücreye tam oturur (0.5.28; GitPoulpi). Sixel ve kitty asla ulaşmaz (ConPTY onları düşürür), bu yüzden seçilecek bir şey yoktur | ölçülmedi (GPU gerekir) |
| **Alacritty** | metin: **çalışır** (terminalde resim desteği yok) | metin: **kısmen** (resim yok; görsel diff'i bloklarla) | metin: **çalışır** |
| **Tabby** | **çalışır**: saydamlıklı sixel (yoklama; GitPoulpi Linux'ta Tabby'de çalıştırılmadı) | **çalışır**: sixel, hizalı (GitPoulpi) | **çalışır**: sixel, saydamlık korunur (GitPoulpi) |
| **Warp** | ölçülmedi (XWayland gerektirir ve sentetik tuş vuruşlarını yanlış çözer, bu yüzden hiçbir şey yazılamadı) | **çalışmaz** (terminal: sixel, iTerm2 veya kitty resmi yok); GitPoulpi ölçülmedi (Warp'ın giriş kutusundan başlatılamadı) | ölçülmedi (GPU gerekir) |
| **xterm** | yoklama: sixel `-ti vt340` ile **çalışır**; GitPoulpi onu aygıt özniteliklerinden seçer (başlangıç sorgusu çalıştırmadan sonra geldi) ya da `ui.image_protocol = "sixel"` ayarlayın | yok | metin: **çalışır**; sixel çizer (yoklama) ve `ui.image_protocol = "sixel"` ister |
| **iTerm2** | yok | yok | **çalışır**: 3.7.7 ve yenisinde kitty protokolü, iletişim kutusu yok (GitPoulpi). Eski sürümler satır içi görselleri opak zeminle alır. Sixel çizer ama ayarlanmamış pikselleri opak boyar, bu yüzden `ui.image_protocol = "sixel"` düz bir dikdörtgen verir |
| **Terminal.app** | yok | yok | metin: **çalışır** (resim yok) |
| **Windows Terminal** | yok | **çalışır**: sixel (GitPoulpi, 1.25) | yok |
| **conhost** | yok | metin: **kısmen** (karakterli grafik, yarım blokla görsel diff'i) | yok |
| **mintty** | yok | **çalışmaz** (ConPTY arkasındaki terminal: hiçbir resim çizilmez, imleç gösterilerek yerleştirilen sade bir `OSC 1337` resmi bile; grafik sütunu boş kalır; metin ve palet sorunsuz) | yok |

### Fare işaretçisi şekli (OSC 22)

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **çalışır** | yok | ölçülmedi (GPU gerekir) |
| **Ghostty** | **çalışır** | yok | ölçülmedi (GPU gerekir) |
| **WezTerm** | **çalışmaz** (terminal: denenen derlemede uygulanmadı) | resmî: **çalışmaz** (terminal). İşaretçi şekilleri ekleyen upstream WezTerm PR #8092 ile derleme: **çalışır** (el, I-beam, yeniden boyutlandırma okları) | ölçülmedi (GPU gerekir) |
| **foot** | **çalışır** | yok | yok |
| **Rio** | **çalışmaz** (terminal: ayrıştırılır ve atılır) | **çalışmaz** (terminal, aynı neden; ConPTY değil) | **çalışmaz** (terminal, aynı kaynak; yalnızca Linux ve Windows'ta çalıştırıldı) |
| **Alacritty** | **çalışmaz** (terminal) | **çalışmaz** (terminal) | **çalışmaz** (terminal) |
| **Tabby** | **çalışmaz** (terminal: xterm.js) | **çalışmaz** (terminal) | **çalışmaz** (terminal) |
| **Warp** | ölçülmedi | **çalışmaz** (terminal) | ölçülmedi |
| **xterm** | **çalışmaz** (terminal: CSS adları bilinmiyor) | yok | ölçülmedi (işaretçi pencerenin dışındaydı) |
| **iTerm2** | yok | yok | **çalışmaz** (terminal) |
| **Terminal.app** | yok | yok | **çalışmaz** (terminal) |
| **Windows Terminal** | yok | **çalışmaz** (terminal) | yok |
| **conhost** | yok | **çalışmaz** (terminal) | yok |
| **mintty** | yok | mintty'nin kendi adlarıyla **çalışır**; GitPoulpi onları gönderir | yok |

Hücrede "çalışmaz" yazan yerlerde fare altındaki ayırıcı veya düğme yine de yanar.

### Klavye protokolü

GitPoulpi, Linux ve macOS'ta kitty klavye protokolünü ister. Windows'ta istemez: konsol
değiştiricileri kendisi bildirir ve aşağıdaki akorlar terminalin kendine ayırdıklarıyla
sınırlıdır.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **çalışır** | yok | ölçülmedi (GPU gerekir) |
| **Ghostty** | **çalışır** | yok | ölçülmedi (GPU gerekir) |
| **WezTerm** | **kısmen**: `wezterm.lua` içinde `enable_kitty_keyboard = true` olana kadar yanıt yok (varsayılan `false`) | **kısmen**: WezTerm `Ctrl+Shift+N`'yi kendine ayırır (penceresini açar); `Ctrl+,` iletilmez (resmî derleme). PR #8092 derlemesi iki akoru da ayırmaz, ama `Ctrl+,` yine Ayarlar'ı açmaz (ConPTY üzerinden kitty klavye yanıtı yok) | ölçülmedi (GPU gerekir) |
| **foot** | **çalışır** | yok | yok |
| **Rio** | **çalışır** (varsayılan olarak açık) | **kısmen**: Rio `Ctrl+Shift+N`'yi kendine ayırır (penceresini açar) | ölçülmedi (GPU gerekir) |
| **Alacritty** | **çalışır** (0.13 ve sonrası) | **kısmen**: `Ctrl+Shift+N`, `Ctrl+N` olarak ulaşır, `Ctrl+,` iletilmez | **çalışır** |
| **Tabby** | **çalışmaz** (terminal: yanıt yok; `Ctrl+Shift+N` ve `Ctrl+,` tuşlarını kendine ayırır) | **kısmen**: Tabby `Ctrl+Shift+N` ve `Ctrl+,` tuşlarını kendine ayırır | **çalışmaz** (terminal: yanıt yok); bazı akorlar ulaşmadı |
| **Warp** | ölçülmedi | ölçülmedi | ölçülmedi (GPU gerekir) |
| **xterm** | **çalışmaz** (terminal) | yok | **çalışmaz** (terminal) |
| **iTerm2** | yok | yok | **çalışır** (3.7.7; çalışmazsa *Profiles > Keys*'i denetleyin) |
| **Terminal.app** | yok | yok | **çalışmaz** (terminal: yanıt yok; eski tuşlar) |
| **Windows Terminal** | yok | **kısmen**: WT `Ctrl+Shift+N`'yi (yeni pencere) ve `Ctrl+,`'yi (kendi ayarları) kendine ayırır | yok |
| **conhost** | yok | ölçülmedi | yok |
| **mintty** | yok | **kısmen**: iki akorun da görünür bir etkisi olmadı | yok |

Bir akor ulaşmadığında `Alt` varyantlarını kullanın
([Klavye kısayolları](keybindings.md#terminalin-gönderemediği-tuşlar-ve-kitty-klavye-protokolü)),
ya da akoru *Ayarlar > Klavye ve fare* içinde yeniden atayın.

### Pano (kopyalama eylemleri)

Yerel bir ekran varken kopyalama her zaman sistem panosunu kullanır; bu yüzden bu hücreler
terminalin `OSC 52` yolunu anlatır; o yol SSH üzerinde, kapsayıcılarda ve yedek olarak
önemlidir. Sistem panosunu kullanan her GitPoulpi kopyalama testi çalıştı.

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty** | **çalışır** | yok | ölçülmedi (GPU gerekir) |
| **Ghostty** | **çalışır** (tmux, zellij, herdr arkasında görüldü); çıplak: ölçülmedi (pano yazması klavye odağı gerektirir, test bunu vermedi) | yok | ölçülmedi (GPU gerekir) |
| **WezTerm** | **çalışır** | **çalışır** | ölçülmedi (GPU gerekir) |
| **foot** | ölçülmedi (yazma klavye odağı gerektirir, test bunu vermedi) | yok | yok |
| **Rio** | ölçülmedi (aynı neden) | **çalışır** | ölçülmedi (GPU gerekir) |
| **Alacritty** | ölçülmedi (aynı neden) | **çalışmaz** (varsayılan yapılandırmayla pano değişmedi; araştırılmadı) | **çalışır** |
| **Tabby** | **çalışır** | **çalışır** | **çalışır** |
| **Warp** | ölçülmedi | **çalışmaz** (pano değişmedi) | ölçülmedi (GPU gerekir) |
| **xterm** | ölçülmedi | yok | **çalışmaz** (pano değişmedi) |
| **iTerm2** | yok | yok | *Settings > General > Selection > Applications in terminal may access clipboard* açılana kadar **çalışmaz** |
| **Terminal.app** | yok | yok | **çalışmaz** (terminal) |
| **Windows Terminal** | yok | **çalışır** (odaktayken) | yok |
| **conhost** | yok | ölçülmedi (yoklama çalıştırılmadı) | yok |
| **mintty** | yok | **çalışmaz** (varsayılan ayarlarıyla pano değişmedi) | yok |

### Yapıştırma

Linux ve macOS'ta GitPoulpi bracketed paste ister, bu yüzden çok satırlı bir yapıştırma tek
yapıştırmadır. Bu terminal terminal çalıştırılmadı (terminal başına **ölçülmedi**).
Windows'ta konsolun yapıştırma ayraçları yoktur; bu yüzden önce sormayan bir terminal metni
tuş vuruşları olarak iletir ve her satır sonu bir `Enter` olurdu. GitPoulpi bu yüzden 30 ms
içinde birbirini izleyen tuş vuruşlarını (bir iç `Enter` ya da sekiz veya daha fazla
karakter) tek yapıştırma sayar. Basılı tutulan tuşlar ve elle yazılan "abc, `Enter`"
yapıştırma değildir.

| Terminal | Windows (yeni dal isteminde iki satırlı yapıştırma) |
|---|---|
| **Windows Terminal** | **çalışır**: WT önce "birkaç satır yapıştırılsın mı?" diye sorar (kendi uyarısı), sonra hiçbir dal oluşmaz |
| **WezTerm**, **Rio**, **Alacritty**, **Tabby**, **mintty** | **çalışır** (dal oluşmadı) |
| **Warp** | ölçülmedi (GitPoulpi onun içinde başlatılamadı) |
| **conhost** | ölçülmedi (testin gönderdiği `Ctrl+V` ve `Shift+Insert` klasik konsola yapıştırmadı) |

### Emoji ve simge genişliği

| Terminal | Linux | Windows | macOS |
|---|---|---|---|
| **kitty**, **Ghostty**, **WezTerm**, **foot**, **Rio**, **Alacritty** | **çalışır** (geniş hücreler hizalı; kitty dişli U+2699'u boş çizer, bilinen bir yazı tipi meselesi) | WezTerm, Rio, Tabby, Windows Terminal: **çalışır**. Alacritty: **kısmen** (emoji tek renkli ana hatlar olarak) | Alacritty, Tabby: **çalışır**. Diğerleri: ölçülmedi (GPU gerekir) |
| **Tabby** | **çalışır** | **çalışır** | **çalışır** |
| **Warp** | ölçülmedi | **çalışır** | ölçülmedi |
| **xterm** | **kısmen** (kutular: yazı tipi) | yok | **kısmen** (kutular: yazı tipi) |
| **iTerm2**, **Terminal.app** | yok | yok | **çalışır** |
| **conhost** | yok | **kısmen**: renkli emoji çizilir, geniş hücreler hizalı | yok |
| **mintty** | yok | **çalışır** | yok |

### Denenen sürümler

kitty 0.49.2 (macOS'ta 0.50.0 başlamadı), Ghostty 1.4.0, Linux'ta WezTerm 20260716 ve
Windows'ta 20240203 (son etiketli sürüm), foot 1.28.0, Rio 0.5.28 (Linux, Windows) ve
0.5.29 (macOS), Alacritty 0.17.0, Tabby 1.0.238, Warp 0.2026.11.04, xterm 411 (Linux) ve 378
(XQuartz), iTerm2 3.7.7, Terminal.app 455.1 (macOS 15.8), Windows Terminal 1.25.2733, mintty
3.8.3. Terminaller hızlı değişir: bunlardan yeni bir sürüm farklı olabilir.

## Her terminal için

### Windows

- **Windows Terminal.** Başlatıcının varsayılanı ve Windows'ta en iyi resim desteği: sixel
  grafik ve avatarlar, OSC 52, iyi emoji ve hücre boyutu sorusunu yanıtlar. Bir Nerd Font'u
  *Profiller > Varsayılanlar > Görünüm > Yazı tipi* altından seçin. Fare yakalanmışken metin
  seçmek için `Shift` tuşunu basılı tutun. `Ctrl+Shift+N` ve `Ctrl+,` tuşlarını kendine
  ayırır: `Alt` varyantlarını kullanın ya da yeniden atayın. İşaretçi şekli değişmez.
- **WezTerm.** `wezterm.lua` içinde `config.enable_kitty_keyboard = true` (Linux ve macOS
  derlemeleri) ve emoji yedekli bir yazı tipi ayarlayın, örneğin
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  Resimler iTerm2 protokolünü kullanır ve hiçbir şey ayarlamak gerekmez: son etiketli
  sürüm 20240203-110809 (Şubat 2024) onları doğru satırlara sabitler (güncel GitPoulpi ile
  Kasım 2026'da yeniden denetlendi; aşağıda anlatılan ConPTY düzeltmesinden önce
  sabitlemiyordu). Upstream WezTerm PR #8092 işaretçi şekilleri ekler.
- **Rio** (MSI, UAC ister). Resimler (grafik, avatarlar, görsel diff'i) saydamlıklı iTerm2
  protokolünü kullanır ve hücrelerine tam oturur: 0.5.28'de varsayılan yazı tipinde ve 18
  ile 22,5 px'te ölçüldü (hücreler 8x16, 11x21 ve 13x26 piksel, hep tam piksel), 20x10
  hücrelik ve 4x27 hücrelik bir resim de hücreye tam oturdu. Hiçbir şey ayarlamak
  gerekmez. Sözde konsol hücre boyutu sorusunu yutar; bu yüzden GitPoulpi Rio'nun
  hücrelerinin genişliğin iki katı yüksekliğinde olduğunu varsayar (üç boyutta da doğru);
  başka oranlı bir yazı tipiyle yuvarlak commit işaretleri için
  `GITPOULPI_TERMINAL_CELL_PX=WxH` (örneğin `10x24`) ayarlayın. Rio'nun hiçbir işletim
  sisteminde işaretçi şekli yoktur.
- **Tabby.** Sixel grafik ve avatarlar, kopyalama ve yapıştırma çalışır. Tabby
  `Ctrl+Shift+N` ve `Ctrl+,` tuşlarını kendine ayırır.
- **Alacritty.** OpenGL 3.3 gerekir. Karakterli grafik; `OSC 52` ile kopyalama varsayılan
  yapılandırmayla açık değildi.
- **mintty** (Git Bash, MSYS2, Cygwin). İşaretçi şekilleri çalışır (mintty'nin kendi
  adları). Resimler görünmez (grafik sütunu boş kalır; mintty 3.8.3, sade bir yoklamayla
  yerleştirilse bile ConPTY arkasında hiçbir `OSC 1337` resmi çizmez) ve orada hücre boyutu
  sorusunu yanıtlamaz: `ui.graph_renderer = "text"` ayarlayın.
- **Warp.** GitPoulpi ile ölçülmedi: Warp'ın giriş kutusundan başlatılamadı. Terminalde
  sixel, iTerm2 veya kitty resmi yok.
- **conhost, cmd, bir PowerShell penceresi.** Karakterli grafik, yarım blokla görsel
  diff'i. Varsayılan olarak böyle bir pencereden başlatılan GitPoulpi kendini seçtiğiniz
  terminalde yeniden açar; bkz. "Yeniden başlatma kuralı".

**ConPTY sınırı.** Windows her terminalin programını bir sözde konsolun (ConPTY) arkasında
çalıştırır. Güncel birini paket olarak içeren Windows Terminal ve WezTerm derlemeleri
resim dizilerini geçirir; Windows'un içindeki (Rio ve mintty'nin kullandığı) ve WezTerm
20240203'ün içindeki geçirmez. Kaybolan: DCS (sixel) ve APC (kitty grafik) dizileri
düşer, bu yüzden yalnızca iTerm2 resimleri (`OSC 1337`) ulaşır; Rio ve WezTerm'in onları
kullanmasının nedeni budur; `CSI 16 t` (hücre boyutu) yalnızca Windows Terminal ve
Tabby'de yanıtlanır. Düzeltilen: bir `OSC 1337` ConPTY'yi hemen geçer ve ConPTY'nin en son
boyadığı imlece iner; ConPTY imleci yalnızca görünürken boyar. GitPoulpi imleci gizli
tutuyordu, bu yüzden resimleri bayat satırlara iniyordu (grafik alt bilginin üzerine,
kaydırmadan sonra yırtık bir ekran). Artık Windows'ta yazdığı her resmin çevresinde, aynı
yazmanın içinde imleci gösterir ve yeniden gizler; Rio ve WezTerm 20240203 her resmi
hücresine sabitler (`pty_conpty_picture_anchor` ConPTY arkasındaki imleç hareketini geri
okur). Windows dışında veya sixel için hiçbir şey değişmez.

**Windows yapıştırması.** Yukarıdaki "Yapıştırma" tablosuna bakın: GitPoulpi bir tuş
vuruşu patlamasını tek yapıştırmaya katar; bu yüzden çok satırlı bir yapıştırma denenen
terminallerde bir istemi asla göndermez.

**Yeniden başlatma kuralı.** Exe'ye çift tıklamak onu kendi klasik konsol penceresinde
(conhost) başlatır. `ui.launch_terminal` başka bir terminali adlandırdığında (varsayılan:
Windows Terminal) GitPoulpi kendini orada yeniden açar ve ilk pencere kapanır
(`ui.relaunch_in_terminal`). Bunu **yalnızca** klasik bir konsol penceresinde tek
başınayken yapar. Exe'yi doğrudan çalıştıran bir terminal (`wt.exe gitpoulpi.exe`, WezTerm,
Rio, Alacritty `-e`, Tabby `run`) tam olarak olmak istediğiniz yerdir ve hiçbir şey yeniden
başlatılmaz; bir kabuktan başlatıldığında da tek başına olmadığı için bir şey olmaz.
`ui.relaunch_in_terminal = false` ile kapatın.

### Linux

- **kitty.** Hiçbir ayar olmadan her şey çalışır. `Shift+tık` kitty'nin kendisine aittir;
  bir seçimi genişletmek için `Alt+tık` kullanın ([Fare ve hareketler](mouse-and-gestures.md)).
- **Ghostty.** Hiçbir ayar olmadan her şey çalışır. Simgeler delik olarak görünüyorsa
  `font-family` değerine bir emoji ya da Nerd Font ekleyin (yedek eklemek için anahtarı
  yineleyin).
- **WezTerm.** Resimler iTerm2 protokolüyle, saydamlık karıştırılarak. Tam akorlar için
  `enable_kitty_keyboard = true` ayarlayın. İşaretçi şekli yok.
- **foot.** Sixel grafik ve avatarlar, işaretçi şekilleri, tam klavye, hızlı.
  `foot.ini` içinde `font=JetBrainsMono Nerd Font:size=11` ayarlayın. `OSC 52` ile
  kopyalamayı `security.osc52` yönetir.
- **Rio.** Saydamlıklı iTerm2 protokolüyle resimler, tam klavye. İşaretçi şekli yok (Rio
  `OSC 22`'yi atar).
- **Alacritty.** Karakterli grafik, 0.13'ten beri kitty klavyesi. Resim desteği yok.
- **Tabby.** Sixel resimler; kitty klavyesi yok, işaretçi şekli yok (Electron, xterm.js).
  `Ctrl+Shift+N` ve `Ctrl+,` tuşlarını kendine ayırır.
- **xterm.** Sixel için `xterm -u8 -ti vt340` olarak başlatın (kutu çizimi için UTF-8
  gerekir). Emoji yerine kutular: yazı tipi.
- **Warp.** Ölçülmedi: XWayland gerektirir ve testte sentetik tuş vuruşlarını yanlış çözdü.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Kasım 2026 turuna dahil değil. Karakterlerle ve
  eski tuşlarla çalışırlar: `Alt` varyantlarını ve yineleme için `Ctrl+Y` kullanın. VTE
  `OSC 52`'yi yok sayar, bu yüzden SSH üzerinden kopyalama panoya ulaşmaz. Bir giriş
  yöntemi (IBus, Fcitx) `Ctrl+Space` stage anahtarını ele geçirebilir; onu *Ayarlar >
  Klavye ve fare* içinde yeniden atayın. Konsole, kendi belgelerinden sixel için tanınır.
- **Wayland, Hyprland, Omarchy.** Fare işaretçisi ipucu (`OSC 22`) kitty, Ghostty ve
  foot'ta çalışır. Omarchy başlatıcı girişi `xdg-terminal-exec` üzerinden geçer.

### macOS

- **iTerm2.** 3.7.7 ve yenisi resimleri kitty protokolüyle, soru sormadan çizer. Yardımcı
  olan ayarlar:
  - *Settings > General > Selection > Applications in terminal may access clipboard*:
    `OSC 52` ile kopyalama için gerekir (aksi halde iTerm2 "the terminal attempted to
    access the clipboard but it was denied" gösterir).
  - *Profiles > Keys > Left Option key: Esc+*: `Alt` akorları için gerekir.
  - `ui.image_protocol = "iterm2"` ayarlarsanız (ya da 3.7.7'den eski bir sürüm
    kullanırsanız) "Allow Terminal-Initiated Display?" iletişim kutusunda "Remember my
    choice"ı işaretleyin. Resimler o zaman opak bir zemine sahip olur (arka planınız
    temanınkinden farklıysa görünür bir dikdörtgen; `ui.graph_renderer = "text"` onu
    kaldırır).
  - Kitty klavye protokolü varsayılan olarak açıktır (3.6.9 ve yenisi).
- **Terminal.app.** Yalnızca 256 renk: GitPoulpi `ui.color_depth = 256` değerini kendisi
  seçer ve temalar daha düz görünür. Resim yok, kitty klavyesi yok, `OSC 52` ile pano yok.
  `Alt` akorları için *Use Option as Meta key*'i işaretleyin.
- **Alacritty, Tabby.** Alacritty: karakterli grafik, kitty klavyesi, `OSC 52` kopyalama;
  `Alt` akorları için `option_as_alt = "Both"` ayarlayın. Tabby: sixel grafik ve avatarlar.
- **Option as Meta.** Terminal onu Meta'ya çevirmedikçe Option karakter oluşturur
  (Option+B bir integral işareti yazar): iTerm2 *Left Option key: Esc+*, Terminal.app *Use
  Option as Meta key*, Alacritty `option_as_alt`. GitPoulpi'nin `Alt` akorları buna ihtiyaç
  duyar ve hiçbir yerde varsayılan olarak açık değildir.
- **kitty, Ghostty, WezTerm, Rio, Warp**: macOS'ta ölçülmedi (GPU gerekir). Linux
  davranışını bekleyin.
- **xterm** (XQuartz): `xterm -u8 -ti vt340`; sixel `ui.image_protocol = "sixel"` ister.

### VS Code ve Warp

VS Code terminali (`TERM_PROGRAM=vscode`) tura dahil değildir. `terminal.integrated.enableImages`
açıkken aygıt özniteliklerinden tanınır, aksi halde GitPoulpi karakter çizer;
`ui.image_protocol = "sixel"` resimleri zorlar. Warp: yukarıdaki işletim sistemi
notlarına bakın.

## Çoklayıcılar

tmux, zellij, screen, herdr, psmux ve terminallerin kendi panelleri GitPoulpi ile terminalin
arasında durur ve nelerin geçtiğini değiştirir. Kısa özet: tmux sixel'i ve
`allow-passthrough on` ile işaretçi şekillerini ve kitty resimlerini geçirir; zellij ve
screen ne resimleri ne işaretçi şekillerini geçirir; herdr kitty resimlerini geçirir. Tüm
ayrıntılar ve eklenecek yapılandırma [Çoklayıcılar](multiplexers.md) sayfasındadır.

## Neler eksilir ve nasıl

| Eksik olan | Onun yerine ne alırsınız |
|---|---|
| Resim protokolü | Karakterli grafik. Avatar, `ui.avatar_fallback` ne derse desin renkli baş harf rozetine (tek harf) dönüşür: metin hücrelerine sıkıştırılmış bir identicon tanınmaz. Görsel diff'i ve Dosya Görünümü renkli yarım blokları ya da bir bilgi kartını kullanır. |
| Gerçek renk | Renkler 256 ya da 16 renkli palete eşlenir (`ui.color_depth`). |
| Kitty klavye protokolü | Eski tuşlar; bazı akorlar başka bir tuş olarak ulaşır. Ayarlar uyarır; yeniden atarsınız ya da `Alt` varyantlarını kullanırsınız. |
| OSC 52 | Yerelde hiçbir şey değişmez (sistem panosu kullanılır). SSH üzerinden veya ekransız, kopyalama eylemleri panoya hiçbir şey koymaz: fareyle seçin (`Shift` tuşunu basılı tutun) ve terminalin kendi kopyalamasını kullanın. |
| OSC 22 | İşaretçi değişmez; farenin altındaki ayırıcı ya da düğme yine de yanar. |
| Emoji yazı tipi | Simgeler yerine kutular ya da delikler; grafik ve metin yine de çalışır. Bir Nerd Font ya da emoji yazı tipi kurun ve yedek olarak listeleyin. |

## Bilinmesi gereken ayarlar

| Ayar | Kullanım |
|---|---|
| `ui.image_protocol` | `auto` (varsayılan), `kitty`, `iterm2`, `sixel`, `none`. Resimlerin olması gereken yerde bozuk görüntü görüyorsanız `none` yapın. Yeniden başlatma gerekir. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` her zaman doğrudur (ConPTY sınırının geçerli olduğu yerlerde kullanın). Yeniden başlatma gerekir. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | Bir terminal ya da çoklayıcı yanlış davranırsa `auto` veya `off`. Yeniden başlatma gerekir. |
| `ui.text_cursor` | `terminal` (varsayılan) veya `drawn`. |
| `ui.mouse_enabled` | Kapalıyken fare terminale kalır. Yeniden başlatma gerekir. |
| `ui.launch_terminal`, `ui.relaunch_in_terminal` | İşletim sistemi kısayollarının açtığı terminal ve çift tıklamanın GitPoulpi'yi orada yeniden açıp açmayacağı ([Ayarlar](settings.md)). |

Ayrıca bkz. [Çoklayıcılar](multiplexers.md), [Görseller](image-view.md),
[Klavye kısayolları](keybindings.md) ve [Sorun giderme ve SSS](troubleshooting.md).

## Kaynaklar

Belgeler Ekim 2026'da denetlendi, davranış Kasım 2026'da denendi. Aksi belirtilmedikçe birincil kaynaklar.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty klavyesi [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) üzerinden (ikincil)
- Windows Terminal: [1.22 announcement, sixel](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (ikincil), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [escape sequence support](https://rioterm.com/docs/escape-sequence-support) (`OSC 22` satırı yok), [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52, bir [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) üzerinden (ikincil)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (gerçek renk yok): [Apple discussion](https://discussions.apple.com/thread/255315820) (ikincil)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Karşılaştırma siteleri (ikincil, yalnızca çapraz denetim için kullanıldı): [Terminal Trove](https://terminaltrove.com/compare/terminals/)
