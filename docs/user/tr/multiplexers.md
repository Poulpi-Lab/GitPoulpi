[English](../multiplexers.md) · **Türkçe**

# Çoklayıcılar ve paneller

Bir çoklayıcı GitPoulpi ile terminaliniz arasında durur: hücreleri kendisi yeniden çizer ve
yalnızca anladığı kaçış dizilerini iletir. Bu yüzden terminalin tek başına yaptığı şey
([Terminal seçmek](terminals.md)) tmux, zellij, screen veya herdr içinde çalışmayabilir.
Bu sayfa nelerin geçtiğini ve eklenecek yapılandırmayı söyler. Sonuçlar, terminal
tablolarıyla aynı Kasım 2026 test turundan gelir: tmux 3.8 (MSYS2: 3.7c), zellij 0.45.1 ve
0.46.0, GNU screen 5.0 (macOS 4.00.03 ile gelir), herdr 0.9.5, psmux 3.4.0.

GitPoulpi bir çoklayıcıyı `TMUX`, `ZELLIJ`, `STY`, `TERM=screen*` veya `tmux*` ve
`HERDR_*` değişkenlerinden algılar. Birinin içindeyken resimler, aşağıdaki tablonun
GitPoulpi'nin çalışan bir yol algıladığını söylediği yerler dışında karakter çizmeye
döner. `GITPOULPI_TERMINAL_IMAGE=off` tüm resimleri kapatır, `ui.image_protocol` bir
protokolü zorlar.

## Bir bakışta

