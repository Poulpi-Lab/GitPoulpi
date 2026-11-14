[English](../mcp-and-agents.md) · [Čeština](../cs/mcp-and-agents.md) · [Deutsch](../de/mcp-and-agents.md) · [Español](../es/mcp-and-agents.md) · [Français](../fr/mcp-and-agents.md) · [Italiano](../it/mcp-and-agents.md) · [日本語](../ja/mcp-and-agents.md) · [한국어](../ko/mcp-and-agents.md) · [Nederlands](../nl/mcp-and-agents.md) · [Polski](../pl/mcp-and-agents.md) · [Português (Brasil)](../pt-BR/mcp-and-agents.md) · [Português (Portugal)](../pt-PT/mcp-and-agents.md) · [Русский](../ru/mcp-and-agents.md) · [Svenska](../sv/mcp-and-agents.md) · **Türkçe** · [Українська](../uk/mcp-and-agents.md) · [简体中文](../zh-CN/mcp-and-agents.md)

# MCP ve ajanlar

Bir YZ ajanı (Claude Code, Cursor, Codex, ...) GitPoulpi'yi **MCP sunucusu** veya
**CLI**'ı üzerinden yönetebilir. İkisi de pencereyle aynı arka plan daemon'ı
üzerinden geçer; bu yüzden ajanın yaptığı, açık GitPoulpi'nizde canlı görünür ve
ikisi de her komut için tamamen aynı kodu çalıştırır.

Bu sayfa *harici* ajanlarla ilgilidir. GitPoulpi'nin kendi YZ düğmeleri (commit
mesajları, açıklamalar, commit composer) ayrı yapılandırılır: bkz. [YZ
desteği](ai.md).

## Hızlı kurulum

1. `gitpoulpi` ve `gitpoulpi-mcp` dosyalarını `PATH` üzerine koyun ([Kurulum](installation.md)).
2. Sunucuyu ajanınıza kaydedin. stdio konuşur: başlatılacak bir şey yok, port yok.
3. İsteğe bağlı olarak ajanın neler yapabileceğini `config.toml` dosyasının
   `[mcp]` tablosunda seçin (aşağıda). Varsayılanlar, deponun okunmasına izin
   verir (siz `allow_mutations = true` yapana kadar değişiklikler reddedilir),
   size bir şey göstermeyi yalnızca siz onayladıktan sonra yapar ve iletişim
   kutularını asla önceden doldurmaz.

**Claude Code**

```
claude mcp add gitpoulpi -- gitpoulpi mcp serve --stdio
```

`--scope project` bunu bunun yerine deponun `.mcp.json` dosyasına yazar, böylece
projeyi açan herkes alır. Aynı dosya elle, isteğe bağlı eklerle:

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "command": "gitpoulpi",
      "args": ["mcp", "serve", "--stdio", "--ui-control", "auto"],
      "env": { "GITPOULPI_CONFIG_DIR": "/home/me/.config/gitpoulpi-agent" }
    }
  }
}
```

- `--ui-control auto|ask|off`, yalnızca bu ajan için `[mcp] ui_control` değerini
  geçersiz kılar.
- `GITPOULPI_CONFIG_DIR` ajana kendi `config.toml` dosyasını verir (kendi `[mcp]`
  sınırları, ayarlarınıza erişim yok). Sizinkini paylaşmak için bırakın.
- `gitpoulpi --repo <path> mcp serve --stdio` sunucuyu tek bir depoya sabitler;
  aksi hâlde bir araç, sunucunun çalışma klasörünü çevreleyen depo üzerinde ya da
  çağrının `repo` argümanındaki depo üzerinde çalışır.

**Diğer istemciler**: stdio sunucusu başlatan herhangi bir MCP istemcisi.

| İstemci | Nerede | Ne |
|---|---|---|
| Cursor, Windsurf, Claude Desktop, VS Code | istemcinin MCP JSON dosyası | yukarıdaki `mcpServers` bloğu (VS Code anahtarı `servers` olarak adlandırır) |
| OpenAI Codex CLI | `~/.codex/config.toml` | `[mcp_servers.gitpoulpi]`, `command = "gitpoulpi"`, `args = ["mcp", "serve", "--stdio"]` |
| Başka herhangi biri | kendi belgeleri | `gitpoulpi-mcp` komutu (bağımsız ikili, aynı sunucu, argüman gerekmez) |

**HTTP üzerinden** (bir URL'ye bağlanan istemciler):

```
gitpoulpi-mcp --http                         # [mcp] bind, varsayılan http://127.0.0.1:8808/mcp
gitpoulpi mcp serve --http 127.0.0.1:9000
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp
```

Durumsuz JSON, aynı araçlar, aynı `[mcp]` kapıları. Her istek bearer token'ı
gerektirir (`[mcp] http_auth`, varsayılan olarak açık): onu `gitpoulpi mcp token
--show` ile yazdırın (veya `--copy`, `--rotate`; token, özel veri klasörünün
altındaki yalnızca kullanıcıya ait bir dosyada durur) ve istemciye bir başlık
olarak verin:

```
claude mcp add --transport http gitpoulpi http://127.0.0.1:8808/mcp \
  --header "Authorization: Bearer <token>"
