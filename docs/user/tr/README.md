[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · [Français](../fr/README.md) · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · **Türkçe** · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# GitPoulpi kullanıcı belgeleri

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="GitPoulpi ahtapotu" width="128"></p>

GitPoulpi, tıklanabilir bir commit grafiği, bir komut paleti, barındırma hizmeti
ve issue takipçisi entegrasyonu, isteğe bağlı YZ yardımı ve arayüzün yaptığı her
şeyi yapan bir CLI ile MCP sunucusu sunan, terminalde çalışan bir Git
istemcisidir. Bu sayfalar okuma sırasındadır: en üstten başlayın, gerektiğinde
başvuru sayfalarına atlayın.

![GitPoulpi ana penceresi](../images/hero.png)

## Buradan başlayın

1. **[Kurulum](installation.md)**: nelere ihtiyacınız olduğu, sürüm arşivleri,
   paket yöneticileri, kaynaktan derleme. Daha küçük bir ikili için
   **[Özel derlemeler](custom-builds.md)** sayfasına bakın. GitPoulpi'nin kendini
   nasıl güncel tuttuğu (sürüm çipi, güncelleme paneli, önceki sürüme dönme):
   **[Güncellemeler](updates.md)**. Bir kapsayıcıda çalıştırmak için:
   **[Docker'da çalıştırma](docker.md)**. Yöneticiler için, iç kaynaklar, yansılar
   ve güncelleme politikası: **[Kurumsal dağıtım](enterprise-distribution.md)**.
2. **[Başlarken](getting-started.md)**: ilk beş dakikanız, başlatmaktan ilk
   commit'inize kadar.
3. **[Arayüz turu](interface-tour.md)**: pencerenin her bölgesinin adı,
   görsellerle.

## Bir özelliği bulun

| Şunu yapmak istiyorum... | Sayfa | Uygulamada nasıl ulaşılır |
|---|---|---|
| İki dalda aynı anda çalışmak | [Worktree'ler](worktrees.md) | Refs kenar çubuğunun **WORKTREE'LER** bölümü (`+` bir tane oluşturur; biri olana kadar gizlidir: palet **Sol panel: Worktree'ler Bölümünü Göster**); palet **Refs: Worktree Oluştur...** |
| İç içe depolarla çalışmak | [Alt modüller](submodules.md) | Refs kenar çubuğunun **ALT MODÜLLER** bölümü; palet **Git: Tüm Alt Modülleri Güncelle**, **Refs: Alt Modül Güncelle (Seçili)** |
| Bir merge çakışmasını çözmek | [Merge çakışmalarını çözmek](merge-conflicts.md) | Durum panelinde bir `!` satırında `Enter`, banner'ın **Çakışmaları çöz...** düğmesi veya Diff görünümünde `Shift+E`; palet **Çakışmalar: Merge Düzenleyiciyi Aç** |
| Bir pull veya merge request'i incelemek | [Bir pull veya merge request'i incelemek](review.md) | **PULL REQUEST'LER** bölümünün bir satırında `Enter`, bir Radar satırında `e`, çalışma alanı aramasında `Alt+R`; palet **Barındırma: İsteği İncele...** |
| Pull request'lere ve issue'lara göz atmak, onları oluşturmak ve yönetmek | [Pull request'ler ve issue'lar](pull-requests-and-issues.md) | Refs kenar çubuğunun **PULL REQUEST'LER** ve **ISSUE'LAR** bölümleri (`n` bir tane oluşturur); palet **Pull Request'ler: Pull Request Oluştur...**, **Barındırma: Yeni Issue...**. Önce bir hesap bağlayın: [Barındırma hizmetleri ve issue takipçileri](hosting-integration.md) |
| Bir issue veya isteğin tartışmasını okumak ve yanıtlamak | [Konuşmalar](conversations.md) | Bir issue satırında, bir pano kartında veya bir Radar satırında `Enter`; palet **Barındırma: Issue Konuşmasını Aç...** |
| Issue'larımı pano olarak görmek | [Issue panosu](pull-requests-and-issues.md#issue-panosu) | ISSUE'LAR başlığının **Pano** düğmesi; palet **Issue'lar: Panoyu Aç** |
| Issue'ları Markdown notları olarak tutmak, token olmadan | [Yerel issue'lar ve Obsidian](local-issues.md) | ISSUE'LAR bölümü, `Local Markdown - <ad>` sağlayıcısı; palet **Yerel Issue'lar: Obsidian'da Aç...** |
| Birçok commit boyunca yazarları, tarihleri, mesajları veya dosyaları yeniden yazmak | [Geçmişi yeniden yazma](history-rewrite.md) | `Ctrl+G Shift+H`; palet **Yeniden Yaz: Geçmiş Ekranı** |
| Bir işlemi geri almak (veya yinelemek) | [Geri al ve yinele](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y`; `Alt+Z` geri alma geçmişini açar; palet **Düzenle: Geri Al** |
| Tek bir tuşla birkaç eylem çalıştırmak (stage, YZ mesajı, commit, push...) | [Diziler](sequences.md) | Ayarlar > Diziler; palet **Dizi: <ad>** |
| Büyük dosyaları Git LFS ile izlemek | [Git LFS](lfs.md) | Palet **LFS: Deseni İzle...**, **LFS: Bu Depo İçin Başlat** |
| Depolar arasında bir dalı, commit'i, pull request'i veya issue'yu bulmak | [Arama](search.md) | `Ctrl+Alt+F` veya başlık çubuğundaki Genel arama düğmesi; `f` geçerli deponun commit grafiğinde arar |
| Beni bekleyen her şeyi tek listede görmek | [Radar](radar.md) | `Alt+R`; palet **Radar: Aç** |
| Depoları gruplamak ve birlikte fetch veya pull etmek | [Çalışma alanları](workspaces.md) | `Alt+W` değiştirir, `Alt+J` Çalışma alanları sekmesini açar; palet **Çalışma Alanı: Yönet...** |
| YZ yardımı almak (commit mesajları, açıklamalar, tanımlar) | [YZ desteği](ai.md) | `Ctrl+,` > **YZ** > **YZ'yi etkinleştir**, sonra `Ctrl+G i` bir commit mesajı yazar; palet **YZ: ...** |
| Herhangi bir komutu veya tuşu bulmak | [Klavye kısayolları](keybindings.md) | Uygulamada `?` tuşları listeler, `Ctrl+P` her komutu bulur |

## Günlük kullanım

4. **[Log ile çalışmak](log-and-history.md)**: grafiği okuma, arama, dallar,
   tag'ler, stash'ler, ref'leri sürükleme, rebase, cherry-pick,
   reword ve amend, bannerlar, pull ve push, auto-stash.
5. **[Stage'leme ve commit'leme](staging-and-committing.md)**: çalışma değişiklikleri,
   diff'ler, hunk'ları ve tek tek satırları stage'leme, commit yazma, commit
   composer, blame, dosya geçmişi ve yerleşik düzenleyici.
6. **[Görseller](image-view.md)**: Diff görünümü ve Dosya görünümündeki
   görseller, önce ve sonra, yakınlaştırma, Şununla düzenle.
7. **[Merge çakışmalarını çözmek](merge-conflicts.md)**: merge düzenleyicisi (A
   ve B, Çıktı'nın üzerinde yan yana), satır ve parça seçme, ikili, LFS, silinmiş,
   sembolik bağlantı ve alt modül çakışmaları için kartlar ve bir ajanın nasıl
   çözüm önerdiği.
8. **[Worktree'ler](worktrees.md)**: başka bir dalda ikinci bir checkout edilmiş
   klasör: WORKTREE'LER bölümü, bağlı worktree'leri oluşturma, açma, merge etme ve
   kaldırma.
9. **[Alt modüller](submodules.md)**: ALT MODÜLLER bölümü, bunları güncelleme ve
   checkout'larla pull'larla uyum içinde tutma.
10. **[Geri al ve yinele](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, nelerin geri
    alınabildiği ve alınamadığı, ne zaman reddettiği veya sorduğu, ne kadar geriye
    gittiği, kapatma.
11. **[Git LFS](lfs.md)**: büyük dosyalar, izleme, kilitler ve `git-lfs` kurulu
    değilken neler olduğu.
12. **[Temalar](themes.md)**: yerleşik temalar, her temayı her şerit paletiyle
    kullanma, canlı önizleme, kendi tema dosyalarınız, `gitpoulpi theme`.
13. **[Çalışma alanları](workspaces.md)**: kendi sekmeleri ve isteğe bağlı bir
    varsayılan commit profili olan, adlandırılmış depo grupları, sekme çubuğu
    rozeti, Çalışma alanları sekmesi, Tümünü fetch et, Tümünü pull
    et, Eksikleri klonla.
14. **[Arama](search.md)**: çalışma alanının veya açık sekmelerin tüm depolarındaki
    dallar, tag'ler, worktree'ler, stash'ler, commit'ler, pull/merge request'ler
    ve issue'lar üzerinde tek bir bulanık arama (`Ctrl+Alt+F`).
15. **[Radar](radar.md)**: bir çalışma alanının (veya açık sekmelerinizin)
    pull/merge request'leri, issue'ları ve süren işleri tek listede: filtreler,
    sabitleme, erteleme ve her satırın yapabilecekleri.
16. **[Pull request'ler ve issue'lar](pull-requests-and-issues.md)**: kenar
    çubuğunun PULL REQUEST'LER ve ISSUE'LAR bölümleri, issue panosu, bir pull
    request veya issue oluşturma, araç çubuğu çipi, commit mesajlarındaki
    referanslar.
17. **[Konuşmalar](conversations.md)**: bir issue veya isteğin tartışmasını
    pencerenin içinde okuma ve yanıtlama, şu adla gönderme, her barındırıcının
    yapamadıkları.
18. **[Bir pull veya merge request'i incelemek](review.md)**: yorum dizileri,
    bekleyen bir inceleme ve Onayla / Değişiklik iste ile inceleme görünümü.
19. **[Şurada aç](open-in.md)**: etkin sekmenin klasörünü bir terminalde,
    çoklayıcıda, düzenleyicide veya dosya yöneticisinde açan başlık çubuğu
    listesi.
20. **[Etkin sekmenin klasörünü izlemek](following-the-tab-folder.md)**:
    "Çık ve cd yap" ve bash, zsh, fish ve PowerShell için `gitpoulpi shell-init`
    sarmalayıcısı (`gp`).
    GitPoulpi'yi en iyi hangi terminal gösterir: **[Terminal seçmek](terminals.md)**.
    tmux, zellij, screen, herdr ve terminal panellerinden nelerin geçtiği:
    **[Çoklayıcılar](multiplexers.md)**.

Ayrıca: **[Bir dal için birden çok uzak depo](multiple-remotes.md)**: push hedefleri ve pull kaynakları,
Uzak depolar penceresi, tüm uzak depolara veya seçililere Push, ilerleyen bir uzak depoyu çözme, çakışmadan sonra
Devam, CLI ve düz git takma adı.

**[Dal iş akışları](branching-workflows.md)**, Git Flow, GitHub Flow, GitLab
Flow ve Trunk-based hazır şablonları veya kendi temel dallarınız, dal adı desenleri
ve konu türleri (`gitpoulpi flow ...`).

**[Geçmişi yeniden yazma](history-rewrite.md)**: birçok commit boyunca yazarları,
tarihleri, iletileri ve dosya içeriklerini yeniden yazan yeniden yazma ekranı
(`Ctrl+G Shift+H`); önizleme, Lua betikleri ve tek bir geri alma ile.

## Klavye ve fare

21. **[Klavye kısayolları](keybindings.md)**: bölgeye göre her tuş ve paletteki
    her komut. Uygulamada `?` aynı listeyi gösterir ve `Ctrl+P` her komutu bulur.
22. **[Fare ve hareketler](mouse-and-gestures.md)**: tıklama, sürükleme, seçme
    ve kopyalama, kaydırma çubukları ve üzerine gelme geri bildirimi.

## Ayarlar ve entegrasyonlar

23. **[Ayarlar](settings.md)**: Ayarlar ekranının her sayfası ve seçeneği, arama
    kutusu, profiller ve kısayolları yeniden atama.
24. **[YZ desteği](ai.md)**: GitPoulpi'nin kendi YZ'si (varsayılan olarak
    kapalı): `claude -p` gibi yerel bir ajan, Anthropic, OpenAI, OpenRouter,
    Ollama veya LM Studio; yönergeler; commit mesajları, açıklamalar, pull
    request açıklamaları ve commit composer; gizlilik ve sorun giderme.
25. **[Barındırma hizmetleri ve issue takipçileri](hosting-integration.md)**:
    GitHub, GitLab, Gitea ve Forgejo, Azure DevOps, Bitbucket, Gerrit ve issue
    takipçileri: token'lar ve kapsamları, bağlanma, bir depoyu bağlama, SSH
    anahtarları, bilinen sınırlar. Ayrıca
    **[Yerel issue'lar ve Obsidian](local-issues.md)**: issue kaynağı olarak bir
    Markdown notları klasörü, token yok, bir Obsidian Base ile.

## Otomasyon

26. **[CLI başvurusu](cli-reference.md)**: her `gitpoulpi` komutu, betikler için
    `--json` ile.
27. **[MCP ve ajanlar](mcp-and-agents.md)**: harici bir ajanın (Claude Code,
    Cursor, Codex, ...) GitPoulpi'yi yönetmesine izin verin: sunucuyu kaydetme,
    `.mcp.json`, izin anahtarları ve varsayılanları, her araç ve bir ajanın
    pencerenizde size bir şeyi nasıl gösterdiği. Hazır ajan skill'i
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md) dosyasıdır.

## Başvuru ve yardım

28. **[Yapılandırma başvurusu](config-reference.md)**: her `config.toml` ve `tui.toml` anahtarı,
    varsayılanı ve etkisi, depo başına ayarlar, ortam değişkenleri.
29. **[Sorun giderme ve SSS](troubleshooting.md)**: simgeler, grafik çizimi,
    ulaşmayan tuşlar, takılı kalmış daemon'lar, hız, LFS işaretçileri.

## Ekran görüntüleri

Bu sayfalardaki her görsel, uydurma yazarlarla küçük bir demo depoda, bir sürüm
derlemesinden alınmış gerçek bir ekran görüntüsüdür. Görseller
[`images/`](../images/) içindedir. `scripts/docs-screenshots.sh` demo depoları
oluşturur ve görüntüleri yeniden alır. Görüntüler, commit grafiğini ve yazar
avatarlarını resim olarak çizen `kitty` terminalinde alınmıştır. Grafik protokolü
olmayan terminaller bunun yerine karakterlerden oluşan bir grafik gösterir.

## Katkıda bulunanlar için

[CLAUDE.md](../../../CLAUDE.md), [mimariye genel bakış](../../architecture/overview.md)
ve [yol haritası](../../roadmap.md) tasarımı ve nelerin planlandığını anlatır;
[CHANGELOG.md](../../../CHANGELOG.md) her sürümde nelerin değiştiğini listeler.

---

GitKraken, ilgili sahibinin ticari markasıdır. GitPoulpi bağımsız bir projedir; bu marka sahibiyle bağlantılı değildir, onun tarafından onaylanmamış veya desteklenmemiştir; ad yalnızca uyumluluğu ve ilham kaynağını anlatmak için anılır.

GitPoulpi içinde ve belgelerinde anılan diğer tüm ürün ve hizmet adları ilgili sahiplerinin ticari markalarıdır; örneğin kod barındırma hizmetleri (GitHub, GitLab, Bitbucket vb.), iş takip araçları (Jira, Trello, Linear vb.) ile GitPoulpi'nin açabildiği düzenleyiciler ve araçlar (Visual Studio Code, JetBrains IDE'leri, lazygit vb.). Bu adlar yalnızca GitPoulpi'nin birlikte çalıştığı ürünleri belirtmek için kullanılır; GitPoulpi bunların hiçbiriyle bağlantılı değildir, hiçbiri tarafından onaylanmamış veya desteklenmemiştir.
