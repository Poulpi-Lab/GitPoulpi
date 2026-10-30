[English](../terminals.md) · [Čeština](../cs/terminals.md) · [Deutsch](../de/terminals.md) · [Español](../es/terminals.md) · [Français](../fr/terminals.md) · [Italiano](../it/terminals.md) · [日本語](../ja/terminals.md) · [한국어](../ko/terminals.md) · [Nederlands](../nl/terminals.md) · [Polski](../pl/terminals.md) · [Português (Brasil)](../pt-BR/terminals.md) · [Português (Portugal)](../pt-PT/terminals.md) · [Русский](../ru/terminals.md) · [Svenska](../sv/terminals.md) · **Türkçe** · [Українська](../uk/terminals.md) · [简体中文](../zh-CN/terminals.md)

# Terminal seçmek

GitPoulpi fare desteği olan her terminalde çalışır, ancak resim çizen, gerçek renk
gösteren ve klavyenin tamamını gönderen bir terminalde en iyi görünür ve en iyi
hissettirir. Bu sayfa terminalleri işletim sistemine göre sıralar, her birinin size
ne sunduğunu ve hangi ayarı değiştirmeniz gerektiğini söyler. Bilgiler, Ekim 2026'da her
projenin belgelerine, değişiklik günlüğüne veya issue takipçisine karşı denetlendi
([Kaynaklar](#kaynaklar)'a bakın); buradaki hiçbir şey her terminalde denenmedi.
**(doğrulanmadı)** işaretli her şey birincil bir kaynaktan doğrulanamadı.

## En iyi seçimler

| İşletim sistemi | En iyi seçimler, sırayla | Neden |
|---|---|---|
| **Windows** | **WezTerm**, sonra **Windows Terminal** | WezTerm grafiği ve avatarları saydamlıkla birlikte resim olarak çizer. Windows Terminal'in işletim sistemi entegrasyonu en iyisidir (başlatıcının varsayılanı), avatarları sixel resim olarak, grafiği ise karakterlerle çizer. |
| **Linux** (X11, Wayland, Hyprland, Omarchy) | **kitty**, **Ghostty**, **WezTerm**, sonra **foot** | kitty ve Ghostty resim ve klavyenin tamamını sunar. WezTerm de resim sunar. foot avatarları sixel resim olarak çizer, hızlıdır ve klavyenin tamamını sunar. Omarchy'de resim için Ghostty veya kitty seçin. |
| **macOS** | **kitty**, **Ghostty**, **WezTerm**, sonra **iTerm2** | Aynı gerekçe. Terminal.app çalışır ama ne resim ne de gerçek renk sunar. |

Hangi işletim sisteminde olursa olsun **tmux, zellij ve screen**, terminalin resim
protokolünü GitPoulpi'den gizler; bu yüzden karakter çizer ([Çoklayıcılar](#çoklayıcılar)'a bakın).

## GitPoulpi'nin kullandıkları

- **Resimler** (grafik sütunu, yazar avatarları, görsel diff'i ve Dosya Görünümü):
  Unicode yer tutucularıyla **kitty grafik protokolü**, **iTerm2 satır içi
  görselleri** (`OSC 1337`) ya da **sixel**, bu tercih sırasıyla. Algılama
  varsayılan olarak reddeder; `TERM`, `TERM_PROGRAM`, `KITTY_WINDOW_ID`,
  `GHOSTTY_RESOURCES_DIR`, `WEZTERM_EXECUTABLE`, `WT_SESSION`, `KONSOLE_VERSION` ve
  çoklayıcı değişkenlerine bakar, klavyeyi asla okumaz (yani terminale aygıt
  özniteliklerini de sormaz). kitty ve Ghostty kitty protokolünü kullanır. WezTerm
  (bilerek) iTerm2 protokolünü kullanır ve PNG'nin saydamlığını karıştırır. iTerm2,
  Rio ve mintty, opak bir zeminle iTerm2 protokolünü kullanır. Windows Terminal, foot,
  Konsole ve mlterm **sixel** kullanır. Bunların hiçbiri yoksa avatarlar baş harf
  rozetidir.
- **Sixel** **grafik sütunu** için (şeritler, düğümler ve `ui.avatars = "graph"` ile commit dairelerinin içindeki avatarlar), başka yerlerde **avatarlar** için (Log, commit ayrıntısı, Dosya Geçmişi, Blame, görsel görünümü) çizilir. Sixel'de alfa, kimlik ve ölçekleme yoktur; bu yüzden resim tam olarak `sütun x hücre genişliği` çarpı `satır x hücre yüksekliği` pikseldir, yalnızca bir şey olan yerde, arka planını koruyan boş hücrelerin üzerine çizilir; kodlanmış resim önbelleğe alınır, yalnızca şeritler, kaydırma konumu veya yakınlaştırma değişince yeniden kodlanır ve yalnızca altındaki bir hücre yeniden boyandığında tekrar gönderilir. Sixel'i silmenin yolu yoktur: GitPoulpi kaldırmak için hücrelerinin üzerine metin yazar. Sixel hücrenin gerçek piksel boyutunu gerektirir: Windows Terminal'e başlangıçta ve yakınlaştırmadan sonra bir kez sorulur (`CSI 16 t`, yaklaşık 10 ms içinde yanıt verir, sessiz kalırsa hiçbir şeyi engellemez); başka yerlerde pencere boyutundan veya `GITPOULPI_TERMINAL_CELL_PX=10x20` değerinden gelir. Sixel çizen ama tanınmayan bir terminal (`terminal.integrated.enableImages` açık bir VS Code terminali, `-ti vt340` ile başlatılmış bir xterm) `ui.image_protocol = "sixel"` ister. 1.22 öncesi Windows Terminal sixel çizmez; orada `ui.image_protocol = "none"` yapın (grafik karakterli kalır).
- **Gerçek renk**: temalar RGB'dir. `ui.color_depth = "auto"`, `COLORTERM=truecolor`
  olmayan bir `*-256color` `TERM` üzerinde (ve Terminal.app'te) 256 renge, Linux
  konsolunda 16 renge düşer. kitty, WezTerm ve Windows Terminal gerçek renkli sayılır.
- **Fare**: düğmeler, sürükleme ve hareketle SGR fare raporlaması (üzerine gelme
  geri bildirimi, sürükle ve bırak, sağ tık, çift tık).
- **Klavye**: **kitty klavye protokolü** (yalnızca disambiguate bayrağı) başlangıçta
  100 ms'lik bir zaman aşımıyla istenir. `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` ve
  `Ctrl+Shift+Z` tuşlarının ulaşmasını sağlar. Onsuz eski tuşlar kalır (`Ctrl+Y`
  yineler, `Alt` varyantları bazı akorların yerini alır). Bkz.
  [Klavye kısayolları](keybindings.md#terminalin-gönderemediği-tuşlar-ve-kitty-klavye-protokolü).
- **Pano**: kopyalama eylemleri `OSC 52` yazar.
- **Fare işaretçisi şekli** (ayırıcılar üzerinde yeniden boyutlandırma işaretçisi,
  tıklanabilir öğeler üzerinde el): `OSC 22`; uygulandığı yerde bir ek özelliktir.
- **Odak olayları** etkindir (pencere odağı kaybedince ipuçları kapanır). GitPoulpi
  bracketed paste, OSC 8 köprülerini veya undercurl'ü istemez. Yalnızca resim
  yolunda her kare eşzamanlı bir güncellemeye sarılır (mode 2026; bilmeyen
  terminaller yok sayar); böylece bir resim, bir açılır pencerenin üzerinde asla
  yanıp sönmez.
- **Emoji ve semboller**: GitPoulpi yalnızca varsayılan emoji sunumlu ya da tek
  genişlikli BMP şekilli glifleri kullanır; çünkü terminaller ve yazı tipleri
  varyasyon seçicileri konusunda anlaşamaz.

## Yetenek tablosu

"Resimler", GitPoulpi'nin o terminalde bugün çizdiği şeydir (**kitty** ya da
**iTerm2** ya da **sixel** protokolü ya da **metin**); terminalin yapabileceği şey değil.

| Terminal | İşletim sistemi | Resimler (grafik, avatarlar, görsel diff'i) | Gerçek renk | Kitty klavyesi (tam akorlar) | OSC 52 kopyalama | OSC 22 işaretçi |
|---|---|---|---|---|---|---|
| **kitty** (0.49) | Linux, macOS | kitty | evet | evet | evet | evet |
| **Ghostty** (1.3) | Linux, macOS (Windows sürümü yok) | kitty | evet | evet | evet | evet, 1.0'dan beri |
| **WezTerm** | Windows, Linux, macOS | iTerm2, saydamlıkla | evet | yalnızca `enable_kitty_keyboard = true` ile (varsayılan `false`) | evet (yalnızca yazma) | hayır (pull request hâlâ açık) |
| **foot** (1.28) | Linux (Wayland) | sixel (grafik ve avatarlar; kitty grafiği yok) | evet | evet, 1.10.3'ten beri | evet (`security.osc52`) | evet, 1.12'den beri |
| **Alacritty** (0.17) | tümü | metin (yukarı akışta görsel desteği yok) | evet | evet, 0.13'ten beri | evet (yazma; yapıştırma varsayılan olarak kapalı) | hayır (uygulanmadı) |
| **Konsole** | Linux | sixel (grafik ve avatarlar) | evet | 2026'da ekleniyor (merge request !1188); sürüm (doğrulanmadı) | yalnızca yazma, 2024 ortasında birleştirildi | (doğrulanmadı) |
| **GNOME Terminal, Ptyxis** (VTE) | Linux | metin | evet | hayır (yayımlanmış bir VTE'de yok) | hayır (VTE yok sayar) | hayır |
| **iTerm2** (3.7) | macOS | iTerm2, opak zemin | evet | evet, 3.6.9'dan beri (Profiles > Keys'i denetleyin) | varsayılan olarak kapalı: "Applications in terminal may access clipboard"u açın | (doğrulanmadı) |
| **Terminal.app** | macOS | metin | hayır (256 renk) | hayır | (doğrulanmadı) | (doğrulanmadı) |
| **Windows Terminal** (1.25) | Windows | sixel (grafik ve avatarlar) | evet | evet, 1.25'ten beri kararlı (Ekim 2026); Preview 1.25'ten (Mart 2026) | evet (yazma; yalnızca odaktayken) | hayır (uygulanmadı) |
| **Rio** | Windows, Linux, macOS | iTerm2, opak zemin (kitty grafiğini de konuşur) | evet | evet (varsayılan olarak açık) | evet | hayır (uygulanmadı) |
| **Warp** | macOS, Linux, Windows | metin (algılanmaz) | evet | evet | evet (diziyi dikkate alır) | (doğrulanmadı) |
| **VS Code terminali** (xterm.js) | tümü | metin (sixel ve iTerm2 görselleri `terminal.integrated.enableImages` arkasında var; `ui.image_protocol = "sixel"` sixel'i kullanır) | evet | (doğrulanmadı) | (doğrulanmadı) | hayır |
| **mintty** (Cygwin, Git Bash) | Windows | iTerm2, opak zemin | evet | (doğrulanmadı; belgelenmemiş) | (doğrulanmadı; belgelenmemiş) | evet |
| **Windows konsolu** (conhost, cmd, PowerShell penceresi) | Windows | metin | yalnızca yeni Windows 10/11 | gerekmez: değiştiricileri kendisi bildirir | hayır | hayır |
| **ConEmu** | Windows | metin | (doğrulanmadı) | (doğrulanmadı) | (doğrulanmadı) | (doğrulanmadı) |
| **tmux, zellij, screen** | tümü | metin, her zaman | ayar gerekir | ayar gerekir | ayar gerekir | hayır |

OSC 22 için "hayır (uygulanmadı)" hücreleri, bir test değil, bir projenin değişiklik
günlüğünde ve belgelerinde bulunmamasından ve kullanıcı raporlarından gelir; onları
"belgelenmemiş" olarak okuyun. Sayfa emoji ve varyasyon seçici genişliğini tablolamaz:
terminale ve yazı tipine göre değişir; GitPoulpi'nin VS16 glifleri kullanmamasının
nedeni budur ([GitPoulpi'nin kullandıkları](#gitpoulpinin-kullandıkları)'na bakın).

## Her terminal için

### Windows

- **WezTerm.** Windows'ta en iyi resim desteği. `ui.image_protocol` değerini `auto`
  bırakın. `wezterm.lua` içinde, tam akorlar için `config.enable_kitty_keyboard = true`
  ve emoji yedekli bir yazı tipi ayarlayın, örneğin
  `config.font = wezterm.font_with_fallback { 'JetBrainsMono Nerd Font', 'Segoe UI Emoji' }`.
  WezTerm'in etiketlenmiş son kararlı sürümü 20240203-110809-5046fc22'dir (Şubat 2024);
  daha yeni düzeltmeler nightly sürümlerde gelir. WezTerm'de henüz OSC 22 işaretçi şekli yoktur.
- **Windows Terminal.** Gerçek renk, fare, OSC 52 ve iyi emoji. GitPoulpi avatarları
  sixel resim olarak çizer (Windows Terminal 1.22 veya sonrası); grafik karakterlerdir. Windows Terminal'in kendi ayarlarından (*Profiller > Varsayılanlar > Görünüm >
  Yazı tipi*) bir Nerd Font seçin. Fare yakalanmışken metin seçmek için
  `Shift` tuşunu basılı tutun. İşaretçi şekli değişmez. 1.25'ten eski bir sürümde (kitty
  klavye protokolü yok) `Alt` varyantlarını kullanın.
- **Rio**, opak bir resim zeminiyle çalışır; daha az denenmiştir.
- **mintty** (Git Bash, MSYS2, Cygwin) resimleri iTerm2 protokolüyle, opak bir zeminle
  çizer; daha az denenmiştir.
- **Windows konsolu, ConEmu, cmd**: kaçının. Resimler kötü çizilir ya da hiç çizilmez.
  Exe'ye çift tıklamak GitPoulpi'yi seçtiğiniz terminalde yeniden açar
  (*Ayarlar > Genel > Başlatıcı*).

### Linux

- **kitty.** Hiçbir ayar olmadan her şey çalışır. `Shift+tık` kitty'nin kendisine aittir;
  bir seçimi genişletmek için `Alt+tık` kullanın ([Fare ve hareketler](mouse-and-gestures.md)).
- **Ghostty.** Hiçbir ayar olmadan her şey çalışır. Simgeler delik olarak görünüyorsa
  `font-family` değerine bir emoji ya da Nerd Font ekleyin (yedek eklemek için anahtarı
  yineleyin).
- **WezTerm.** Resimler iTerm2 protokolüyle, saydamlık karıştırılarak. Windows'taki
  `enable_kitty_keyboard` notu aynen geçerlidir.
- **foot.** Karakterli grafik, tam klavye, hızlı. Avatarlar sixel resimdir, grafik
  karakterlerdir. `foot.ini` içinde `font=JetBrainsMono Nerd Font:size=11` ayarlayın.
- **Alacritty.** Karakterli grafik, 0.13'ten beri kitty klavyesi. Alacritty'de görsel
  desteği yoktur.
- **Konsole, GNOME Terminal, Ptyxis (VTE).** Karakterlerle ve eski tuşlarla çalışır:
  `Alt` varyantlarını ve yineleme için `Ctrl+Y` kullanın (yeni bir Konsole kitty klavye
  protokolünü ekleyebilir). VTE'de kopyalama eylemleri (`OSC 52`) panoya hiçbir şey koymaz.
  Bir giriş yöntemi (IBus, Fcitx) `Ctrl+Space` stage anahtarını ele geçirebilir -
  **(doğrulanmadı)**; onu *Ayarlar > Klavye ve fare* içinde yeniden atayın.
- **Wayland, Hyprland, Omarchy.** Fare işaretçisi ipucu (`OSC 22`) kitty, Ghostty ve
  foot'ta çalışır. Omarchy başlatıcı girişi `xdg-terminal-exec` üzerinden geçer.

### macOS

- **kitty, Ghostty, WezTerm**: Linux'taki gibi.
- **iTerm2.** Opak zeminli resimler (arka planınız temanınkinden farklıysa görünür bir
  dikdörtgen; `ui.graph_renderer = "text"` onu kaldırır). Kopyalama için *Settings >
  General > Selection > Applications in terminal may access clipboard*'u, `Alt` akorları
  için *Profiles > Keys > Left Option key: Esc+*'i etkinleştirin.
- **Terminal.app.** Yalnızca 256 renk: GitPoulpi `ui.color_depth = 256` değerini kendisi
  seçer ve temalar daha düz görünür. Resim yok. `Alt` akorları için *Use Option as Meta
  key*'i işaretleyin.

### VS Code ve Warp

VS Code terminali (`TERM_PROGRAM=vscode`) ve Warp resim terminali olarak tanınmaz;
GitPoulpi karakter çizer. İkisi de kullanılabilir. VS Code'da `terminal.integrated.enableImages`'ı açın ve avatar resimleri için
`ui.image_protocol = "sixel"` yapın. Warp, kitty klavye protokolü desteğini
belgeler; VS Code terminalinin desteği **(doğrulanmadı)**.

## Çoklayıcılar

GitPoulpi tmux, zellij, screen ve herdr'i algılar (`TMUX`, `ZELLIJ`, `STY`,
`TERM=screen*` ya da `tmux*`, `HERDR_*`) ve **karakter** çizer; çünkü bir çoklayıcı
resim kaçış dizilerini değil, hücreleri iletir. Diğer özellikler için `~/.tmux.conf`
içinde:

```
set -g mouse on
set -as terminal-features ',*:RGB'     # gerçek renk
set -g extended-keys on                # tam akorlar (tmux 3.2 veya sonrası)
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a veya sonrası; yalnızca resimleri zorlarsanız, aşağıya bakın
```

Resimleri ileten bir çoklayıcıdan geçirmeye zorlamak için `ui.image_protocol` değerini
`kitty` veya `iterm2` yapın. kitty, Ghostty veya WezTerm
içindeki `kitty_graphics = true` ayarlı herdr kendiliğinden algılanır; bkz.
[Sorun giderme ve SSS](troubleshooting.md#grafik-resim-olarak-değil-karakterlerle-çiziliyor-herdr-tmux-diğer-terminaller).
`GITPOULPI_TERMINAL_IMAGE=off` tüm resimleri kapatır.

tmux kitty klavye protokolünü kendisi iletmez: `extended-keys` değiştirilmiş tuşları
`CSI u` ya da `xterm` dizileri olarak göndermesini sağlar. zellij 0.41 ve sonrası,
değişiklik günlüğüne göre kitty klavye protokolünü konuşur ve OSC 52'yi iletir.

## Neler eksilir ve nasıl

| Eksik olan | Onun yerine ne alırsınız |
|---|---|
| Resim protokolü | Karakterli grafik. Avatar, `ui.avatar_fallback` ne derse desin renkli baş harf rozetine (tek harf) dönüşür: metin hücrelerine sıkıştırılmış bir identicon tanınmaz. Görsel diff'i ve Dosya Görünümü renkli yarım blokları ya da bir bilgi kartını kullanır. |
| Gerçek renk | Renkler 256 ya da 16 renkli palete eşlenir (`ui.color_depth`). |
| Kitty klavye protokolü | Eski tuşlar; bazı akorlar başka bir tuş olarak ulaşır. Ayarlar uyarır; yeniden atarsınız ya da `Alt` varyantlarını kullanırsınız. |
| OSC 52 | Kopyalama eylemleri panoya hiçbir şey koymaz: fareyle seçin (`Shift` tuşunu basılı tutun) ve terminalin kendi kopyalamasını kullanın. |
| OSC 22 | İşaretçi değişmez; farenin altındaki ayırıcı ya da düğme yine de yanar. |
| Emoji yazı tipi | Simgeler yerine kutular ya da delikler; grafik ve metin yine de çalışır. Bir Nerd Font ya da emoji yazı tipi kurun ve yedek olarak listeleyin. |

## Bilinmesi gereken ayarlar

| Ayar | Kullanım |
|---|---|
| `ui.image_protocol` | `auto` (varsayılan), `kitty`, `iterm2`, `sixel`, `none`. Resimlerin olması gereken yerde bozuk görüntü görüyorsanız `none` yapın. Yeniden başlatma gerekir. |
| `ui.graph_renderer` | `auto`, `image`, `text`. `text` her zaman doğrudur. Yeniden başlatma gerekir. |
| `ui.color_depth` | `auto`, `truecolor`, `256`, `16`. |
| `ui.kitty_keyboard` | Bir terminal ya da çoklayıcı yanlış davranırsa `auto` veya `off`. Yeniden başlatma gerekir. |
| `ui.mouse_enabled` | Kapalıyken fare terminale kalır. Yeniden başlatma gerekir. |
| `ui.launch_terminal` | İşletim sistemi kısayollarının açtığı terminal ([Ayarlar](settings.md)). |

Ayrıca bkz. [Görseller](image-view.md), [Klavye kısayolları](keybindings.md) ve
[Sorun giderme ve SSS](troubleshooting.md).

## Kaynaklar

Ekim 2026'da denetlendi. Aksi belirtilmedikçe birincil kaynaklar.

- kitty: [changelog](https://sw.kovidgoyal.net/kitty/changelog/), [pointer shapes](https://sw.kovidgoyal.net/kitty/pointer-shapes/)
- Ghostty: [OSC 22](https://ghostty.org/docs/vt/osc/22), [OSC 52](https://ghostty.org/docs/vt/osc/52), [download](https://ghostty.org/download)
- WezTerm: [enable_kitty_keyboard](https://wezterm.org/config/lua/config/enable_kitty_keyboard.html), [escape sequences](https://wezterm.org/escape-sequences.html), [OSC 22 pull request](https://github.com/wezterm/wezterm/pull/6292), [releases](https://github.com/wezterm/wezterm/releases)
- foot: [changelog](https://codeberg.org/dnkl/foot/raw/branch/master/CHANGELOG.md)
- Alacritty: [changelog](https://raw.githubusercontent.com/alacritty/alacritty/master/CHANGELOG.md), [releases](https://github.com/alacritty/alacritty/releases)
- Konsole: [OSC 52 merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/767), [kitty keyboard merge request](https://invent.kde.org/utilities/konsole/-/merge_requests/1188)
- VTE: [OSC 52 issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2495), [kitty keyboard issue](https://gitlab.gnome.org/GNOME/vte/-/issues/2601)
- iTerm2: [downloads](https://iterm2.com/downloads.html); kitty klavyesi [terminfo.dev matrix](https://terminfo.dev/extensions/kitty-keyboard-protocol) üzerinden (ikincil)
- Windows Terminal: [1.22 announcement](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-22-release), [Preview 1.25](https://devblogs.microsoft.com/commandline/windows-terminal-preview-1-25-release/) (ikincil), [v1.25.2733.0](https://github.com/microsoft/terminal/releases/tag/v1.25.2733.0)
- Rio: [features](https://rioterm.com/docs/features), [kitty keyboard](https://rioterm.com/docs/features/kitty-keyboard-protocol)
- Warp: [full-screen apps](https://docs.warp.dev/terminal/more-features/full-screen-apps); OSC 52, bir [Warp OSC 52 advisory write-up](https://www.superagent.sh/blog/when-terminal-output-owns-your-clipboard-osc-52-in-warp) üzerinden (ikincil)
- VS Code: [advanced terminal](https://code.visualstudio.com/docs/terminal/advanced)
- mintty: [control sequences](https://raw.githubusercontent.com/mintty/mintty/master/wiki/CtrlSeqs.md), [tips](https://raw.githubusercontent.com/mintty/mintty/master/wiki/Tips.md)
- Terminal.app (gerçek renk yok): [Apple discussion](https://discussions.apple.com/thread/255315820) (ikincil)
- tmux: [CHANGES](https://raw.githubusercontent.com/tmux/tmux/master/CHANGES); zellij: [CHANGELOG](https://raw.githubusercontent.com/zellij-org/zellij/main/CHANGELOG.md)
- Karşılaştırma siteleri (ikincil, yalnızca çapraz denetim için kullanıldı): [Terminal Trove](https://terminaltrove.com/compare/terminals/)

Hâlâ doğrulanmamış olanlar: Konsole ve mlterm'de sixel (projelerin kendi özellik listelerinden alındı, bir sürüme karşı yeniden denetlenmedi ve hiçbir terminalde gerçek bir pencerede denenmedi); Konsole, iTerm2, Terminal.app ve Warp'ta OSC 22; Terminal.app,
VS Code ve mintty'de OSC 52; VS Code, mintty ve ConEmu'da kitty klavye protokolü; ConEmu'nun
her hücresi; ve terminal başına emoji genişliği. Konsole'un kitty klavye protokolünü hangi
sürümde sunduğu doğrulanmadı.
