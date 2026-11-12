[English](../ai.md) · [Čeština](../cs/ai.md) · [Deutsch](../de/ai.md) · [Español](../es/ai.md) · [Français](../fr/ai.md) · [Italiano](../it/ai.md) · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · [Nederlands](../nl/ai.md) · [Polski](../pl/ai.md) · [Português (Brasil)](../pt-BR/ai.md) · [Português (Portugal)](../pt-PT/ai.md) · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · **Türkçe** · [Українська](../uk/ai.md) · [简体中文](../zh-CN/ai.md)

# YZ desteği

GitPoulpi hiçbir zaman hesap gerektirmez ve **YZ varsayılan olarak kapalıdır**.
YZ açıkken commit ve stash mesajları yazabilir, değişiklikleri açıklayabilir,
pull request açıklamaları yazabilir, merge çakışması çözümleri önerebilir ve
çalışma değişikliklerinizi birkaç commit'e bölebilir. Model sizindir: sahip
olduğunuz bir API anahtarı, yerel bir sunucu ya da zaten kurulu bir ajan CLI'sı
(örneğin `claude -p`).

Her YZ denetimi mor `✦` simgesini gösterir. YZ kapalıyken:

- Hiçbir yere hiçbir şey gönderilmez.
- Hiçbir YZ denetimi gösterilmez: `✦` düğmesi, `YZ:` palet komutu, YZ menü
  satırı ve YZ tuş ipucu yoktur. YZ tuşları hiçbir şey yapmaz.
- Deponun YZ sayfası yalnızca "YZ kapalı (Ayarlar > YZ)" mesajını gösterir.
- MCP sunucusu `ai_*` araçlarını listelemez. `gitpoulpi ai ...` CLI komutları
  aynı mesajla reddeder.

**YZ'yi etkinleştir** hemen etkili olur. Yeniden başlatmanız gerekmez.

GitPoulpi ile YZ kullanmanın iki yolu vardır:

| | Yerleşik YZ (bu sayfa) | Harici bir ajan |
|---|---|---|
| Ne | GitPoulpi bir modelden commit mesajı, açıklama, ... ister | Claude Code, Cursor, Codex... GitPoulpi'yi bir araç olarak kullanır |
| Ayarlandığı yer | Ayarlar > YZ (`[ai]`) | ajanın MCP yapılandırması, sınırlar `[mcp]` içinde |
| Rehber | aşağıda | [MCP ve ajanlar](mcp-and-agents.md) |

## YZ'yi yapılandırma