```

```json
{
  "mcpServers": {
    "gitpoulpi": {
      "type": "http",
      "url": "http://127.0.0.1:8808/mcp",
      "headers": { "Authorization": "Bearer <token>" }
    }
  }
}
```

Eksik veya yanlış bir token `WWW-Authenticate: Bearer` ile `401` alır. Değiştirilen
bir token sunucuyu yeniden başlatmadan hemen geçerli olur. `http_auth` değerini
kapatmak, her yerel hesaba güvendiğiniz bir loopback uç noktası içindir; sunucu bu
durumda stderr'e uyarı yazar.

### Uç noktaya başka bir makineden erişmek

En güvenli kurulum hiç uzak dinleyici gerektirmez: loopback adresini koruyun ve
portu ajanı çalıştıran makineden SSH ile iletin, `ssh -L
8808:127.0.0.1:8808 host`. Bu durumda `--allow-remote` devreye girmez.

`--allow-remote` (loopback olmayan bir adres), kimlik doğrulama bir token ile açık
**ve** trafik şifreli olmadıkça reddedilir. Ret, nelerin eksik olduğunu adlandırır.
Şifrelenmenin iki yolu vardır:

- **Sunucuda TLS** (`[mcp] tls = "self_signed"` veya `"files"`). Sunucu TLS'yi
  kendisi sonlandırır: anahtarı tutar, el sıkışmayı yürütür ve HTTP katmanına yalnızca
  tamamlanmış bir el sıkışma ulaşır; düz HTTP kullanan bir istemci el sıkışmada
  başarısız olur ve yanında düz bir dinleyici yoktur. Yani şifreleme, bir varsayım
  değil, sunucunun her bağlantıda kurduğu bir olgudur. `self_signed` ile ilk başlatma
  özel veri klasörünün altında bir çift üretir ve SHA-256 parmak izini yazdırır
  (Ayarlarda da vardır); bir istemci sunucuya o parmak izini ya da sertifika dosyasını
  sabitleyerek güvenir. `files` ile `tls_cert` ve `tls_key` değerlerini PEM
  dosyalarınıza ayarlayın; istemciler sertifikayı CA'sıyla doğrular. İstemciler bearer
  token ile kimlik doğrular; istemci sertifikaları kullanılmaz.
- **Bir tünel veya ters vekil sunucu** (`[mcp] behind_proxy = true`). Bir tünel ajanı
  veya vekil sunucu, herhangi bir yerel veya LAN istemcisi gibi bağlanır ve başlıkları
  taklit edilebilir; bu yüzden sunucu içeriden onun orada olup olmadığını söyleyemez.
  Bu yüzden bir algılama yerine açık bir onaydır (Ayarlarda kırmızı bir uyarıyla
  gösterilir). Mümkün olduğunda tüneli aynı makinede çalıştırın ve loopback adresini
  koruyun. Bir vekilin arkasındaki loopback olmayan bir adreste, vekil ile sunucu
  arasındaki bağlantı o ağda düz metindir.

**MCP olmadan**, kabuğu olan bir ajan CLI'ı kullanır: `gitpoulpi --json <command>`
([CLI başvurusuna](cli-reference.md) bakın). Ona, ham `git` yerine GitPoulpi'nin ne
zaman ve nasıl kullanılacağını söyleyen hazır
[`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md) becerisini verin.

## İzinler