| | Resimler | İşaretçi şekli (OSC 22) | Pano (OSC 52) | Tuşlar |
|---|---|---|---|---|
| **tmux** | arkadaki terminal kabul ediyorsa sixel (algılanır); `allow-passthrough on` ve `ui.image_protocol = "kitty"` ile kitty resimleri | `allow-passthrough on` ile kitty ve foot'ta; Ghostty'de değil | `set-clipboard on` ile | kitty klavyesi yok; `Ctrl+Enter` ve `Ctrl+,` asla ulaşmaz |
| **zellij** | karakterler | hayır | evet | kitty klavyesi çalışır |
| **GNU screen** | karakterler | hayır | hayır | eski tuşlar |
| **herdr** | kitty resimleri (`kitty_graphics = true` ile algılanır) | hayır | evet | ölçülmedi |
| **psmux** (Windows) | karakterler (tasarım gereği) | hayır (Windows Terminal'de yok) | sistem panosu çalışır | ölçülmedi |
| **MSYS2 tmux** (Windows) | karakterler (tasarım gereği) | hayır | sistem panosu çalışır | ölçülmedi; ekran bozuk |
| **WezTerm panelleri**, **Windows Terminal panelleri**, **kitty ve Ghostty bölmeleri**, **iTerm2 bölünmüş panelleri** | terminalin kendisi gibi | terminal gibi | terminal gibi | terminal gibi |
| **iTerm2 `tmux -CC`** | karakterler | hayır | hayır (iTerm2 varsayılan olarak reddeder) | ölçülmedi |

## tmux

**Resimler.**

- **Sixel**, tmux'ın kendisi tarafından, onu destekleyen bir istemci için yeniden kodlanarak
  çizilir. GitPoulpi başlangıçta tmux'a bir kez sorar (`tmux display-message -p
  '#{client_termfeatures}'`, 400 ms ile sınırlı) ve istemci onu listelerse sixel kullanır.
  foot, WezTerm, Rio ve Tabby arkasında durum budur; kitty ve Ghostty arkasında değildir
  (tmux o zaman "SIXEL IMAGE" metni yazdırır). Sixel'li derlenmiş bir tmux gerekir (Arch'ın
  ve Homebrew'unki öyledir; `tmux -V` ve bir sixel testiyle denetleyin). Ayar yok. tmux'ın
  aygıt öznitelikleri sorgusuna kendi yanıtı, arkasında ne olursa olsun sixel olduğunu
  söyler; GitPoulpi bu yüzden istemcinin özelliklerini sorar.
- **tmux arkasındaki iTerm2** sixel bildirir ama ayarlanmamış pikselleri opak boyar; bu
  yüzden çoğunlukla saydam olan grafik düz bir dikdörtgen olurdu. GitPoulpi iTerm2'yi
  (`LC_TERMINAL`, `ITERM_SESSION_ID`, istemci türü) tanır ve karakter çizer.
- **kitty resimleri** tmux'ın DCS geçişinden (passthrough) geçer (kitty, Ghostty; ham
  yoklamada iTerm2 ve Rio'da da görüldü). `set -g allow-passthrough on` (tmux 3.3a veya
  yenisi) **ve** GitPoulpi'de `ui.image_protocol = "kitty"` gerekir; GitPoulpi o zaman
  yüklemeleri ve silmeleri sarar. tmux seçeneği olmadan sarmalayıcı düşer ve grafik boş
  kalır.
- iTerm2 satır içi görselleri tmux'tan geçer ama tmux onları konumlandıramaz: GitPoulpi
  orada onları asla kullanmaz.

**İşaretçi şekli.** GitPoulpi `OSC 22`'yi geçiş için sarar. `allow-passthrough on` ile
kitty ve foot'ta pencereye ulaşır. Ghostty sarılıyken göstermez (ok kalır). WezTerm, Rio,
Alacritty, Tabby, iTerm2'de gösterilecek bir `OSC 22` yoktur.

**Pano.** `set -g set-clipboard on`. GitPoulpi ayrıca geçişe sarılmış biçimi de gönderir
(`allow-passthrough on` ile çalışır). kitty, Ghostty, Alacritty ve Tabby arkasında ulaşır;
iTerm2 pano izni açılana kadar reddeder.

**Tuşlar.** tmux kitty klavye sorgusunu yanıtlamaz ve `extended-keys` `Ctrl+Enter` veya
`Ctrl+,` tuşlarını geçirmez: kitty içindeki tmux 3.8 ile ölçüldü, `extended-keys` kapalı,
açık veya always iken ikisi de sade `Enter` ve `,` olarak ulaşır. `Ctrl+Shift+Z` gibi harf
akorları yalnızca `extended-keys` açık veya always **ve** `extended-keys-format csi-u`
(tmux 3.5 veya yenisi) **ve** genişletilmiş tuşları isteyen bir uygulama olduğunda
ulaşır; GitPoulpi asla istemez, bu yüzden ayar burada hiçbir şeyi değiştirmez. tmux
içinde `Alt` varyantlarını kullanın
([Klavye kısayolları](keybindings.md#terminalin-gönderemediği-tuşlar-ve-kitty-klavye-protokolü)).

**Yapılandırma** (`~/.tmux.conf`):

```
set -g mouse on
set -as terminal-features ',*:RGB'     # gerçek renk
set -g set-clipboard on                # OSC 52
set -g allow-passthrough on            # tmux 3.3a veya sonrası: işaretçi şekilleri, zorlanmış kitty resimleri
```

**WSL içindeki tmux.** Ölçülmedi. Linux derlemesidir, bu yüzden dıştaki terminal olarak
Windows Terminal ile yukarıdakinin aynısını bekleyin.

## zellij

Resimler: zellij kitty protokolünü kabul eder ama Unicode yer tutucularını kabul etmez ve
sixel veya iTerm2 görseli taşımaz; bu yüzden GitPoulpi karakter çizer (bilinçli bir veto).
İşaretçi şekli: yutulur, bu yüzden GitPoulpi göndermez. Pano: `OSC 52` iletilir (kitty,
Ghostty, Alacritty, Tabby arkasında çalışır; iTerm2 ve Terminal.app reddeder veya yok
sayar). Tuşlar: zellij kitty klavye sorgusunu yanıtlar, bu yüzden klavyenin tamamı çalışır.

zellij 0.46 temiz bir başlangıçta bir "Welcome to Zellij" penceresi açar: `Esc`'e basın.
Çıkmış bir oturum adını korur ve aynı adla bir sonraki başlangıç ne yapılacağını sorar.
Yapılandırma gerekmez.

## GNU screen

Hiçbir şey geçmez: resimler ve `OSC 22` karakter olarak yazdırılır veya düşer, `OSC 52` yok
sayılır ve klavye eskisidir. GitPoulpi karakter çizer. screen 4.00.03 (macOS'un getirdiği
sürüm) arayüzde renkleri ve geniş glifleri de bozar; screen 5 yalnızca Linux'ta denendi.
Kullanılabilir bir oturum için tmux veya zellij kullanın.

## herdr

herdr, **kitty resimlerini** yeniden yayan bir terminal gömer. GitPoulpi onları herdr'in
`config.toml` dosyası `kitty_graphics = true` dediğinde (`[terminal]` altında veya eski
sürümlerde `[experimental]`) ve herdr'in çevresindeki terminal onları çizdiğinde (kitty,
Ghostty, WezTerm) kullanır: algılanır, ayar yok. Yapılandırma başka yerdeyse (örneğin
GitPoulpi'nin okumadığı `HERDR_HOME` altında) `ui.image_protocol = "kitty"` ayarlayın.
SSH üzerinden (`herdr --remote`) GitPoulpi yerel istemciyi göremez, bu yüzden karakter
çizer; bkz.
[Sorun giderme](troubleshooting.md#grafik-resim-olarak-değil-karakterlerle-çiziliyor-herdr-tmux-diğer-terminaller).
`OSC 22` herdr 0.9.5 tarafından iletilmez, bu yüzden GitPoulpi göndermez. `OSC 52` kitty,
Ghostty, Alacritty ve Tabby arkasında ulaşır. Ham yoklamada herdr kitty resimlerini kitty,
Ghostty ve iTerm2'de, varsayılan yapılandırmayla da çizdi. Tuşlar: ölçülmedi.

## psmux ve MSYS2 tmux (Windows)

- **psmux** (Rust, ConPTY, tmux komut dili; `winget install psmux`) `TMUX` ayarlar, bu
  yüzden GitPoulpi orada tasarım gereği karakter çizer. Windows Terminal içinde normal
  çalışır ve kopyalama sistem panosu üzerinden işler; işaretçi değişmez (Windows
  Terminal'de `OSC 22` yok). Diğer ana terminaller ölçülmedi.
- **MSYS2 veya Cygwin'den tmux** da `TMUX` ayarlar (tasarım gereği karakterler) ve testte
  ekran bozuktu: mintty'de ve Windows Terminal'de (`-u` ile ve onsuz; onsuz kutu çizimi
  `qqqq` olarak yazdırıldı) bayat kareler üst üste çizildi. Bir MSYS2 sözde terminalinin
  içindeki yerel bir Windows programı tmux tarafından doğru yeniden çizilmez. Bu
  GitPoulpi'de düzeltilemez: psmux, Windows Terminal panelleri veya WezTerm panelleri
  kullanın.
- zellij ve screen'in Windows sürümü yoktur. herdr 0.9.5 kuruldu ama çalıştırılmadı.

## Terminal panelleri

Terminalin kendisine ait paneller GitPoulpi için çoklayıcı değildir:

- **WezTerm panelleri** (`wezterm cli split-pane`, `wezterm connect`, SSH etki alanları):
  WezTerm her şeyi çizer, bu yüzden resimler çalışır (panelde grafik ve avatarlar) ve
  işaretçi WezTerm'in kendisindeki gibi davranır (şekiller upstream WezTerm PR #8092'li bir
  derlemede çalışır; panel ayırıcısı yeniden boyutlandırma işaretçisini verir). Linux veya
  macOS'ta ölçülmedi.
- **Windows Terminal panelleri** (`wt ; split-pane`): sixel grafik ve avatarlar panelde
  çalışır, kopyalama çalışır, işaretçi şekilleri yok.
- **kitty, Ghostty ve iTerm2 bölmeleri**: yerel, terminal gibi. kitty ve Ghostty için
  macOS'ta ölçülmedi (GPU gerekir).

## iTerm2 `tmux -CC` (denetim kipi)

iTerm2 her tmux penceresi için yerel bir pencere açar ve bir "[tmux bash]" ağ geçidi
penceresi tutar. `TMUX` ayarlıdır, bu yüzden GitPoulpi yerel pencerede karakter çizer.
Görseller denetim bağlantısı üzerinden iTerm2'ye gerçekten ulaşır (bir `OSC 1337` testi
için "Allow Terminal-Initiated Display?" iletişim kutusu çıktı), ama GitPoulpi onları orada
kullanmaz. Zorlanmış bir `ui.image_protocol = "kitty"` değerinin kitty yükü ağ geçidi
penceresinde yazdırılır, bu yüzden onu `auto` bırakın. `OSC 22` ve `OSC 52` ulaşmaz.

## Bilinmesi gereken ayarlar

| Ayar | Kullanım |
|---|---|
| `ui.image_protocol` | `kitty`, kitty resimlerini onları ileten bir çoklayıcıdan geçmeye zorlar (`allow-passthrough on` ile tmux, GitPoulpi'nin okumadığı bir yapılandırmalı herdr); karakterler için `none`. |
| `ui.graph_renderer` | `text` her zaman doğrudur. |
| `ui.kitty_keyboard` | Bir çoklayıcı klavye sorgusuyla yanlış davranırsa `off`. |

Çoklayıcının arkasındaki terminal için [Terminal seçmek](terminals.md) sayfasına ve grafik
karakterlerle çizildiğinde [Sorun giderme](troubleshooting.md) sayfasına bakın.