1. `Ctrl+,` > **YZ** > **YZ'yi etkinleştir** kutusunu işaretleyin.
2. Profiller, **Profiller (bağlantılar)** sekmesinin listesidir ve sayfanın solundadır (dar bir terminalde üstte durur):
   birini seçin, sağda düzenleyin (`Enter` onun satırlarına geçer; paletteki
   **YZ: Profili Düzenle...** komutu doğrudan listeye götürür). *Default* adlı bir
   profil her zaman bulunur. **+ Ekle** bir tane daha oluşturur, **Çoğalt** seçili olanı
   (anahtarı olmadan) kopyalar, **Kaldır** önce sorar. **Sağlayıcı**yı seçin ve alanlarını
   doldurun:

   | Sağlayıcı | Alanlar | Varsayılanlar |
   |---|---|---|
   | **Yerel ajan** (anahtar yok, kendine ait hesap yok) | **Ajan türü** (hangi ajan olduğu), **Komut** (tam program: `PATH` üzerinde, bir takma ad ya da sarmalayıcı betik veya tam yol), etkileşimsiz çalışmasını sağlayan **Argümanlar** ve isteğe bağlı **Ortam değişkenleri**. İstem standart girdisine gider, yanıt standart çıktısından okunur. | Claude Code: `claude`, `-p --model haiku` |
   | **Anthropic** | **API anahtarı**, isteğe bağlı **Model** ve **Temel URL** | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **API anahtarı**, Model, Temel URL | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **API anahtarı**, Model, Temel URL | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **OpenAI uyumlu** (Ollama, LM Studio, vLLM, bir şirket ağ geçidi) | **Temel URL** (`.../v1`), **Model**, isteğe bağlı anahtar | `http://localhost:11434/v1` (Ollama; LM Studio için `http://localhost:1234/v1`), `llama3` |

   **Ön ayarlar** (her zaman başka bir değer yazabilirsiniz):

   ![Yerel ajan profilinin Ajan türü açılır listesi: Claude Code, OpenAI Codex CLI, Gemini CLI, opencode ve diğer ajan CLI'ları](../images/settings-ai-presets.png)

   - **Yerel ajan > Ajan türü**, yaygın ajan CLI'larının en çok kullanılandan
     başlayan bir açılır listesidir: Claude Code, OpenAI Codex CLI, Gemini CLI,
     opencode, Cursor Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose,
     Crush, Amp veya *Diğer (öneri yok)*. Tür, **komuttan ayrıdır**:
     yalnızca hangi önerileri alacağınızı belirler. Birini seçmek **Komut**
     alanını (hâlâ önceki türün varsayılanıysa) ve **Argümanlar** alanını
     başsız çağrıyla (`claude -p`, `codex exec -`, `gemini -p ...`, ...) önceden
     doldurur. **Komut** serbest metindir: `/opt/bin/work-claude`, bir sarmalayıcı
     betik ya da bir takma ad hedefi yazın; tür, seçenekleri, model listesi ve
     değişkenleriyle birlikte kalır. **Ajan türü** boşsa GitPoulpi onu komuttan
     çıkarır.
   - **Argüman seçenekleri**, GitPoulpi'nin tek seferlik kullanımına uygun
     seçenekleri ekler veya çıkarır; her birinin tek satırlık bir gerekçesi
     vardır: araç yok, kayıtlı oturum yok, düz metin çıktı, düşük akıl yürütme
     eforu, salt okunur kip, izin istemi yok, güncelleme denetimi veya analitik
     yok. Liste ajana göre değişir, çünkü ajanlar birbirinden farklıdır
     (yalnızca bazıları araçları komut satırından kapatabilir; opencode için
     bu bir ortam değişkenidir). **Model** o ajanın modellerini listeler ve
     doğru bayrağı yazar (Claude Code için `--model haiku`, Codex için `-m
     gpt-6-luna`, opencode için `--model provider/model`); *(varsayılan - 
     bayrak yok)* onu kaldırır. Amp'in model bayrağı yoktur. `aider` istemini
     `/dev/stdin` üzerinden okur, bu yüzden yalnızca Linux ve macOS'ta çalışır.
   - **Ortam değişkenleri** yalnızca o ajanın sürecine ayarlanır; GitPoulpi'ye
     veya diğer profillere asla. **Ortam değişkeni ekle**, bu ajan için önemli
     olan belgelenmiş değişkenleri, her birinin ne yaptığıyla birlikte sunar
     (telemetriyi ve güncelleme denetimlerini kapatma, efor düzeyi, yapılandırma
     klasörü, API anahtarı, temel URL veya yerel sunucu, ...); `NAME` veya
     `NAME=value` da yazabilirsiniz. Sonra her değişkenin kendi **Env NAME**
     satırı olur; bu satırda değeri bir düğme olarak görürsünüz (yenisini yazmak için ona
     basın; alandan çıktığınızda kaydedilir) ve değişkeni silen bir **[x]** düğmesi vardır. Bir değer, GitPoulpi'nin kendi
     değişkenlerinden birini ödünç almak için `${env:NAME}` olabilir (API
     anahtarları için kullanışlıdır, böylece `config.toml` dosyasına
     yazılmazlar). Gizli bilgiye benzeyen değerler (`_KEY`, `_TOKEN`, `_SECRET`,
     ... ile biten adlar ya da bilinen anahtar önekleri) noktalarla görünür ve
     baştan yazılır. Değerler asla günlüğe yazılmaz.
   - **HTTP sağlayıcıları > Sağlayıcı ön ayarı**; OpenAI, Anthropic,
     OpenRouter, Google Gemini, xAI, DeepSeek, Mistral, Groq, Together,
     Fireworks, Cerebras, Perplexity, Moonshot, Qwen (DashScope), Zhipu (GLM) ve
     yerel sunucular Ollama, LM Studio, llama.cpp, vLLM, Jan ve LocalAI için
     temel URL'yi ve bir modeli doldurur. **Model**, o sağlayıcının
     modellerinin açılır listesidir; serbest metin de olur.

3. **Bağlantıyı sına** düğmesine basın: *Son sınama* satırında `bağlandı: model
   yanıt verdi` veya `başarısız: <neden>` yazar.
4. Birden çok profil varsa en alttan **Varsayılan profil**i seçin; her eylem
   yine de kendi profilini belirtebilir.

![Yerel bir ajan için YZ sayfasının Profiller bloğu: Ajan türü (Claude Code), Komut, Argümanlar, Model, iki Env satırı, Ortam değişkeni ekle ve kaldır, Bağlantıyı sına ve Varsayılan profil](../images/settings-ai-agent.png)

**API anahtarları**, **API anahtarı** satırında `Enter` ile girilir ve sistem
kimlik bilgisi yardımcınızda (Git Credential Manager, libsecret / `pass`,
Keychain) `ai:<profil kimliği>` altında saklanır; `config.toml` içine asla
yazılmaz. Yardımcı yoksa GitPoulpi, anahtarı saklıyormuş gibi davranmak yerine
bunu söyler. *Ayarlar > SSH ve kimlik bilgileri > Tüm Kullanıcı Adlarını ve
Parolaları Unut* bunları kaldırır.

`config.toml` içinde aynısı (geri kalan her şey [yapılandırma
başvurusunda](config-reference.md#ai-yerleşik-yz-varsayılan-olarak-kapalı)):

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # serbest: bir takma ad, bir sarmalayıcı betik, mutlak bir yol
harness = "claude"        # hangi ajan olduğu; boş = `command` değerinden çıkarılır
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # yalnızca bu ajanın süreci; ${env:NAME} bizimkini ödünç alır
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Çalıştırma modu: başsız veya sanal terminal

Yerel bir ajan profili ajanını iki yoldan biriyle çalıştırır; seçimi **Çalıştırma modu** yapar
(Claude Code, OpenAI Codex CLI ve Gemini CLI için gösterilir):

- **Başsız (etkileşimsiz)**, varsayılan: ajanın tek seferlik modu
  (`claude -p`), istem standart girdiye, yanıt standart çıktıdan.
- **Etkileşimli (sanal terminal)**: GitPoulpi ajanın kendi ekranını gizli bir
  terminalde başlatır, isteği bir insan gibi yazar, yanıtı bekler ve ajanı
  kapatır. Bazı ajanlar başsız modlarını etkileşimli modlarından farklı
  faturalandırır veya sınırlar; hangisinin sizin için geçerli olduğu ve
  sağlayıcının koşullarının her ikisi hakkında ne dediği sağlayıcıya bağlıdır,
  bu yüzden sağlayıcınınkileri kontrol edin.

Her mod kendi **Argümanlar**, **Argüman seçenekleri** ve **Model** değerlerini
tutar: mod değiştirmek hiçbir şeyi kaybettirmez ve etkileşimli mod yalnızca
başsız bayrağı olmadan anlamlı olan seçenekleri sunar. İki satır daha her iki
mod için de geçerlidir:

- **Çalışma klasörü**: *Özel klasör*, GitPoulpi'nin sahip olduğu boş bir
  klasördür; ajan hiçbir proje görmez ve sizden ona bir kez güvenmenizi ister.
  *Geçerli depo* ajana projenin kendi bağlamını (örneğin `CLAUDE.md` dosyasını)
  verir ve ajan depo başına bir kez güven ister. *Varsayılan*, etkileşimli modda
  özel klasördür; başsız modda ise eskisi gibi GitPoulpi'nin kendisinin
  çalıştığı klasördür.
- **Zaman aşımı (saniye)**: GitPoulpi vazgeçip ajanı durdurmadan önce ajanın ne
  kadar sürebileceği. Boş bırakılırsa başsızda 120, etkileşimlide 180 saniyedir.

**İlk çalıştırma sorularını siz yanıtlarsınız.** Bir klasörde hiç çalışmamış
bir ajan önce bir şey sorar (bu klasöre güven, oturum aç, tema seç).
GitPoulpi sizin yerinize asla yanıtlamaz: istek bir iletiyle durur ve
**Ajanı aç...** (profilde ya da komut paletinde *YZ: Ajanı Aç*) ajanı bir kez,
görünür bir terminalde, kullanacağı çalışma klasöründe açar. Soruyu yanıtlayın,
ajanı kapatın ve yeniden deneyin; yanıtı ajan hatırlar.

Yanıt, Claude Code için ajanın oturum dökümünden okunur (tam metni, sonradan
silinir); Codex ve Gemini için ise isteğin istediği iki işaret satırı arasındaki
ekrandan okunur. Aynı anda en fazla iki sanal terminal çalıştırması olur ve bir
istek sırasında `Esc` tuşuna basmak ajanı durdurur.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # başsız küme, korunur
run_mode = "terminal"               # headless (varsayılan) | terminal
terminal_args = ["--effort", "low"] # etkileşimli küme
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 veya yok: başsızda 120, terminalde 180
```

## Talimatlar

Modele söylenenlere eklediğiniz metin. Her eylemin yerleşik istemi (yanıtın
biçimini belirler) her zaman önce gelir ve düzenlenemez; ardından, yalnızca boş
olmayanlar olmak üzere genelden özele dört düzey gelir:

1. **Genel talimatlar** (her eylem). Düzenleyicinin *Hızlı ekle* çipleri yaygın
   cümleleri ekler: *<dilde> yanıt ver*, *Yanıtları kısa tut*, *Emir kipi kullan*, *Teknik
   üslup*, *Rahat üslup*.
2. **Çalışma alanı talimatları** (her eylem, [çalışma alanının](workspaces.md#bir-çalışma-alanının-yz-talimatları)
   her deposu): tüm eylemler için ve isteğe bağlı olarak eylem başına. Çalışma
   alanının Düzenle formunda düzenlenir.
3. **Depo talimatları** (her eylem, yalnızca bu depo), *Genelle birleştir* ile:
   **Genele ekle** (onlardan sonra) veya **Geneli değiştir** (onların yerine).
4. **Eylemin talimatları** (*Commit Mesajı Üretimi*, *Değişiklikleri Açıkla*,
   *Stash Mesajı Üretimi*, *Pull Request*, *Commit Composer*, *Çakışma Çözümü*,
   *Kod incelemesi*, *Commit Bölme*), her bloğun düzenleyicisinde kendi çipleri (örn. *Conventional commits biçimi*, *50 karakter
   sınırı*) ve kendi **Profil** açılır listesiyle (`Varsayılan (<ad>)`,
   varsayılan profili izler).

Bir *Özel Talimatlar* satırında `Enter`, çok satırlı bir düzenleyici açar
(`Ctrl+S` kaydeder, `Esc` iptal eder). Metnin üstünde, o bölümün çiplerini
barındıran bir *Hızlı ekle* satırı bulunur: birine tıklamak cümlesini imlecin
olduğu yere ekler. Klavyeyle `Tab` satıra geçer, `Left`/`Right` çip seçer,
`Enter` veya `Space` ekler ve metne döner (`Tab` veya `Esc` eklemeden döner).
Cümlesi talimatlarda zaten bulunan bir çip soluk görünür ve yeniden eklemek
hiçbir şey yapmaz. Metin tam bir düzenleyicidir: tıklama imleci yerleştirir,
sürükleme veya `Shift`+oklar seçer, `Ctrl+A` tümünü seçer, `Ctrl+C` / `Ctrl+X`
seçimi kopyalar ve keser, `Ctrl`+oklar sözcük atlar; `Home` / `End`, `PageUp` /
`PageDown` ve fare tekerleği uzun metinde gezdirir. Sayfanın kendisi her
talimatın yalnızca bir önizlemesini gösterir.

![YZ sayfası: genel talimatlar, ardından profil açılır listesi ve özel talimatlarla Commit Mesajı Üretimi ve Değişiklikleri Açıkla blokları](../images/settings-ai.png)

**Depo başına**: *Ayarlar > Depoya Özel Ayarlar > YZ* (bir depo açıkken)
depo talimatlarını ve her eylem için genel olanları geçersiz kılan bir profil
ile talimatları barındırır. Boş bir alan genel ayarı kullanır (içinde gri
görünür); **Genele sıfırla** bir geçersiz kılmayı kaldırır. Deponun
`.gitpoulpi/repo-state.toml` dosyasına kaydedilir ve CLI tarafından da dikkate
alınır.

![Deponun YZ sayfası: Genelle birleştir açılır listesiyle Depo Talimatları, ardından her eylem için bir profil ve talimatlar](../images/settings-repo-ai.png)

### Düzeyler nasıl birleşir

Gönderilen sıra: yerleşik istem, **genel**, **çalışma alanı**, **depo**, ardından
eylemin kendi talimatları. Bir düzeyin kipi, üstündeki düzeylere ne olacağını
belirler:

- **Ekle** (varsayılan): düzeyin metni, daha genel olanlardan sonra eklenir.
- **Değiştir**: düzeyin metni, daha genel her düzeyin *yerine* kullanılır (genel
  ve, depo değiştiriyorsa, çalışma alanı). Metni olmayan bir düzey hiçbir şeyi
  değiştirmez.

Eylemin kendi talimatları da aynı fikri izler: o eylem için genel olanlar, sonra
o eylem için çalışma alanınınkiler (onlardan sonra eklenir ya da onların yerine
geçer) ve deponun eylem için kendi geçersiz kılması ikisine de baskın gelir.
Kullanılan çalışma alanı, depoyu içeriyorsa penceredeki etkin çalışma alanıdır,
yoksa onu içeren ilk çalışma alanı; komut satırında `GITPOULPI_WORKSPACE`
içindeki, o da yoksa ilk çalışma alanıdır.

Tam olarak ne gönderildiğini, düzey düzey görmek için
`gitpoulpi ai prompt --action <action> --repo .` komutunu çalıştırın (`base`,
`global`, `workspace`, `repository`, `task` ve son `system_prompt`). Hiçbir model
çağırmaz ve YZ kapalıyken de çalışır. Bir çalışma alanının talimatlarını bir
betikten ayarlamak için
`gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode
append|replace] --text ...` (veya `--clear`) kullanın.

### İstem değişkenleri

Her talimat metni (genel, çalışma alanı, depo, eylem başına) `${name}` değişkenleri içerebilir.
Bunlar, istem gönderilmeden hemen önce değerleriyle değiştirilir (kayıtlı metin
`${...}` hâlini korur). Tabloda olmayan bir ad yazıldığı gibi gönderilir ve
düzenleyici bu konuda uyarır; gerçek bir `${` için `$${` yazın.

| Değişken | Şuna açılır | Örnek |
|---|---|---|
| `${uilang}` | Arayüz dili, İngilizce adıyla | `French` |
| `${repo}` | Deponun klasör adı | `gitpoulpi` |
| `${branch}` | Geçerli dal | `main` |
| `${ref}` / `${commit}` | Eylemin ilgilendiği commit'in kısa sha'sı (*Açıkla*), yoksa HEAD | `4c0443d` |
| `${commit_subject}` | O commit'in konu satırı | `feat: add search` |
| `${date}` | Yerel tarih, ISO | `2026-10-03` |
| `${time}` | Yerel saat, `SS:DD` | `14:05` |
| `${datetime}` | Yerel tarih ve saat | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | Varsayılan uzak depo (`origin`, yoksa ilki) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (başka bir barındırıcı) veya `none` | `github` |
| `${files}` | Eylemin diff'indeki dosya sayısı | `3` |
| `${action}` | Çalıştırılan eylem | `commit_message` |

Talimat düzenleyicisinde: `${` yazın, bir açılır pencere eşleşen değişkenleri
listeler (`Up`/`Down` seçer, `Enter` veya `Tab` ekler, `Esc` kapatır); ya da
sol alttaki **Değişkenler** açılır listesini kullanın (`Ctrl+T` veya komut
paletinde *YZ: İstem Değişkeni Ekle*); bu, imlecin olduğu yere ekler. Her
girdinin bir açıklaması vardır; üzerine gelirken veya vurgularken açılır
listede ve düğmelerin üstündeki satırda gösterilir.

## Neler yapabilir

| Eylem | Pencerede | CLI |
|---|---|---|
| **Commit mesajı** | Değişiklikleri stage'leyin, Özet alanının sağındaki `✦` düğmesine, `Ctrl+G i` tuşlarına ya da palette `YZ: Commit Mesajı Oluştur` komutuna basın. Özet ve açıklama düzenlemeniz için doldurulur; hiçbir şey commit edilmez. | `ai commit-message` (yazdırır), `commit --ai` (commit eder) |
| **Değişiklikleri açıkla** | Çalışma değişiklikleri panelinin sağ üstündeki `✦` (`Ctrl+G w`: seçili commit, yoksa çalışma değişiklikleri) ya da bir commit'in menüsündeki *Değişiklikleri açıkla (YZ)*. Bir açılır pencere: oklar kaydırır, `c` kopyalar, `Esc` kapatır. | `ai explain [rev] [--wip]` |
| **Stash mesajı** | `Ctrl+G m` stash istemini açar; içindeyken `Ctrl+G` modele sorar. Paletteki `YZ: Stash Mesajı Oluştur` ikisini de yapar. | `stash push --ai` |
| **Pull request başlığı ve açıklaması** | Pull Request oluştur formundaki *Başlık ve açıklama oluştur* (dalın commit'leri ve diff'i yerelde okunur). | |
| **Merge çakışması çözümü** | Merge çakışması düzenleyicisinde: araç çubuğundaki `✦ Öner` düğmesi, bir çakışmanın sağ tık menüsündeki *✦ Çözüm Öner (YZ)*, `Alt+I` veya palette `YZ: Merge Çakışması Çözümü Öner`. Öneri, Çıktı'nın *geçerli çakışmaya ait* bölgesinin yerine tek bir düzenleme olarak geçer (düzenlendi işaretli, tek geri alma adımı); siz kaydedene kadar hiçbir şey kaydedilmez veya stage'lenmez. | `ai resolve-conflict <path> [--conflict N]` (yazdırır) |
| **Kod incelemesi** | Bir pull veya merge request'in [incelemesinde](review.md#-yz-inceleme-taslağı): `✦ İnceleme taslağı...` düğmesi, `s` veya palette `İnceleme: ✦ İnceleme Taslağı...` (inceleme seçeneklerini açar, `Enter` çalıştırır). Model, isteğin diff'ini, açıklamasını ve mevcut yorumlarını (ve varsayılan olarak projenin `CLAUDE.md` ile `AGENTS.md` dosyalarını: bkz. [İnceleme bağlamı](#i̇nceleme-bağlamı)) okur ve yorumlar önerir; her birini kabul eder, düzenler veya yoksayarsınız. İncelemeyi gönderene kadar hiçbir şey barındırıcıya ulaşmaz. | `hosting review suggest <n>` (JSON yazdırır, hiçbir şey göndermez), `ai review-context` (varsayılanlar) |
| **Commit bölme** | Bir commit'in menüsündeki *Bu commit'i böl (YZ)...* ya da palette `YZ: Seçili Commit'i Böl...`. Bölme düzenleyicisi modelin gruplamasıyla dolu olarak açılır; aşağıya bakın. | `ai split-plan <rev>` (yazdırır) |
| **Geçmiş yeniden yazma betiği** | [Geçmişi yeniden yazma ekranında](history-rewrite.md#yz-ile-betik-yazma): Betik düzenleyicisinin `✦ YZ ile yaz...` düğmesi ya da palette `Yeniden Yaz: YZ ile Betik Yaz...`. Bir iletişim kutusu tam olarak neyin gönderileceğini gösterir; betik işaretlenmiş olarak düzenleyiciye düşer ve sizin yerinize asla çalıştırılmaz veya uygulanmaz. | `ai rewrite-script "<yönerge>" [--sample N]` (yazdırır) |
| **Geçmiş yeniden yazma kuralları** | [Geçmişi yeniden yazma ekranında](history-rewrite.md#yz-ile-kural-oluşturma): `✦ YZ ile kural oluştur...` düğmesi, `Alt+I` ya da palette `Yeniden Yaz: YZ ile Kural Oluştur...`. Çok satırlı bir iletişim kutusu tam olarak neyin gönderileceğini gösterir; kurallar ve ayarlar ekrana yerleşir, sizin yerinize hiçbir şey önizlenmez veya uygulanmaz. | `ai rewrite-rules "<açıklama>" [--context]` (yazdırır) |
| **Commit composer** | WIP satırının menüsündeki *Commit'leri oluştur (YZ)...*, `Ctrl+G C`, palette `YZ: Çalışma Değişikliklerinden Commit'ler Oluştur...` | |

`Esc`, çalışan bir isteği iptal eder (yerel bir ajanın süreci sonlandırılır).

### Merge çakışması çözümü

Model; dosyanın yolunu ve dilini, çakışmanın düzenleyicinin gösterdiği etiketlerle
temel (Base), geçerli (A) ve gelen (B) bloklarını, öncesinde ve sonrasında zaten
birleştirilmiş en çok 20 satır metni ve talimatlarınızı (genel, depo ve Ayarlar >
YZ içindeki *Çakışma Çözümü* talimatları) alır. Yanıtını, yalnızca o tek
çakışmanın çözülmüş metni olarak bir kod bloğunda verir; kod yerine çakışma
işaretleri veya sohbet içeren bir yanıt bir kez geri gönderilir, sonra
reddedilir. 40 KB'ı aşan bir çakışma (`ai.limits.conflict_bytes`, *Ayarlar > YZ >
Sınırlar*) bir uyarının arkasında tutulur; uyarı boyutunu ve sınırı söyler ve *Yine
de gönder* (yalnızca bu istek, gizlilik denetimleri yine uygulanır), *Ayarı aç* ve
*İptal* düğmeleri sunar; sınır GitPoulpi'nin kendi korumasıdır, bu yüzden modeliniz
yer açıyorsa artırın. YZ kapalıyken denetimler gizlidir ve bir öneri hiçbir şeyi kaydetmez, stage'lemez veya
çözmez: Çıktı'yı gözden geçirin, düzenleyin ve her zamanki gibi kaydedin.

![Parıltı düğmesinden sonra commit formu: modelin yazdığı, düzenlemeye hazır bir özet ve açıklama](../images/ai-commit-message.png)

![Çalışma değişikliklerini açıkla penceresi: bir açıklama ve iki kez denetlenecek şeylerin listesi](../images/ai-explain.png)

### Commit composer

Model, çalışma değişikliklerini birkaç commit'e böler (her değişen dosya tam
olarak birinde; geçersiz bir plan bir kez geri gönderilir, sonra reddedilir).
Dosyalarda hâlâ çakışma varken hiçbir şey sorulmaz: bir iletişim kutusu onları
listeler ve çakışma çözme görünümünü açar, çünkü composer çakışmasız bir ağaç
ister. Hiçbir şey commit edilmeden önce planı gözden geçirirsiniz:

![Composer incelemesi: dosyalarıyla birlikte dört önerilen commit ve Sıfırla, İptal ve 4 commit oluştur düğmeleri](../images/composer.png)

| Tuş | Fare | Ne yapar |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | bir satıra tıkla | commit'ler ve dosyalar üzerinde gezin |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | oklar, sürükle ve bırak | bir commit'i taşı; bir dosyayı başka bir commit'e taşı |
| `Right` / `Left`, `*` | satırın oku | bir commit'i, hepsini genişlet / daralt (`ui.composer_files` başlangıcı belirler) |
| `e` / `Enter` | kalem | mesajı düzenle |
| `d` / `Delete` | `Drop` / `geri yükle` | bir commit'i dışarıda bırak (dosyaları commit'lenmemiş kalır) |
| `n` | `+ kendi commit'i` | bir dosyayı kendi commit'ine taşı |
| `r` | `Sıfırla` | modelin planına geri dön |
| `Ctrl+S` / `a` | `N commit oluştur` | sırayla oluştur |
| `Esc` | `İptal` | çık, hiçbir şey yapılmaz |

Oluşturma, her commit'in dosyalarını stage'ler ve sırayla commit eder; çalışma
ağacına asla dokunulmaz. Yalnızca kısmen stage'lenmiş bir dosya bütün olarak
commit edilir. Bir adım başarısız olursa durur ve kaç commit yapıldığını söyler.

### Commit'i bölme

Model commit'in değişikliklerini, [bölme düzenleyicisinin](log-and-history.md#commiti-bölme)
numaraladığı gibi numaralanmış olarak okur ve her biri bir mesajla birkaç commit önerir;
hunk'ları ve tek tek satırları gruplayabilir. Bir değişikliği dışarıda bırakan, bir değişikliği
iki kez atayan ya da var olmayan bir şeyi adlandıran yanıt reddedilir. Her durumda bölme
düzenleyicisi açılır: öneri doldurulmuş olarak ya da model başarısız olduysa nedeni bir notta
yazılı olarak, böylece elle bölebilirsiniz. Siz **Böl**'e (`Ctrl+S`) basana kadar hiçbir şey
bölünmez; gruplama yanlışsa önce değişiklikleri taşıyın, commit'leri yeniden adlandırın ya da
bir tane ekleyin. Profil ve talimatlar, Ayarlar > YZ içindeki *Commit Bölme* olanlardır.

### Bir geçmiş yeniden yazma betiği

Yeniden yazmanın ne yapması gerektiğini anlatırsınız ("geceleri yapılan her commit'i sonraki
sabaha taşı", "eski e-postamı yenisiyle değiştir, yalnızca 2024'ten öncesinde") ve model, Betik
kuralı için başlığı, `rewrite_commit(c, ctx)` işlevi ve korumalı alanın yardımcılarıyla bir
[Lua betiği](history-rewrite.md#lua-betikleri) yazar. Daemon yanıtı derler; derlenmezse hata
modele bir kez geri gönderilir. Geriye kalan düzenleyicide gösterilir: okuyun, deneyin, sonra
önizleyin. Ayrıntılar ve iletişim kutusu [Geçmişi yeniden yazma](history-rewrite.md#yz-ile-betik-yazma)
içindedir. Profil ve talimatlar, Ayarlar > YZ içindeki *Geçmiş yeniden yazma betiği* olanlardır.

### Geçmiş yeniden yazma kuralları

Geçmişin nasıl yeniden yazılması gerektiğini kendi sözlerinizle anlatırsınız ("eski e-postam
a@b.org'u c@d.org ile değiştir", "secrets.txt dosyasını tüm geçmişten sil") ve model, ekranın kendi
diliyle bir kapsam, birkaç ayar ve bir [yeniden yazma kuralları](history-rewrite.md#yz-ile-kural-oluşturma)
listesi önerir. **Yalnızca hazırlar**: yanıt ekranı doldurur, hiçbir şey önizlenmez veya uygulanmaz;
kullanılamayan bir girdi nedeniyle listelenir, geçerli olanlar korunur. Bu yolla asla Betik kuralı
oluşturulmaz. İsterseniz kapsam, dal ve tag adları ve en yeni commit'lerin yazarları da gönderilir
(varsayılan olarak kapalı bir kutu, "ne gönderilir" bölmesinde görünür). Profil ve talimatlar,
Ayarlar > YZ içindeki *Geçmiş yeniden yazma kuralları* olanlardır.

## Anahtarın gittiği yer

Bir API anahtarı (OpenAI, Anthropic, OpenRouter, OpenAI uyumlu bir sunucu) saklandığı adrese bağlıdır: profilin temel URL'sinin kökeni, yalnızca kullanıcıya ait özel veri klasöründeki `trust.toml` dosyasına kaydedilir. Temel URL sonradan başka bir yeri gösterirse, istek hiçbir şey gönderilmeden reddedilir ve anahtar olduğu yerde kalır. Anahtarı bilerek yeni adrese taşımak için `gitpoulpi ai trust <profile> --accept-origin <url>` komutunu çalıştırın (ya da uygulamada kabul edin).

- Düz `http://` bu makinede serbesttir (`localhost` üzerinde Ollama, LM Studio); başka her yerde `gitpoulpi ai trust <profile> --allow-insecure-http on` gerekir.
- Sertifikası bilinen bir otoriteye zincirlenmeyen, kendi barındırılan bir sunucu yalnızca `--accept-certificate <sha256>` ile sabitlediğiniz anahtar parmak izi için kabul edilir; TLS el sıkışması yine de sunucunun o anahtarı taşıdığını kanıtlar.
- Bu denetim var olmadan önce zaten anahtarı olan bir profil, ilk kullanıldığında mevcut temel URL'sinden sessizce kaydedilir.
- Yerel bir ajanın (`claude -p` ve benzerleri) anahtarı ve HTTP'si yoktur, bu yüzden bağlanacak bir şey yoktur.

## İnceleme bağlamı

Bir kod inceleme taslağı (`✦` veya `hosting review suggest`) isteğin diff'ini, başlığını, açıklamasını ve
mevcut yorumlarını okur. Diff'ten önce yerleştirilen bir **Proje kuralları** bloğunda projenin kendi
kuralları da verilebilir ve yerel bir ajan depo içinde çalışabilir. Varsayılanlar şunlardır:

| Ayar | Varsayılan | Anlamı |
|---|---|---|
| `run_in_repository` | `profile` | `profile`: YZ profilinin *Çalışma klasörü* belirler. `yes`: profil ne derse desin, yerel bir ajan inceleme için depoda çalışır. `no`: orada değil (*Depo* olarak ayarlanmış bir profil bunun yerine ayrılmış klasörü kullanır). HTTP profillerinin çalışma klasörü olmadığından onlar için bir şey değişmez. |
| `include_claude_md` | açık | Modele deponun `CLAUDE.md` dosyasını ver. |
| `include_agents_md` | açık | Modele deponun `AGENTS.md` dosyasını ver. |
| `extra_file` | yok | Bir dosya daha: depo köküne göre bir yol, mutlak bir yol veya bir bağlantı. |

Talimatlar gibi katmanlıdır ve en özel olan alan alan kazanır: önce genel `[ai.review]` tablosu, sonra
deponun kendi değerleri (ayarlanmamış bir alan devralır), sonra tek bir çalıştırmanın seçimleri. Klonlanmamış
bir depo kendi değerlerini `host/project` ile `remote-reviews.toml` içinde, inceleme taslaklarının yanında
tutar.

Bir depo genel ek dosyayı **kapatabilir** de: kendi `no_extra_file = true` değeri (*Depoya özel ayarlar > YZ*
içindeki **Ek dosya yok** onay kutusu, ✦ İnceleme seçenekleri penceresi ve `ai review-context` komutunun
`--no-extra-file` bayrağı) `[ai.review]` ne belirtirse belirtsin "bu depo için ek dosya yok" demektir. **Boş**
bir **Ek dosya** alanı aynı şey değildir: genel dosyayı devralır. Deponun bir dosyası ile bu anahtar birbirini
dışlar (biri ayarlanınca öteki temizlenir); elle düzenlenmiş bir dosyada ikisi birden varsa anahtar kazanır.
Bir deponun eski `extra_file = ""` değeri hâlâ "hiçbiri" anlamına gelir.

Dosyaların geldiği yer:

- `CLAUDE.md`, `AGENTS.md` ve göreli bir ek dosya, depo **kökünden**, isteğin hedeflediği dalda okunur; asla
  çalışma ağacından değil (o başka bir dalda olabilir veya yerel düzenlemeler içerebilir). Klon varsa
  isteğin temel commit'inden, yoksa sunucunun API'sinden.
- Mutlak bir yol bu makineden okunur.
- Bir kod barındırıcısındaki dosyaya bağlantı (GitHub `.../blob/<dal>/<yol>`, GitLab `.../-/blob/<dal>/<yol>`,
  Gitea ve Forgejo `.../src/branch/<dal>/<yol>` ya da ham bir bağlantı), sunucusu eşleşen entegrasyon profili
  üzerinden okunur; böylece özel bir depo, zaten verdiğiniz belirteçle çalışır. Hiçbir profilin eşleşmediği bir
  bağlantı ya da başka herhangi bir adres anonim olarak getirilir. Eğik çizgi içeren bir dal adı bağlantıdan
  okunamaz: bir yol kullanın veya eğik çizgisiz bir daldaki bağlantıyı.
- Eksik bir `CLAUDE.md` ya da `AGENTS.md` sessizce atlanır; eksik bir ek dosya bildirilir (CLI'nin
  JSON'unda `context.problems`).
- Her dosya **8 KB**'ta, dosyalar birlikte **16 KB**'ta kesilir; modelin okuduğu metinde görünür bir
  `[CLAUDE.md cut: showing ... of ... bytes]` notu bulunur.

Yerel bir ajan depoda çalıştığında istem ona, checkout edilmiş dosyaların isteğin head'iyle
uyuşmayabileceğini ve diff'in esas olduğunu söyler.

İnceleme görünümünde `s`, bu değerlerin doldurulduğu **✦ İnceleme seçenekleri** iletişim kutusunu açar: deponunkiler
genel olanların üstünde. Yalnızca bu çalıştırma için değiştirin ya da kalıcı olması için *Bu deponun varsayılanları
olarak kaydet* kutusunu işaretleyin (bkz. [İnceleme seçenekleri](review.md#i̇nceleme-seçenekleri)). Varsayılanlar
ayrıca *Ayarlar > YZ* içinde (*Kod incelemesi* bloğu, dört satır) ve depo başına *Depoya özel ayarlar > YZ* içinde
düzenlenebilir; orada her satır *genel değeri kullan* seçeneğine sahiptir ve bir *Genele sıfırla* düğmesi deponun
kendi değerlerini siler; Ayarlar araması bunları bulur (`AGENTS.md'yi ekle`, `Ek dosya`). Modele verilemeyen bir şey
(eksik bir ek dosya) ya da sığması için kısaltılan bir şey taslak kartında gösterilir.

Komut satırından:

```sh
gitpoulpi ai review-context                      # genel varsayılanlar ve bir incelemenin aldığı
gitpoulpi ai review-context --agents-md off      # genel bir varsayılanı kaydet
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # yeniden genel varsayılanları izle
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` model gerektirmez ve YZ kapalıyken de çalışır. `ai prompt --action code_review`, etkin
bağlamı `review_context` olarak yazdırır. Aynı dört ayar `config.toml` içinde `ai.review.*` olarak bulunur
([başvuru](config-reference.md#ai-yerleşik-yz-varsayılan-olarak-kapalı)).

## Girdi sınırları

GitPoulpi'nin bir isteğe koyduğu her girdinin bir boyut sınırı vardır: `[ai.limits]`
altında bir ayar (*Ayarlar > YZ > Sınırlar*). Sınırlar GitPoulpi'nin kendi korumasıdır
(bir modelin bağlam penceresi modele göre değişir): modeliniz yer açıyorsa artırın,
istekleri küçük tutmak için azaltın.

| Ayar (`ai.limits.`) | Varsayılan | Neyi sınırlar |
|---|---|---|
| `conflict_bytes` | 40000 | Bir merge çakışmasının temel, geçerli ve gelen blokları |
| `diff_bytes` | 60000 | Bir commit mesajı, bir açıklama, bir stash adı, bir pull request açıklaması, bir commit planı veya bir bölme isteğinin diff'i; bir kod incelemesinin tüm diff'i |
| `review_file_bytes` | 12000 | Bir kod incelemesinde tek bir dosyanın diff'i |
| `review_text_bytes` | 4000 | İncelenen pull request'in açıklaması |
| `review_threads_bytes` | 6000 | Zaten var olan inceleme yorumları |
| `review_context_file_bytes` | 8192 | Bir incelemenin tek bir proje bağlam dosyası (`CLAUDE.md`, `AGENTS.md`, ek dosya) |
| `review_context_bytes` | 16384 | Bir incelemenin tüm bağlam dosyaları birlikte |

**Hiçbir şey sessizce kesilmez.** Bir girdi sınırını aştığında istek bekler ve
*YZ sınırı aşıldı* neyin aştığını listeler: hangi parça (diff, bir dosya, açıklama,
bir bağlam dosyası), boyutu, sınır ve ayar. Bir inceleme sınırı aşan her parçayı tek
bir iletişim kutusunda listeler. Düğmeler, her biri tuşuyla:

- *Tümünü gönder* (`T`): yalnızca bu istek, hiçbir şey kesilmez. Yine gizlilik
  denetimlerinden (gizli bilgiler değiştirilir, hassas dosyalar dışarıda bırakılır) ve
  açıksa sıradaki *Neler gönderiliyor* panelinden geçer.
- *Kısaltarak gönder* (`K`): sınırı aşan kesilir, eskiden olduğu gibi; iletişim
  kutusunun her satırı neyin dışarıda kaldığını söyler (diff'in sonu, son hunk'lar,
  sığmayan dosyalar).
- *Ayarı aç* (`A`): isteği iptal eder ve *Ayarlar > YZ > Sınırlar* sayfasını listelenen
  ilk sınırda açar. Odak buradan başlar: Enter hiçbir zaman göndermez.
- *İptal* (`P`, `Esc`, çarpı ya da dışına tıklama): hiçbir şey göndermez.

Çakışma istisnadır: asla kesilmez. *Yine de gönder* onu bir kez bütün olarak gönderir
(`G`).

Hiçbir şey bir YZ isteğini kendiliğinden başlatmaz, bu yüzden soracak bir arka plan
çalıştırması yoktur: her istek sizin başlattığınız bir istektir. Soramayan bir çalıştırma
(CLI ve MCP) eskisi gibi kısaltır ve bunu söyler; aşağıya bakın.

**Komut satırında** hiçbir şey bir iletişim kutusunu yanıtlayamaz, bu yüzden sınırı aşan
bir girdi eskisi gibi kısaltılır ve yanıt neyin kesildiğini söyler:
`truncated: [{part, detail?, bytes, limit, setting}]` (yalnızca bir şey kesildiğinde
bulunur; `part` şunlardan biridir: `diff`, `split_hunks`, `review_diff`, `review_file`,
`review_text`, `review_threads`, `context_file`, `context_total`). `--send-all`
(takma ad `--no-truncate`) bunun yerine her şeyi bütün olarak gönderir: `ai` üzerinde
(tüm komutları, sınırını aşan bir çakışmanın aksi halde reddedildiği `ai resolve-conflict`
dahil), `commit --ai`, `stash push --ai` ve `hosting review suggest` üzerinde. MCP
araçlarında aynı `send_all` argümanı ve aynı `truncated` alanı vardır.

## Gizlilik

- Bir düğmeye basana veya bir komut çalıştırana kadar hiçbir şey gönderilmez.
  Arka planda YZ yok, telemetri yok, GitPoulpi hesabı yok.
- Gönderilen: istem (önce yerleşik olan, sonra sizin talimatlarınız) ve
  sorduğunuz diff; bir pull request için ayrıca dalın commit konuları. 60 KB'ı
  (`ai.limits.diff_bytes`) aşan bir diff asla sessizce kesilmez: bkz.
  [Girdi sınırları](#girdi-sınırları).
- Bir geçmiş yeniden yazma betiği isteği, yönergenizi ve sabit betik sözleşmesini (betik
  korumalı alanının API'si) gönderir. En fazla 20 commit'in yazarı, tarihi ve konusu
  **yalnızca** iletişim kutusunda *Örnek gönder* kutusunu işaretlerseniz gider (varsayılan
  kapalı; CLI için `--sample N` gerekir). Dosya içerikleri ve gizli değerler asla gönderilmez.
- Yalnızca o eylem için seçilen profile gider: o API sunucusuna ya da yerel
  ajan komutunuzun standart girdisine (o da ajanın yaptığı her şeyi yapar).
- API anahtarlarını `gitpoulpi` sürecinin kendisi kullanır; daemon'a, pencerenin
  diğer istemcilerine veya bir MCP istemcisine asla ulaşmaz.
- Üretilen metin kendiliğinden asla commit edilmez, push edilmez veya
  uygulanmaz.

### Bir istekten neler temizlenir

Her istek, hangi eylem gönderirse göndersin ve hangi sağlayıcı (barındırılan bir
API veya yerel bir ajan) alırsa alsın, önce aynı temizleme adımından geçer:

- **Gizli bilgiler sansürlenir** (`[ai.privacy] redact_secrets`, varsayılan olarak
  açık). Bilinen önekli bir token, bir özel anahtar bloğu, bir JWT, bir URL içindeki
  parola ve değeri rastgele görünen bir `password = value` satırı `[REDACTED:<rule>]`
  ile değiştirilir. Kapsamlı değildir: yaygın biçimleri bulur (`rewrite scan-secrets`
  ile aynı tarayıcı).
- **Hassas dosyalar dışarıda bırakılır** (`[ai.privacy] exclude_sensitive_files`,
  varsayılan olarak açık). `[privacy] sensitive_patterns` ile eşleşen bir dosyanın
  diff'i (`.example`, `.sample` ve `.template` dosyaları dışında `.env` ve `.env.*`,
  `*.pem`, `*.key`, SSH özel anahtarları, `*.p12`, `*.pfx`, `*.jks`, `*.keystore`,
  `*.crt`, `*.cer`, `*.der`, `*.kdbx`) isteğin parçası olmaz. Böyle bir dosyayı
  adlandıran bölünmüş bir parça veya besteci dosya listesi aynı şekilde kesilir; bu
  yüzden onun için bir plan eksik diye reddedilebilir: o dosyayı kendiniz stage'leyin
  veya bölün.

İkisinden birini kapatmak bir seçimdir ve Ayarlar > YZ, kapalı kaldığı sürece altında
kırmızı bir satır gösterir.

**Ne gönderildiği, gönderilmeden önce.** Bir istek çıkmadan önce TUI **Gönderilenler**
iletişim kutusunu gösterir: talimatlar ve temizlemeden sonraki istek (geçmiş yeniden yazma
iletişim kutularıyla aynı bölme, kaydırılabilir, `Alt+W` gösterir veya gizler), kaç gizli
bilginin değiştirildiği ve hangi dosyaların dışarıda bırakıldığı (bir kural ve `ghp_****abcd`
gibi maskeli bir önizleme, asla bir değer), bir **<eylem> için bir daha gösterme** kutusu ve
**Gönder** / **İptal** (`Esc`, kapatma çarpısı ve dışına bir tıklama iptal eder; Gönder'e
kadar hiçbir şey çıkmaz). Kutu `[ai.sent_preview] <action> = false` yazar; Ayarlar > YZ >
*Göndermeden önce gönderileni göster...* her eylem için bir anahtar ve **Hepsini yeniden
göster** düğmesi sunar. İki geçmiş yeniden yazma iletişim kutusu isteklerini zaten gösterir,
bu yüzden anahtarları yoktur.

**Komut satırında**, her `ai ...` yanıtı (ve `hosting review suggest`)
`privacy: {redactions: N, excluded_files: [...]}` taşır ve `--print-sent`, temizlenmiş
sistem ve kullanıcı metnini `{sent: [{system, user}], privacy, dry_run: true}` olarak,
modeli çağırmadan yazdırır (`ai.enabled = true` gerekir, anahtar ve ağ gerekmez).

**Bir sınır.** Çalışma klasörü depo olan bir yerel ajan (`workdir = repository` ya da bir
inceleme için *Ajanı depoda çalıştır*) dosyaları kendisi okuyabilir: temizleme,
GitPoulpi'nin ona gönderdiğini kapsar, kendi başına açtıklarını değil. Onu depodan uzak
tutmak için `dedicated` çalışma klasörünü kullanın.

## Sorun giderme

| Belirti | Çözüm |
|---|---|
| Hiçbir yerde `✦` yok | YZ kapalı: *Ayarlar > YZ > YZ'yi etkinleştir*. |
| `✦` gri | Hiçbir şey stage'lenmemiş (commit mesajı). |
| *Bağlantıyı sına*: komut bulunamadı | Yerel ajanın **Komut**u, GitPoulpi'nin gördüğü `PATH` üzerinde değil: GitPoulpi'yi `claude --version` komutunun çalıştığı yerde başlatın ya da tam yolu verin. |
| Yerel bir ajan donuyor veya etkileşimli bir ekran gösteriyor | *Başsız* çalıştırma modunda, **Argümanlar**'a başsız bayrağını (Claude Code için `-p`) ekleyin ya da başsız çağrıyı geri yüklemek için **Ajan türü**nü yeniden seçin. Etkileşimli ekran, *Etkileşimli (sanal terminal)* çalıştırma modunun işidir. |
| "Use "Open the agent"..." | Ajan bir ilk çalıştırma sorusu sordu (klasöre güven, oturum aç). Profildeki **Ajanı aç...** düğmesini kullanın, yanıtlayın, ajanı kapatın ve yeniden deneyin. |
| Bir sanal terminal isteği zaman aşımına uğruyor | İleti, ajanın ekranının son satırlarıyla biter: bir izin istemi, bir oturum açma ya da bir güncelleme sorusu orada görünür. **Ajanı aç...** ile yanıtlayın ya da **Zaman aşımı**nı artırın. |
| `başarısız:` 401 | Yanlış anahtar ya da başka bir sağlayıcının anahtarı: yeniden girin. |
| `başarısız:` 404 | **Model** orada yok: varsayılanı kullanmak için temizleyin. |
| `başarısız:` bağlantı hatası (Ollama, LM Studio) | Sunucu çalışmıyor ya da **Temel URL**'de port yanlış veya `/v1` eksik. |
| Anahtar, yazıldıktan sonra `(ayarlanmadı)` gösteriyor | Kimlik bilgisi yardımcısı yok: birini yapılandırın (`git config --global credential.helper ...`) ve yeniden girin. |
| *YZ sınırı aşıldı* | İsteğin bir girdisi `[ai.limits]` sınırlarından birini aşıyor: *Tümünü gönder*, *Kısaltarak gönder*, *Ayarı aç* veya *İptal* seçin (bkz. [Girdi sınırları](#girdi-sınırları)). |
| "Çakışma sınırdan büyük" | Çakışma `ai.limits.conflict_bytes` değerini aşıyor (varsayılan 40000). *Yine de gönder* onu bir kez gönderir; *Ayarı aç* Ayarlar > YZ > Sınırlar'daki sınıra götürür. |
| Yanlış dil veya üslup | Talimat ekleyin; düzenleyicinin Hızlı ekle çipleri bunu hızlandırır. |
| Composer planın geçersiz olduğunu söylüyor | Model bir dosyayı atladı veya yineledi: yeniden deneyin ya da daha güçlü bir model kullanın. |

## Eski yapılandırma dosyaları

`[[ai.presets]]` ve `[ai.defaults]` içeren eski bir `config.toml` hâlâ yüklenir:

- Her eylemin varsayılan ön ayarı kendi `[ai.actions.<action>]` bölümü olur
  (`system_prompt`, `instructions` olur).
- Her eski ön ayar `[[ai.legacy_presets]]` içinde tutulur. Ayarlar > YZ bunları
  *Eski ön ayarlar* altında, *Arşivi sil* ile birlikte gösterir.
- Bir deponun eski `[preferences.ai_presets]` bölümü, depo geçersiz kılması olur.

Yerleşik istem artık her zaman önce gelir. Eski bir istem yerleşik bir istemi
kopyalamışsa, kopyalanan kısmı kaldırın ve yalnızca eklemek istediğinizi bırakın.