| Anahtar | Varsayılan | Etki |
|---|---|---|
| `[mcp] enabled` | `true` | `false`: `gitpoulpi mcp serve` / `gitpoulpi-mcp` başlamayı reddeder (Ayarlar > Ajanlar ve MCP > *MCP sunucusu*). Sunucu başlarken okunur: çalışan bir sunucu ayarlarını korur. |
| `[mcp] bind` | `"127.0.0.1:8808"` | Hiçbiri verilmediğinde `--http`'nin kullandığı adres (`--allow-remote` olmadıkça yalnızca loopback; bu kimlik doğrulama ve şifreleme ister, bkz. yukarısı). Ayarlar > Ajanlar ve MCP > *MCP sunucu portu* portu düzenler ve ana makine kısmını korur. |
| `[mcp] http_auth` | `true` | `--http`, `Authorization: Bearer <token>` ister (`gitpoulpi mcp token`). Ayarlar > Ajanlar ve MCP > *HTTP kimlik doğrulaması*. |
| `[mcp] tls`, `tls_cert`, `tls_key` | `"off"` | `off`, `self_signed` veya `files` (PEM yollarınız): sunucu TLS'yi kendisi sonlandırır. Ayarlar > Ajanlar ve MCP > *TLS*. |
| `[mcp] behind_proxy` | `false` | Bir tünelin veya ters vekil sunucunun trafiği şifrelediğinin sorumluluğunu üstlenirsiniz; bu, `--allow-remote` komutunun TLS olmadan çalışmasına izin verir. |
| `[mcp] allow_mutations` | `false` | Kapalı: bir şeyi değiştiren her araç reddedilir; okumalar, `ui_*` ve öneriler yine çalışır. `true`, ajanın commit, push, reset... yapmasına izin verir. |
| `[mcp] mutation_allowlist` | `[]` | Boş değil: yalnızca bu araçlar bir şey değiştirebilir, örn. `["stage", "unstage", "commit"]`. |
| `[mcp] allow_ui` | `true` | `false`: ajan pencerenizi hareket ettiremez (`ui_state` / `ui_viewers` yine yanıt verir). |
| `[mcp] ui_control` | `"ask"` | `ask`: pencerenizde onayladığınız bir çip; `auto`: meşgul olmadığınız sürece uygulanır; `off`: reddedilir. Ajan başına: `--ui-control`. |
| `[mcp] allow_ui_propose` | `false` | Ajanın, sizin onaylayacağınız önceden doldurulmuş iletişim kutuları açmasına izin verir (Ayarlar > Ajanlar ve MCP > *YZ ajanlarının onaylayacağım değişiklikler önermesine izin ver*). |
| `[mcp] allow_sensitive_staging` | `false` | Yalnızca `config.toml` içinde. Kapalıyken `stage` aracı `allow_sensitive` sunmaz ve hassas bir dosya (bir özel anahtar, bir `.env`, tanınan bir token) ekleyecek bir stage `{error: "sensitive_files", files}` ile reddedilir: ajan size soracaktır. `true`, onun `allow_sensitive` geçmesine izin verir. Bkz. [Stage'leme ve commit'leme](staging-and-committing.md#hassas-dosyalar). |
| `[mcp] allow_history_purge` | `false` | Yalnızca `config.toml` içinde. `true`, bir geçmiş yeniden yazmanın eski geçmişini (ve onun geri almasını) kalıcı olarak silen `rewrite_purge` aracını listeler. Kapalıyken ajan onu görmez. Yine de `allow_mutations` gerekir. |
| `[ui] allow_agent_view_control` | `true` | Pencerenizin ana anahtarı (Ayarlar > Ajanlar ve MCP > *YZ ajanlarının görünümü yönetmesine izin ver*). Kapalı: hiçbir ajan pencerenize ulaşamaz. |
| `[ui] launch` | `"off"` | Hiçbir pencere açık değilken bir ajanın GitPoulpi penceresi açıp açamayacağı ([aşağıya](#hiçbir-pencere-açık-olmadığında-veya-başka-bir-oturumda-olduğunda) bakın). |

Pratikte bunun anlamı:

- **Kutudan çıktığı gibi ajan salt okunurdur**: okuyabilir, size pencerenizde
  bir şeyler gösterebilir (`ui_*`) ve izin verirseniz sizin onaylayacağınız
  değişiklikler önerebilir. Deponun değiştirilmesine (commit, push, reset...)
  izin vermek için `allow_mutations = true` yapın; dar bir izin için onu `true`
  bırakıp bir `mutation_allowlist` kullanın. Her araç MCP ek açıklamaları taşır
  (`readOnlyHint`, `destructiveHint`, `openWorldHint`), bu yüzden Claude Code gibi
  bir istemci riskli olanlardan önce yine size sorar.
- **Çakışmalar**: `conflicts_check` (bir merge'den önce yapılan tahmin), `conflicts_list`,
  `conflicts_show` ve `conflicts_state` okumadır; `conflicts_resolve` (tutulmayan tarafı atar, `--delete` dosyayı
  kaldırır) ve `conflicts_abort` (tüm merge/rebase/cherry-pick'i bırakır)
  yıkıcıdır; `conflicts_unresolve` düz bir yazmadır. Reddedilen bir çağrı
  (`stale`, `not_conflicted`, `pick_incomplete`...) başarısız olmak yerine çıkış
  kodu 2 ile `{"refused": {...}}` yanıtlar.
- **Takılı durumlar**: `conflicts_am_continue` bir yazmadır, `conflicts_am_skip`,
  `conflicts_am_abort` ve `index_lock_remove` yıkıcıdır, `index_lock_show` bir
  okumadır.
- **Geçmişi yeniden yazma**: `rewrite_tools`, `rewrite_preview`, `rewrite_rows`,
  `rewrite_analyze` (boyut analizi), `rewrite_scan_secrets` (gizlenmiş önizlemeli bulgular, asla bir değer değil),
  `rewrite_script_check`, `rewrite_script_test` ve `rewrite_discard` okumadır (önizleme
  hiçbir şeyi değiştirmez; yalnızca planı daemon'da ve git dizininde diskte tutar, bu
  yüzden `rewrite_apply` yeniden başlatmadan sonra da çalışır). `rewrite_apply` ve
  `rewrite_run` yıkıcıdır: birçok ref'i yeniden yazılmış commit'lere tek bir geri alma
  girdisi olarak taşırlar (`undo` onları geri koyar) ve asla push etmezler. Kural setini
  `recipe_json` (ya da yol olarak `recipe`) ile, gizli değerleri `secrets_file` ile
  verin, asla satır içinde değil. Kabuk, Python ve harici Lua betikleri (`script_language`)
  korumalı alanda değildir ve **bir ajan onlara güvenemez**: `rewrite_script_trust` aracı
  ve `trust` argümanı yoktur; bu yüzden böyle bir betik yalnızca bir kişi tam o metin üzerinde
  `gitpoulpi rewrite script trust DOSYA` çalıştırdıktan sonra çalışır (değişen bir betik yeniden
  reddedilir, `script_not_trusted` ve özet `refused.script` içinde) ve yalnızca
  `rewrite.external_scripts` dilini açıksa (`scripts_disabled`).
  Bir ret `{"refused": {"reason", "message", "detail"}}`
  yanıtlar; push edilmiş ref'leri taşıyan bir plan, çağrı `yes` vermedikçe
  `{"needs_confirmation": ...}` (çıkış kodu 3) yanıtlar - önce kullanıcıya gösterin.
- **Bir ajan tarafından asla erişilemez**: bir token saklamak (`hosting login`),
  bir SSH anahtarı oluşturmak ve yüklemek, `config set` (kendi sınırlarını
  yeniden yazabilirdi) ve daemon yaşam döngüsü. Bir token ajana asla gösterilmez.
- **Hesap geneli listeler**: `hosting_radar` bir okumadır (bir profilin tüm depolarında incelemenizi
  bekleyen request'ler, açık request'ler, atanan issue'lar; `refresh` verilmedikçe beş dakikalık önbellekten);
  `hosting_integrations_radar` bir profilin `[integrations.radar]` tablosunu değiştirir, bu yüzden
  `allow_mutations` onu kapılar (yalnızca ayarları yazdıran, seçeneksiz çağrıldığında bile).
- **İnceleme bağlamı**: `ai_review_context`, bir kod inceleme taslağına diff dışında nelerin verildiğini gösterir
  (CLAUDE.md, AGENTS.md, bir ek dosya, yerel bir ajanın çalıştığı yer). Bir ayar argümanıyla bir varsayılanı kaydeder
  (genel, ya da `repo`nun veya `project`in kendi değeri); bu yüzden, bir ayarı değiştiren diğer araçlar gibi,
  `allow_mutations` onu kapılar; hiçbiri verilmezse yalnızca yazdırır. Her `ai_*` aracı gibi yalnızca `ai.enabled`
  açıkken listelenir. `hosting_review_suggest` aynı argümanları yalnızca tek bir çalıştırma için alır.
- **`[mcp]` kapıları MCP sunucusuna uygulanır.** CLI sizin kendi kabuğunuzdur ve
  `allow_mutations` ile kapılanmaz; yalnızca `ui_control` ve `allow_ui_propose`
  ayrıca `gitpoulpi ui ...` için de geçerlidir.
- Reddedilen bir çağrı sıradan bir araç hatasıdır (`isError: true`, neden
  `structuredContent.error` içinde), asla sessiz bir işlemsizlik değildir.

```toml
[mcp]
allow_mutations = false        # true: ajan depoyu değiştirebilir
mutation_allowlist = []        # örn. ["stage", "unstage", "commit", "branch_create"]
allow_ui = true
ui_control = "ask"             # "auto" ajanın görünümünüzü sormadan hareket ettirmesine izin verir
allow_ui_propose = false       # true: ajan Reword / Split / commit / dal / tag iletişim kutularını önceden doldurabilir
allow_sensitive_staging = false  # true: ajan stage koruması işaretleyen dosyaları stage'leyebilir
allow_history_purge = false    # true: ajan rewrite_purge aracını görür (eski geçmişi kalıcı olarak siler)
```

## Araçlar

Her CLI komutu için bir araç, yolundan `_` ile adlandırılır (`branch create`,
`branch_create` olur). Her biri komutun argümanlarını ve ayrıca `repo` (depo;
varsayılan: sunucununki) ile `session` alır. Başarılı bir çağrının
`structuredContent` değeri tam olarak `gitpoulpi --json <command>` komutunun
yazdırdığıdır. **Tür**: *read* hiçbir şeyi değiştirmez; *write* ve *destructive*
değişikliklerdir (kapılı); *network* bir uzak depoyla ya da barındırıcıyla
konuşur; *view* yalnızca pencerenizi hareket ettirir; *propose* sizin
onaylayacağınız bir iletişim kutusu açar.

<!-- mcp-tools:begin -->
| Tool | CLI | Kind |
|---|---|---|
| `ai_commit_message` | `gitpoulpi ai commit-message` | read, network |
| `ai_explain` | `gitpoulpi ai explain` | read, network |
| `ai_prompt` | `gitpoulpi ai prompt` | read |
| `ai_resolve_conflict` | `gitpoulpi ai resolve-conflict` | read, network |
| `ai_review_context` | `gitpoulpi ai review-context` | write |
| `ai_rewrite_rules` | `gitpoulpi ai rewrite-rules` | read, network |
| `ai_rewrite_script` | `gitpoulpi ai rewrite-script` | read, network |
| `ai_split_plan` | `gitpoulpi ai split-plan` | read, network |
| `alias_status` | `gitpoulpi alias status` | read |
| `amend` | `gitpoulpi amend` | destructive |
| `apply_commit` | `gitpoulpi apply-commit` | write |
| `autostash_start_over` | `gitpoulpi autostash start-over` | destructive |
| `blame` | `gitpoulpi blame` | read |
| `branch_create` | `gitpoulpi branch create` | write |
| `branch_delete` | `gitpoulpi branch delete` | destructive |
| `branch_delete_remote` | `gitpoulpi branch delete-remote` | destructive, network |
| `branch_fast_forward` | `gitpoulpi branch fast-forward` | write |
| `branch_list` | `gitpoulpi branch list` | read |
| `branch_rename` | `gitpoulpi branch rename` | write |
| `branch_set_upstream` | `gitpoulpi branch set-upstream` | write |
| `branch_track` | `gitpoulpi branch track` | write |
| `branch_unset_upstream` | `gitpoulpi branch unset-upstream` | write |
| `branch_upstream_add` | `gitpoulpi branch upstream add` | write |
| `branch_upstream_list` | `gitpoulpi branch upstream list` | read |
| `branch_upstream_primary` | `gitpoulpi branch upstream primary` | write |
| `branch_upstream_pull` | `gitpoulpi branch upstream pull` | write |
| `branch_upstream_remove` | `gitpoulpi branch upstream remove` | write |
| `checkout` | `gitpoulpi checkout` | write |
| `cherry_pick` | `gitpoulpi cherry-pick` | write |
| `clone` | `gitpoulpi clone` | write, network |
| `commit` | `gitpoulpi commit` | write |
| `config_get` | `gitpoulpi config get` | read |
| `config_path` | `gitpoulpi config path` | read |
| `conflicts_abort` | `gitpoulpi conflicts abort` | destructive |
| `conflicts_am_abort` | `gitpoulpi conflicts am-abort` | destructive |
| `conflicts_am_continue` | `gitpoulpi conflicts am-continue` | write |
| `conflicts_am_skip` | `gitpoulpi conflicts am-skip` | destructive |
| `conflicts_check` | `gitpoulpi conflicts check` | read |
| `conflicts_list` | `gitpoulpi conflicts list` | read |
| `conflicts_resolve` | `gitpoulpi conflicts resolve` | destructive |
| `conflicts_show` | `gitpoulpi conflicts show` | read |
| `conflicts_skip` | `gitpoulpi conflicts skip` | destructive |
| `conflicts_state` | `gitpoulpi conflicts state` | read |
| `conflicts_unresolve` | `gitpoulpi conflicts unresolve` | write |
| `convert` | `gitpoulpi convert` | write |
| `diff` | `gitpoulpi diff` | read |
| `discard` | `gitpoulpi discard` | destructive |
| `fetch` | `gitpoulpi fetch` | write, network |
| `file_history` | `gitpoulpi file-history` | read |
| `flow_config_init` | `gitpoulpi flow config init` | write |
| `flow_config_show` | `gitpoulpi flow config show` | read |
| `flow_detect` | `gitpoulpi flow detect` | read |
| `flow_finish` | `gitpoulpi flow finish` | destructive, network |
| `flow_import` | `gitpoulpi flow import` | write |
| `flow_issue_new` | `gitpoulpi flow issue new` | write, network |
| `flow_issue_wait` | `gitpoulpi flow issue wait` | write, network |
| `flow_pick_up` | `gitpoulpi flow pick-up` | write, network |
| `flow_promote` | `gitpoulpi flow promote` | destructive |
| `flow_set_parent` | `gitpoulpi flow set-parent` | write |
| `flow_start` | `gitpoulpi flow start` | write, network |
| `flow_status` | `gitpoulpi flow status` | read |
| `flow_sync` | `gitpoulpi flow sync` | destructive |
| `flow_templates` | `gitpoulpi flow templates` | read |
| `hosting_board_move` | `gitpoulpi hosting board move` | write, network |
| `hosting_board_show` | `gitpoulpi hosting board show` | read, network |
| `hosting_checkout_pr` | `gitpoulpi hosting checkout-pr` | write, network |
| `hosting_clone` | `gitpoulpi hosting clone` | write, network |
| `hosting_edit_url` | `gitpoulpi hosting edit-url` | read, network |
| `hosting_file` | `gitpoulpi hosting file` | read, network |
| `hosting_integrations_add` | `gitpoulpi hosting integrations add` | write |
| `hosting_integrations_list` | `gitpoulpi hosting integrations list` | read |
| `hosting_integrations_radar` | `gitpoulpi hosting integrations radar` | write |
| `hosting_integrations_remove` | `gitpoulpi hosting integrations remove` | destructive |
| `hosting_integrations_test` | `gitpoulpi hosting integrations test` | read, network |
| `hosting_issue_base` | `gitpoulpi hosting issue base` | write |
| `hosting_issue_close` | `gitpoulpi hosting issue close` | write |
| `hosting_issue_create` | `gitpoulpi hosting issue create` | write, network |
| `hosting_issue_edit` | `gitpoulpi hosting issue edit` | write, network |
| `hosting_issue_move` | `gitpoulpi hosting issue move` | write |
| `hosting_issue_options` | `gitpoulpi hosting issue options` | read, network |
| `hosting_issue_reopen` | `gitpoulpi hosting issue reopen` | write |
| `hosting_issues` | `gitpoulpi hosting issues` | read, network |
| `hosting_labels` | `gitpoulpi hosting labels` | read, network |
| `hosting_link` | `gitpoulpi hosting link` | write |
| `hosting_list` | `gitpoulpi hosting list` | read |
| `hosting_logout` | `gitpoulpi hosting logout` | destructive |
| `hosting_members` | `gitpoulpi hosting members` | read, network |
| `hosting_milestones` | `gitpoulpi hosting milestones` | read, network |
| `hosting_open` | `gitpoulpi hosting open` | read |
| `hosting_pr` | `gitpoulpi hosting pr` | write, network |
| `hosting_pr_edit` | `gitpoulpi hosting pr-edit` | write, network |
| `hosting_pr_for_branch` | `gitpoulpi hosting pr-for-branch` | read, network |
| `hosting_projects` | `gitpoulpi hosting projects` | read, network |
| `hosting_prs` | `gitpoulpi hosting prs` | read, network |
| `hosting_radar` | `gitpoulpi hosting radar` | read, network |
| `hosting_refs` | `gitpoulpi hosting refs` | read |
| `hosting_review_abandon` | `gitpoulpi hosting review abandon` | write, network |
| `hosting_review_comment` | `gitpoulpi hosting review comment` | write, network |
| `hosting_review_diff` | `gitpoulpi hosting review diff` | read, network |
| `hosting_review_fetch` | `gitpoulpi hosting review fetch` | write, network |
| `hosting_review_push` | `gitpoulpi hosting review push` | write, network |
| `hosting_review_reply` | `gitpoulpi hosting review reply` | write, network |
| `hosting_review_resolve` | `gitpoulpi hosting review resolve` | write, network |
| `hosting_review_show` | `gitpoulpi hosting review show` | read, network |
| `hosting_review_submit` | `gitpoulpi hosting review submit` | write, network |
| `hosting_review_submit_change` | `gitpoulpi hosting review submit-change` | write, network |
| `hosting_review_suggest` | `gitpoulpi hosting review suggest` | read, network |
| `hosting_review_threads` | `gitpoulpi hosting review threads` | read, network |
| `hosting_ssh_key_delete` | `gitpoulpi hosting ssh-key delete` | destructive, network |
| `hosting_ssh_key_list` | `gitpoulpi hosting ssh-key list` | read, network |
| `hosting_use` | `gitpoulpi hosting use` | write |
| `ignore` | `gitpoulpi ignore` | write |
| `index_lock_remove` | `gitpoulpi index-lock remove` | destructive |
| `index_lock_show` | `gitpoulpi index-lock show` | read |
| `job_cancel` | `gitpoulpi job cancel` | write |
| `job_list` | `gitpoulpi job list` | read |
| `job_wait` | `gitpoulpi job wait` | read |
| `lfs_checkout` | `gitpoulpi lfs checkout` | write |
| `lfs_fetch` | `gitpoulpi lfs fetch` | write, network |
| `lfs_files` | `gitpoulpi lfs files` | read |
| `lfs_install` | `gitpoulpi lfs install` | write |
| `lfs_lock` | `gitpoulpi lfs lock` | write, network |
| `lfs_locks` | `gitpoulpi lfs locks` | read, network |
| `lfs_prune` | `gitpoulpi lfs prune` | destructive |
| `lfs_pull` | `gitpoulpi lfs pull` | write, network |
| `lfs_push` | `gitpoulpi lfs push` | destructive, network |
| `lfs_status` | `gitpoulpi lfs status` | read |
| `lfs_track` | `gitpoulpi lfs track` | write |
| `lfs_unlock` | `gitpoulpi lfs unlock` | destructive, network |
| `lfs_untrack` | `gitpoulpi lfs untrack` | write |
| `licenses` | `gitpoulpi licenses` | read |
| `log` | `gitpoulpi log` | read |
| `lost_commits` | `gitpoulpi lost-commits` | read |
| `mailmap_add` | `gitpoulpi mailmap add` | write |
| `mailmap_check` | `gitpoulpi mailmap check` | read |
| `mailmap_list` | `gitpoulpi mailmap list` | read |
| `merge` | `gitpoulpi merge` | write |
| `patch_apply` | `gitpoulpi patch apply` | write |
| `patch_create` | `gitpoulpi patch create` | write |
| `path_status` | `gitpoulpi path status` | read |
| `pull` | `gitpoulpi pull` | write, network |
| `push` | `gitpoulpi push` | destructive, network |
| `rebase` | `gitpoulpi rebase` | destructive |
| `rebase_plan` | `gitpoulpi rebase-plan` | read |
| `redo` | `gitpoulpi redo` | destructive |
| `reflog` | `gitpoulpi reflog` | read |
| `reflog_restore` | `gitpoulpi reflog-restore` | destructive |
| `refs_hide` | `gitpoulpi refs hide` | write |
| `refs_show` | `gitpoulpi refs show` | write |
| `refs_solo` | `gitpoulpi refs solo` | write |
| `remote_add` | `gitpoulpi remote add` | write |
| `remote_clear_push_url` | `gitpoulpi remote clear-push-url` | write |
| `remote_default` | `gitpoulpi remote default` | write |
| `remote_list` | `gitpoulpi remote list` | read |
| `remote_remove` | `gitpoulpi remote remove` | destructive |
| `remote_rename` | `gitpoulpi remote rename` | write |
| `remote_set_url` | `gitpoulpi remote set-url` | write |
| `remote_group_abort` | `gitpoulpi remote-group abort` | destructive |
| `remote_group_continue` | `gitpoulpi remote-group continue` | destructive, network |
| `remote_group_plan` | `gitpoulpi remote-group plan` | read, network |
| `remote_group_retry` | `gitpoulpi remote-group retry` | write, network |
| `remote_group_status` | `gitpoulpi remote-group status` | read |
| `repo_close` | `gitpoulpi repo close` | write |
| `repo_discover` | `gitpoulpi repo discover` | read |
| `repo_init` | `gitpoulpi repo init` | write |
| `repo_list` | `gitpoulpi repo list` | read |
| `repo_open` | `gitpoulpi repo open` | write |
| `repo_recent` | `gitpoulpi repo recent` | read |
| `repo_auth_show` | `gitpoulpi repo-auth show` | read |
| `repo_auth_test` | `gitpoulpi repo-auth test` | write, network |
| `repo_config_set` | `gitpoulpi repo-config set` | write |
| `repo_config_show` | `gitpoulpi repo-config show` | read |
| `repo_config_unset` | `gitpoulpi repo-config unset` | write |
| `reset` | `gitpoulpi reset` | destructive |
| `revert` | `gitpoulpi revert` | write |
| `reword` | `gitpoulpi reword` | destructive |
| `rewrite_analyze` | `gitpoulpi rewrite analyze` | read |
| `rewrite_apply` | `gitpoulpi rewrite apply` | destructive |
| `rewrite_check_signing` | `gitpoulpi rewrite check-signing` | read |
| `rewrite_diff` | `gitpoulpi rewrite diff` | read |
| `rewrite_discard` | `gitpoulpi rewrite discard` | read |
| `rewrite_maps` | `gitpoulpi rewrite maps` | write |
| `rewrite_note` | `gitpoulpi rewrite note` | read |
| `rewrite_preview` | `gitpoulpi rewrite preview` | read |
| `rewrite_purge` | `gitpoulpi rewrite purge` | destructive |
| `rewrite_push` | `gitpoulpi rewrite push` | destructive, network |
| `rewrite_rows` | `gitpoulpi rewrite rows` | read |
| `rewrite_run` | `gitpoulpi rewrite run` | destructive |
| `rewrite_scan_secrets` | `gitpoulpi rewrite scan-secrets` | read |
| `rewrite_script_check` | `gitpoulpi rewrite script check` | read |
| `rewrite_script_test` | `gitpoulpi rewrite script test` | read |
| `rewrite_tools` | `gitpoulpi rewrite tools` | read |
| `rm_file` | `gitpoulpi rm-file` | destructive |
| `search` | `gitpoulpi search` | read |
| `session_delete` | `gitpoulpi session delete` | destructive |
| `session_list` | `gitpoulpi session list` | read |
| `session_new` | `gitpoulpi session new` | write |
| `session_rename` | `gitpoulpi session rename` | write |
| `show` | `gitpoulpi show` | read |
| `sparse_checkout_add` | `gitpoulpi sparse-checkout add` | write |
| `sparse_checkout_disable` | `gitpoulpi sparse-checkout disable` | write |
| `sparse_checkout_set` | `gitpoulpi sparse-checkout set` | write |
| `sparse_checkout_status` | `gitpoulpi sparse-checkout status` | read |
| `split_commit` | `gitpoulpi split-commit` | destructive |
| `split_source` | `gitpoulpi split-source` | read |
| `ssh_command_clear` | `gitpoulpi ssh-command clear` | write |
| `ssh_command_set` | `gitpoulpi ssh-command set` | write |
| `ssh_command_show` | `gitpoulpi ssh-command show` | read |
| `stage` | `gitpoulpi stage` | write |
| `stash_apply` | `gitpoulpi stash apply` | write |
| `stash_drop` | `gitpoulpi stash drop` | destructive |
| `stash_from_commit` | `gitpoulpi stash from-commit` | write |
| `stash_list` | `gitpoulpi stash list` | read |
| `stash_pop` | `gitpoulpi stash pop` | write |
| `stash_push` | `gitpoulpi stash push` | write |
| `status` | `gitpoulpi status` | read |
| `submodule_init` | `gitpoulpi submodule init` | write |
| `submodule_list` | `gitpoulpi submodule list` | read |
| `submodule_update` | `gitpoulpi submodule update` | write, network |
| `tag_annotate` | `gitpoulpi tag annotate` | write |
| `tag_create` | `gitpoulpi tag create` | write |
| `tag_delete` | `gitpoulpi tag delete` | destructive, network |
| `tag_delete_remote` | `gitpoulpi tag delete-remote` | destructive, network |
| `tag_fetch` | `gitpoulpi tag fetch` | write, network |
| `tag_list` | `gitpoulpi tag list` | read |
| `tag_push` | `gitpoulpi tag push` | write, network |
| `theme_export` | `gitpoulpi theme export` | read |
| `theme_list` | `gitpoulpi theme list` | read |
| `treat` | `gitpoulpi treat` | write |
| `ui_back` | `gitpoulpi ui back` | view |
| `ui_highlight` | `gitpoulpi ui highlight` | view |
| `ui_notify` | `gitpoulpi ui notify` | view |
| `ui_propose_branch` | `gitpoulpi ui propose branch` | propose |
| `ui_propose_commit_message` | `gitpoulpi ui propose commit-message` | propose |
| `ui_propose_resolution` | `gitpoulpi ui propose resolution` | propose |
| `ui_propose_reword` | `gitpoulpi ui propose reword` | propose |
| `ui_propose_rewrite` | `gitpoulpi ui propose rewrite` | propose |
| `ui_propose_split` | `gitpoulpi ui propose split` | propose |
| `ui_propose_tag` | `gitpoulpi ui propose tag` | propose |
| `ui_show_commit` | `gitpoulpi ui show commit` | view |
| `ui_show_conflict` | `gitpoulpi ui show conflict` | view |
| `ui_show_diff` | `gitpoulpi ui show diff` | view |
| `ui_show_file` | `gitpoulpi ui show file` | view |
| `ui_show_home` | `gitpoulpi ui show home` | view |
| `ui_show_issue` | `gitpoulpi ui show issue` | view |
| `ui_show_keys` | `gitpoulpi ui show keys` | view |
| `ui_show_palette` | `gitpoulpi ui show palette` | view |
| `ui_show_pr` | `gitpoulpi ui show pr` | view |
| `ui_show_ref` | `gitpoulpi ui show ref` | view |
| `ui_show_search` | `gitpoulpi ui show search` | view |
| `ui_show_settings` | `gitpoulpi ui show settings` | view |
| `ui_show_tab` | `gitpoulpi ui show tab` | view |
| `ui_show_wip` | `gitpoulpi ui show wip` | view |
| `ui_state` | `gitpoulpi ui state` | view |
| `ui_viewers` | `gitpoulpi ui viewers` | view |
| `undo` | `gitpoulpi undo` | destructive |
| `undo_forget` | `gitpoulpi undo-forget` | write |
| `undo_restore` | `gitpoulpi undo-restore` | write |
| `unstage` | `gitpoulpi unstage` | write |
| `workspace_add` | `gitpoulpi workspace add` | write |
| `workspace_create` | `gitpoulpi workspace create` | write |
| `workspace_delete` | `gitpoulpi workspace delete` | destructive |
| `workspace_list` | `gitpoulpi workspace list` | read |
| `workspace_remove` | `gitpoulpi workspace remove` | write |
| `workspace_rename` | `gitpoulpi workspace rename` | write |
| `workspace_set_ai_instructions` | `gitpoulpi workspace set-ai-instructions` | write |
| `workspace_set_profile` | `gitpoulpi workspace set-profile` | write |
| `workspace_show` | `gitpoulpi workspace show` | read |
| `worktree_add` | `gitpoulpi worktree add` | write |
| `worktree_changes` | `gitpoulpi worktree changes` | read |
| `worktree_list` | `gitpoulpi worktree list` | read |
| `worktree_remove` | `gitpoulpi worktree remove` | destructive |

Not exposed as tools:

| CLI | Why |
|---|---|
| `gitpoulpi hosting integrations trust` | accepting a changed address, account or certificate is the user's decision; an agent is told the security check needs the user |
| `gitpoulpi ai trust` | binding an API key to another address is the user's decision |
| `gitpoulpi mcp serve` | the MCP server cannot spawn/serve a copy of itself |
| `gitpoulpi mcp token` | prints, copies and rotates the key that guards this very server: an agent must not read or replace it |
| `gitpoulpi completions` | shell completion generation, not a GitBackend operation |
| `gitpoulpi shell-init` | prints a wrapper function for the user's own shell, not a GitBackend operation |
| `gitpoulpi desktop shortcut status` | launcher shortcuts are the user's desktop, not a repository; they need the TUI's icon renderer |
| `gitpoulpi desktop shortcut install` | writes files into the user's launcher, Start menu and Desktop |
| `gitpoulpi desktop shortcut remove` | deletes files from the user's launcher, Start menu and Desktop |
| `gitpoulpi alias create` | writes a command into the user's PATH; an agent never changes the user's shell |
| `gitpoulpi alias remove` | deletes a command from the user's PATH; an agent never changes the user's shell |
| `gitpoulpi path add` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi path remove` | edits the user's shell rc files or Windows user PATH |
| `gitpoulpi uninstall` | removes GitPoulpi's own program files, aliases, PATH entries and shortcuts from the machine |
| `gitpoulpi launch` | starts a terminal window on the user's desktop; the launcher shortcuts' own entry point |
| `gitpoulpi daemon run` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon start` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon stop` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon restart` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi daemon status` | daemon process lifecycle, not a GitBackend operation |
| `gitpoulpi rewrite script trust` | trusting an unsandboxed script is a person's decision; an agent can only run scripts a person trusted |
| `gitpoulpi config set` | an agent must not be able to rewrite its own gating config |
| `gitpoulpi hosting ssh-key create` | writes a private key and uploads it; too dangerous to let an agent automate |
| `gitpoulpi hosting login` | takes a secret token on stdin; a token must not pass through an agent's tool-call transcript |
| `gitpoulpi repo-auth set` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi repo-auth clear` | which identity a repository authenticates with is the user's decision; an agent could switch a repository to a more privileged token |
| `gitpoulpi flow trust-hooks` | trusting the shell commands a repository commits is a person's decision; an agent can only run hooks a person trusted |
<!-- mcp-tools:end -->

### Dal iş akışı eylemleri planla-sonra-çalıştır biçimindedir

`flow_start`, `flow_finish`, `flow_sync`, `flow_promote` ve `flow_set_parent`
([Dal iş akışları](branching-workflows.md#eylemler)) yalın bir çağrıyla asla çalışmaz.
`dry_run: true` planı (her git adımı ve komutu, uyarılar, Undo'nun geri alamayacakları,
engeller) normal bir sonuç olarak döndürür; `yes: true` onu çalıştırır; ikisi de yoksa çağrı
reddedilir ve planı taşır. İş akışı dosyasının `[hooks]` bölümü ekibin paylaştığı bir dosyadan
gelen kabuk komutlarıdır, bu yüzden yalnızca açık bir `run_hooks: true` ile çalışır: plan onları
listeler (`hooks`, `hooks_run`) ki kullanıcıya tam komutları önce gösterebilesiniz. Çakışmayla
duran bir merge `status: "conflicted"` yanıtını verir (bir hata sonucu, çıkış kodu 2); çakışma
araçları dosyaları çözdükten sonra aynı aracı `continue_: true` veya `abort: true` ile çağırın.
Bir merge request açmak (`merge-request` kipinde `finish`) geri alınamaz ve plan bunu söyler.

### Bir ajanın düz `git` ile yaptıkları da geri alma geçmişindedir

Dal ve worktree oluşturan, commit yapan, reset yapan veya stash atan bir ajan bunu
kendi `git`'iyle yapıyorsa (bu araçlarla değil) yine de görünürdür: daemon
deponun ref'lerini en son gördükleriyle karşılaştırır ve her değişikliği
`"source": "outside"`, git'in nedeni (`outside.message`) ve git'in kaydettiği
committer ile bir `undo --list` girdisi olarak kaydeder. `undo`, `redo` ve
`undo_restore` araçları bu girdileri diğerleri gibi işler; böylece bir ajan (ya da
kullanıcı, `Ctrl+Z` ile) silinmiş bir dalı geri getirebilir. Dosya düzenlemeleri
hiçbir zaman yakalanmaz. Bkz. [Geri al ve yinele](undo-redo.md#gitpoulpi-dışında-yapılan-değişiklikler).

## Kullanıcıya bir şeyler göstermek

Bir ajan, bir şeyi anlatmak yerine ona işaret edebilir: `ui_show_commit`,
`ui_show_diff` (bir dosya bir satırda, satır içi veya bölünmüş), `ui_show_file`
(içerik, blame veya geçmiş bir satırda), `ui_show_ref`, `ui_show_pr`,
`ui_show_issue`, `ui_show_conflict` (çakışmalı bir dosyada, bir çakışmada merge
düzenleyicisi), `ui_show_wip` (`composer`, imleci commit özetine koyar, hiçbir
şey yazmaz), `ui_show_tab`, `ui_show_home`, `ui_show_search`, `ui_show_palette`
(sorgu yazılır, asla çalıştırılmaz), `ui_show_keys`, `ui_show_settings` (bir sayfa
ve bir ayar, asla düzenlenmez), `ui_highlight` (açık diff'in veya dosyanın
satırlarını renklendirir), `ui_notify` (bir bildirim) ve `ui_back` (ajanın son
görünüm değişikliğini geri alır). `ui_state` bir pencerenin neyi gösterdiğini
okur (`text: true` görünen metni ekler); `ui_viewers` pencereleri listeler.

Bu araçlar **depoyu asla değiştirmez** ve yazı yazamaz, tıklayamaz veya bir
ayarı düzenleyemez. Revizyonlar ve yollar, herhangi bir şey pencereye ulaşmadan
önce denetlenir.

**Kullanıcının gördüğü ve karar verdiği**:

- `ask` (varsayılan) ya da yazı yazıyorsanız, bir iletişim kutusu veya Ayarlar
  açıksa, sürüklüyorsanız veya son 1,5 sn içinde bir tuşa bastıysanız: ajan
  hemen `deferred` alır ve durum çubuğunuzda bir çip belirir: *"claude-code ...
  göstermek istiyor"*. `Ctrl+N Enter` veya bir tık gösterir; `Esc` veya
  `Ctrl+N Backspace` kapatır. 60 sn sonra süresi dolar.
- Uygulandı: görünüm hareket eder, 4 sn'lik bir bildirim onu kimin hareket
  ettirdiğini söyler ve sekme, bir sonraki tuşunuza kadar mor bir `✦` alır.
  `Ctrl+N Backspace` (palet `Ajan: Geri`) geri gider; pencere on görünümü
  hatırlar.
- Gizli hiçbir şey gösterilmez ve hiçbir şey checkout edilmez: solo filtrenizin
  gizlediği bir commit `partial` (`hidden_by_filter`) yanıtlar; bir sağlayıcıya
  bağlı olmayan bir PR veya issue listesi `hosting_not_linked` yanıtlar.

**Yanıtlar**: `{"outcome": "applied" | "partial" | "deferred" | "refused" |
"not_found" | "no_viewer" | "proposed", ...}`, uygulandığında pencerenin
`state` değeriyle. CLI çıkış kodları: 0 applied/partial/proposed, 3 deferred, 1
diğer durumlar. Çağrılar hız sınırlıdır (5'lik bir salvo, sonra saniyede bir).

```console
$ gitpoulpi --json ui show diff src/lib.rs --rev 7f3c2a1 --line 120
{"outcome":"applied","state":{"central":"diff","diff":{"path":"src/lib.rs","cursor_line":120,...}}}
$ gitpoulpi --json ui notify "Hata, yukarıdaki erken return."
$ gitpoulpi --json ui back
```

### Bir değişiklik önermek (`ui_propose_*`)

`allow_ui_propose = true` ile bir ajan, iletişim kutularınızdan birini **önceden
doldurulmuş** olarak açabilir ve *✦ <agent> önerdi* ile işaretler. O iletişim
kutusunun kendi onayına basana kadar hiçbir şey olmaz; önce düzenleyebilir veya
iptal edebilirsiniz.

| Araç | Açar | Şununla onaylarsınız |
|---|---|---|
| `ui_propose_reword` (rev, message) | Reword açılır penceresi | *Mesajı Güncelle* (push edilmiş bir commit yine force push'u sorar) |
| `ui_propose_commit_message` (summary, body) | doldurulmuş commit formu; asla commit etmez | Commit düğmesi |
| `ui_propose_branch` (name, start point) | satır içi Yeni Dal kutusu | `Enter` |
| `ui_propose_tag` (name, target, message) | satır içi Yeni Tag kutusu (mesaj varsa açıklamalı) | `Enter` |
| `ui_propose_split` (rev, plan file) | commit'in üzerinde, plandan doldurulmuş bölme düzenleyicisi | *Böl* (`Ctrl+S`); `Esc` reddeder |
| `ui_propose_rewrite` (kural seti dosyası) | kural setinin kurallarının içinde olduğu geçmişi yeniden yazma ekranı; hiçbir şey önizlenmez veya uygulanmaz | sizin kendi *Önizle* ve ardından *Uygula* adımlarınız (`Ctrl+Enter`); `Esc` reddeder. Betikli bir kural seti reddedilir, gizli değerler çıkarılır |
| `ui_propose_resolution` (path, pick or content file) | çakışmalı bir dosyada, seçimler (veya tüm metin) Çıktı'da hazır olarak merge düzenleyicisi | *Kaydet ve Çözüldü İşaretle* (`Ctrl+S`); kapatmak reddeder |

Aynı anda tek öneri; bir commit mesajı önerisi, composer'da kendi taslağınız
varken, bir çözüm önerisi ise merge düzenleyicisi sizin kaydedilmemiş işinizi
tutarken veya dosyanın stage durumu ajan onu okuduktan sonra değiştiyse
reddedilir. Onaylamadan önce değiştirdiğiniz bir çözüm `accepted_edited` olarak
bildirilir. Yanıtta bir `proposal` kimliği vardır; ne kararlaştırdığınız
`ui_state` değerinin `proposals` alanındadır (`offered`, `open`, `accepted`,
`accepted_edited`, `rejected`, `expired`, `failed`).

### Hiçbir pencere açık olmadığında veya başka bir oturumda olduğunda

- Bir pencere, başlatıldığı oturuma aittir. Ajanın oturumunda hiç yoksa, `ui show`
  deponun açık olduğu başka bir oturumdaki **bir** pencereyi arar ve onu kullanır
  (yanıtta `routed_session`). Birden çoğu eşleşirse: onları adlandıran
  `no_viewer`; `--session` verin.
- **Ajan için bir pencere açmak** varsayılan olarak kapalıdır. `[ui] launch`
  değerini (Ayarlar > Ajanlar ve MCP > *Ajanlar için pencere aç*) bir terminal
  veya çoklayıcı ön ayarına ya da bir `custom` komutuna ayarlayın; o zaman
  `launch: true` içeren bir çağrı (CLI `gitpoulpi ui --launch show ...`) bir
  tane açar ve 10 sn'ye kadar bekler. `ui_control = "auto"` ile bayrak olmadan
  da açılır; `ask` ile ajan `no_viewer` alır ve size söylemelidir.

| Ön ayar | Komut |
|---|---|
| `kitty` | `kitty --directory {repo} {gitpoulpi} --session {session}` |
| `foot` | `foot --working-directory {repo} {gitpoulpi} --session {session}` |
| `alacritty` | `alacritty --working-directory {repo} -e {gitpoulpi} --session {session}` |
| `wezterm` / `wezterm-tab` | `wezterm start --cwd {repo} -- ...` / `wezterm cli spawn --cwd {repo} -- ...` |
| `ghostty` | `ghostty --working-directory={repo} -e {gitpoulpi} --session {session}` |
| `gnome-terminal` | `gnome-terminal --working-directory={repo} -- {gitpoulpi} --session {session}` |
| `xdg-terminal-exec` | `xdg-terminal-exec {gitpoulpi} --session {session}` (varsayılan terminaliniz) |
| `windows-terminal` / `windows-terminal-tab` | `wt.exe -d {repo} ...` / `wt.exe -w 0 new-tab -d {repo} ...` |
| `tmux` (yeni pencere), `zellij` (yeni bölme), `cmux` (yeni çalışma alanı) | çalışan bir çoklayıcının içinde, deponun klasöründe |
| `custom` | `[ui] launch_command`, örn. `my-term --cd {repo} {gitpoulpi} --session {session}`; argümanlara bölünür, kabuk yok |

Bir ön ayar programınızın sürümüyle çalışmıyorsa `custom` kullanın.

## Hatalar

- Hatalı biçimli bir çağrı (bilinmeyen araç veya argüman, eksik zorunlu bir
  argüman) bir JSON-RPC `-32602 invalid_params` hatasıdır.
- Bir ret ya da bir Git veya daemon hatası (kötü revizyon, çakışma, ...) bir araç
  hatasıdır: `isError: true`, mesaj `content` ve `structuredContent.error`
  içinde.
