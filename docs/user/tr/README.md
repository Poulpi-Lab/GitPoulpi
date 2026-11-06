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

## Günlük kullanım

4. **[Log ile çalışmak](log-and-history.md)**: grafiği okuma, arama, dallar,
   tag'ler, stash'ler, worktree'ler, ref'leri sürükleme, rebase, cherry-pick,
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
8. **[Geri al ve yinele](undo-redo.md)**: `Ctrl+Z` / `Ctrl+Y`, nelerin geri
   alınabildiği ve alınamadığı, ne zaman reddettiği veya sorduğu, ne kadar geriye
   gittiği, kapatma.
9. **[Git LFS](lfs.md)**: büyük dosyalar, izleme, kilitler ve `git-lfs` kurulu
   değilken neler olduğu.
10. **[Alt modüller](submodules.md)**: ALT MODÜLLER bölümü, bunları güncelleme ve
    checkout'larla pull'larla uyum içinde tutma.
11. **[Temalar](themes.md)**: yerleşik temalar, her temayı her şerit paletiyle
    kullanma, canlı önizleme, kendi tema dosyalarınız, `gitpoulpi theme`.
12. **[Çalışma alanları](workspaces.md)**: kendi sekmeleri ve isteğe bağlı bir
    varsayılan commit profili olan, adlandırılmış depo grupları, sekme çubuğu
    rozeti, Çalışma alanları sekmesi, Tümünü fetch et, Tümünü pull
    et, Eksikleri klonla.
13. **[Arama](search.md)**: çalışma alanının veya açık sekmelerin tüm depolarındaki
    dallar, tag'ler, worktree'ler, stash'ler, commit'ler, pull/merge request'ler
    ve issue'lar üzerinde tek bir bulanık arama (`Ctrl+Alt+F`).
14. **[Radar](radar.md)**: bir çalışma alanının (veya açık sekmelerinizin)
    pull/merge request'leri, issue'ları ve süren işleri tek listede: filtreler,
    sabitleme, erteleme ve her satırın yapabilecekleri.
    Bu request'leri uygulamada okumak ve incelemek, yorum dizileri, bekleyen bir
    inceleme ve Onayla / Değişiklik iste ile:
    **[Bir pull veya merge request'i incelemek](review.md)**.
15. **[Şurada aç](open-in.md)**: etkin sekmenin klasörünü bir terminalde,
    çoklayıcıda, düzenleyicide veya dosya yöneticisinde açan başlık çubuğu
    listesi.
16. **[Etkin sekmenin klasörünü izlemek](following-the-tab-folder.md)**:
    "Çık ve cd yap" ve bash, zsh, fish ve PowerShell için `gitpoulpi shell-init`
    sarmalayıcısı (`gp`).
    GitPoulpi'yi en iyi hangi terminal gösterir: **[Terminal seçmek](terminals.md)**.

Ayrıca: **[Dal iş akışları](branching-workflows.md)**, Git Flow, GitHub Flow, GitLab
Flow ve Trunk-based hazır şablonları veya kendi temel dallarınız, dal adı desenleri
ve konu türleri (`gitpoulpi flow ...`).

**[Geçmişi yeniden yazma](history-rewrite.md)**: birçok commit boyunca yazarları,
tarihleri, iletileri ve dosya içeriklerini yeniden yazan yeniden yazma ekranı
(`Ctrl+G Shift+H`); önizleme, Lua betikleri ve tek bir geri alma ile.

## Klavye ve fare

17. **[Klavye kısayolları](keybindings.md)**: bölgeye göre her tuş ve paletteki
    her komut. Uygulamada `?` aynı listeyi gösterir ve `Ctrl+P` her komutu bulur.
18. **[Fare ve hareketler](mouse-and-gestures.md)**: tıklama, sürükleme, seçme
    ve kopyalama, kaydırma çubukları ve üzerine gelme geri bildirimi.

## Ayarlar ve entegrasyonlar

19. **[Ayarlar](settings.md)**: Ayarlar ekranının her sayfası ve seçeneği, arama
    kutusu, profiller ve kısayolları yeniden atama.
20. **[YZ desteği](ai.md)**: GitPoulpi'nin kendi YZ'si (varsayılan olarak
    kapalı): `claude -p` gibi yerel bir ajan, Anthropic, OpenAI, OpenRouter,
    Ollama veya LM Studio; yönergeler; commit mesajları, açıklamalar, pull
    request açıklamaları ve commit composer; gizlilik ve sorun giderme.
21. **[Barındırma hizmetleri ve issue takipçileri](hosting-integration.md)**:
    GitHub, GitLab, Gitea ve Forgejo, Azure DevOps, Bitbucket, Gerrit ve issue
    takipçileri: token'lar ve kapsamları, bağlanma, bir depoyu bağlama,
    pull request'ler ve issue'lar, commit mesajlarındaki referanslar, SSH
    anahtarları, bilinen sınırlar. Ayrıca
    **[Yerel issue'lar ve Obsidian](local-issues.md)**: issue kaynağı olarak bir
    Markdown notları klasörü, token yok, bir Obsidian Base ile.

## Otomasyon

22. **[CLI başvurusu](cli-reference.md)**: her `gitpoulpi` komutu, betikler için
    `--json` ile.
23. **[MCP ve ajanlar](mcp-and-agents.md)**: harici bir ajanın (Claude Code,
    Cursor, Codex, ...) GitPoulpi'yi yönetmesine izin verin: sunucuyu kaydetme,
    `.mcp.json`, izin anahtarları ve varsayılanları, her araç ve bir ajanın
    pencerenizde size bir şeyi nasıl gösterdiği. Hazır ajan skill'i
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md) dosyasıdır.

## Başvuru ve yardım

24. **[Yapılandırma başvurusu](config-reference.md)**: her `config.toml` anahtarı,
    varsayılanı ve etkisi, depo başına ayarlar, ortam değişkenleri.
25. **[Sorun giderme ve SSS](troubleshooting.md)**: simgeler, grafik çizimi,
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
