[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · [Français](../fr/keybindings.md) · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · **Türkçe** · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Klavye kısayolları

Bu sayfa **varsayılan** tuşları listeler. Çoğunu yeniden atayabilirsiniz. Uygulama
geçerli tuşları her zaman gösterir: komut paletinde, `?` yardımında,
durum çubuklarında, bağlam menülerinde ve düğme ipuçlarında. Bir tuşu
değiştirdiğinizde bu yerler yeni tuşu gösterir. Fare için bkz.
[Fare ve hareketler](mouse-and-gestures.md).

## Yeniden atama

**Ayarlar > Klavye ve fare** sayfasını açın. Sayfa
[Ayarlar](settings.md#klavye-ve-fare) içinde anlatılır. Değişiklikler hemen
uygulanır ve `config.toml` dosyasının `[keybindings]` tablosuna kaydedilir. Bu
tabloyu elle de düzenleyebilirsiniz:

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux ve GNU screen sıklıkla Ctrl+B ve Ctrl+A kullanır
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

Sözdizimi, çakışma kuralları ve uyarılar
[Yapılandırma başvurusu](config-reference.md) içinde `[keybindings]` başlığı
altındadır. Kısaca:

- Bir hata durum çubuğunda bildirilir ve varsayılan tuş kalır.
- Aynı bağlamda aynı tuş birleşimine iki eylem atanması reddedilir.
- Bir önek `Ctrl` veya `Alt` ister.
- Genel bir tuş birleşimini gizleyen bir panel tuşu yalnızca uyarı verir.

Aşağıdaki tablo yeniden atayabileceğiniz her tuşu listeler. Bazı tuşlar sabittir
ve yeniden atanamaz: liste hareketi (`j` `k` `h` `l`, oklar, `PageUp` /
`PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, rakamlar (sayı öneki),
metin düzenleme tuşları ve açılır pencerelerin `Enter` / `Tab` / ok tuşları.
Palette `Uygulama: Tuş Atamaları...` ve `Tuş Atamaları: Tümünü Sıfırla...`
komutları vardır.

## Varsayılan kısayollar

Yeniden atayabileceğiniz her tuş; `[keybindings.<context>]` içinde kullanılacak
kimlik ve varsayılanıyla birlikte. Tablo uygulamadan üretilir (bir test onu güncel
tutar). Bir önekten sonraki tuşlar önekle birlikte gösterilir (`Ctrl+G s`).

<!-- keymap:begin -->
| Bağlam | Eylem kimliği | Ne yapar | Varsayılan tuşlar |
|---|---|---|---|
| `global` | `git_prefix` | Git Eylemleri Öneki | `Ctrl+G` |
| `global` | `nav_prefix` | Gezinme Öneki | `Ctrl+N` |
| `global` | `palette` | Komut Paleti | `Ctrl+P` |
| `global` | `settings` | Ayarlar | `Ctrl+,` |
| `global` | `home` | Ana Sayfa (Depo Yönetimi) | `Ctrl+O` |
| `global` | `clone` | Depo Klonla | `Ctrl+Shift+N` |
| `global` | `new_tab` | Yeni Sekme | `Alt+T` |
| `global` | `workspace_switch` | Çalışma Alanını Değiştir | `Alt+W` |
| `global` | `radar` | Radar'ı Aç | `Alt+R` |
| `global` | `workspaces` | Çalışma Alanlarını Aç | `Alt+J` |
| `global` | `tab_menu` | Sekme Menüsü | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Şurada aç (liste) | `Alt+O` |
| `global` | `tab_open_in_default` | Varsayılanda aç | `Ctrl+Alt+O` |
| `global` | `close_tab` | Sekmeyi Kapat | `Ctrl+W` |
| `global` | `next_tab` | Sonraki Sekme | `Ctrl+PageDown`, `Ctrl+Tab` |
| `global` | `prev_tab` | Önceki Sekme | `Ctrl+PageUp`, `Ctrl+Shift+Tab` |
| `global` | `help` | Yardım | `?` |
| `global` | `quit` | Çık | `q` |
| `global` | `refresh` | Yenile | `r` |
| `global` | `refresh_all` | Yenile (Fetch, Pull Request'ler, issue'lar) | `F5` |
| `global` | `find_in_log` | Commit Grafiğinde Ara | `f` |
| `global` | `workspace_search` | Çalışma Alanında Ara | `Ctrl+Alt+F` |
| `global` | `select_tab` | Sekme Seç | `t` |
| `global` | `undo` | Son İşlemi Geri Al | `Ctrl+Z` |
| `global` | `redo` | Geri Alınan İşlemi Yinele | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Geri alma geçmişini aç | `Alt+Z` |
| `global` | `commit_column` | Commit sütununu göster veya gizle | `Alt+C` |
| `global` | `undo_offer` | Dalı engelli eylemin bıraktığı yere geri yükle | `Alt+Shift+Z` |
| `global` | `context_menu` | Bağlam Menüsü | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Commit Mesajı Yaz | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Stage'lenen Değişiklikleri Şimdi Commit Et | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | YZ: Değişiklikleri Commit'ler Olarak Böl | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | YZ: Değişiklikleri Açıkla | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash (Mesajla) | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | YZ: Commit Mesajı Yaz | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Tüm Değişiklikleri Stage'le | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Tüm Değişiklikleri At | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Seçili Dosyayı Stage'le | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Seçili Dosyayı Unstage'le | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `pull_all_branches` | Pull: Tüm Dallar | `Ctrl+G Ctrl+P` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Bağlı Pull Request Aç | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push: Tüm Tag'ler | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch: Tüm Uzak Depolar | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Yeni Tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Yeni Dal | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash: Değişiklikler | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Seçili Commit Cherry-pick | `Ctrl+G y` |
| `git_prefix` | `revert` | Seçili Commit Revert | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Commit Mesajını Düzenle | `Ctrl+G e` |
| `git_prefix` | `amend` | Seçili Commit Öğesini Değiştir | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Seçili Commit Öğesinin Yazarını Değiştir | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Seçili Commit İçin Reset (Mixed) | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase: Seçili Dalın Üstüne | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop Stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Seçili Ref'i Sil veya Stash At | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | LFS menüsünü aç | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Flow menüsünü aç | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS: Fetch (tüm nesneler) | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS: Checkout (tüm dosyalar) | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS: Pull (nesneler) | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS: Push (tüm nesneler) | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS: Yerel Nesneleri Temizle | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Geçmişi Yeniden Yazma Ekranını Aç | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Refs Panelini Daralt veya Geri Yükle | `Ctrl+N Shift+H` |
| `nav_prefix` | `collapse_status` | Durum Panelini Daralt veya Geri Yükle | `Ctrl+N Shift+L` |
| `nav_prefix` | `settings` | Ayarları Aç | `Ctrl+N ,` |
| `nav_prefix` | `about` | Hakkında Penceresini Aç | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Etkinlik Günlüğünü Aç | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Genel etkinlik günlüğünü aç | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Arka plan görevlerini göster | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Güncelleme Panelini Göster | `Ctrl+N u` |
| `nav_prefix` | `help` | Yardımı Aç/Kapat | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Sekme Seç | `Ctrl+N Tab` |
| `nav_prefix` | `change_theme` | Temayı Değiştir | `Ctrl+N t` |
| `nav_prefix` | `change_language` | Dili Değiştir | `Ctrl+N l` |
| `nav_prefix` | `change_graph_color_theme` | Grafik Renk Temasını Değiştir | `Ctrl+N Shift+T` |
| `nav_prefix` | `toggle_graph_row_tints` | Grafik Satır Tonunu Aç/Kapat | `Ctrl+N Shift+Y` |
| `nav_prefix` | `grow_section` | Refs Bölümünü Büyüt | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Refs Bölümünü Küçült | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Commit Mesaj Bloğunu Büyüt | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Commit Mesaj Bloğunu Küçült | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Refs Panelini Genişlet | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Refs Panelini Daralt | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Durum Panelini Genişlet | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Durum Panelini Daralt | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Ajan: Bekleyen Görünümü Göster | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Ajan: Geri (veya Bekleyen Görünümü Kapat) | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | Açık Pencereyi Düzenle (Taşı, Boyutlandır) | `Ctrl+N m` |
| `refs` | `filter` | Ref'leri Ada Göre Filtrele | `/` |
| `refs` | `solo` | Seçimi Solo Yap | `o` |
| `refs` | `clear_solo` | Solo'yu Durdur | `Shift+O` |
| `refs` | `toggle_hierarchy` | Dal Hiyerarşisi Görünümünü Aç/Kapat | `y` |
| `refs` | `show_all` | Gizli Tüm Ref'leri Göster | `Shift+H` |
| `refs` | `reveal` | Grafikte göster | `g` |
| `refs` | `mark_drop` | Bırakma Kaynağı Olarak İşaretle | `m` |
| `refs` | `drop_marked` | İşaretli Ref'i Buraya Bırak | `Shift+M` |
| `refs` | `expand_all` | Tüm Klasörleri Genişlet | `=` |
| `refs` | `collapse_all` | Tüm Klasörleri Daralt | `-` |
| `refs` | `expand_below` | Klasörü ve Alt Klasörlerini Genişlet | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Klasörü ve Alt Klasörlerini Daralt | `_`, `Shift+Left` |
| `refs` | `scroll_left` | Paneli sola kaydır | `Alt+Left` |
| `refs` | `scroll_right` | Paneli sağa kaydır | `Alt+Right` |
| `log` | `toggle_file_mode` | Dosyalar: Yol veya Ağaç | `m` |
| `log` | `lfs_only` | Dosyalar: Yalnızca LFS | `o` |
| `log` | `view_all_files` | Dosyalar: Tüm Dosyaları Göster | `a` |
| `log` | `compare_selection` | Seçili satırları karşılaştır | `v` |
| `log` | `compare_swap` | Karşılaştırma uçlarını değiştir | `x` |
| `log` | `compare_next` | Sonraki karşılaştırılan satıra git | `n` |
| `log` | `filter_author` | Log'u yazara göre filtrele | `Shift+A` |
| `log` | `pick_column` | Taşımak veya boyutlandırmak için bir sütun seç | `Shift+C` |
| `log` | `scroll_left` | Sütunları Sola Kaydır | `Shift+Left` |
| `log` | `scroll_right` | Sütunları Sağa Kaydır | `Shift+Right` |
| `log` | `details_scroll_left` | Paneli sola kaydır | `Alt+Left` |
| `log` | `details_scroll_right` | Paneli sağa kaydır | `Alt+Right` |
| `log` | `copy_mermaid` | Commit'ler Mermaid gitGraph Olarak Kopyala | `Shift+Y` |
| `log_columns` | `next` | Sonraki sütunu seç | `Tab` |
| `log_columns` | `prev` | Önceki sütunu seç | `Shift+BackTab` |
| `log_columns` | `move_left` | Sütunu sola taşı | `<` |
| `log_columns` | `move_right` | Sütunu sağa taşı | `>` |
| `log_columns` | `wider` | Sütunu genişlet | `]` |
| `log_columns` | `narrower` | Sütunu daralt | `[` |
| `log_columns` | `reset_width` | Sütun genişliklerini sıfırla | `0` |
| `log_columns` | `fit` | Sütunu İçeriğine Sığdır | `=` |
| `log_columns` | `fit_all` | Tüm Sütunları İçeriğe Sığdır | `+` |
| `log_columns` | `layout` | Yana Kaydır veya Bölmeye Sığdır | `s` |
| `log_files` | `collapse_all` | Tüm Klasörleri Daralt veya Genişlet | `z` |
| `log_files` | `filter` | Değişen Dosyaları Filtrele | `/` |
| `log_files` | `expand_all` | Tüm Klasörleri Genişlet | `=` |
| `log_files` | `collapse_all_folders` | Tüm Klasörleri Daralt | `-` |
| `log_files` | `expand_below` | Klasörü ve Alt Klasörlerini Genişlet | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Klasörü ve Alt Klasörlerini Daralt | `_`, `Shift+Left` |
| `log_files` | `scroll_left` | Paneli sola kaydır | `Alt+Left` |
| `log_files` | `scroll_right` | Paneli sağa kaydır | `Alt+Right` |
| `status` | `compose` | Commit Mesajına Odaklan | `c` |
| `status` | `toggle_file_mode` | Dosyalar: Yol veya Ağaç | `m` |
| `status` | `lfs_only` | Dosyalar: Yalnızca LFS | `o` |
| `status` | `view_all_files` | Dosyalar: Tüm Dosyaları Göster | `a` |
| `status` | `amend` | Amend'i Aç/Kapat | `Shift+A` |
| `status` | `skip_hooks` | Hook'ları Atlamayı Aç/Kapat | `Shift+H` |
| `status` | `collapse_all` | Tüm Klasörleri Daralt veya Genişlet | `z` |
| `status` | `stage_toggle` | Seçili Satırı Stage'le veya Unstage'le | `Ctrl+Space` |
| `status` | `open_diff` | Seçili Dosyanın Diff Görünümünü Aç | `d` |
| `status` | `expand_all` | Tüm Klasörleri Genişlet | `=` |
| `status` | `collapse_all_folders` | Tüm Klasörleri Daralt | `-` |
| `status` | `expand_below` | Klasörü ve Alt Klasörlerini Genişlet | `*`, `Shift+Right` |
| `status` | `collapse_below` | Klasörü ve Alt Klasörlerini Daralt | `_`, `Shift+Left` |
| `status` | `scroll_left` | Paneli sola kaydır | `Alt+Left` |
| `status` | `scroll_right` | Paneli sağa kaydır | `Alt+Right` |
| `home` | `favorite` | Depoyu Yıldızla veya Yıldızı Kaldır | `f`, `*` |
| `home` | `close_repo` | Satırı Kapat, Kaldır veya Yıldızını Kaldır | `x`, `Delete` |
| `home` | `copy_path` | Depo Yolunu Kopyala | `y` |
| `home` | `open_in` | Depoyu Varsayılan Araçta Aç | `o` |
| `home` | `browse` | Depo Gözat | `b` |
| `home` | `clone` | Depo Klonla | `c` |
| `home` | `init` | Depo Başlat | `i` |
| `home` | `collapse_all` | Tüm Bölümleri Daralt | `-` |
| `home` | `expand_all` | Tüm Bölümleri Genişlet | `+`, `=` |
| `home` | `search` | Depolarda Ara | `/` |
| `home` | `workspace_new` | Yeni Çalışma Alanı | `n` |
| `home` | `workspace_edit` | Çalışma Alanını Düzenle | `e` |
| `home` | `workspace_from_tabs` | Açık Sekmelerden Yeni Çalışma Alanı | `a` |
| `home` | `workspace_fetch` | Çalışma Alanında Tümünü Fetch Et | `Shift+F` |
| `home` | `workspace_pull` | Çalışma Alanında Tümünü Pull Et | `Shift+P` |
| `home` | `workspace_clone_missing` | Eksik Depoları Klonla | `m` |
| `home` | `workspace_locate` | Eksik Depoyu Bul | `l` |
| `refs` | `hosting_filter` | Pull Request'ler / Issue'lar: Filtre Seç | `v` |
| `refs` | `hosting_edit` | Pull Request: Barındırıcıda Düzenle | `e` |
| `refs` | `new_pull_request` | Pull Request'ler: Yeni Oluştur | `n` |
| `refs` | `add_remote` | Uzak Depolar: Uzak Depo Ekle | `+` |
| `refs` | `remotes` | Dal: Uzak Depolarını Düzenle | `u` |
| `diff` | `close` | Diff Görünümünü Kapat | `q` |
| `diff` | `edit` | Çalışma Ağacında Düzenle | `e` |
| `diff` | `lfs_raw` | LFS: Kart veya Ham İşaretçi | `r` |
| `diff` | `copy` | Seçimi veya Satırı Kopyala | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Hunk'ı Stage'le veya Unstage'le | `Space` |
| `diff` | `stage_lines` | Seçili Satırları Stage'le veya Unstage'le | `Enter` |
| `diff` | `discard_hunk` | Hunk'ı At | `x` |
| `diff` | `discard_lines` | Seçili Satırları At | `Shift+X` |
| `diff` | `select_lines` | Satır Seçimini Başlat veya Bırak | `v` |
| `diff` | `next_hunk` | Sonraki Hunk | `n` |
| `diff` | `prev_hunk` | Önceki Hunk | `p` |
| `diff` | `toggle_layout` | Yerleşim: Satır İçi veya Bölünmüş | `m` |
| `diff` | `toggle_hunks_only` | Yalnızca Hunk'lar veya Tüm Dosya | `Shift+H` |
| `diff` | `toggle_minimap` | Minimap'i Aç/Kapat | `Shift+M` |
| `diff` | `choose_encoding` | Kodlamayı Seç | `Shift+C` |
| `diff` | `toggle_wrap` | Sözcük Kaydırmayı Aç/Kapat | `w` |
| `diff` | `toggle_whitespace` | Yalnızca Boşluk Değişikliklerini Göster veya Gizle | `s` |
| `diff` | `merge_editor` | Merge Düzenleyiciyi Aç (Çakışmalı Dosya) | `Shift+E` |
| `file_insight` | `close` | Dosya Görünümünü Kapat | `q` |
| `file_insight` | `edit` | Çalışma Ağacında Düzenle | `e` |
| `file_insight` | `copy` | Seçimi veya Satırı Kopyala | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Son Satıra Atla | `Shift+G` |
| `file_insight` | `open_commit` | Satırın Commit Kaydına Git | `Enter` |
| `history` | `show_in_log` | Log'da göster | `Enter` |
| `history` | `copy_sha` | Commit SHA'sını kopyala | `y` |
| `history` | `copy_summary` | Özeti kopyala | `Shift+Y` |
| `history` | `show_diff` | Bu Commit öğesinin diff'ini göster | `d`, `Alt+1` |
| `history` | `open_file` | Dosyayı bu Commit anında aç | `o`, `Alt+2` |
| `history` | `blame` | Bu Commit anında Blame | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Çalışma ağacı ile karşılaştır | `c` |
| `history` | `restore` | Bu sürümü geri yükle | `Shift+R` |
| `history` | `range_up` | Seçimi yukarı genişlet | `Shift+Up` |
| `history` | `range_down` | Seçimi aşağı genişlet | `Shift+Down` |
| `history` | `switch_focus` | Liste ile diff arasında geçiş yap | `Tab` |
| `history` | `widen_list` | Listeyi genişlet | `Alt+Right` |
| `history` | `narrow_list` | Listeyi daralt | `Alt+Left` |
| `history` | `reset_split` | Liste genişliğini sıfırla | `Alt+0` |
| `commit_column` | `scroll_left` | Sütunu sola kaydır | `Shift+Left` |
| `commit_column` | `scroll_right` | Sütunu sağa kaydır | `Shift+Right` |
| `commit_column` | `next_part` | Sonraki bölümü seç | `Tab` |
| `commit_column` | `move_left` | Bölümü sola taşı | `<` |
| `commit_column` | `move_right` | Bölümü sağa taşı | `>` |
| `commit_column` | `wider_part` | Bölümü genişlet | `]` |
| `commit_column` | `narrower_part` | Bölümü daralt | `[` |
| `commit_column` | `reset_widths` | Bölüm genişliklerini sıfırla | `0` |
| `commit_column` | `fit` | Sütunu İçeriğine Sığdır | `=` |
| `commit_column` | `fit_all` | Tüm Sütunları İçeriğe Sığdır | `+` |
| `file_insight` | `blame_card` | Blame: Ayrıntı kartını göster veya gizle | `i` |
| `file_insight` | `blame_next` | Blame: Sonraki hunk | `n` |
| `file_insight` | `blame_prev` | Blame: Önceki hunk | `p` |
| `file_insight` | `blame_next_same` | Blame: Aynı Commit öğesinin sonraki hunk'ı | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame: Aynı Commit öğesinin önceki hunk'ı | `Shift+P` |
| `file_insight` | `blame_goto` | Blame: Satıra git | `g` |
| `file_insight` | `blame_copy_sha` | Blame: Commit SHA değerini kopyala | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame: Commit diff'ini göster | `d` |
| `file_insight` | `blame_parent` | Blame: Commit ebeveyninde blame yap | `b` |
| `file_insight` | `blame_history` | Blame: Dosya geçmişini aç | `Shift+H` |
| `file_insight` | `blame_wider` | Blame: Açıklama sütununu genişlet | `>` |
| `file_insight` | `blame_narrower` | Blame: Açıklama sütununu daralt | `<` |
| `image` | `zoom_in` | Görseli yakınlaştır | `+`, `=` |
| `image` | `zoom_out` | Görseli uzaklaştır | `-` |
| `image` | `fit` | Görseli görünüme sığdır | `0` |
| `image` | `actual_size` | Görseli gerçek boyutta göster | `1` |
| `image` | `flip_side` | Diğer tarafı göster (önce / sonra) | `Tab` |
| `image` | `widen_before` | Önce bölmesini genişlet | `Alt+Right` |
| `image` | `widen_after` | Sonra bölmesini genişlet | `Alt+Left` |
| `image` | `reset_split` | Önce / sonra bölünmesini sıfırla | `Alt+0` |
| `image` | `edit_in` | Birlikte aç... | `Shift+O` |
| `image` | `edit_default` | Varsayılan uygulamayla aç | `o` |
| `editor` | `save` | Kaydet | `Ctrl+S` |
| `editor` | `undo` | Geri Al | `Ctrl+Z` |
| `editor` | `redo` | Yinele | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Bul | `Ctrl+F` |
| `editor` | `find_next` | Sonrakini Bul (Shift: Öncekini) | `F3` |
| `editor` | `paste` | Yapıştır | `Ctrl+V` |
| `editor` | `toggle_minimap` | Minimap'i Aç/Kapat | `Alt+M` |
| `editor_prompt` | `save` | Kaydedilmemiş Değişiklikler: Kaydet ve Kapat | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Kaydedilmemiş Değişiklikler: At ve Kapat | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Kaydedilmemiş Değişiklikler: Düzenlemeye Devam Et | `c`, `Shift+C` |
| `merge` | `next_conflict` | Sonraki Çakışma | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Önceki Çakışma | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Sonraki Çakışmalı Dosya | `Alt+PageDown` |
| `merge` | `prev_file` | Önceki Çakışmalı Dosya | `Alt+PageUp` |
| `merge` | `next_pane` | Sonraki Bölmeye Odaklan | `F6` |
| `merge` | `prev_pane` | Önceki Bölmeye Odaklan | `Shift+F6` |
| `merge` | `toggle_panes` | Yan Panelleri Göster veya Gizle (Refs ve Durum) | `F9` |
| `merge` | `toggle_base` | Temel Bölmesini Göster veya Gizle | `Alt+D` |
| `merge` | `toggle_full_file` | Tüm Dosya Diff'ini Aç/Kapat | `Alt+F` |
| `merge` | `undo` | Geri Al (Seçimler ve Yazma) | `Ctrl+Z` |
| `merge` | `redo` | Yinele (Seçimler ve Yazma) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Mevcut (A) Bölmesini Genişlet | `Alt+Right` |
| `merge` | `widen_incoming` | Gelen (B) Bölmesini Genişlet | `Alt+Left` |
| `merge` | `reset_panes` | Bölme Boyutlarını Sıfırla | `Alt+0` |
| `merge` | `save` | Çözüldü İşaretlemeden Kaydet | `Alt+S` |
| `merge` | `save_resolve` | Kaydet ve Çözüldü İşaretle | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Çözüm Öner (YZ) | `Alt+I` |
| `merge` | `unpick` | Bu Çakışmanın Seçimini Kaldır (Çözülmemişe Dön) | `Alt+U` |
| `merge` | `reset_all` | Tüm Dosyayı Özgün Çakışmalara Sıfırla | `Alt+R` |
| `merge` | `focus_a` | Mevcut (A) Bölmesine Odaklan | `Alt+1` |
| `merge` | `focus_b` | Gelen (B) Bölmesine Odaklan | `Alt+2` |
| `merge` | `focus_output` | Çıktı Bölmesine Odaklan | `Alt+3` |
| `merge` | `focus_base` | Temel Bölmesine Odaklan | `Alt+4` |
| `merge_sides` | `toggle_line` | Satırı Seç veya Bırak | `Space` |
| `merge_sides` | `toggle_chunk` | Tüm Parçayı Seç veya Bırak | `Enter` |
| `merge_sides` | `take_a` | Yalnızca Mevcut (A) Al | `a` |
| `merge_sides` | `take_b` | Yalnızca Gelen (B) Al | `b` |
| `merge_sides` | `take_both` | İkisini de Al, Önce A Sonra B | `c` |
| `merge_sides` | `take_both_reversed` | İkisini de Al, Önce B Sonra A | `Shift+C` |
| `merge_sides` | `clear` | Bu Çakışma İçin Hiçbirini Alma | `x` |
| `merge_sides` | `unpick` | Bu Çakışmanın Seçimini Kaldır (Çözülmemişe Dön) | `u` |
| `merge_sides` | `reset_all` | Tüm Dosyayı Özgün Çakışmalara Sıfırla | `Shift+U` |
| `merge_sides` | `swap_order` | İki Tarafın Sırasını Değiştir | `s` |
| `merge_sides` | `next_conflict` | Sonraki Çakışma | `n` |
| `merge_sides` | `prev_conflict` | Önceki Çakışma | `Shift+N` |
| `merge_sides` | `take_all_a` | Tüm Mevcut (A) Al | `Shift+A` |
| `merge_sides` | `take_all_b` | Tüm Gelen (B) Al | `Shift+B` |
| `merge_sides` | `toggle_base` | Temel Bölmesini Göster veya Gizle | `d` |
| `merge_sides` | `scroll_left` | Bölmeleri Sola Kaydır | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Bölmeleri Sağa Kaydır | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Tüm Dosya Diff'ini Aç/Kapat | `f` |
| `merge_sides` | `cycle_height` | Çıktı Yüksekliğini Döngüyle Değiştir | `z` |
| `merge_card` | `take_a` | A'yı (Mevcut) Olduğu Gibi Koru | `a` |
| `merge_card` | `take_b` | B'yi (Gelen) Olduğu Gibi Al | `b` |
| `merge_card` | `take_base` | Temeli Al (Ortak Ata) | `o` |
| `merge_card` | `keep` | Kalan Dosyayı Koru | `k` |
| `merge_card` | `delete` | Silmeyi Kabul Et | `Shift+D` |
| `merge_card` | `external_tool` | Harici Merge Aracında Aç | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (Amend İçin Dur) | `e` |
| `rebase` | `autosquash` | Autosquash'i Aç/Kapat | `a` |
| `rebase` | `start` | Planı Başlat | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Planı Sıfırla | `Ctrl+R` |
| `rebase` | `split` | Satırdaki Commit'i Böl | `t` |
| `rebase` | `move_up` | Satırı Yukarı Taşı | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Satırı Aşağı Taşı | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Commit'ler Oluştur | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Satır Seçimini Aç/Kapat | `v` |
| `split` | `move_up` | Commit'i Yukarı Taşı | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Commit'i Aşağı Taşı | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Bölmeyi Sıfırla | `Shift+R` |
| `rewrite` | `preview` | Yeniden Yazmayı Önizle | `F5` |
| `rewrite` | `apply` | Yeniden Yazmayı Uygula | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Önizlemeyi İptal Et | `Ctrl+X` |
| `rewrite` | `add_rule` | Kural Ekle | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Kuralı Aç veya Kapat | `Space` |
| `rewrite` | `move_up` | Kuralı Yukarı Taşı | `Alt+Up` |
| `rewrite` | `move_down` | Kuralı Aşağı Taşı | `Alt+Down` |
| `rewrite` | `delete_rule` | Kuralı Sil | `d`, `Delete` |
| `rewrite` | `restore_rule` | Silinen Kuralı Geri Getir | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Kuralı Çoğalt | `Ctrl+D` |
| `rewrite` | `next_tab` | Sonraki Önizleme Sekmesi | `Alt+Right` |
| `rewrite` | `prev_tab` | Önceki Önizleme Sekmesi | `Alt+Left` |
| `rewrite` | `save_shared` | Kural Setini Depoya Kaydet | `Alt+S` |
| `rewrite` | `save_mine` | Kural Setini Yalnızca Bana Kaydet | `Alt+M` |
| `rewrite` | `load` | Kural Seti Yükle | `Alt+L` |
| `rewrite` | `name` | Kural Setini Yeniden Adlandır | `F2` |
| `rewrite` | `protected_all` | Tüm Korumalı Dalları Dahil Et | `Alt+A` |
| `rewrite` | `protected_none` | Tüm Korumalı Dalları Hariç Tut | `Alt+U` |
| `rewrite` | `check_tools` | Araçları Yeniden Denetle | `F6` |
| `rewrite` | `analyze` | Geçmiş Boyutlarını Analiz Et | `F7` |
| `rewrite` | `host_howto` | Barındırıcının Dal Korumasını Açıkla | `Alt+H` |
| `rewrite` | `ai_rules` | YZ ile yeniden yazma kuralları oluştur | `Alt+I` |
| `reword` | `apply` | Yeni Mesajı Uygula | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Geçmişi daha kapsamlı değiştir | `Alt+H` |
| `composer` | `commit` | Composer'dan Commit | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Issue referansı ekle | `Alt+R` |
| `md_editor` | `preview` | Markdown Düzenleyici: Yaz ve Önizleme Arasında Geç | `Alt+P` |
| `md_editor` | `split` | Markdown Düzenleyici: Yan Yana Görünümü Aç veya Kapat | `Alt+L` |
| `md_editor` | `submit` | Markdown Düzenleyici: Metni Gönder | `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` |
| `md_editor` | `ai` | Markdown Düzenleyici: YZ ile Yaz | `Alt+E` |
| `dialog` | `browse` | Klasör Gözat | `Ctrl+B` |
| `dialog` | `sources` | Klon kaynaklarına git | `Alt+S` |
| `dialog` | `submit` | Pull Request Formunu Gönder | `Ctrl+S` |
| `dialog` | `web` | Sağlayıcının web formunu aç | `Alt+W` |
| `folder_picker` | `choose` | Klasörü Seç | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Yeni Klasör | `Alt+N` |
| `folder_picker` | `filter` | Klasör Girdilerini Filtrele | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Kardeş Klasörleri Göster veya Gizle | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Gizli Klasörleri Göster veya Gizle | `Alt+H` |
| `preferences` | `search` | Ayarlarda Ara | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Liste Öğesi Ekle | `Insert` |
| `preferences` | `list_duplicate` | Seçili Liste Öğesini Çoğalt | `Ctrl+D` |
| `preferences` | `list_remove` | Seçili Liste Öğesini Kaldır | `Delete` |
| `preferences` | `list_move_up` | Seçili Liste Öğesini Yukarı Taşı | `Alt+Up` |
| `preferences` | `list_move_down` | Seçili Liste Öğesini Aşağı Taşı | `Alt+Down` |
| `preferences` | `prev_tab` | Ayarlar Sayfasının Önceki Sekmesi | `Alt+Left` |
| `preferences` | `next_tab` | Ayarlar Sayfasının Sonraki Sekmesi | `Alt+Right` |
| `ai_popup` | `copy` | Sonucu Kopyala | `c` |
| `activity_log` | `scroll_left` | Etkinlik Günlüğünü Sola Kaydır | `Left` |
| `activity_log` | `scroll_right` | Etkinlik Günlüğünü Sağa Kaydır | `Right` |
| `integrations` | `add` | Profil Ekle | `a` |
| `integrations` | `edit` | Seçili Profili Düzenle | `e` |
| `integrations` | `open_token` | Token Sayfasını Aç | `o` |
| `integrations` | `remove` | Seçili Profili Kaldır | `d`, `Delete` |
| `prompt` | `ai_stash_message` | YZ: Stash Mesajı Oluştur | `Ctrl+G` |
<!-- keymap:end -->

### Terminalin gönderemediği tuşlar ve kitty klavye protokolü

Eski bir terminal bazı tuş birleşimlerini gönderemez: `Ctrl+,`, `Ctrl+Shift+N`,
`Ctrl+Enter` ve `Ctrl+<rakam>`. Hiçbir şey göndermez ya da başka bir tuş gönderir:
örneğin `Ctrl+Shift+N` `Ctrl+N` olarak, `Ctrl+I` `Tab` olarak ve `Ctrl+M` `Enter`
olarak gelir. Ayarlar sayfası böyle bir atama için uyarır ("bu terminal ...
göndermez").

**Kitty klavye protokolü** olan terminaller bu tuş birleşimlerini gönderebilir.
Örnekler: kitty, WezTerm, Ghostty, foot, Alacritty 0.13 ve sonrası ile zellij (tmux
yanıt vermez: bkz. [Çoklayıcılar](multiplexers.md#tmux)). GitPoulpi protokolü varsayılan olarak kullanır:

- Başlangıçta GitPoulpi terminale sorar ve yanıt için en fazla yaklaşık 100 ms
  bekler. Terminal yanıt vermezse eski tuşlar kalır.
- Terminal yanıt verirse GitPoulpi protokolü açar. Çıkarken, hata verirken veya
  çökerken yeniden kapatır.
- Ayarlar sayfasındaki uyarılar protokolü izler: açıkken yukarıdaki tuş
  birleşimleri gönderilebilir.
- **Ayarlar > Klavye ve fare > Kitty klavye protokolü** (`ui.kitty_keyboard`)
  `off` yapılırsa terminalin klavye kipine hiç dokunulmaz. Bir terminal veya
  çoklayıcı sorun çıkarıyorsa kullanın. Yeniden başlatma gerekir.

Windows konsolları değiştirici tuşları kendileri bildirir ve protokole ihtiyaç
duymaz.

## Komut paleti (`Ctrl+P`)

Palet uygulamanın **her** komutunu listeler, böylece kısayolsuz her şeye
ulaşırsınız. Tam liste [aşağıdadır](#her-palet-komutu).

| Tuş | Eylem |
|---|---|
| `Ctrl+P` | Paleti aç |
| *(yaz)* | Listeyi filtrele. `Kategori: ` önekinden sonraki tam eşleşme önce gelir, sonra etiketin herhangi bir yerindeki eşleşme, sonra bulanık eşleşme |
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Vurguyu taşı (bkz. [Filtrelenebilir açılır listeler](#filtrelenebilir-açılır-listeler)) |
| `Enter` veya çift tık | Vurgulanan komutu çalıştır |
| `Esc` | Kapat |

`j` ve `k` filtreye yazılır. Vurguyu taşımazlar.

Her satır kısayolunu sağda gösterir. Şu anda geçerli olmayan bir komut (örneğin
seçili ref yok) listede gri olarak kalır ve nedeni listenin altında görünür.
Etiketler bir kategoriyle başlar; örneğin `Gezinme:`, `Refs:`, `Git:`,
`Görünüm:`, `Diff:` veya `Uygulama:`. Tüm grubu görmek için bir kategori yazın.

### Her komut için bir tuş

Paletin her satırı bir tuş düğmesiyle (`[k]`) biter ve `Alt+K` vurgulu satırda aynı işi yapar: o komutun kısayoluna göre süzülmüş **Ayarlar > Klavye ve fare** sayfasını açar, yeniden atamaya hazırdır. Kendi kısayolu olmayan bir komut burada bir tane alır: **Palet komutları** altında listelenir, varsayılan olarak atanmamıştır ve ona verdiğiniz tuş, komutu palette seçmekle aynı şekilde çalıştırır (şu an çalıştırılamayan bir komut nedenini söyler). Satır bundan sonra tuşunuzu gösterir.

`config.toml` içinde bu, komutun adını taşıyan `[keybindings.palette]` tablosudur (`toggle_minimap = "Ctrl+Alt+M"`). Başka bir eylemle çakışan tuş kırmızı gösterilir ve satırında `c` tuşu, çakıştığı tüm eylemleri listeler.

Listenin altındaki ipucu çubuğu bu tuşu gösterir (varsayılan `Alt+K`). Bu, `palette.configure_shortcut` eylemidir; diğerleri gibi yeniden atanabilir.

Palet ve seçiciler (tema, dil, sekme, çalışma alanı araması) bir pencere gibi taşınır ve boyutlandırılır: üst kenarlığı sürükleyin, `Ctrl+N` ardından `m` tuşlarına basın (her pencerede olduğu gibi) ya da `Alt+G` tuşuna basın. `Alt+G`, `palette.arrange` eylemidir ("Pencere: Öndeki Pencereyi Taşı veya Boyutlandır"): her pencerede ve açılır pencerede öndekinin düzenleme kipini başlatan ek bir yoldur (paletteki `Pencere: Taşı veya Boyutlandır` ve `Pencere: Konumu ve Boyutu Sıfırla` satırları palet açıkken paletin kendisi üzerinde çalışır). Oklar taşır, `Shift`+oklar boyutlandırır, `0` sıfırlar, `Enter` veya `Esc` kipten çıkar ve açılır pencere açık kalır. Yerin korunup korunmayacağını `ui.dialog_geometry` belirler.

`Ctrl+N` burada da gezinme önekidir: filtre kutusunda, listenin ilk satırında ve kapatma çarpısında bir rozet gösterir; ardından gelen tuş o denetime basar ya da `m` açılır pencereyi düzenler. Artık vurguyu aşağı taşımaz, `Ctrl+P` de yukarı taşımaz: filtre alanı olan bir listeyi yalnızca oklar, `PageUp` / `PageDown` ve `Home` / `End` taşır.

### Her palet komutu

Uygulamadan üretilir (bir test onu güncel tutar). YZ komutları yalnızca YZ
açıkken görünür; diğerleri geçerli olmadıklarında nedenleriyle birlikte gri olur.
Tuşlar varsayılanlardır.

<!-- palette:begin -->
| Komut | Varsayılan tuş |
|---|---|
| Gezinme: Refs Paneline Odaklan | `Ctrl+N <badge>` |
| Gezinme: Log Paneline Odaklan | `Ctrl+N <badge>` |
| Gezinme: Durum Paneline Odaklan | `Ctrl+N <badge>` |
| Gezinme: Refs Filtresine Odaklan | `Ctrl+N <badge>` |
| Gezinme: Refs > Yerel'e Odaklan | `Ctrl+N <badge>` |
| Gezinme: Refs > Uzak'a Odaklan | `Ctrl+N <badge>` |
| Gezinme: Refs > Worktree'lere Odaklan | `Ctrl+N <badge>` |
| Gezinme: Refs > Alt Modüllere Odaklan | `Ctrl+N <badge>` |
| Gezinme: Refs > Stash'lere Odaklan | `Ctrl+N <badge>` |
| Gezinme: Refs > Tag'lere Odaklan | `Ctrl+N <badge>` |
| Sol panel: Bölümleri Göster/Gizle... |  |
| Sol panel: Bölümleri Varsayılana Sıfırla |  |
| Sol panel: Bu Bölümleri Tüm Depolar İçin Kullan |  |
| Sol panel: Yerel Dallar Bölümünü Göster/Gizle |  |
| Sol panel: Uzak Depolar Bölümünü Göster/Gizle |  |
| Sol panel: Worktree'ler Bölümünü Göster/Gizle |  |
| Sol panel: Worktree'ler Bölümünü Göster |  |
| Sol panel: Stash'ler Bölümünü Göster/Gizle |  |
| Sol panel: Tag'ler Bölümünü Göster/Gizle |  |
| Sol panel: Pull Request'ler Bölümünü Göster/Gizle |  |
| Sol panel: Issue'lar Bölümünü Göster/Gizle |  |
| Sol panel: Alt Modüller Bölümünü Göster/Gizle |  |
| Gezinme: Durum > Unstaged'a Odaklan | `Ctrl+N <badge>` |
| Gezinme: Durum > Staged'a Odaklan | `Ctrl+N <badge>` |
| Gezinme: Odağı Sonraki Panele Geçir | `Tab` |
| Gezinme: Odağı Önceki Panele Geçir | `Shift+Tab` |
| Gezinme: Commit'lerde Ara... | `f` |
| Log: Yazara Göre Filtrele... | `A` |
| Log: Yazar Filtresini Temizle |  |
| Gezinme: Refs Panelini Daralt/Genişlet | `Ctrl+N H` |
| Gezinme: Refs Panelini Genişlet | `Ctrl+N Right` |
| Gezinme: Refs Panelini Daralt | `Ctrl+N Left` |
| Gezinme: Durum Panelini Genişlet | `Ctrl+N Shift+Left` |
| Gezinme: Durum Panelini Daralt | `Ctrl+N Shift+Right` |
| Gezinme: Durum Panelini Daralt/Genişlet | `Ctrl+N L` |
| Gezinme: Refs Bölümünü Büyüt | `Ctrl+N +` |
| Gezinme: Refs Bölümünü Küçült | `Ctrl+N -` |
| Gezinme: Refs Bölüm Boyutlarını Sıfırla |  |
| Gezinme: Yerleşimi Sıfırla |  |
| Commit Ayrıntıları: Mesajı Büyüt | `Ctrl+N ]` |
| Commit Ayrıntıları: Mesajı Küçült | `Ctrl+N [` |
| Commit Ayrıntıları: Mesaj Boyutunu Sıfırla |  |
| Commit Ayrıntıları: Mesajı Aşağı Kaydır |  |
| Commit Ayrıntıları: Mesajı Yukarı Kaydır |  |
| Commit Ayrıntıları: Mesajdaki Bağlantıyı Aç... |  |
| Dosyalar: Commit Dosya Sırasını Değiştir (A-Z / Z-A) |  |
| Refs: Seçili Ref'te Checkout | `çift tık` |
| Refs: Ref Adını Kopyala |  |
| Refs: Tam Ref'i Kopyala (refs/heads/...) |  |
| Refs: Ref Hedef SHA'sını Kopyala |  |
| Refs: Seçili Dalı Geçerliye Merge Et |  |
| Refs: Geçerli Dalı Seçilinin Üstüne Rebase Et | `Ctrl+G r` |
| Refs: Seçili Ref'i Checkout Yapılmış Dala Bırak... |  |
| Refs: Bırakma Kaynağı Olarak İşaretle | `m` |
| Refs: İşaretli Ref'i Buraya Bırak... | `M` |
| Refs: Seçili Dalın Adını Değiştir... |  |
| Refs: Seçili Dalı Geçerliye Fast-Forward Et |  |
| Refs: Geçerli Dalı Seçiliye Fast-Forward Et |  |
| Refs: Seçili Dal veya Tag'den Worktree Oluştur... |  |
| Refs: Seçili Dalın Reflog'u... |  |
| Refs: Seçili Daldan Pull Request Başlat... |  |
| Refs: Worktree'yi Sekmede Aç | `Enter / çift tık` |
| Refs: Worktree'yi Şurada Aç... |  |
| Refs: Worktree Dalını Başka Dala Merge Et... |  |
| Refs: Worktree'yi Kaldır... |  |
| Refs: Worktree Oluştur... | `+` |
| Refs: Geçerli Daldan Worktree Oluştur |  |
| Refs: Dal Seçerek Worktree Oluştur |  |
| Refs: Alt Modül Güncelle (Seçili) |  |
| Refs: Alt Modül Başlat (Seçili) |  |
| Refs: Alt Modülü Sekmede Aç | `Enter / çift tık` |
| Git: Tüm Alt Modülleri Güncelle |  |
| Git: Seçili Commit'te Worktree Oluştur... |  |
| Git: Seçili Commit'ten Yama Oluştur... |  |
| Git: Seçili Commit'ten HEAD'e Yama Oluştur... |  |
| Git: Çalışma Ağacı Değişikliklerinden Yama Oluştur... |  |
| Yeniden Yaz: Eski Geçmişi Temizle... |  |
| Git: Yamayı Uygula... |  |
| Yeniden Yaz: Geçmiş Ekranı | `Ctrl+G H` |
| Yeniden Yaz: Bir Yazarı Değiştir... |  |
| Yeniden Yaz: Yazarları Anonimleştir... |  |
| Yeniden Yaz: Geçmişe .mailmap Uygula... |  |
| Yeniden Yaz: Kimlikleri Merge Et (.mailmap)... |  |
| Yeniden Yaz: Commit Tarihlerini Kaydır... |  |
| Yeniden Yaz: Commit Saat Dilimini Değiştir... |  |
| Yeniden Yaz: Commit Saatlerini Belirli Aralıkta Tut... |  |
| Yeniden Yaz: Commit Tarihini Yazar Tarihi Yap |  |
| Yeniden Yaz: Mesajlarda Bul ve Değiştir... |  |
| Yeniden Yaz: Mesaj Satırlarını Sil... |  |
| Yeniden Yaz: Trailer Ekle veya Kaldır... |  |
| Yeniden Yaz: İmzaları Kaldır... |  |
| Yeniden Yaz: Mesaj Kodlamasını Düzelt... |  |
| Yeniden Yaz: Dosyalarda Bul ve Değiştir... |  |
| Yeniden Yaz: Dosyaları Geçmişten Kaldır... |  |
| Yeniden Yaz: Büyük Dosyaları Geçmişten Kaldır... |  |
| Yeniden Yaz: Yolları Taşı veya Yeniden Adlandır... |  |
| Yeniden Yaz: Dosyaları LFS'ye Taşı... |  |
| Yeniden Yaz: Dosyaları LFS'den Geri Getir... |  |
| Yeniden Yaz: Satır Sonlarını Dönüştür... |  |
| Yeniden Yaz: Çalıştırma Bitini Kaldır... |  |
| Yeniden Yaz: Dosya Kodlamasını Dönüştür... |  |
| Yeniden Yaz: Betik Çalıştır... |  |
| ✦ Yeniden Yaz: YZ ile Betik Yaz... |  |
| ✦ Yeniden Yaz: YZ ile Kural Oluştur... |  |
| Yeniden Yaz: Seçili Commit'ten HEAD'e... |  |
| Yeniden Yaz: Seçili Commit'ler... |  |
| LFS: Bu Depo İçin Başlat |  |
| LFS: Deseni İzle... |  |
| LFS: Deseni İzlemeyi Bırak... |  |
| LFS: Nesneleri Fetch Et |  |
| LFS: Menüyü Aç | `Ctrl+G l` |
| LFS: Fetch (tüm nesneler) | `Ctrl+G F` |
| LFS: Checkout (tüm dosyalar) | `Ctrl+G K` |
| LFS: Push (tüm nesneler) | `Ctrl+G J` |
| LFS: Nesneleri Pull Et | `Ctrl+G L` |
| LFS: Nesneleri Push Et |  |
| LFS: Yerel Nesneleri Prune Et | `Ctrl+G V` |
| LFS: Kilitleri Göster |  |
| LFS: Kurulum Sayfasını Aç |  |
| LFS: Diff'te Ham İşaretçiyi Aç/Kapat | `r` |
| LFS: Seçili Dosyayı İzle |  |
| LFS: Seçili Dosyanın Uzantısını İzle |  |
| LFS: Seçili Dosyayı Kilitle |  |
| LFS: Seçili Dosyanın Kilidini Aç |  |
| LFS: Seçili Dosyanın Kilidini Zorla Aç |  |
| Refs: Seçili Dalı veya Tag'i Sil... | `Ctrl+G d` |
| Refs: Seçili Dalı Uzak Deposundan Sil... |  |
| Refs: Seçili Dalı Yerelde ve Uzak Depoda Sil... |  |
| Refs: Uzak Depodaki Seçili Dalın Bağlantısını Kopyala |  |
| Git: Uzak Depodaki Seçili Commit'in Bağlantısını Kopyala |  |
| Git: Seçili Commit'te Açıklamalı Tag... |  |
| Refs: Seçili Tag'i Varsayılan Uzak Depoya Push Et |  |
| Refs: Seçili Tag'i Uzak Deposundan Sil... |  |
| Git: Seçili Tag için Fetch |  |
| Git: Seçili Tag İçin Uzak Deponun Sürümünü Kullan |  |
| Git: Seçili Tag Uzak Depoya Zorla Push Et |  |
| Refs: Seçili Tag'e Not Ekle... |  |
| Refs: Seçili Tag Seçili Commit Üzerine Taşı... |  |
| Refs: Grafikte Gizle/Göster |  |
| Refs: Grafikte Göster | `g` |
| Refs: Seçili Ref'te Solo'yu Aç/Kapat | `o` |
| Refs: Solo'yu Durdur (tüm solo ref'leri temizle) | `O` |
| Refs: Gizli Tüm Ref'leri Göster | `H` |
| Refs: Seçili Klasör/Uzak Depoda Gizle/Göster |  |
| Refs: Seçili Klasör/Uzak Depoda Solo'yu Aç/Kapat | `o` |
| Refs: Ada Göre Filtrele... | `/` |
| Refs: Ad Filtresini Temizle | `Esc` |
| Refs: Tüm Klasörleri Genişlet | `=` |
| Refs: Tüm Klasörleri Daralt | `-` |
| Refs: Seçili Klasörü ve Alt Klasörlerini Genişlet | `*` |
| Refs: Seçili Klasörü ve Alt Klasörlerini Daralt | `_` |
| Görünüm: Paneli Sola Kaydır | `Alt+Left` |
| Görünüm: Paneli Sağa Kaydır | `Alt+Right` |
| Refs: Sonraki Bölüm | `l` |
| Refs: Önceki Bölüm | `h` |
| Refs: Etkin Bölümü Genişlet veya Daralt | `Enter` |
| Refs: Dal Hiyerarşisi Görünümünü Aç/Kapat | `y` |
| Git: Seçili Dosyayı Stage'le | `Ctrl+G s` |
| Git: Seçili Dosyayı Unstage'le | `Ctrl+G u` |
| Git: Tüm Değişiklikleri Stage'le | `Ctrl+G a` |
| Çalışma Değişiklikleri: Seçili Dosyayı At... |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Yok Say |  |
| Çalışma Değişiklikleri: Seçili Dosyanın Uzantısını Yok Say |  |
| Çalışma Değişiklikleri: Seçili Dosyayı İkili Olarak Ele Al... |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Metin Olarak Ele Al... |  |
| Çalışma Değişiklikleri: Seçili Dosyanın Uzantısını İkili Olarak Ele Al... |  |
| Çalışma Değişiklikleri: Seçili Dosyanın Uzantısını Metin Olarak Ele Al... |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Stash'le |  |
| Çalışma Değişiklikleri: Seçili Dosyanın Geçmişi |  |
| Çalışma Değişiklikleri: Seçili Dosyanın Blame'i |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Harici Diff Aracında Aç |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Harici Düzenleyicide Aç |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Varsayılan Programda Aç |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Klasörde Göster |  |
| Çalışma Değişiklikleri: Seçili Dosya Yolunu Kopyala |  |
| Çalışma Değişiklikleri: Seçili Dosyadan Yama Oluştur... |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Düzenle |  |
| Çalışma Değişiklikleri: Seçili Dosyayı Sil... |  |
| Çalışma Değişiklikleri: Seçili Klasör Menüsünü Aç |  |
| Çalışma Değişiklikleri: Dosya Oluştur |  |
| Çalışma Değişiklikleri: Tüm Değişiklikleri At... | `Ctrl+G x` |
| Git: Commit... | `Ctrl+G c` |
| Git: Stage'lenen Değişiklikleri Şimdi Commit Et | `Ctrl+G Enter` |
| Git: WIP Commit Mesajını Düzenle (Log'da) | `Log'da c` |
| Git: Tüm Uzak Depoları Fetch Et | `Ctrl+G f` |
| Git: Pull | `Ctrl+G p` |
| Git: Pull (mümkünse fast-forward) |  |
| Git: Pull (yalnızca fast-forward) |  |
| Git: Pull (rebase) |  |
| Git: Pull: Tüm Dallar | `Ctrl+G Ctrl+P` |
| Git: Pull: Tüm Dallar (mümkünse fast-forward) |  |
| Git: Pull: Tüm Dallar (yalnızca fast-forward) |  |
| Git: Pull: Tüm Dallar (Rebase) |  |
| Git: Pull Seçenekleri... |  |
| Git: Push | `Ctrl+G P` |
| Git: Uzak Depoya Push Et... |  |
| Git: Push Seçili Uzak Depolara... |  |
| Git: Push Tüm Hedeflere |  |
| Git: Pull Tüm Kaynaklardan |  |
| Git: Pull Tek Bir Uzak Depodan... |  |
| Git: Toplu Push İşlemini Göster |  |
| Git: Toplu Push İşlemine Devam Et |  |
| Git: Toplu Push İşlemini İptal Et |  |
| Git: Başarısız Push İşlemlerini Yeniden Dene |  |
| Git: Force Push (lease ile)... |  |
| Git: Dalın Uzak Depoları... |  |
| Remote: Uzak Depo Ekle... |  |
| Remote: Uzak Depoyu Kaldır... |  |
| Remote: Uzak Depoyu Düzenle... |  |
| Remote: Varsayılan Uzak Depoyu Ayarla... |  |
| Remote: Uzak Depo Adını Kopyala |  |
| Remote: Uzak Depo Bağlantısını Kopyala |  |
| Remote: Uzak Depoyu Fetch Et... |  |
| Remote: Seçili Uzak Depo Eylemleri... | `.` |
| Depo: Kısmi Checkout... |  |
| Depo: SSH ve Kimlik Bilgileri... |  |
| Depo: Bağlantıyı Sına |  |
| Depo: Varsayılan Kimlik Bilgilerini Kullan |  |
| Flow: Menüyü Aç | `Ctrl+G W` |
| Flow: Dal Başlat... |  |
| Flow: Yeni Issue... |  |
| Flow: Barındırıcı Dalını Devral... |  |
| Flow: Geçerli Dalı Bitir... |  |
| Flow: Üst Dalla Eşitle... |  |
| Flow: Yükselt... |  |
| Flow: Üst Dalı Ayarla... |  |
| Flow: Durdurulmuş Eyleme Devam Et veya İptal Et... |  |
| Flow: Gruplu Dalları Aç/Kapat |  |
| Flow: İş Akışı Ayarları... |  |
| Remote: Uzak Depoyu Grafikte Gizle... |  |
| Remote: Uzak Depoyu Grafikte Göster... |  |
| Git: HEAD'de Yeni Dal... | `Ctrl+G b` |
| Git: HEAD'de Yeni Tag... | `Ctrl+G t` |
| Git: Gönderilmemiş Tüm Tag'leri Push Et | `Ctrl+G U` |
| Git: Tüm Değişiklikleri Stash'le | `Ctrl+G T` |
| Git: Tüm Değişiklikleri Mesajla Stash'le... | `Ctrl+G m` |
| Git: Stash Pop | `Ctrl+G o` |
| Git: Stash Apply |  |
| Git: Stash Drop... |  |
| Git: Seçili Commit'i HEAD Üstüne Cherry-Pick Et | `Ctrl+G y` |
| Git: Seçili Commit'te Checkout (ayrık HEAD) |  |
| Git: Seçili Commit'te Dal Oluştur... |  |
| Git: Seçili Commit'te Tag Oluştur... |  |
| Commit: Mesajı Kopyala |  |
| Commit: Yazar Adını Kopyala |  |
| Commit: Yazar E-postasını Kopyala |  |
| Commit: Tarihi Kopyala |  |
| Commit: Kısa SHA'yı Kopyala |  |
| Commit: Tam SHA'yı Kopyala |  |
| Git: Seçili Commit'i Revert Et... | `Ctrl+G v` |
| Git: Commit Mesajını Düzenle... | `Ctrl+G e` |
| Commit: Değiştir... | `Ctrl+G E` |
| Commit: Yazarı Değiştir... | `Ctrl+G A` |
| Commit: Özeti Değiştir... |  |
| Commit: Açıklamayı Değiştir... |  |
| Git: Seçili Commit'i Böl... |  |
| Git: Seçili Commit'ten Etkileşimli Rebase... |  |
| Rebase: Devam Et |  |
| Rebase: Atla |  |
| Rebase: İptal Et |  |
| git am: Devam et |  |
| git am: Bu yamayı atla |  |
| git am: İptal et |  |
| Depo: Eski index.lock dosyasını sil |  |
| Banner: Bekleyen Otomatik Stash'i Pop Et |  |
| Banner: Bekleyen Otomatik Stash İşlemini Bitir (sonucu koru) |  |
| Banner: Bekleyen Otomatik Stash'i Göster |  |
| Çakışmalar: Tahmin Edilen Çakışmaları Göster |  |
| Banner: Bekleyen Otomatik Stash'i At... |  |
| Banner: Baştan Başla... |  |
| Git: Dalı Seçili Commit'e Reset Et (soft - indeksi ve çalışma ağacını koru) |  |
| Git: Dalı Seçili Commit'e Reset Et (mixed - çalışma ağacını koru) | `Ctrl+G R` |
| Git: Dalı Seçili Commit'e Reset Et (hard - her şeyi at)... |  |
| Git: Çakışmayı Ours ile Çöz (bu dal) |  |
| Git: Çakışmayı Theirs ile Çöz (gelen) |  |
| Git: Çakışmayı Temel ile Çöz (ortak ata) |  |
| Çakışmalar: Merge Düzenleyiciyi Aç | `Çakışmalı dosyada Enter` |
| Merge: Sonraki Çakışma | `F7` |
| Merge: Önceki Çakışma | `Shift+F7` |
| Merge: Sonraki Çakışmalı Dosya | `Alt+PageDown` |
| Merge: Önceki Çakışmalı Dosya | `Alt+PageUp` |
| Merge: Parçayı A'dan Al |  |
| Merge: Parçayı B'den Al |  |
| Merge: Bu Çakışma İçin Mevcut'u Al | `a` |
| Merge: Bu Çakışma İçin Gelen'i Al | `b` |
| Merge: İkisini de Al, Önce A Sonra B | `c` |
| Merge: İkisini de Al, Önce B Sonra A | `C` |
| Merge: Bu Çakışma İçin Temeli Al |  |
| Merge: Bu Çakışmayı Temizle | `x` |
| Merge: Sırayı Değiştir | `s` |
| Merge: Tüm Mevcut'u Al | `A` |
| Merge: Tüm Gelen'i Al | `B` |
| Merge: Bu Çakışmayı Çıktıda Düzenle |  |
| Merge: Yan Panelleri Göster veya Gizle | `F9` |
| Merge: Bölmeleri Sola Kaydır | `Shift+Left` |
| Merge: Bölmeleri Sağa Kaydır | `Shift+Right` |
| Merge: Temel Bölmesini Aç/Kapat | `Alt+D` |
| Merge: Tüm Dosya Diff'ini Aç/Kapat | `Alt+F` |
| Merge: Geri Al | `Ctrl+Z` |
| Merge: Yinele | `Ctrl+Y` |
| Merge: Çözüldü İşaretlemeden Kaydet | `Alt+S` |
| Merge: Kaydet ve Çözüldü İşaretle | `Ctrl+S` |
| Merge: Çıktıyı Merge Sonucuna Sıfırla | `Alt+R` |
| Merge: Çıktı Yüksekliğini Döngüyle Değiştir | `z` |
| Merge: Mevcut Bölmeyi Genişlet | `Alt+Right` |
| Merge: Gelen Bölmeyi Genişlet | `Alt+Left` |
| Merge: Bölme Boyutlarını Sıfırla | `Alt+0` |
| Merge: Bu Çakışmanın Seçimini Kaldır | `Alt+U` |
| Merge: Mevcut (A) Bölmesine Odaklan | `Alt+1` |
| Merge: Gelen (B) Bölmesine Odaklan | `Alt+2` |
| Merge: Çıktı Bölmesine Odaklan | `Alt+3` |
| Merge: Temel Bölmesine Odaklan | `Alt+4` |
| Merge: Soldaki Bölmeye Geç (Aynı Çakışma) |  |
| Merge: Sağdaki Bölmeye Geç (Aynı Çakışma) |  |
| Merge: Sonraki Bölmeye Odaklan | `F6` |
| Merge: Önceki Bölmeye Odaklan | `Shift+F6` |
| Merge: Diskteki Dosyadan Devam Et |  |
| Merge: Merge'den Baştan Başla |  |
| Merge: Çözümü Geri Al ve Yeniden Aç |  |
| Merge: Dosyayı Yeniden Yükle |  |
| Merge: Mevcut Dosyayı Olduğu Gibi Kullan | `a` |
| Merge: Gelen Dosyayı Olduğu Gibi Kullan | `b` |
| Merge: Temel Dosyayı Olduğu Gibi Kullan | `o` |
| Merge: Kalan Dosyayı Koru | `k` |
| Merge: Silmeyi Kabul Et | `D` |
| Merge: Harici Merge Aracında Aç | `m` |
| ✦ YZ: Merge Çakışması Çözümü Öner | `Alt+I` |
| Merge: Merge Düzenleyiciyi Kapat |  |
| Git: Merge, Cherry-pick veya Revert'i İptal Et |  |
| Git: Cherry-pick veya Revert'i Atla |  |
| Depo: Aç veya Geç (Depo Yönetimi)... | `Ctrl+O` |
| Depo: Depo Gözat... | `b (yönetim sekmesi)` |
| Depo: Yeni Oluştur (Init)... | `i (yönetim sekmesi)` |
| Depolar: Seçiliyi Aç | `Enter` |
| Depolar: Seçiliyi Favorilere Ekle veya Kaldır | `f` |
| Depolar: Seçiliyi Kapat veya Kaldır | `x` |
| Depolar: Seçili Depo Menüsü... | `.` |
| Depolar: Seçilen Yolu Kopyala | `y` |
| Depolar: Seçileni Varsayılan Araçta Aç | `o` |
| Depolar: Ara | `/ / Ctrl+F` |
| Depolar: Tüm Bölümleri Daralt | `-` |
| Depolar: Tüm Bölümleri Genişlet | `+` |
| Depolar: Tüm Sekmeleri Kapat... |  |
| Depolar: Seçiliyi Listeden Kaldır |  |
| Depolar: Tüm Son Kullanılanları Kaldır |  |
| Çalışma Alanı: Yeni... | `n` |
| Çalışma Alanı: Açık Sekmelerden Yeni... | `a` |
| Çalışma Alanı: Değiştir... | `Alt+W` |
| Çalışma Alanı: Düzenle... | `e` |
| Çalışma Alanı: Kapat (Çalışma Alanı Yok) |  |
| Çalışma Alanı: Yönet... | `Alt+J` |
| Çalışma Alanı: Sekmeyi Kapat |  |
| Çalışma Alanı: Eksik Depoyu Bul... | `l` |
| Çalışma Alanı: Sil... |  |
| Çalışma Alanı: Tümünü Fetch Et | `F` |
| Çalışma Alanı: Tümünü Pull Et | `P` |
| Çalışma Alanı: Eksikleri Klonla | `m` |
| Çalışma Alanı: Sekmeyi Çalışma Alanına Ekle |  |
| Çalışma Alanı: Sekmeyi Çalışma Alanından Çıkar |  |
| Radar: Aç | `Alt+R` |
| Radar: Yenile | `F5` |
| Radar: Ara | `/` |
| Radar: Tarayıcıda Aç | `o` |
| Radar: Depoyu Aç | `Enter` |
| Radar: Dalı Checkout Et | `c` |
| Radar: Worktree Oluştur... | `t` |
| Radar: Depoda Göster | `s` |
| Radar: Issue Üzerinde Çalışmaya Başla... | `n` |
| Radar: Sabitle veya Sabitlemeyi Kaldır | `p` |
| Radar: Ertele... | `z` |
| Radar: İsteği İncele... | `e` |
| Radar: Issue Konuşmasını Aç | `i` |
| Radar: İsteği Merge Et... | `m` |
| Radar: Kapat... | `x` |
| Radar: Hazır Olarak İşaretle | `y` |
| Radar: Depoyu Klonla... | `g` |
| Radar: API Üzerinden İncele |  |
| Radar: Klonla ve İncele... |  |
| Radar: Atananları Düzenle... |  |
| Radar: İnceleyenleri Düzenle... |  |
| Radar: Etiketleri Düzenle... |  |
| Radar: Kilometre Taşı Ayarla... |  |
| Radar: Kapsam Seç | `w` |
| Radar: Görünüm Seç | `v` |
| Radar: Filtre Seç | `f` |
| Radar: Klonlanmamış İstekleri Göster | `a` |
| Radar: Taslakları Göster | `d` |
| Radar: Hesap Geneli İnceleme İsteklerini Etkinleştir... | `T` |
| Radar: Satır Menüsü... | `.` |
| Radar: Sütun Seç | `C` |
| Radar: Sütunu Genişlet | `]` |
| Radar: Sütunu Daralt | `[` |
| Radar: Sütun Genişliklerini Sıfırla | `0` |
| Radar: Sütunu İçeriğine Sığdır | `=` |
| Radar: Tüm Sütunları İçeriğe Sığdır | `+` |
| Radar: Ana Sayfaya Dön | `Esc` |
| Radar: Sekmeyi Kapat |  |
| Sekme: Yeni Sekme | `Alt+T` |
| Sekme: Yeni... |  |
| Sekme: Eylem Menüsü... | `Ctrl+T` |
| Sekme: Diğer Sekmeleri Kapat... |  |
| Sekme: Sağdaki Sekmeleri Kapat... |  |
| Sekme: Favorilere Ekle veya Kaldır |  |
| Sekme: Depo Yolunu Kopyala |  |
| Sekme: Dosya Yöneticisinde Aç |  |
| Sekme: Terminalde Aç |  |
| Sekme: Şurada Aç... | `Alt+O` |
| Sekme: Burada Terminal |  |
| ✦ Sekme: Burada YZ Ajanı Aç |  |
| Sekme: Varsayılanda Aç | `Ctrl+Alt+O` |
| Depo: Profili Değiştir... |  |
| Yardım: GitPoulpi Hakkında | `Ctrl+N a` |
| Yardım: Lisanslar ve Emeği Geçenler |  |
| Görünüm: Etkinlik Günlüğü | `Ctrl+N A` |
| Görünüm: Genel Etkinlik Günlüğü | `Ctrl+N G` |
| Yardım: Sürüm Notları |  |
| Güncellemeler: Güncellemeyi Göster | `Ctrl+N u` |
| Güncellemeler: Güvenlik Bildirimini Aç |  |
| Güncellemeler: Güncelleme Komutunu Kopyala |  |
| Güncellemeler: Güncelleme Komutunu Terminalde Çalıştır |  |
| Güncellemeler: Şimdi Denetle |  |
| Güncellemeler: İndir |  |
| Güncellemeler: Şimdi Güncelle |  |
| Güncellemeler: Güncellemek İçin Yeniden Başlat |  |
| Güncellemeler: Önceki Sürüme Dön |  |
| Güncellemeler: Bu Sürümü Atla |  |
| Sürüm Notları: Sekmeyi Kapat |  |
| Görevler: Arka Plan Görevlerini Göster | `Ctrl+N J` |
| Görevler: İptal Edilebilen Her Şeyi İptal Et |  |
| Görünüm: Tanılamayı kopyala |  |
| Etkinlik Günlüğü: Hata Düzeyini Göster veya Gizle |  |
| Etkinlik Günlüğü: Uyarı Düzeyini Göster veya Gizle |  |
| Etkinlik Günlüğü: Bilgi Düzeyini Göster veya Gizle |  |
| Etkinlik Günlüğü: Hata Ayıklama Düzeyini Göster veya Gizle |  |
| Etkinlik Günlüğü: İzleme Düzeyini Göster veya Gizle |  |
| Etkinlik Günlüğü: Tüm Düzeyleri Göster |  |
| Etkinlik Günlüğü: Hiçbir Düzeyi Gösterme |  |
| Etkinlik Günlüğü: Uygulama Günlüğünü Temizle |  |
| Etkinlik Günlüğü: Kapsamı Seç |  |
| Etkinlik Günlüğü: Git İşlemleri |  |
| Etkinlik Günlüğü: Uygulama Günlüğü |  |
| Etkinlik Günlüğü: Bu Satırın İşlemini Göster |  |
| Etkinlik Günlüğü: Sonraki Başarısız İşlem |  |
| Etkinlik Günlüğü: Önceki Başarısız İşlem |  |
| Etkinlik Günlüğü: Sola Kaydır | `Left` |
| Etkinlik Günlüğü: Sağa Kaydır | `Right` |
| Etkinlik Günlüğü: Son Hatayı Kopyala |  |
| Etkinlik Günlüğü: Önerilen Eylemi Çalıştır |  |
| Etkinlik Günlüğü: Son Hatayı Aç |  |
| Sekme: Çık ve cd Yap |  |
| Depo: Klonla... | `Ctrl+Shift+N` |
| Depo: Sonraki Sekme | `Ctrl+PageDown / Ctrl+Tab` |
| Depo: Önceki Sekme | `Ctrl+PageUp / Ctrl+Shift+Tab` |
| Sekme: Sola Taşı |  |
| Sekme: Sağa Taşı |  |
| Depo: Sekmeyi Kapat | `Ctrl+W` |
| Depo: Sekme Seç... | `t` |
| Ara: Çalışma Alanı... | `Ctrl+Alt+F` |
| Düzenle: Geri Al | `Ctrl+Z` |
| Düzenle: Yinele | `Ctrl+Y` |
| Düzenle: Engellenen geri alma girdisini unut |  |
| Düzenle: Geri Alma Geçmişi... | `Alt+Z` |
| Düzenle: Reflog... |  |
| Düzenle: Kaybolan Commit'ler Kurtar... |  |
| Düzenle: Dalı Engelli Eylemin Bıraktığı Yere Geri Yükle | `Alt+Shift+Z` |
| Düzenle: Tüm Diziyi Geri Al |  |
| İnceleme: Genel Bakışı Göster |  |
| İnceleme: Dosyaları Göster |  |
| İnceleme: Commit'leri Göster |  |
| İnceleme: Satıra Yorum Yap... | `c` |
| İnceleme: Konuşmaya Yorum Yap... | `C` |
| İnceleme: Yorum Dizisini Yanıtla... | `r` |
| İnceleme: Yorum Dizisini Çöz veya Yeniden Aç | `x` |
| İnceleme: Genel Bakışta Yoruma Dön | `Esc` |
| İnceleme: Yorum Dizisini Daralt veya Genişlet | `z` |
| İnceleme: Bekleyen Yorumu Düzenle... | `e` |
| İnceleme: Bekleyen Yorumu Sil | `d` |
| İnceleme: Dosyayı Görüldü Olarak İşaretle | `v` |
| İnceleme: Sonraki Dosya | `]` |
| İnceleme: Önceki Dosya | `[` |
| İnceleme: İncelemeyi Bitir... | `f` |
| İnceleme: Bekleyen İncelemeyi At... |  |
| İnceleme: Yenile | `F5` |
| İnceleme: Tarayıcıda Aç | `o` |
| İnceleme: Dalı Checkout Et | `b` |
| İnceleme: Kapat | `Esc` |
| İnceleme: Atananları Düzenle... | `a` |
| İnceleme: İnceleyenleri Düzenle... | `p` |
| İnceleme: Etiketleri Düzenle... | `l` |
| İnceleme: Kilometre Taşı Ayarla... | `m` |
| İnceleme: Yama Kümesi Seç... | `P` |
| İnceleme: Yama Kümelerini Karşılaştır... | `D` |
| İnceleme: Değişikliği Teslim Et... | `U` |
| İnceleme: Değişiklikten Vazgeç... | `Q` |
| İnceleme: Satır İçi veya Bölünmüş Yerleşim | `V` |
| İnceleme: Tüm Dosya veya Yalnızca Hunk'lar | `H` |
| İnceleme: Yalnızca Boşluk Değişikliklerini Göster veya Gizle | `S` |
| İnceleme: Sözcük Kaydırmayı Aç/Kapat | `W` |
| İnceleme: Sonraki Hunk | `J` |
| İnceleme: Önceki Hunk | `K` |
| İnceleme: Kodlamayı Seç... | `E` |
| İnceleme: Minimap'i Aç/Kapat | `M` |
| İnceleme: Diff'i Sola Kaydır | `Left` |
| İnceleme: Diff'i Sağa Kaydır | `Right` |
| İnceleme: Seçili Metni veya Satırı Kopyala | `Ctrl+C` |
| Pano: Kartı Şuraya Taşı... | `m` |
| Pano: Kartı Sola Taşı |  |
| Pano: Kartı Sağa Taşı |  |
| Pano: Kartı Aç | `Enter` |
| Pano: Atananları Düzenle... |  |
| Pano: Etiketleri Düzenle... |  |
| Pano: Kilometre Taşı Ayarla... |  |
| Pano: Bu Sütunda Yeni Issue | `n` |
| Pano: Atanana Göre Filtrele... | `a` |
| Pano: Etikete Göre Filtrele... | `l` |
| Pano: Kilometre Taşına Göre Filtrele... | `M` |
| Pano: Metne Göre Filtrele | `/` |
| Pano: Filtreleri Temizle | `x` |
| Pano: Sütun Etiketlerini Ayarla... | `c` |
| Pano: Yenile | `r` |
| Pano: Kapat | `Esc` |
| Konuşma: Geri |  |
| Konuşma: Yenile | `F5` |
| Konuşma: Eski Kayıtları Yükle | `m` |
| Konuşma: Önceki Kayıt | `Up` |
| Konuşma: Sonraki Kayıt | `Down` |
| Konuşma: Sonraki Yeni Kayıt | `n` |
| Konuşma: Kayda Yanıt Ver | `r` |
| Konuşma: Kaydı Alıntıla | `Q` |
| Konuşma: Yeni Yorum | `c` |
| Konuşma: Kaydın Bağlantıları... | `l` |
| Konuşma: Kaydın Diyagramını Aç | `g` |
| Konuşma: Dizinin Kodunu Aç | `G` |
| Konuşma: Kaydın Bağlantısını Kopyala | `y` |
| Konuşma: Metni Kopyala | `Ctrl+C` |
| Konuşma: Tarayıcıda Aç | `o` |
| Konuşma: Yorumu Düzenle | `e` |
| Konuşma: Yorumu Sil | `d` |
| Konuşma: Olayları Göster veya Gizle | `v` |
| Konuşma: Yaz ve Önizleme Arasında Geç |  |
| Konuşma: Yan Yana Görünümü Aç veya Kapat |  |
| Konuşma: Yorumu Gönder |  |
| Konuşma: Dahili Notu Aç veya Kapat |  |
| Konuşma: Kapat | `Esc` |
| Markdown Düzenleyici: YZ ile Yaz |  |
| Markdown Düzenleyici: YZ Bağlamını Yenile |  |
| Barındırma: Issue Konuşmasını Aç... |  |
| İnceleme: ✦ İnceleme Taslağı... | `s` |
| İnceleme: Öneriyi Kabul Et | `y` |
| İnceleme: Öneriyi Düzenle ve Kabul Et... | `w` |
| İnceleme: Öneriyi Yoksay | `n` |
| İnceleme: Tüm Önerileri Kabul Et | `A` |
| İnceleme: Tüm Önerileri Yoksay | `N` |
| İnceleme: YZ Özetini İnceleme Metninde Kullan | `i` |
| İnceleme: ✦ Taslağı Kullan... | `u` |
| İnceleme: ✦ Taslak Metnini Düzenle... | `t` |
| İnceleme: ✦ Taslağı At | `N` |
| Geri Alma Geçmişi: Buraya Kadar Geri Al... | `u` |
| Geri Alma Geçmişi: Buraya Kadar Yinele... | `y` |
| Geri Alma Geçmişi: Tüm Diziyi Geri Al | `x` |
| Geri Alma Geçmişi: Yalnızca Bunu Geri Yükle... | `r` |
| Geri Alma Geçmişi: Dalı Eski Yerine Geri Yükle... | `o` |
| Geri Alma Geçmişi: Yenile |  |
| Geri Alma Geçmişi: Kapat | `Esc` |
| Reflog: Log'da Göster | `l` |
| Reflog: Burada Dal Oluştur... | `b` |
| Reflog: Burada Tag Oluştur... | `t` |
| Reflog: Değişiklikleri Çalışma Ağacına Uygula | `w` |
| Reflog: Değişiklikleri Stash Olarak Kaydet | `s` |
| Reflog: Bu Konuma Geri Yükle... | `r` |
| Reflog: Daha Fazla Yükle | `m` |
| Reflog: Yenile |  |
| Kaybolan Çalışma: Burada Dal Oluştur... | `b` |
| Kaybolan Çalışma: Burada Tag Oluştur... | `t` |
| Kaybolan Çalışma: Cherry-pick | `c` |
| Kaybolan Çalışma: Değişiklikleri Çalışma Ağacına Uygula | `w` |
| Kaybolan Çalışma: Değişiklikleri Stash Olarak Kaydet | `s` |
| Kaybolan Çalışma: Yeniden Tara |  |
| Reflog / Kaybolan Çalışma: Sonraki Panel | `Tab` |
| Reflog / Kaybolan Çalışma: Dosya Listesi Yol veya Ağaç | `v` |
| Reflog / Kaybolan Çalışma: Panel Boyutlarını Sıfırla |  |
| Pencere: Taşı veya Boyutlandır | `Ctrl+N m` |
| Pencere: Konumu ve Boyutu Sıfırla |  |
| Görünüm: Yenile | `r` |
| Görünüm: Yenile (fetch, Pull Request'ler, issue'lar) | `F5` |
| Görünüm: Yardım Katmanını Aç/Kapat | `?` |
| Görünüm: Ref Sütununu Aç/Kapat |  |
| Görünüm: Grafik Sütununu Aç/Kapat |  |
| Görünüm: Mesaj Sütununu Aç/Kapat |  |
| Görünüm: Yazar Sütununu Aç/Kapat |  |
| Görünüm: Tarih Sütununu Aç/Kapat |  |
| Görünüm: Değişiklikler Sütununu Aç/Kapat |  |
| Görünüm: SHA Sütununu Aç/Kapat |  |
| Görünüm: Log sütunu: taşımak veya boyutlandırmak için bir sütun seç | `C` |
| Görünüm: Log sütunu: sonraki sütunu seç | `Tab` |
| Görünüm: Log sütunu: önceki sütunu seç | `Shift+BackTab` |
| Görünüm: Log sütunu: sütunu sola taşı | `<` |
| Görünüm: Log sütunu: sütunu sağa taşı | `>` |
| Görünüm: Log sütunu: sütunu genişlet | `]` |
| Görünüm: Log sütunu: sütunu daralt | `[` |
| Görünüm: Log sütunu: sütun genişliklerini sıfırla | `0` |
| Görünüm: Log sütunu: Sütunu İçeriğine Sığdır | `=` |
| Görünüm: Log sütunu: Tüm Sütunları İçeriğe Sığdır | `+` |
| Görünüm: Log: Sütunları Sola Kaydır | `Shift+Left` |
| Görünüm: Log: Sütunları Sağa Kaydır | `Shift+Right` |
| Görünüm: Log: Sütunlar için Yana Kaydırmayı Aç/Kapat | `s` |
| Görünüm: Log: Gösterilecek sütunları seç |  |
| Diff: Seçili Dosyanın Diff'ini Aç | `Enter` |
| Diff: Diff Görünümünü Kapat | `Esc` |
| Diff: Sonraki Hunk | `n` |
| Diff: Önceki Hunk | `p` |
| Diff: İmleç Altındaki Hunk'ı Stage'le | `Space` |
| Diff: İmleç Altındaki Hunk'ı Unstage'le | `Space` |
| Diff: Satırları Seç (veya seçimi temizle) | `v` |
| Diff: Seçili Satırları Stage'le | `Enter` |
| Diff: Seçili Satırları Unstage'le | `Enter` |
| Diff: İmleç Altındaki Hunk'ı At... | `x` |
| Diff: Seçili Satırları At... | `X` |
| Kopyala: Seçili Metin (önce metnin üzerinde sürükleyin) | `Ctrl+C` |
| Kopyala: İmleç Altındaki Satır (veya seçili satırlar) | `y` |
| Diff: İmleç Altındaki Hunk'ı Kopyala |  |
| Diff: Yerleşim Satır İçi (+/- tek sütunda) | `m` |
| Diff: Yerleşim Bölünmüş (önce ve sonra) | `m` |
| Diff: Yalnızca Hunk'ları Aç/Kapat (tüm dosya / yalnızca değişiklikler) | `H` |
| Dosyalar: Yol/Ağaç Aç/Kapat | `m` |
| Dosyalar: Tüm Dosyaları Göster Aç/Kapat | `a` |
| Dosyalar: Yalnızca LFS'yi Aç/Kapat | `o` |
| Commit: Hook'ları Atlamayı Aç/Kapat | `H` |
| Dosyalar: Satırın veya Seçimin Menüsünü Aç | `.` |
| Dosyalar: Tüm Klasörleri Genişlet | `=` |
| Dosyalar: Tüm Klasörleri Daralt | `-` |
| Dosyalar: Seçili Klasörü ve Alt Klasörlerini Genişlet | `*` |
| Dosyalar: Seçili Klasörü ve Alt Klasörlerini Daralt | `_` |
| Dosyalar: Tümünü Seç |  |
| Dosyalar: Seçimi Temizle |  |
| Dosyalar: Geçerli Satırın Seçimini Aç/Kapat |  |
| Log: Mermaid gitGraph Olarak Kopyala... | `Y` |
| Log: Seçimi karşılaştır | `v` |
| Log: Karşılaştırma uçlarını değiştir | `x` |
| Log: Sonraki karşılaştırılan satıra git | `n` |
| Diff: Sözcük Kaydırmayı Aç/Kapat | `w` |
| Diff: Kodlamayı Seç... | `C` |
| Görünüm: Minimap'i Aç/Kapat | `M` |
| Diff: Değişen Sözcük Vurgusunu Aç/Kapat |  |
| Görünüm: Commit sütununu aç/kapat | `Alt+C` |
| Görünüm: Commit sütunu: gizle |  |
| Görünüm: Commit sütunu: dosya geçmişi |  |
| Görünüm: Commit sütunu: tüm Commit'ler |  |
| Görünüm: Commit sütununa odaklan |  |
| Görünüm: Commit sütununu genişlet |  |
| Görünüm: Commit sütununu daralt |  |
| Görünüm: Commit sütunu: sola kaydır | `Shift+Left` |
| Görünüm: Commit sütunu: sağa kaydır | `Shift+Right` |
| Görünüm: Commit sütunu: sonraki bölümü seç | `Tab` |
| Görünüm: Commit sütunu: bölümü sola taşı | `<` |
| Görünüm: Commit sütunu: bölümü sağa taşı | `>` |
| Görünüm: Commit sütunu: bölümü genişlet | `]` |
| Görünüm: Commit sütunu: bölümü daralt | `[` |
| Görünüm: Commit sütunu: bölüm genişliklerini sıfırla | `0` |
| Görünüm: Commit sütunu: Sütunu İçeriğine Sığdır | `=` |
| Görünüm: Commit sütunu: Tüm Sütunları İçeriğe Sığdır | `+` |
| Görünüm: Commit sütunu: seçili satırın menüsü | `.` |
| Görünüm: Commit sütunu: sütunları seç |  |
| Görünüm: Commit sütunu: Ref sütununu aç/kapat |  |
| Görünüm: Commit sütunu: Grafik sütununu aç/kapat |  |
| Görünüm: Commit sütunu: Yazar sütununu aç/kapat |  |
| Görünüm: Commit sütunu: Tarih sütununu aç/kapat |  |
| Görünüm: Commit sütunu: Değişim sütununu aç/kapat |  |
| Görünüm: Commit sütunu: SHA sütununu aç/kapat |  |
| Görünüm: Commit sütunu: seçili satırı Log'da göster |  |
| Görünüm: Commit sütunu: dosyanın seçili satırdaki değişikliklerini göster |  |
| Görünüm: Commit sütunu: dosyayı seçili satırdaki haliyle göster |  |
| Görünüm: Commit sütunu: dosyayı seçili satırda Blame ile göster |  |
| Düzenleyici: Seçimi İzlemeyi Aç/Kapat |  |
| Görsel: Yakınlaştır | `+` |
| Görsel: Uzaklaştır | `-` |
| Görsel: Görünüme Sığdır | `0` |
| Görsel: Gerçek Boyut (1:1) | `1` |
| Görsel: Önce / Sonra Arasında Geçiş | `Tab` |
| Görsel: Önce Bölmesini Genişlet | `Alt+Right` |
| Görsel: Sonra Bölmesini Genişlet | `Alt+Left` |
| Görsel: Bölme Boyutlarını Sıfırla | `Alt+0` |
| Görsel: Şununla Düzenle... | `O` |
| Görsel: Varsayılan Düzenleyiciyle Düzenle | `o` |
| Dosya Geçmişi: Log'da Göster | `Enter` |
| Dosya Geçmişi: Commit SHA'sını Kopyala | `y` |
| Dosya Geçmişi: Özeti Kopyala | `Y` |
| Dosya Geçmişi: Bu Commit Öğesinin Diff'ini Göster | `d` |
| Dosya Geçmişi: Dosyayı Bu Commit Anında Aç | `o` |
| Dosya Geçmişi: Bu Commit Anında Blame | `b` |
| Dosya Geçmişi: Çalışma Ağacı ile Karşılaştır | `c` |
| Dosya Geçmişi: Bu Sürümü Geri Yükle | `R` |
| Dosya Geçmişi: Seçimi Yukarı Genişlet | `Shift+Up` |
| Dosya Geçmişi: Seçimi Aşağı Genişlet | `Shift+Down` |
| Dosya Geçmişi: Liste ile Diff Arasında Geçiş Yap | `Tab` |
| Dosya Geçmişi: Listeyi Genişlet | `Alt+Right` |
| Dosya Geçmişi: Listeyi Daralt | `Alt+Left` |
| Dosya Geçmişi: Liste Genişliğini Sıfırla | `Alt+0` |
| Dosya: Birlikte Aç... | `O` |
| Dosya: Varsayılan Uygulamayla Aç | `o` |
| Diff: Yalnızca Boşluk Değişikliklerini Aç/Kapat | `s` |
| ✦ YZ: Commit Mesajı Oluştur | `Ctrl+G i` |
| ✦ YZ: Stash Mesajı Oluştur |  |
| ✦ YZ: İstem Değişkeni Ekle |  |
| ✦ YZ: Profili Düzenle... |  |
| ✦ YZ: Ajanı Aç... |  |
| ✦ YZ: Seçili Commit'i Açıkla | `Ctrl+G w` |
| ✦ YZ: Seçili Commit'i Böl... |  |
| ✦ YZ: Çalışma Değişikliklerini Açıkla | `Ctrl+G w` |
| ✦ YZ: Çalışma Değişikliklerinden Commit'ler Oluştur... | `Ctrl+G C` |
| Hassas Dosyalar: Dosyada Göster | `S` |
| Composer: Dosyayı Bir Commit'e Taşı... | `m` |
| Diff: Blame |  |
| Blame: Commit öğesini Log'da göster | `Enter` |
| Blame: Commit SHA değerini kopyala | `Y` |
| Blame: Commit diff'ini göster | `d` |
| Blame: Commit ebeveyninde blame yap | `b` |
| Blame: Dosya geçmişini aç | `H` |
| Blame: Ayrıntı kartını göster veya gizle | `i` |
| Blame: Sonraki hunk | `n` |
| Blame: Önceki hunk | `p` |
| Blame: Aynı Commit öğesinin sonraki hunk'ı | `N` |
| Blame: Aynı Commit öğesinin önceki hunk'ı | `P` |
| Blame: Satıra git | `g` |
| Blame: Açıklama sütununu genişlet | `>` |
| Blame: Açıklama sütununu daralt | `<` |
| Diff: Dosya Geçmişi |  |
| Diff: Dosya Görünümü |  |
| Dosya: Çalışma Ağacında Düzenle | `e` |
| Düzenleyici: Kodlamayı Değiştir |  |
| Düzenleyici: Kodlamayla Yeniden Aç |  |
| Düzenleyici: Kodlamaya Dönüştür |  |
| Düzenleyici: Satır Sonlarını Dönüştür |  |
| Düzenleyici: Girintiyi Değiştir |  |
| Uygulama: Ayarlar... | `Ctrl+, / Ctrl+N ,` |
| ✦ Dizi: YZ ile Oluştur... |  |
| Uygulama: Ayarlarda Ara... | `Ayarlar'da /` |
| Uygulama: Tuş Atamaları... |  |
| Ajan: Bekleyen Görünümü Göster | `Ctrl+N Enter` |
| Ajan: Bekleyen Görünümü Kapat |  |
| Ajan: Geri | `Ctrl+N Backspace` |
| Dizi: İptal |  |
| Dizi: Devam |  |
| Dizi: Bu Deponun Dizilerini İncele |  |
| Dizi: Bu Deponun Dizilerine Güven |  |
| Dizi: Bu Deponun Dizilerini Yok Say |  |
| Tuş Atamaları: Tümünü Sıfırla... |  |
| Entegrasyonlar: Bağlı Pull Request Aç | `Ctrl+G g` |
| Entegrasyonlar: Bağlı Pull Request Barındırıcıda Düzenle | `e (satırda)` |
| Entegrasyonlar: Bağlı Pull Request URL'sini Kopyala |  |
| Entegrasyonlar: Pull Request'ler ve Issue'ları Yenile |  |
| Entegrasyonlar: Pull Request Checkout... |  |
| Entegrasyonlar: Commit Referansını Aç (#issue / !istek)... |  |
| Entegrasyonlar: Issue Aç... |  |
| Barındırma: İsteği İncele... |  |
| Barındırma: İnceleme İçin Push... |  |
| Barındırma: Atananları Düzenle... |  |
| Barındırma: İnceleyenleri Düzenle... |  |
| Barındırma: Etiketleri Düzenle... |  |
| Barındırma: Kilometre Taşı Ayarla... |  |
| Yerel Issue'lar: Issue'yu Kapat... |  |
| Yerel Issue'lar: Issue'yu Yeniden Aç... |  |
| Yerel Issue'lar: Obsidian'da Aç... |  |
| Issue'lar: Panoyu Aç |  |
| Yerel Issue'lar: Obsidian Base Oluştur |  |
| Pull Request'ler: Pull Request Oluştur... | `n` |
| Barındırma: Yeni Issue... | `n` |
| Pull Request'ler: Ara | `Arama satırında Enter` |
| Pull Request'ler: Filtre Seç (Açık / Merged / Tümü)... | `v` |
| Pull Request'ler: Sağlayıcı Seç... | `Sağlayıcı satırında Enter` |
| Pull Request'ler: Depo Seç... | `Depo satırında Enter` |
| Pull Request'ler: Kaynak Bağlantısını Kaldır (Yok) |  |
| Issue'lar: Sağlayıcı Seç... | `Sağlayıcı satırında Enter` |
| Issue'lar: Kaynak Bağlantısını Kaldır (Yok) |  |
| Entegrasyonlar: Profil Bağlantısını Kaldır (Otomatik) |  |
| Issue'lar: Depo Seç... | `Depo satırında Enter` |
| Issue'lar: Ara | `Arama satırında Enter` |
| Issue'lar: Filtre Seç... | `v` |
| Issue'lar: Sorguyu / Listeleri Düzenle... |  |
| Commit: Issue Referansı Ekle... |  |
| Entegrasyonlar: Bu Depo İçin Profil Kullan... |  |
| Entegrasyonlar: Bu Deponun Profilini Yönet |  |
| Entegrasyonlar: Entegrasyon Bağla... |  |
| Ayarlar: Bir profilin Radar seçenekleri... |  |
| Entegrasyonlar: Güvenlik Denetimini İncele |  |
| Uygulama: Çık | `q` |
| Uygulama: Tema... |  |
| Çakışmalar: Temel Dal Listelerini Düzenle... |  |
| Çakışmalar: Bu Deponun Temel Dalları... |  |
| Uygulama: Temayı Değiştir... | `Ctrl+N t` |
| Uygulama: Dili Değiştir... | `Ctrl+N l` |
| Başlatıcı: Kısayol Oluştur |  |
| Başlatıcı: Masaüstü Kısayolu Oluştur |  |
| Başlatıcı: Kısayolları Kaldır |  |
| Başlatıcı: Komut Takma Adı Oluştur |  |
| Başlatıcı: Komut Takma Adını Kaldır |  |
| MCP: Boş Bir Port Öner |  |
| Başlatıcı: PATH'e Ekle |  |
| Başlatıcı: PATH'ten Kaldır |  |
| Başlatıcı: GitPoulpi'de Aç Menüsünü Kur |  |
| Başlatıcı: GitPoulpi'de Aç Menüsünü Kaldır |  |
| Başlatıcı: Birincil Kurulum Ol |  |
| Başlatıcı: Eski Başlatıcıları Kaldır |  |
| Başlatıcı: Kayıp Kurulumları Unut |  |
| Uygulama: Grafik Renk Temasını Değiştir... | `Ctrl+N T` |
| Uygulama: Grafik Satır Tonunu Aç/Kapat | `Ctrl+N Y` |
<!-- palette:end -->

## Gezinme

| Tuş | Eylem |
|---|---|
| `Tab` / `Shift+Tab` | Sonraki / önceki panele odaklan |
| `Ctrl+N`, sonra bir rozet | Pencerenin o bölümüne odaklan ya da öndeki pencerenin veya açılır pencerenin o denetimine bas (bkz. [Gezinme öneki](#gezinme-öneki-ctrln)) |
| `j` / `k`, `Up` / `Down` | Seçimi taşı, her panelde |
| `PageUp` / `PageDown`, `Home` / `End` | Bir sayfa taşı; ilk / son satıra git |
| bir sayı, sonra `j` / `k` / `Up` / `Down` | Sayı öneki: `4j` dört satır taşır. `PageUp` / `PageDown` ile sayı, sayfa sayısıdır. Durum çubuğu bekleyen sayıyı gösterir (`4...`). `Esc` veya başka herhangi bir tuş onu temizler. Baştaki `0` asla bir sayı başlatmaz |
| `h` / `l`, `Left` / `Right` | Refs paneli: bölüm değiştir. Log: bir commit'in değişen dosyalar listesinden çık (`h`) veya ona gir (`l`) |
| `?` | Yardımı aç veya kapat (bkz. [Uygulama içi yardım](#uygulama-içi-yardım-)) |
| `r` | Yenile |
| `F5` | Her uzak depoyu fetch et, pull request'leri ve issue'ları yeniden yükle ve yenile |
| `q` | Çık |
| `Ctrl+Z` | Son işlemi **geri al**. Bkz. [Geri al ve yinele](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Yinele**. `Ctrl+Shift+Z` kitty klavye protokolü ister; `Ctrl+Y` her zaman çalışır |
| `.`, `Menu`, `Shift+F10` | Seçili satırın bağlam menüsünü aç |

### Bir metin alanında Ctrl+Z / Ctrl+Y

Bir metin alanı klavyeyi tutarken `Ctrl+Z` ve `Ctrl+Y` (`Ctrl+Shift+Z` de) o alanın
metni üzerinde işler, depo üzerinde asla: klavyeyi tutan alan her zaman kazanır; bu
yüzden commit mesajındaki `Ctrl+Z` bir commit'i geri alamaz. Başka her yerde küresel
anlamlarını korurlar (yukarıdaki geri al / yinele günlüğü). Bu her metin alanı ve çok
satırlı düzenleyici için geçerlidir: commit mesajı, Ayarlar alanları ve metin
düzenleyicileri, pull request ve issue formları, YZ iletişim kutuları ve her filtre veya
arama kutusu.

- **Yazma** bir sözcük bir sözcük geri alınır (bir harf dizisi, bir boşluk dizisi, bir
  noktalama dizisi) ve bir saniyeden uzun duraklayan yazma yeni bir adım başlatır.
  Art arda Backspace'ler tek adımdır. Bir yapıştırma, bir kesme, `Ctrl+U` / `Ctrl+K`,
  eklenen bir kod parçası ya da kabul edilen bir tamamlama birer adımdır.
- Bir geri alma metni, imleci ve seçimi geri getirir. Geri almadan sonraki yeni bir
  düzenleme, yinelenebilecek olanı düşürür.
- Her alan sınırlı bir geçmiş tutar (son 200 adım, yaklaşık 1 MB metin) ve alan dışarıdan
  değiştirildiğinde (seçicide seçilen bir klasör, yeniden yüklenen bir form) ya da başka
  bir alan klavyeyi aldığında onu unutur.
- Geri alınacak bir şey kalmadığında `Ctrl+Z` hiçbir şey yapmaz: bir alan klavyeyi
  tutarken depo geri almasına asla düşmez.
- YZ ayarlarının **Yönergeler düzenleyicisinde** ayrıca Kaydet ve İptal'in yanında,
  gidilecek bir adım olmadığında devre dışı kalan **Geri al** ve **Yinele** düğmeleri
  vardır.
- Kod düzenleyicisi ve merge düzenleyicisi kendi geri almalarını korur (`Düzenle: Geri Al`,
  `Merge: Geri Al`); bu, seçimlerini de kapsar.

## Filtrelenebilir açılır listeler

Filtrelenebilir tüm açılır listeler aynı tuşları kullanır: komut paleti ve
seçicileri (tema, dil, grafik renkleri), sekme seçici ve arama satırı olan her
açılır liste (Ayarlar seçenekleri, pull request formu, issue referansı seçici,
**Şurada aç** listesi). Listeyi filtrelemek için yazın. Şu tuşlar vurguyu taşır:

| Tuş | Eylem |
|---|---|
| `Up` / `Down`, `Ctrl+K` / `Ctrl+J` | Bir satır yukarı / aşağı taşı |
| `Home` / `End` | İlk / son satır |
| `PageUp` / `PageDown` | Görünen bir sayfa taşı |
| `Enter` | Vurgulanan satırı seç |
| `Esc` | Kapat |

Bu açılır pencerelerde `Home` ve `End` filtre alanının imlecini değil listeyi
taşır; `Ctrl+K` / `Ctrl+J` de listeyi taşır, bu yüzden `Ctrl+K` artık filtreyi
sonuna kadar silmez (`Ctrl+U` hâlâ temizler; bir listenin dışındaki sıradan bir metin
alanında `Ctrl+K` satır sonuna kadar silmeyi sürdürür). `Ctrl+A` ve `Ctrl+E` imleci yine
başa ve sona taşır. `Ctrl+J` / `Ctrl+K` diğer her listeyi de taşır, metin alanıyla
veya onsuz (`Ctrl+N` gezinme önekidir ve `Ctrl+P` komut paletidir, bu yüzden liste
tuşu değildirler). Düz `j` ve `k` filtreye yazılır; metin alanı olmayan listelerde
`j` / `k` aşağı / yukarı gitmenin ek bir yoludur.

Fareyle:

- Tekerlek listeyi **kaydırır**. Vurguyu taşımaz.
- Bir tık bir satırı **seçer**. Çift tık, `Enter` gibi onu **seçip uygular**.
- Açılır penceresinden uzun bir liste sağ kenarında bir kaydırma çubuğu gösterir.
  Onu sürükleyebilirsiniz.
- Arama satırı olmayan kısa bir açılır liste (iki veya üç seçenek) ilk tıkta
  seçer.

**Canlı önizleme.** Tema, grafik renk teması ve dil seçicileri vurgulanan satırı
açılır pencerenin arkasında hemen uygular. `Enter` onu korur. `Esc` veya açılır
pencerenin dışına tıklamak önceki değeri geri yükler.

**Tema filtresi.** Tema seçicinin filtre alanının altında bir satırı vardır:
**Tümü**, **Koyu**, **Açık** ve **Renkli**. `Tab` / `Shift+Tab` sonrakine /
öncekine gider. `Alt+1` ile `Alt+4` doğrudan birini seçer. Onlara tıklayabilirsiniz
de.

## Uygulama içi yardım (`?`)

`?`, `Ctrl+N ?` veya paletin `Görünüm: Yardım Katmanını Aç/Kapat` komutu yardımı
açar. Uygulamanın her bölümünün tuşlarını ve fare hareketlerini listeler. Her
şeyin üzerinde açılır, Ayarlar ekranı dahil.

| Tuş | Eylem |
|---|---|
| `?` | Yardımı kapat (ve filtreyi temizle) |
| *(yaz)* | Filtre: yalnızca metni içeren bölümleri tut. `j`, `k` ve `q` de yazılır |
| `Backspace`, `Ctrl+U` | Son karakteri sil / filtreyi temizle |
| `Esc` | Filtreyi temizle; boşsa yardımı kapat |
| `Up` / `Down` | Bir satır kaydır |
| `PageUp` / `PageDown`, `Home` / `End` | Bir sayfa kaydır; en üste / en alta git |
| `Ctrl+D` veya bağlantıya tık | Bu belgeleri tarayıcıda GitHub'da aç |

Tekerlek yardımı kaydırır ve kaydırma çubuğunu sürükleyebilirsiniz. Yardımın
dışına tıklamak onu kapatır.

## Gezinme öneki (`Ctrl+N`)

`Ctrl+N` bir tuş daha bekler. Durum çubuğu kabul ettiği tuşları gösterir. Sonraki
tuş bir eylem çalıştırır veya iptal eder.

| Tuş | Eylem |
|---|---|
| `Ctrl+N`, sonra bir rozet | Bu rozeti gösteren pencere bölümüne odaklan |
| `Ctrl+N Shift+H` / `Ctrl+N Shift+L` | Refs panelini / sağ paneli daralt veya geri yükle |
| `Ctrl+N Left` / `Ctrl+N Right` | Refs paneli ile Log arasındaki ayırıcıyı bir adım sola / sağa kaydır (daralmış Refs paneli önce geri gelir) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Log ile sağ panel arasındaki ayırıcıyı bir adım sola / sağa kaydır |
| `Ctrl+N +` / `Ctrl+N -` | Etkin Refs bölümünü bir satır büyüt / küçült |
| `Ctrl+N ]` / `Ctrl+N [` | Sağ panelin commit mesajı bloğunu bir satır büyüt / küçült |
| `Ctrl+N Tab` | Bir sekme seç (sekme seçici; tek başına `t` aynısını yapar) |
| `Ctrl+N t` / `Ctrl+N l` / `Ctrl+N Shift+T` | Temayı / dili / grafik renk temasını değiştir |
| `Ctrl+N Shift+Y` | Grafik satır tonlarını aç veya kapat |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Bir ajanın hazırladığı görünümü göster / geri dön (bkz. [MCP ve ajanlar](mcp-and-agents.md)) |
| `Ctrl+N ,` | Ayarlar'ı aç |
| `Ctrl+N ?` | Yardımı aç veya kapat |
| `Esc` veya başka herhangi bir tuş | İptal |

### Rozetler

Önek beklerken, odağı alabilen her pencere bölümü üst sol kenarlığında tek
karakterlik bir rozet gösterir. Odaklanmak için o karakteri basın. Rozetler her
seferinde ekran sırasına göre yeniden dağıtılır: önce `1` ile `9`, sonra `a`,
`b`, `c`... Bu yüzden ekrandaki rozeti okuyun: bir bölüm her zaman aynı rozeti almaz.

Rozeti olan bölümler: Refs filtresi ve görünen her Refs bölümü, Log (veya Diff
görünümü), açıksa Dosya Görünümü, Unstaged ve Staged listeleri, commit
composer'ın özet alanı ve commit dosya listesinin denetimleri. Ana Sayfa
sekmesinde rozetler arama alanı, dört bölüm ve araç çubuğu düğmeleridir (bir düğmenin rozeti ona basar). Radar sekmesinde ise süzgeç alanı,
Kapsam, Görünüm ve Göster açılır listeleri (bir açılır listenin rozeti onu açar),
Yenile ve Geri düğmeleri ve ekrandaki her bölüm başlığıdır. Bir bölümün rozeti
seçimi başlığına taşır; ardından `Enter` veya `Space` bölümü daraltır.

Gözat klasör seçicisinde ve Klonla, Init, Worktree oluştur, Çalışma alanı, Kısmi checkout, Flow, Yeni issue ve pull request iletişim kutularında rozetler alanlar, onay kutuları ve düğmelerdir (bir düğmenin rozeti ona basar, bir alanın rozeti imleci içine koyar; bir listede tek rozet ilk satırdadır; Gözat klasör seçicisinde Hızlı erişim listesi için bir, Klasörler listesi için bir rozet vardır, her listenin ilk satırının yanında, ve birine basmak klavyeyi o listeye bir satır seçmeden koyar). Etkinlik Günlüğü'nde rozetler iki sekme, Kapsam ve Düzey kutuları, süzgeç, düzey onay kutuları, düğmeler ve listenin ilk girdisidir. Bir rozet, o denetime tıklamakla aynı şeyi yapar. Klasör seçicide `Ctrl+N` rozetleri başlatır; bu yüzden Yeni klasör tuşu `Alt+N`'dir.

Değiştir (ve Commit mesajını düzenle) iletişim kutusunda rozetler özet, açıklama, yazarın adı, e-postası ve tarihi, **Geçmişi daha kapsamlı değiştir...**, **İptal**, **Uygula** ve kapatma çarpısıdır; geçmişi yeniden yazma ekranında kapsam kutusu, onay kutuları ve ref düğmesi, tarif adı, her kural, onay kutusu ve düğmeleri, **+ Kural ekle** düğmesi, seçili kuralın alanları, önizleme sekmeleri, süzgeç kutusu, korumalı dal satırları ve onların Tümünü seç / Hiçbirini seçme düğmeleri ile Önizleme, Uygula, İptal, Kaydet, Yükle ve YZ ile Kural Oluştur düğmeleridir (Gizli bilgiler, Kimlikler, Boyut ve sonuç sayfaları kendi düğme ve satırlarını ekler; Commit'ler sekmesinin satırları, bir liste oldukları için, rozet almaz). Bir rozet tıklamayla aynı şeyi yapar. Bir açılır liste, bir soru, yazılmakta olan bir alan, klasör seçici, temizleme ekranı, betik düzenleyici, YZ iletişim kutuları, blob diff'i ya da barındırıcı yönergesi açıkken ve Değiştir iletişim kutusu "vazgeçilsin mi?" diye sorarken hiçbir şey rozetlenmez; `Esc` hiçbir şeyi kapatmadan rozetleri kaldırır.

Ayarlar'da rozetler arama kutusunda, sayfa listesinde (geçerli sayfada tek rozet), alt sekmelerde, listede ve düğmelerinde, sayfanın ekrandaki her alanında ve kapatma çarpısında bulunur. Bir rozet, o denetime tıklamakla aynı şeyi yapar: metin alanı düzenlemeye açılır, onay kutusu değişir, açılır liste açılır, düğme çalışır. Bir alana yazılırken ya da açılır liste, düzenleyici veya arama açıkken `Ctrl+N` kendi anlamını korur.

Paletin aynı eylemleri vardır; örneğin `Gezinme: Refs > Yerel'e Odaklan` veya
`Gezinme: Durum > Staged'a Odaklan`.

## Depolar ve sekmeler

| Tuş | Eylem |
|---|---|
| `Ctrl+O` veya `⌂` üzerine tık | Ana sekmeye git (Ana sekmede: arama alanına odaklan; Radar ya da sürüm notları açıkken: önce onu kapat) |
| `Ctrl+Shift+N` | Ana sekmeyi Klonla iletişim kutusuyla aç |
| `Ctrl+PageDown` / `Ctrl+PageUp`, `Ctrl+Tab` / `Ctrl+Shift+Tab` | Sonraki / önceki sekme (`Ctrl+Tab` çifti yalnızca terminal gönderiyorsa) |
| `t` | Filtrelenebilir bir listeden sekme seç |
| `Ctrl+W` | Geçerli sekmeyi kapat (önce sorar; depo değişmez) |
| `Ctrl+T` veya bir sekmeye sağ tık | Sekme menüsü (satırları [Sekme satırı](interface-tour.md#sekme-satırı) içinde listelenir) |
| `Alt+O` / `Ctrl+Alt+O` | Sekmenin klasörünü aç: **Şurada aç** listesinden seç / varsayılan girişi kullan. Bkz. [Şurada aç](open-in.md) |

### Ana sekme

`⌂` sekmesi depolarını yönetir. Dört bölümü vardır: **Açık depolar**,
**Favoriler**, **Son depolar** ve **Tüm depolar**. Bkz.
[Arayüz turu](interface-tour.md#ana-sekme).

| Tuş | Eylem |
|---|---|
| `j` / `k`, oklar, `PageUp` / `PageDown`, `Home` / `End` | Taşı |
| `Enter` veya çift tık | Depoyu aç (veya sekmesine geç). Bir bölüm başlığında: katla veya aç |
| `f` veya `*`, ya da yıldıza tık | Favorilere ekle / favorilerden çıkar |
| `x`, `Delete` veya satırın düğmesi | Sekmesini kapat (Açık), listeden kaldır (Son), yıldızı kaldır (Favoriler), unut (Tümü). Diskte hiçbir şey silinmez |
| `.`, sağ tık veya satırın `⋮` düğmesi | Satırın menüsü |
| `/` veya `Ctrl+F` | Tüm bölümlerde ada, dala veya yola göre ara. `Enter` ilk eşleşmeyi açar; `Esc` aramayı temizler, sonra alandan çıkar |
| `Space`, `Left` / `Right` | Geçerli bölümü katla / aç |
| `-` / `+` | Tümünü daralt / Tümünü genişlet |
| `b` / `c` / `i` | Gözat (bir klasör aç) / Klonla / Init |
| `Esc` | Aramayı temizle veya etkin depo sekmesine dön |
| `q` | Çık |

`Ctrl+P`, `Ctrl+,`, `?` ve `Ctrl+N` Ana sekmede çalışır. `Ctrl+G`,
`Ctrl+W` ve `Ctrl+T` bir depo sekmesi ister: Ana sekmede yalnızca bir
mesaj gösterirler. Bir depo gerektiren palet komutları gri olur.

### Klasör seçici

**Gözat** (ve Init, Klonla ve worktree oluştur iletişim kutularının **Gözat...**
düğmesi) klasör seçiciyi açar. Üstte bir yol alanı, solda yerler ve klasör ağacı,
sağda geçerli klasörün içeriği vardır.

| Tuş | Eylem |
|---|---|
| `Tab` / `Shift+Tab` | Yol alanı, sonra sol sütun, sonra içerik |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Odaktaki sütunda taşı. Yol alanında: tamamlamalar arasında gez |
| bir harf (içerik) | Bu harfle başlayan sonraki klasöre git |
| `Enter` / `Right` | Seçili klasörü aç. Yol alanında: yazılan klasöre git; zaten gösterilen klasördeyse onu seç |
| `Backspace` / `Left` / `Alt+Up` | Üst klasör (yol alanında `Backspace` bir karakter siler) |
| `Tab` (yol alanı) | Tek tamamlamayı veya vurgulananı kabul et |
| `Alt+N` | Yeni klasör (`Ctrl+N` rozetleri başlatır, bkz. [Rozetler](#rozetler)) |
| `Alt+H` | Gizli klasörleri göster veya gizle |
| `Ctrl+F` veya yol alanının dışında `/` | Klasörün girdilerini yazdıkça süz (büyük/küçük harf duyarsız; `..` kalır; başlık "x / n" sayar; `Enter` ilk eşleşmeye gider, `Esc` temizler; klasör değiştirince boşalır). `Tab` veya bir tıklamayla da ulaşılır |
| `Ctrl+S`, `Ctrl+Enter` veya onay düğmesi | Seçili alt klasörü, yoksa gösterilen klasörü seç |
| `Esc` | Tamamlamaları kapat veya iptal et |

Yol alanı yeni bir yol gösterdiğinde yolun tamamı seçilidir: bastığınız ilk tuş onu
değiştirir.

### Init ve Klon iletişim kutuları

| Tuş | Eylem |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Sonraki / önceki alan veya düğme |
| `Enter` | Odaktaki düğmeye bas veya oluştur / klonla |
| `Space` | Odaktaki onay kutusunu değiştir veya odaktaki düğmeye bas |
| `Ctrl+B` | Klasöre gözat |
| `j` / `k`, `g` / `G` | Klonla iletişim kutusunun kaynak listesinde ve depo listesinde: aşağı / yukarı, ilk / son (arama alanında değil) |
| `Esc` | Kapat. Klonlama sırasında klonlama arka planda sürer |

Klonla iletişim kutusunda sol sütun **URL ile klonla** ve bağlı her barındırma
profili için bir kaynak listeler (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Bir
kaynağı `Up` / `Down` veya tıklayarak seçin. Sonra **Depo** arama alanına yazın, listede
oklar, `PageUp` / `PageDown` veya tekerlekle gezin ve `Enter` ya da tıklamayla bir satır
seçin. Alanlar için bkz. [Başlarken](getting-started.md).

## Refs paneli

| Tuş | Eylem |
|---|---|
| `Enter` / `Space` | Etkin bölümü veya seçili klasörü genişlet ya da daralt |
| `h` / `l`, `Left` / `Right` | Bölüm değiştir |
| `/` | Ada göre filtrele (`Enter` uygular, `Esc` iptal eder) |
| `y` | Dal hiyerarşisi görünümünü aç veya kapat: dal adlarındaki `/` işaretlerinden klasörler (YEREL ve UZAK) |
| `o` | Seçili ref'i veya seçili klasör ya da uzak depo altındaki her ref'i **Solo** yap. Birleşimlerini görmek için birkaç ref'i solo yapın |
| `Shift+O` | **Solo'yu Durdur**: her solo'yu temizle |
| `Shift+H` | **Tümünü Göster**: gizlenmiş her ref'i yeniden göster |
| `g` | **Grafikte göster**: Log'u seçili ref'in (veya stash'in) commit'ine kaydır ve satırını yanıp söndür; seçim yer değiştirmez |
| `m` | Seçili ref'i bırakma kaynağı olarak **işaretle** (`Esc` ya da üzerinde yeniden `m` işareti kaldırır) |
| `Shift+M` | İşaretli ref'i seçili ref'in üzerine **bırak**: sürükle-bırakın açtığı menüyü açar (Merge, Rebase, Push, Fast-forward...) |
| `+` (UZAK bölümü) | Uzak depo ekle |
| `.`, `Menu`, `Shift+F10` veya sağ tık | Satırın menüsü: bir dal, bir tag, bir uzak deponun adı veya bir klasör |
| Bir WORKTREE'LER satırında `Enter` | O worktree'yi kendi sekmesinde aç |
| `Ctrl+G d` | Seçili dalı veya tag'i sil (önce sorar). Bir UZAK satırında: o uzak dalı sil |

Menülerin ne yaptığı için bkz. [Log ile çalışmak](log-and-history.md).

### Pull request'ler ve issue'lar

Eşleşen bir entegrasyonla Refs panelinde bir pull request bölümü bulunur (GitLab:
merge request'ler). Bağlı bir barındırma profiliyle ISSUE'LAR bölümü de bulunur.
Bkz. [Barındırma hizmetleri ve issue takipçileri](pull-requests-and-issues.md#kenar-çubuğundaki-pull-requestler-ve-issuelar).

| Tuş | Eylem |
|---|---|
| `Enter` | Seçili satırı aç: bir issue (ve Azure DevOps veya Bitbucket'ta bir istek) [konuşma](conversations.md) olarak, incelemesi olan barındırıcılarda bir pull request [incelemede](review.md), geri kalan her şey tarayıcıda. Bir `Yeniden bağlan` satırında: Ayarlar'ı o profilde aç |
| `Space` | Bölümü genişlet veya daralt |
| `v` | Bölümün filtresini aç. Pull request'ler: Açık, Merged / kapanan, Tümü. Issue'lar: Açık, Bana atanan, Kapalı, Tümü |
| `n` | Pull request bölümü: bir pull request oluştur. ISSUE'LAR bölümü: yeni issue. (Başlıktaki `+` düğmesi) |
| `e` | Pull request bölümü: seçili isteği barındırıcıda düzenle (satırın kalemi) |
| Arama satırında `Enter` | Arama alanına yaz. `Enter`, `Tab` veya `Esc` ondan çıkar; `Ctrl+U` temizler |
| Bir grup başlığında `Enter` | Grubu katla veya aç (*Pull Request'ler · Benim*, *Bana Atanan*, *İncelememi Bekleyen*, *Pull Request'ler · Tümü*) |
| Filtre, sağlayıcı veya depo satırında `Enter` | O açılır listeyi aç. Sağlayıcı listesi **Yok (bu deponun bağlantısını kaldır)** ile biter |
| `.` veya sağ tık | Satırın menüsü: konuşmayı aç, tarayıcıda aç, URL'yi veya numarayı kopyala, kaynak dalı checkout et (pull request'ler); issue'dan dal oluştur (issue'lar) |

### Issue panosu

ISSUE'LAR başlığındaki **Pano** (palet: **Issue'lar: Panoyu Aç**), issue kaynağının Kanban panosunu açar. Bkz.
[pano](pull-requests-and-issues.md#issue-panosu) ve [yerel issue'lar](local-issues.md#kanban-panosu). Aşağıdaki her
tuş aynı zamanda bir palet komutudur (**Pano: ...**).

| Tuş | Eylem |
|---|---|
| Oklar | Bir kart (`Up`, `Down`) veya komşu sütun (`Left`, `Right`) seç; `Home`, `End`, `PageUp`, `PageDown` sütunda atlar |
| `Shift+Left`, `Shift+Right` | Seçili kartı önceki veya sonraki sütuna taşı |
| `m` | **Kartı Şuraya Taşı...**: sütunların bir açılır listesi |
| `Enter` | Issue'nun [konuşmasını](conversations.md) aç (barındırıcının okunacak konuşması yoksa barındırıcının sayfası) |
| `e`, `.` | Atananları, etiketleri veya kilometre taşını düzenle; kart menüsü |
| `n` | Seçili sütunda yeni issue |
| `a`, `l`, `M`, `/` | Atanana, etikete, kilometre taşına, metne göre filtrele. `x` filtreleri temizler |
| `c` | Bir GitHub / GitLab panosunun sütunları olan etiketler |
| `r` | Issue'ları yeniden oku |
| `Esc` | Panoyu kapat |

Fare: bir kartı taşımak için başka bir sütuna sürükleyin, seçmek için bir karta tıklayın, açmak için çift tıklayın, kart
menüsü için sağ tıklayın, bir sütunu kaydırmak için tekerlek, sütunları kaydırmak için `Shift`+tekerlek (veya panonun
altındaki çubuk).

### Markdown yazma kutusu ve konuşma görünümü

Metni bir barındırıcıya giden her kutu (bir yorum, bir yanıt, bir inceleme özeti, yeni bir issue veya pull request
açıklaması) aynı yazma kutusudur. Tuşları yukarıdaki `md_editor` tablosundadır ve yeniden atanabilir.

| Tuş | Eylem |
|---|---|
| `Alt+P` | **Yaz** ile **Önizleme**'yi değiştir (**Yan yana**'dan Önizleme'ye geçer) |
| `Alt+L` | **Yan yana**'yı aç veya Yaz'a dön (100 sütun genişliğinde bir pencere ister) |
| `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` | Metni gönder (formun kendi düğmesi aynısını yapar) |
| `Alt+E` | **YZ ile Yaz**: mod çubuğunun altındaki paneli açar veya kapatır (yalnızca YZ açıkken) |
| `Alt+R` | Yalnızca commit composer'da: bir issue referansı ekle |
| `Alt+W` | Yeni issue ve pull request formları: sağlayıcının kendi web sayfasında oluştur |
| `Esc` | **YZ ile Yaz** panelinde: çalışan bir taslağı durdur, yoksa paneli kapat |

Bir issue veya isteğin [konuşma görünümünde](conversations.md) (palet satırları *Konuşma: ...*;
tuşu olan satırlar yukarıdaki palet listesindedir):

| Tuş | Eylem |
|---|---|
| `r` | Seçili kayda yanıt ver (yorum dizisine veya yorum dizisi olmayan bir barındırıcıda metni alıntılanarak) |
| `Q` | Seçili kaydı yeni bir yorumda alıntıla |
| `c` | Yeni yorum |
| `e`, `d` | Kendi yorumlarınızdan birini düzenle, sil |
| `Alt+I` | Yorum kutusunda: **Dahili not** (GitLab, Redmine) |
| `l`, `y`, `o` | Kaydın bağlantıları, adresini kopyala, tarayıcıda aç |
| `n` | Sonraki okunmamış kayıt |
| `v`, `m`, `F5` | Olayları göster veya gizle, eski kayıtları yükle, yenile |
| `g` | Kaydın diyagramını aç (bir `mermaid` bloğu) |
| `Alt+Left`, `Backspace` | Referanslardan açılan öğelerin yığınında geri git |
| `Tab` | Zaman çizelgesi, yorum kutusu, başlık düğmeleri |
| `q`, `Esc` | Kapat |

Radar'da `i` bir issue satırının konuşmasını açar.

Kenar çubuğu bölümlerinin bir pull request veya issue satırında `Enter` konuşmayı, `o` satırı tarayıcıda açar.

### Flow (dal iş akışları)

Depo için bir iş akışı kuruluyken ([dal iş akışları](branching-workflows.md)) araç çubuğunda bir **Flow**
düğmesi bulunur ve kenar çubuğu YEREL ile UZAK bölümlerini dal türüne göre gruplar. Menünün her satırı
bir palet komutudur (**Flow: ...**; **Flow: Feature Başlat...** ve **Flow: Yeni Feature Issue'su...** iş akışının
her konu türü için üretilir). Barındırıcı tarafından yürütülen satırlar (*Yeni <Tür> issue'su*, *Barındırıcının
oluşturduğu dalı devral*) ve iletişim kutuları, bekleme kartı ve "bir dal göründü" kartı
[dal iş akışlarında](branching-workflows.md#uygulamada-barındırıcı-tarafından-yürütülen-akışlar) anlatılır;
`Enter` odaktaki düğmeyi çalıştırır (*Issue oluştur*, *Devral*), `Esc` beklemeyi bırakır veya kapatır.

| Tuş | Eylem |
|---|---|
| `Ctrl+G Shift+W` | Flow menüsünü aç (araç çubuğu düğmesi) |
| `Enter`, `Tab`, `Shift+Tab` | Başlat / Bitir / Eşitle / Yükselt / Üst dalı ayarla iletişim kutusunda: önizle veya çalıştır, sonraki alan, önceki alan |
| `Space` | Bir onay kutusunu işaretle (*Yayımla*, *Hook'ları çalıştır*, *Üzerine yığılmış dalları da güncelle*) |
| `Esc` | Bir açılır listeyi kapat, önizlemeden forma dön, iletişim kutusunu kapat (bir çalıştırma sürerken değil) |
| Bir dalın üzerinde durmak | Türü, üst dalı ve üst dalının ne kadar gerisinde olduğu |

## Log paneli

| Tuş | Eylem |
|---|---|
| `Enter` / `l` | Seçili commit'in değişen dosyalar listesine gir |
| Bir dosya satırında `Enter` / `l` | Dosyanın diff'ini ortada aç |
| `Esc` / `h` | Commit'lere geri dön |
| `Right` | (birden fazla ref'i olan commit) Diğer ref'lerinin listesini aç: `Up` / `Down` taşır, `Enter` checkout eder, `Esc` kapatır |
| `f` | Arama çubuğunu aç (bkz. [Arama](#arama)); bu hiçbir zaman `/` değildi, bu yüzden Log'un araması değişmedi |
| `/` | Refs filtre alanına odaklan (Refs paneli odaktayken aynı tuş bunu yapar); bir metin alanı odaktayken karakter olarak yazılır |
| `c` | (commit'lenmemiş değişiklik varken) WIP satırını seç ve imleci mesaj alanına koy |
| WIP satırında `Enter` | İmleci WIP satırının mesaj alanına koy |
| `.`, `Menu`, `Shift+F10` | Commit menüsü. `Up` / `Down` taşır, `Right` / `l` / `Enter` bir alt menü açar, `Left` / `h` / `Esc` geri döner |
| `m` / `a` | Sağ panelin dosyaları: `Yol` veya `Ağaç` / `Tüm dosyaları göster` |
| `Shift+C` | Taşımak veya boyutlandırmak için bir Log sütunu seç (aşağıya bakın) |

Bir sütun seçiliyken `Tab` / `Shift+Tab` başka birini seçer, `<` / `>` onu taşır, `]` / `[` genişletir veya daraltır, `0` her genişliği otomatiğe döndürür, `Esc` veya `Enter` çıkar (yukarıdaki oluşturulmuş tabloda `log_columns` bağlamı).

WIP satırının mesaj alanı, commit composer'ın özetidir. İmleç ondayken her tuş
yazılır, ancak `Ctrl` birleşimleri yine çalışır. `Enter` composer'ın açıklamasına
geçer. `Esc` alandan çıkar ve metni korur. Bkz.
[Stage'leme ve commit'leme](staging-and-committing.md).

### Birden çok commit seçme

| Tuş veya hareket | Eylem |
|---|---|
| `Space` | Geçerli commit'i seçime ekle veya çıkar |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Seçimi genişlet (Shift'i kendine ayıran terminaller için Alt) |
| `Ctrl+tık` / `Shift+tık` veya `Alt+tık` | Bir commit'i aç/kapat / bir aralık seç |
| `Esc` | Seçimi temizle |
| `Shift+Y` | [Seçimi Mermaid gitGraph olarak kopyala](log-and-history.md#commitleri-mermaid-gitgraph-olarak-kopyalamak) (ayrıntı düzeyleri ve canlı önizlemesi olan bir pencere; `Enter` kopyalar) |

Birden çok commit seçiliyken birine sağ tıklayın ve **N commit'i `<branch>` üstüne
cherry-pick yap...** öğesini seçin. Etkileşimli bir rebase ile aynı görünümü açar.

### Dal veya tag adlandırma

`Ctrl+G b`, `Ctrl+G t`, araç çubuğunun **Dal** ve **Tag** düğmeleri ile
menülerdeki **Burada dal oluştur...** / **Burada tag oluştur...** adı commit'in
satırındaki bir metin kutusunda ister. `Enter` ref'i oluşturur. `Esc` iptal eder.
Geçersiz veya var olan bir ad kırmızıyla gösterilir ve kutu açık kalır.

Bu satır ekranda değilse (bir diff açık, Log tek başına gösteriliyor, Ref
sütunu gizli), ad bunun yerine kapatma çarpısı, **İptal** ve **Oluştur**
düğmeleri olan küçük çerçeveli bir kutuya yazılır. Bir dalın adını
değiştirmek Refs panelindeki satırının üzerine ya da Log satırının Ref
hücresine yazılarak yapılır; dosya oluşturmak Çalışma değişiklikleri
listesinde, klasörün altında yazılır. Her yerde aynı tuşlar geçerlidir:
`Enter` onaylar, `Esc` iptal eder.

Durum çubuğuna hiçbir zaman bir şey yazılmaz. Daha fazla yer isteyen
istemler gerçek düğmeleri olan iletişim kutularıdır: stash mesajı, açıklamalı
bir tag'in mesajı (`Ctrl+Enter` veya `Ctrl+S` uygular, `Enter` satır ekler),
upstream ayarlama (uzak dalların aranabilir bir listesi ve bir Upstream'i
kaldır düğmesi) ve Git LFS izleme ile izlemeyi bırakma desenleri. Paletten
seçilen bir sayı veya metin ayarı, Ayarlar'ı o satırda, düzenlemeye hazır
açar; Refs filtresi (`/`) ise Refs panelinin üstündeki kutuya yazılır.

### Commit mesajını düzenleme

Commit menüsündeki **Commit mesajını düzenle...**, `Ctrl+G e` veya paletin
`Git: Commit Mesajını Düzenle...` komutu **Commit mesajını yeniden yaz**
açılır penceresini açar. Neyi değiştirdiği için bkz.
[Log ile çalışmak](log-and-history.md).

| Tuş | Eylem |
|---|---|
| Özette `Enter` | Açıklamaya git |
| Açıklamada `Enter` | Yeni satır. İlk satırda `Up` özete geri döner |
| `Ctrl+Enter`, `Ctrl+S` veya **Mesajı Güncelle** | Yeni mesajı kaydet |
| `Tab` / `Shift+Tab` | Özet, açıklama ve düğmeler arasında geç |
| `Esc` veya **İptal** | Değişiklik yapmadan kapat |

### Bir commit'i değiştirme

`Ctrl+G Shift+E` (tüm alanlar) ve `Ctrl+G Shift+A` (yazar) Değiştir iletişim kutusunu açar; commit menüsü, paletin `Commit: Değiştir...` satırları ve commit ayrıntıları paneli de açar. Her giriş yolu [Bir commit'i değiştirmek](log-and-history.md#bir-commiti-değiştirmek-mesaj-yazar-tarihler) içinde anlatılır. İki kısayol da yeniden atanabilir: `git_prefix.amend` ve `git_prefix.edit_author`. İletişim kutusundaki tuşlar:

| Tuş | Eylem |
|---|---|
| `Tab` / `Shift+Tab` | Alanlar ve düğmeler arasında geç |
| Tek satırlık alanda `Enter` | Sonraki alan; sonuncudan sonra **Uygula** |
| `Up` / `Down` | Önceki / sonraki alan (açıklamada ilk / son satırdan) |
| `Ctrl+Enter`, `Ctrl+S` veya **Uygula** | Değişen alanları uygula |
| `Esc` veya **İptal** | Değişiklik yapmadan kapat |
| `Alt+H` veya **Geçmişi daha kapsamlı değiştir...** | İletişim kutusunu kapatır ve geçmişi yeniden yazma ekranını bu commit'ten açar (yalnızca geçmiş yeniden yazma içeren derlemelerde); bir alanı düzenlediyseniz önce sorar (`Esc` onaylar). Yeniden atanabilir: `reword.deeper` |

### Worktree oluşturma

**`<ref>` üzerinden worktree oluştur...** (dal ve tag menüleri) ile **Burada
worktree oluştur...** (commit menüsü) bir iletişim kutusu açar: klasör, isteğe
bağlı yeni bir dal ve **Yeni worktree'yi sekme olarak aç**. Tuşlar: `Tab` /
`Shift+Tab` gezer, `Enter` oluşturur, `Space` onay kutusunu değiştirir, `Ctrl+B`
gözatar, `Esc` iptal eder.

### Etkileşimli rebase

Commit menüsündeki **Buradan etkileşimli rebase...** veya
`Git: Seçili Commit'ten Etkileşimli Rebase...` rebase görünümünü açar. Birden çok
commit'in cherry-pick'i aynı görünümü kullanır. Bkz.
[Log ile çalışmak](log-and-history.md).

| Tuş | Eylem |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Satırın eylemini ayarla: Pick / Reword / Squash / Fixup / Drop / Edit (Fixup ve Edit yalnızca rebase'de) |
| `Space` | Satırın eylem listesini aç (`Up` / `Down` seçer, `Enter` uygular, `Esc` listeyi kapatır) |
| `Up` / `Down` | Seçimi taşı |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Seçili satırı yukarı / aşağı taşı |
| `a` | Autosquash önizlemesini aç veya kapat |
| Bir Reword satırında `Enter` | O commit'in yeni mesajını düzenle |
| `Ctrl+S`, `Ctrl+Enter` veya **Rebase'i Başlat** / **Cherry Pick'i Başlat** | Planı çalıştır |
| `Ctrl+R` veya **Sıfırla** | Planı görünüm açıldığındaki haline geri yükle |
| `Esc` veya **İptal** | Değişiklik yapmadan kapat |

Bir rebase durduğunda (bir çakışmada veya bir Edit adımında) Log'un üstündeki bir
banner **Devam et**, **Atla** ve **İptal et** sunar. Paletin `Rebase: Devam Et`,
`Rebase: Atla` ve `Rebase: İptal Et` komutları vardır.

## Arama

`f`, araç çubuğunun **Ara** düğmesi veya `Gezinme: Commit'lerde Ara...` Log'un
üstünde arama çubuğunu açar. Tam commit mesajında, yazar adında ve e-postasında,
SHA'da ve ref adlarında arar. Satırları gizlemez: eşleşmeyen satırlar soluk
gösterilir.

| Tuş | Eylem |
|---|---|
| *(yaz)* | Arama metnini düzenle (her tuş yazılır) |
| `Enter` / `Down` veya çubuğun `↑` düğmesi | Sonraki eşleşme |
| `Shift+Enter` / `Up` veya çubuğun `↓` düğmesi | Önceki eşleşme |
| `Esc` veya çubuğun `✕` düğmesi | Arama çubuğunu kapat |

## Diff görünümü

Değişmiş bir dosya Log'un yerine ortada açılır. Bkz.
[Stage'leme ve commit'leme](staging-and-committing.md).

| Tuş | Eylem |
|---|---|
| `Enter` / `l` veya çift tık (bir dosya satırında) | Dosyanın diff'ini aç |
| `Esc`, `q` veya başlıktaki `✕` | Diff'i kapat |
| `j` / `k`, `Up` / `Down` | Kaydır |
| `PageUp` / `PageDown`, `Home` / `End` | Bir sayfa kaydır; başa / sona git |
| `h` / `l`, `Left` / `Right` | Yana kaydır (sözcük kaydırma açıkken değil) |
| `n` / `p` | Sonraki / önceki hunk |
| `m` | Satır içi veya bölme yerleşimi |
| `Shift+H` | Yalnızca hunk'lar veya tüm dosya |
| `w` | Sözcük kaydırmayı aç veya kapat |
| `s` | Yalnızca boşluk değişikliklerini göster veya gizle |
| `Shift+M` | Minimap'i aç veya kapat |
| `Shift+C` | Dosyanın okunduğu kodlamayı seç (bir liste: önce Otomatik algıla) |
| `e` | Dosyayı yerleşik düzenleyicide düzenle (bkz. [Düzenleyici](#düzenleyici)) |
| `r` | (Git LFS dosyası) LFS kartı veya ham işaretçi. Bkz. [Git LFS](lfs.md) |
| `Shift+E` | (çakışmalı dosya) Merge düzenleyiciyi aç. Bkz. [Merge çakışmalarını çözmek](merge-conflicts.md) |
| `y` / `Ctrl+C` | Seçili metni kopyala. Seçim yoksa: imlecin altındaki satırı veya seçili satırları |
| `.` veya bir hunk'a sağ tık | Hunk menüsü |

### Diff'ten stage'leme ve atma

Bu tuşlar yalnızca çalışma değişikliklerinin diff'inde çalışır (Unstaged veya
Staged listesinden açılan). Atma yalnızca Unstaged listesinde çalışır.

| Tuş | Eylem |
|---|---|
| `Space` | İmlecin altındaki hunk'ı stage'le (Unstaged) / unstage'le (Staged) |
| `v` | Satır seçimini başlat veya bırak; `j` / `k` genişletir |
| `Enter` | Seçili satırları veya imlecin altındaki satırı stage'le / unstage'le |
| `x` | İmlecin altındaki hunk'ı at (önce sorar: `y` / `Enter` onaylar, `n` / `Esc` iptal eder) |
| `Shift+X` | Seçili satırları veya imlecin altındaki satırı at (önce sorar) |

Yalnızca değişen satırlar sayılır: bir seçimdeki bağlam satırları yok sayılır.
İkinci bir hunk'a uzanan bir seçim yalnızca ilk hunk üzerinde işlem yapar.

## Görsel görünümü

Diff görünümünde veya Dosya Görünümünde bir resim. Tuşlar yukarıdaki tablonun
`image` bağlamındadır. Bkz. [Görseller](image-view.md).

## Dosya Görünümü, Blame ve Dosya Geçmişi

| Tuş | Eylem |
|---|---|
| `Esc` / `q` | Kapat (bir metin seçimiyle `Esc` önce onu temizler) |
| `j` / `k`, `Up` / `Down` | Seçimi taşı |
| `PageUp` / `PageDown` | Bir sayfa kaydır |
| `Home` / `End`, `Shift+G` | İlk / son satır |
| `h` / `l`, `Left` / `Right` | Metni yana kaydır |
| `Enter` | (Blame, Dosya Geçmişi) Satırın commit'ine Log'da git |
| `e` | Dosyayı yerleşik düzenleyicide düzenle |
| `y` / `Ctrl+C` | Seçili metni kopyala. Seçim yoksa: seçili satırı (Blame: yalnızca kodu; Dosya Geçmişi: özeti) |

Blame ve dosya geçmişinin kendi tuşları vardır; yukarıdaki üretilmiş tabloda (`file_insight` ve `history` bağlamları) ve paletteki `Blame: ...` / `Dosya Geçmişi: ...` olarak listelenir; hepsi yeniden atanabilir. Ayrıntılar [Blame](log-and-history.md#blame) ve [Dosya geçmişi](log-and-history.md#dosya-geçmişi) bölümlerinde. Dosya geçmişi listesinde tek harfler (`d` `o` `b` `c` `y`) seçili satıra işler, ancak bir Ctrl kombinasyonu (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) her zaman genel anlamını korur.

## Commit sütunu

Bir dosyanın görünümünün solundaki sütun (bkz. [Commit sütunu](interface-tour.md#commit-sütunu)). `Alt+C` onu her yerden gösterir veya gizler; diğer tuşlar sütun klavyeyi aldığında çalışır (`Ctrl+N` ve rozeti, veya `Görünüm: Commit sütununa odaklan`).

| Tuş | Eylem |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Bir commit seç |
| `Enter` / `Right` | Seçilen commit'in dosya listesine geç |
| `Esc` | Klavyeyi görünüme geri ver |
| `.`, `Menu`, `Shift+F10` | Seçilen satırın menüsü |

Diğer tuşları (yana kaydırmak için `Shift+Left` / `Shift+Right`, bölümleri boyutlandırmak için `Tab`, `]`, `[` ve `0`) yukarıdaki üretilmiş tabloda, `commit_column` bağlamındadır.

## Düzenleyici

**Düzenle**, ya da bir diff'te veya Dosya Görünümünde `e`, yerleşik düzenleyiciyi
dosyanın çalışma ağacındaki kopyasında açar. Bkz.
[Stage'leme ve commit'leme](staging-and-committing.md).

| Tuş | Eylem |
|---|---|
| *(yaz)* | İmlecin olduğu yere ekle (seçimin yerine geçer) |
| `Enter` | Geçerli satırın girintisiyle yeni satır |
| `Tab` / `Shift+Tab` | Girintile / girintiyi azalt (birden çok satır seçiliyse seçili satırları) |
| oklar, `Home` / `End` | Taşı. `Home` ilk boşluk olmayan karaktere, sonra 0. sütuna gider. `Shift` seçimi genişletir |
| `Ctrl+Left` / `Ctrl+Right` | Bir sözcük taşı |
| `Ctrl+Home` / `Ctrl+End` | Dosyanın başı / sonu |
| `PageUp` / `PageDown` | Bir sayfa yukarı / aşağı |
| `Ctrl+A` | Tümünü seç |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Kopyala / kes / yapıştır |
| `Ctrl+U` / `Ctrl+K` | Satırın başına / sonuna kadar sil |
| `Ctrl+Z` | Geri al |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Yinele |
| `Ctrl+S` | Kaydet |
| `Ctrl+F` / `F3` | Bul. Bul alanında: `Enter` / `Down` / `F3` sonraki, `Shift+Enter` / `Up` önceki, `Esc` kapatır |
| `Alt+M` | Minimap'i aç veya kapat (**Map**) |
| `Esc` | Kapat. Kaydedilmemiş değişiklik varsa sorar: **Kaydet** (`S` / `Enter`), **At** (`D`), **İptal** (`C` / `Esc`) |

`Tab` düzenleyicide girintiler, bu yüzden odağı taşımaz. Diğer `Ctrl`
birleşimleri (`Ctrl+P`, `Ctrl+N`) yine çalışır.

Durum çubuğundaki **kodlama**, **satır sonları** ve **girinti** düğmedir: her biri bir açılır liste açar
(oklar veya tekerlek, filtrelemek için yazın, seçmek için `Enter`, kapatmak için `Esc`). Komut paleti (`Ctrl+P`)
aynı menüleri açar: *Düzenleyici: Kodlamayı Değiştir*, *Düzenleyici: Kodlamayla Yeniden Aç*, *Düzenleyici: Kodlamaya Dönüştür*, *Düzenleyici: Satır Sonlarını Dönüştür*, *Düzenleyici: Girintiyi Değiştir*. Ayrıntılar [Stage'leme ve commit'leme](staging-and-committing.md) içinde.

## Git eylemleri (`Ctrl+G`)

`Ctrl+G` bir tuş daha bekler:

| Tuş | Eylem |
|---|---|
| `s` / `u` | Seçili dosyayı stage'le / unstage'le |
| `a` | Tüm değişiklikleri stage'le |
| `x` | Tüm değişiklikleri at (önce sorar) |
| `c` | Commit composer'ı aç |
| `Enter` | Stage'lenmiş değişiklikleri formdaki mesajla hemen commit et, her yerden (bkz. [Stage'leme ve commit'leme](staging-and-committing.md#commiti-yazma)) |
| `m` | Tüm değişiklikleri bir mesajla stash'le, küçük bir iletişim kutusuna yazılır (boş olabilir). YZ açıkken YZ düğmesi (iletişim kutusunda `Ctrl+G`) mesajı yazar |
| `Shift+T` | Tüm değişiklikleri (izlenmeyen dosyalar dahil) mesajsız stash'le |
| `o` | Bir stash'i pop et: Log'da seçili stash satırı, yoksa en son stash |
| `p` | Pull, Pull düğmesinin ayarlandığı biçimde (`git.pull_mode`) |
| `Shift+P` | Varsayılan uzak depoya push et (ilk push'ta upstream'i ayarlar) veya uzak depoyu sor |
| `Shift+U` | Tüm tag'leri push et |
| `f` | Her uzak depoyu prune ile fetch et |
| `g` | Checkout edilmiş dalın pull request'ini tarayıcıda aç |
| `b` / `t` | HEAD'de yeni dal / yeni tag (bkz. [Dal veya tag adlandırma](#dal-veya-tag-adlandırma)) |
| `d` | Seçili dalı veya tag'i sil (önce sorar) |
| `e` | Seçili commit'in mesajını düzenle |
| `Shift+E` / `Shift+A` | Seçili commit'i değiştir (tüm alanlar) / yazarını değiştir (bkz. [Bir commit'i değiştirme](#bir-commiti-değiştirme)) |
| `y` | Seçili commit'i cherry-pick yap |
| `v` | Seçili commit'i revert et |
| `Shift+R` | Seçili commit'e reset (mixed) |
| `r` | Seçili dalın üstüne rebase |
| `Shift+W` | Flow menüsünü aç (dal iş akışı olan bir depoda) |
| `Shift+H` | Geçmişi yeniden yazma ekranını aç (yalnızca geçmiş yeniden yazma içeren derlemelerde) |
| `l` | LFS menüsünü aç. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V`: LFS fetch all / checkout all / pull / push all / prune. Bkz. [Git LFS](lfs.md) |
| `i` | ✦ YZ: stage'lenmiş değişiklikler için commit mesajını yaz |
| `w` | ✦ YZ: seçili commit'i veya çalışma değişikliklerini açıkla |
| `Shift+C` | ✦ YZ: çalışma değişikliklerinin commit'lere nasıl bölüneceğini öner |

YZ tuşları yalnızca YZ açıkken çalışır (bkz. [YZ desteği](ai.md)). `Esc` çalışan
bir YZ isteğini iptal eder.

## Sağ panel (çalışma değişiklikleri ve commit ayrıntıları)

WIP satırı seçiliyken sağ panel **Unstaged** ve **Staged** listelerini ve commit
composer'ı gösterir. Bir commit seçiliyken commit ayrıntılarını ve dosyalarını
gösterir.

| Tuş | Eylem |
|---|---|
| `j` / `k`, `Up` / `Down` | Taşı. Bir listenin sonunda seçim diğerine geçer |
| `h` / `l`, `Left` / `Right` | Unstaged / Staged listesine git |
| `Enter` | Etkin listeyi daralt veya genişlet |
| `Space` / `Insert` | Geçerli dosyayı seçime ekle ya da seçimden çıkar |
| `Ctrl+A` | Listedeki tüm dosyaları seç |
| `Ctrl+Space` | Seçili dosyayı stage'le / unstage'le veya bir klasör satırını katla |
| `d` | Seçili dosyanın diff'ini aç |
| `m` | `Yol` (düz liste) veya `Ağaç` (klasörler) |
| `a` | `Tüm dosyaları göster`: yalnızca değişen dosyalar değil, tüm ağaç |
| `z` | Tüm klasörleri daralt veya yeniden genişlet |
| `/` | Bir commit'in değişen dosyalarını ada göre filtrele |
| `.`, `Menu`, `Shift+F10` | Dosyanın menüsü |
| `c` | Commit composer'ın özet alanına odaklan |
| `Shift+A` | **Önceki commit'i amend et** seçeneğini aç veya kapat |
| `Ctrl+N ]` / `Ctrl+N [` | Commit mesajı bloğunu büyüt / küçült |

Commit composer'da:

| Tuş | Eylem |
|---|---|
| `Enter` | Özet: açıklamaya git. Açıklama: yeni satır |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commit (`Ctrl+G Enter` bunu her yerden yapar) |
| `Alt+R` | Bir issue referansı ekle |
| `Esc` | Composer'dan çık |

Dosya menüsü şunları içerir: Stage'le veya Unstage'le, Değişiklikleri at, Yok
say, Dosyayı Stash, Dosya Geçmişi, Dosya Blame'i, Harici diff aracında aç, Harici
düzenleyicide aç, Dosyayı varsayılan programda aç, Klasörde göster, Dosya yolunu
kopyala, Dosya değişikliklerinden yama oluştur..., Dosyayı düzenle ve Dosyayı sil...
Çakışmalı bir dosyada **Merge Düzenleyicide Aç** ve üç **Şununla çöz** satırıyla
başlar. Git LFS açıkken LFS satırları vardır. **Değişiklikleri at...** ve **Dosyayı
sil...** önce sorar.

## Pencere düğmeleri

Her düğmenin bir klavye karşılığı vardır:

| Düğme | Tuş |
|---|---|
| Başlık çubuğunun solundaki <img src="../../../assets/logo/gitpoulpi.png" alt="GitPoulpi simgesi" height="16"> (Hakkında) | Paletteki `Yardım: GitPoulpi Hakkında` |
| Başlık çubuğundaki **Şurada aç** | `Alt+O`, `Ctrl+Alt+O` |
| Başlık çubuğunun sağındaki `×` (çık) | `q` |
| Başlık çubuğundaki sürüm (Sürüm notları) | Paletteki `Yardım: Sürüm Notları` |
| Başlık çubuğunda `⚙`'in solundaki `⌕ Genel arama` | `Ctrl+Alt+F` |
| Başlık çubuğundaki `⚙` (Ayarlar) | `Ctrl+,` |
| Sekme satırındaki çalışma alanı rozeti | `Alt+W` |
| Son sekmeden sonraki `+` | `Alt+T` |
| **Radar** sekmesi | `Alt+R` |
| **Çalışma alanları** sekmesi | `Alt+J` |
| Durum çubuğundaki **Etkinlik** | `Ctrl+N Shift+A` |
| Araç çubuğu: **Geri al**, **Yinele** | `Ctrl+Z`, `Ctrl+Y` |
| Araç çubuğu: **Pull** / `▾` düğmesi | `Ctrl+G p` / `Git: Pull Seçenekleri...` |
| **Pull: tüm dallar** (Pull `▾` listesinin bir satırı) | `Ctrl+G Ctrl+P` |
| Araç çubuğu: **Push** / `▾` düğmesi | `Ctrl+G Shift+P` / `Git: Uzak Depoya Push Et...` |
| Araç çubuğu: **Dal**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Araç çubuğu: **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Araç çubuğu: **LFS** (LFS açıkken) | `Ctrl+G l` |
| Araç çubuğu: **Flow** (depoda bir iş akışı varken) | `Ctrl+G Shift+W` |
| Araç çubuğu: **Ara** | `f` |
| Araç çubuğu: bağlı pull request (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Araç çubuğu: profil listesi (iki veya daha fazla profille) | `Depo: Profili Değiştir...` |

Araç çubuğu [Arayüz turu](interface-tour.md#araç-çubuğu) içinde anlatılır.

## Ayarlar

`Ctrl+,`, `Ctrl+N ,`, `⚙` dişlisi veya `Uygulama: Ayarlar...` Ayarlar
ekranını açar.

| Tuş | Eylem |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Sayfa listesi ile seçenekler arasında geç |
| `j` / `k`, `Up` / `Down` | Seçimi taşı |
| `Enter` / `Space` | Bir sayfa aç, bir onay kutusunu değiştir, bir alanı düzenle, bir açılır liste aç veya bir düğmeye bas |
| `/`, `Ctrl+F` | Tüm seçeneklerde ara |
| `Alt+Left` / `Alt+Right` | Alt sekme değiştir (Genel / liste) |
| `Esc` / `q` | Ayarlar'ı kapat. Bir alan düzenlenirken `Esc` yalnızca düzenlemeyi iptal eder |
| `?` | Yardımı Ayarlar ekranının üzerinde aç |

Bkz. [Ayarlar](settings.md).

## Metin alanları

Tüm metin alanları aynı tuşları paylaşır: filtreler, arama alanları, komut
paleti, istemler, commit composer, WIP satırının alanı, iletişim kutusu alanları
ve Ayarlar alanları. Bir alan klavyeyi tutarken bu tuşlar panel kısayollarına
üstün gelir.

| Tuş | Eylem |
|---|---|
| bir karakter | İmlecin olduğu yere yaz (seçimin yerine geçer) |
| `Left` / `Right` | Bir karakter taşı (seçim varsa: başına / sonuna git) |
| `Ctrl+Left` / `Ctrl+Right` | Bir sözcük taşı |
| `Home` / `End` | Satırın başı / sonu |
| `Shift` + yukarıdaki bir hareket | Seçimi genişlet veya daralt |
| `Up` / `Down` (yalnızca açıklama) | Önceki / sonraki satır |
| `Ctrl+Home` / `Ctrl+End` (yalnızca açıklama) | Metnin başı / sonu |
| `Ctrl+A` | Tümünü seç |
| `Backspace` / `Delete` | İmlecin öncesindeki / sonrasındaki karakteri veya seçimi sil |
| `Ctrl+U` | İmleçten satırın başına kadar sil (veya seçimi) |
| `Ctrl+K` | İmleçten satırın sonuna kadar sil (veya seçimi) |
| `Ctrl+C` / `Ctrl+X` | Seçimi kopyala / kes (parola alanında değil) |
| yapıştır (terminalin kendi yapıştırması) | İmlecin olduğu yere ekle, seçimin yerini al |

Commit açıklamasında `Enter` yeni satır yazar ve `Ctrl+Enter` commit'ler.
Terminallerin hepsi `Ctrl+Enter`'ı aynı biçimde göndermez, bu yüzden `Alt+Enter`
ve `Ctrl+J` de commit'ler.

Bazı konsollar `Ctrl+harf` tuşunu denetim karakteri olarak gönderir (`Ctrl+A`
`0x01` olarak). GitPoulpi iki biçimi de anlar. `AltGr` karakterleri (İsviçre veya
Fransız klavyelerinde `@`, `#`, `€`) yazılır.

## İpuçları

İşaretçi bir hedefin (bir Log hücresi, bir araç çubuğu düğmesi, bağlı pull
request) üzerinde yaklaşık 0,6 sn durduğunda bir ipucu belirir. Bir tuşa basmak,
bir tık, bir kaydırma, yeniden boyutlandırma, açık bir açılır pencere veya menü
ya da işaretçinin hedeften ayrılması onu gizler. Bundan sonra yalnızca işaretçi
yeniden hareket ettiğinde geri gelir.
