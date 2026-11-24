[English](../keybindings.md) · [Čeština](../cs/keybindings.md) · [Deutsch](../de/keybindings.md) · [Español](../es/keybindings.md) · **Français** · [Italiano](../it/keybindings.md) · [日本語](../ja/keybindings.md) · [한국어](../ko/keybindings.md) · [Nederlands](../nl/keybindings.md) · [Polski](../pl/keybindings.md) · [Português (Brasil)](../pt-BR/keybindings.md) · [Português (Portugal)](../pt-PT/keybindings.md) · [Русский](../ru/keybindings.md) · [Svenska](../sv/keybindings.md) · [Türkçe](../tr/keybindings.md) · [Українська](../uk/keybindings.md) · [简体中文](../zh-CN/keybindings.md)

# Raccourcis clavier

Cette page liste les touches **par défaut**. Vous pouvez en réaffecter la plupart.
L'application montre toujours les touches en vigueur : dans la palette de commandes,
l'aide `?`, les lignes d'état, les menus contextuels et les infobulles des boutons.
Quand vous déplacez une touche, ces endroits montrent la nouvelle. Pour la souris, voir
[Souris et gestes](mouse-and-gestures.md).

## Réaffecter

Ouvrez **Paramètres > Clavier et souris**. La page est décrite dans
[Paramètres](settings.md#clavier-et-souris). Les changements s'appliquent aussitôt et sont
enregistrés dans la table `[keybindings]` de `config.toml`. Vous pouvez aussi modifier
cette table à la main :

```toml
[keybindings.global]
git_prefix = "Ctrl+B"      # tmux and GNU screen often use Ctrl+B and Ctrl+A
nav_prefix = "Alt+n"

[keybindings.git_prefix]
stage_file = "z"
```

La syntaxe, les règles de conflit et les avertissements sont dans la
[Référence de configuration](config-reference.md) sous `[keybindings]`. En bref :

- Une erreur est signalée sur la ligne d'état, et la touche par défaut reste.
- Deux actions sur un même enchaînement dans le même contexte sont refusées.
- Un préfixe demande `Ctrl` ou `Alt`.
- Un enchaînement de volet qui masque un enchaînement global ne donne qu'un
  avertissement.

Le tableau ci-dessous liste chaque touche que vous pouvez réaffecter. Certaines touches
sont fixes et ne peuvent pas être réaffectées : le déplacement dans les listes (`j` `k`
`h` `l`, les flèches, `PageUp` / `PageDown`, `Home` / `End`), `Enter`, `Esc`, `Tab`, les
chiffres (le préfixe de nombre), les touches d'édition de texte, et les touches `Enter` /
`Tab` / flèches des fenêtres contextuelles. La palette a `Application : Raccourcis clavier...`
et `Raccourcis : Tout réinitialiser...`.

## Raccourcis par défaut

Chaque touche que vous pouvez réaffecter, avec l'identifiant à utiliser dans
`[keybindings.<contexte>]` et sa valeur par défaut. Le tableau est généré à partir de
l'application (un test le garde à jour). Les touches après un préfixe sont montrées avec le
préfixe (`Ctrl+G s`).

<!-- keymap:begin -->
| Contexte | Id de l'action | Rôle | Touches par défaut |
|---|---|---|---|
| `global` | `git_prefix` | Préfixe des actions Git | `Ctrl+G` |
| `global` | `nav_prefix` | Préfixe de navigation | `Ctrl+N` |
| `global` | `palette` | Palette de commandes | `Ctrl+P` |
| `global` | `settings` | Paramètres | `Ctrl+,` |
| `global` | `home` | Accueil (gestion des dépôts) | `Ctrl+O` |
| `global` | `clone` | Cloner un dépôt | `Ctrl+Shift+N` |
| `global` | `new_tab` | Nouvel onglet | `Alt+T` |
| `global` | `workspace_switch` | Changer d'espace de travail | `Alt+W` |
| `global` | `radar` | Ouvrir le Radar | `Alt+R` |
| `global` | `workspaces` | Ouvrir les espaces de travail | `Alt+J` |
| `global` | `tab_menu` | Menu de l'onglet | `Ctrl+T` |
| `global` | `tab_open_in_popup` | Ouvrir dans (liste) | `Alt+O` |
| `global` | `tab_open_in_default` | Ouvrir dans l'entrée par défaut | `Ctrl+Alt+O` |
| `global` | `close_tab` | Fermer l'onglet | `Ctrl+W` |
| `global` | `next_tab` | Onglet suivant | `Ctrl+PageDown`, `Ctrl+Tab` |
| `global` | `prev_tab` | Onglet précédent | `Ctrl+PageUp`, `Ctrl+Shift+Tab` |
| `global` | `help` | Aide | `?` |
| `global` | `quit` | Quitter | `q` |
| `global` | `refresh` | Actualiser | `r` |
| `global` | `refresh_all` | Actualiser (fetch, pull requests, tickets) | `F5` |
| `global` | `find_in_log` | Chercher dans le graphe des commits | `f` |
| `global` | `workspace_search` | Rechercher dans l'espace de travail | `Ctrl+Alt+F` |
| `global` | `select_tab` | Sélectionner un onglet | `t` |
| `global` | `undo` | Annuler la dernière opération | `Ctrl+Z` |
| `global` | `redo` | Rétablir l'opération annulée | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `global` | `undo_history` | Ouvrir l'historique d'annulation | `Alt+Z` |
| `global` | `commit_column` | Afficher ou masquer la colonne des commits | `Alt+C` |
| `global` | `undo_offer` | Restaurer la branche d'avant le blocage | `Alt+Shift+Z` |
| `global` | `context_menu` | Menu contextuel | `.`, `Menu`, `Shift+F10` |
| `git_prefix` | `compose_commit` | Rédiger le message de commit | `Ctrl+G c` |
| `git_prefix` | `commit_now` | Commiter le stage maintenant | `Ctrl+G Enter` |
| `git_prefix` | `ai_compose` | IA : répartir les modifications en commits | `Ctrl+G Shift+C` |
| `git_prefix` | `ai_explain` | IA : expliquer les modifications | `Ctrl+G w` |
| `git_prefix` | `stash_message` | Stash avec message | `Ctrl+G m` |
| `git_prefix` | `ai_commit_message` | IA : rédiger le message de commit | `Ctrl+G i` |
| `git_prefix` | `stage_all` | Stage de toutes les modifications | `Ctrl+G a` |
| `git_prefix` | `discard_all` | Abandonner toutes les modifications | `Ctrl+G x` |
| `git_prefix` | `stage_file` | Stage du fichier sélectionné | `Ctrl+G s` |
| `git_prefix` | `unstage_file` | Unstage du fichier sélectionné | `Ctrl+G u` |
| `git_prefix` | `pull` | Pull | `Ctrl+G p` |
| `git_prefix` | `pull_all_branches` | Pull de toutes les branches | `Ctrl+G Ctrl+P` |
| `git_prefix` | `push` | Push | `Ctrl+G Shift+P` |
| `git_prefix` | `open_linked_pr` | Ouvrir la pull request liée | `Ctrl+G g` |
| `git_prefix` | `push_tags` | Push de tous les tags | `Ctrl+G Shift+U` |
| `git_prefix` | `fetch_all` | Fetch de tous les distants | `Ctrl+G f` |
| `git_prefix` | `new_tag` | Nouveau tag | `Ctrl+G t` |
| `git_prefix` | `new_branch` | Nouvelle branche | `Ctrl+G b` |
| `git_prefix` | `stash_push` | Stash des modifications | `Ctrl+G Shift+T` |
| `git_prefix` | `cherry_pick` | Cherry-pick du commit sélectionné | `Ctrl+G y` |
| `git_prefix` | `revert` | Revert du commit sélectionné | `Ctrl+G v` |
| `git_prefix` | `edit_message` | Modifier le message de commit | `Ctrl+G e` |
| `git_prefix` | `amend` | Modifier le commit sélectionné | `Ctrl+G Shift+E` |
| `git_prefix` | `edit_author` | Modifier l'auteur du commit sélectionné | `Ctrl+G Shift+A` |
| `git_prefix` | `reset_mixed` | Reset (mixed) au commit sélectionné | `Ctrl+G Shift+R` |
| `git_prefix` | `rebase_onto` | Rebase sur la branche sélectionnée | `Ctrl+G r` |
| `git_prefix` | `stash_pop` | Pop du stash | `Ctrl+G o` |
| `git_prefix` | `delete_ref` | Supprimer la ref ou drop du stash | `Ctrl+G d` |
| `git_prefix` | `lfs_menu` | Ouvrir le menu LFS | `Ctrl+G l` |
| `git_prefix` | `flow_menu` | Ouvrir le menu Flow | `Ctrl+G Shift+W` |
| `git_prefix` | `lfs_fetch_all` | LFS : Fetch de tous les objets | `Ctrl+G Shift+F` |
| `git_prefix` | `lfs_checkout` | LFS : Checkout de tous les fichiers | `Ctrl+G Shift+K` |
| `git_prefix` | `lfs_pull` | LFS : Pull des objets | `Ctrl+G Shift+L` |
| `git_prefix` | `lfs_push_all` | LFS : Push de tous les objets | `Ctrl+G Shift+J` |
| `git_prefix` | `lfs_prune` | LFS : Élaguer les objets locaux | `Ctrl+G Shift+V` |
| `git_prefix` | `rewrite_workbench` | Ouvrir l'atelier de réécriture de l'historique | `Ctrl+G Shift+H` |
| `nav_prefix` | `collapse_refs` | Replier ou rétablir le volet Refs | `Ctrl+N Shift+H` |
| `nav_prefix` | `collapse_status` | Replier ou rétablir le volet Status | `Ctrl+N Shift+L` |
| `nav_prefix` | `settings` | Ouvrir les paramètres | `Ctrl+N ,` |
| `nav_prefix` | `about` | Ouvrir À propos | `Ctrl+N a` |
| `nav_prefix` | `activity_log` | Ouvrir le Journal d'activité | `Ctrl+N Shift+A` |
| `nav_prefix` | `global_log` | Ouvrir le journal d'activité global | `Ctrl+N Shift+G` |
| `nav_prefix` | `jobs` | Afficher les tâches en arrière-plan | `Ctrl+N Shift+J` |
| `nav_prefix` | `updates` | Afficher le panneau de mise à jour | `Ctrl+N u` |
| `nav_prefix` | `help` | Afficher ou masquer l'aide | `Ctrl+N ?` |
| `nav_prefix` | `select_tab` | Sélectionner un onglet | `Ctrl+N Tab` |
| `nav_prefix` | `change_theme` | Changer de thème | `Ctrl+N t` |
| `nav_prefix` | `change_language` | Changer de langue | `Ctrl+N l` |
| `nav_prefix` | `change_graph_color_theme` | Changer le thème de couleurs du graphe | `Ctrl+N Shift+T` |
| `nav_prefix` | `toggle_graph_row_tints` | Basculer la teinte de ligne du graphe | `Ctrl+N Shift+Y` |
| `nav_prefix` | `grow_section` | Agrandir la section Refs | `Ctrl+N +`, `Ctrl+N =` |
| `nav_prefix` | `shrink_section` | Réduire la section Refs | `Ctrl+N -` |
| `nav_prefix` | `grow_message` | Agrandir le bloc du message de commit | `Ctrl+N ]` |
| `nav_prefix` | `shrink_message` | Réduire le bloc du message de commit | `Ctrl+N [` |
| `nav_prefix` | `refs_wider` | Élargir le volet Refs | `Ctrl+N Right` |
| `nav_prefix` | `refs_narrower` | Rétrécir le volet Refs | `Ctrl+N Left` |
| `nav_prefix` | `status_wider` | Élargir le volet Status | `Ctrl+N Shift+Left` |
| `nav_prefix` | `status_narrower` | Rétrécir le volet Status | `Ctrl+N Shift+Right` |
| `nav_prefix` | `agent_show` | Agent : afficher la vue en attente | `Ctrl+N Enter` |
| `nav_prefix` | `agent_back` | Agent : retour (ou fermer la vue en attente) | `Ctrl+N Backspace` |
| `nav_prefix` | `arrange_dialog` | Disposer le dialogue (déplacer, taille) | `Ctrl+N m` |
| `refs` | `filter` | Filtrer les refs par nom | `/` |
| `refs` | `solo` | Isoler la sélection | `o` |
| `refs` | `clear_solo` | Arrêter le solo | `Shift+O` |
| `refs` | `toggle_hierarchy` | Basculer la vue hiérarchique des branches | `y` |
| `refs` | `show_all` | Afficher toutes les refs masquées | `Shift+H` |
| `refs` | `reveal` | Révéler dans le graphe | `g` |
| `refs` | `mark_drop` | Marquer comme source de dépôt | `m` |
| `refs` | `drop_marked` | Déposer la ref marquée ici | `Shift+M` |
| `refs` | `expand_all` | Déplier tous les dossiers | `=` |
| `refs` | `collapse_all` | Replier tous les dossiers | `-` |
| `refs` | `expand_below` | Déplier le dossier et ses sous-dossiers | `*`, `Shift+Right` |
| `refs` | `collapse_below` | Replier le dossier et ses sous-dossiers | `_`, `Shift+Left` |
| `log` | `toggle_file_mode` | Fichiers : chemin ou arbre | `m` |
| `log` | `view_all_files` | Fichiers : voir tous les fichiers | `a` |
| `log` | `compare_selection` | Comparer les lignes sélectionnées | `v` |
| `log` | `compare_swap` | Inverser les extrémités de la comparaison | `x` |
| `log` | `compare_next` | Aller à la ligne comparée suivante | `n` |
| `log` | `pick_column` | Choisir une colonne (déplacer, taille) | `Shift+C` |
| `log` | `scroll_left` | Faire défiler les colonnes vers la gauche | `Shift+Left` |
| `log` | `scroll_right` | Faire défiler les colonnes vers la droite | `Shift+Right` |
| `log` | `copy_mermaid` | Copier les commits en gitGraph Mermaid | `Shift+Y` |
| `log_columns` | `next` | Choisir la colonne suivante | `Tab` |
| `log_columns` | `prev` | Choisir la colonne précédente | `Shift+BackTab` |
| `log_columns` | `move_left` | Déplacer la colonne à gauche | `<` |
| `log_columns` | `move_right` | Déplacer la colonne à droite | `>` |
| `log_columns` | `wider` | Élargir la colonne | `]` |
| `log_columns` | `narrower` | Rétrécir la colonne | `[` |
| `log_columns` | `reset_width` | Réinitialiser les largeurs des colonnes | `0` |
| `log_columns` | `fit` | Ajuster la colonne à son contenu | `=` |
| `log_columns` | `fit_all` | Ajuster toutes les colonnes à leur contenu | `+` |
| `log_columns` | `layout` | Défilement latéral ou ajustement au volet | `s` |
| `log_files` | `collapse_all` | Replier ou déplier tous les dossiers | `z` |
| `log_files` | `filter` | Filtrer les fichiers modifiés | `/` |
| `log_files` | `expand_all` | Déplier tous les dossiers | `=` |
| `log_files` | `collapse_all_folders` | Replier tous les dossiers | `-` |
| `log_files` | `expand_below` | Déplier le dossier et ses sous-dossiers | `*`, `Shift+Right` |
| `log_files` | `collapse_below` | Replier le dossier et ses sous-dossiers | `_`, `Shift+Left` |
| `status` | `compose` | Aller au message de commit | `c` |
| `status` | `toggle_file_mode` | Fichiers : chemin ou arbre | `m` |
| `status` | `view_all_files` | Fichiers : voir tous les fichiers | `a` |
| `status` | `amend` | Basculer l'amendement | `Shift+A` |
| `status` | `skip_hooks` | Basculer Ignorer les hooks | `Shift+H` |
| `status` | `collapse_all` | Replier ou déplier tous les dossiers | `z` |
| `status` | `stage_toggle` | Stage ou unstager la ligne sélectionnée | `Ctrl+Space` |
| `status` | `open_diff` | Ouvrir le diff du fichier sélectionné | `d` |
| `status` | `expand_all` | Déplier tous les dossiers | `=` |
| `status` | `collapse_all_folders` | Replier tous les dossiers | `-` |
| `status` | `expand_below` | Déplier le dossier et ses sous-dossiers | `*`, `Shift+Right` |
| `status` | `collapse_below` | Replier le dossier et ses sous-dossiers | `_`, `Shift+Left` |
| `home` | `favorite` | Marquer ou démarquer le dépôt (étoile) | `f`, `*` |
| `home` | `close_repo` | Fermer, retirer ou démarquer la ligne | `x`, `Delete` |
| `home` | `copy_path` | Copier le chemin du dépôt | `y` |
| `home` | `open_in` | Ouvrir le dépôt avec l'outil par défaut | `o` |
| `home` | `browse` | Parcourir pour trouver un dépôt | `b` |
| `home` | `clone` | Cloner un dépôt | `c` |
| `home` | `init` | Initialiser un dépôt | `i` |
| `home` | `collapse_all` | Replier toutes les sections | `-` |
| `home` | `expand_all` | Déplier toutes les sections | `+`, `=` |
| `home` | `search` | Chercher dans les dépôts | `/` |
| `home` | `workspace_new` | Nouvel espace de travail | `n` |
| `home` | `workspace_edit` | Modifier l'espace de travail | `e` |
| `home` | `workspace_from_tabs` | Espace de travail depuis les onglets | `a` |
| `home` | `workspace_fetch` | Fetch All dans l'espace de travail | `Shift+F` |
| `home` | `workspace_pull` | Pull All dans l'espace de travail | `Shift+P` |
| `home` | `workspace_clone_missing` | Cloner les dépôts absents | `m` |
| `home` | `workspace_locate` | Localiser le dépôt absent | `l` |
| `refs` | `hosting_filter` | Pull requests / tickets : filtre | `v` |
| `refs` | `hosting_edit` | Pull request : modifier sur l'hébergeur | `e` |
| `refs` | `new_pull_request` | Pull requests : en créer une | `n` |
| `refs` | `add_remote` | Distants : ajouter un distant | `+` |
| `refs` | `remotes` | Branche : modifier ses distants | `u` |
| `diff` | `close` | Fermer le diff | `q` |
| `diff` | `edit` | Modifier dans le dossier de travail | `e` |
| `diff` | `lfs_raw` | LFS : carte ou pointeur brut | `r` |
| `diff` | `copy` | Copier la sélection ou la ligne | `y`, `Ctrl+C` |
| `diff` | `stage_hunk` | Stage ou unstage du hunk | `Space` |
| `diff` | `stage_lines` | Stage/unstage des lignes choisies | `Enter` |
| `diff` | `discard_hunk` | Abandonner le hunk | `x` |
| `diff` | `discard_lines` | Abandonner les lignes sélectionnées | `Shift+X` |
| `diff` | `select_lines` | Démarrer ou quitter la sélection de lignes | `v` |
| `diff` | `next_hunk` | Hunk suivant | `n` |
| `diff` | `prev_hunk` | Hunk précédent | `p` |
| `diff` | `toggle_layout` | Disposition : unifiée ou côte à côte | `m` |
| `diff` | `toggle_hunks_only` | Hunks seulement ou fichier entier | `Shift+H` |
| `diff` | `toggle_minimap` | Basculer la minimap | `Shift+M` |
| `diff` | `choose_encoding` | Choisir l'encodage | `Shift+C` |
| `diff` | `toggle_wrap` | Basculer le retour à la ligne | `w` |
| `diff` | `toggle_whitespace` | Afficher/masquer les modifications d'espaces | `s` |
| `diff` | `merge_editor` | Ouvrir l'éditeur de merge (fichier en conflit) | `Shift+E` |
| `file_insight` | `close` | Fermer la vue du fichier | `q` |
| `file_insight` | `edit` | Modifier dans le dossier de travail | `e` |
| `file_insight` | `copy` | Copier la sélection ou la ligne | `y`, `Ctrl+C` |
| `file_insight` | `last_row` | Aller à la dernière ligne | `Shift+G` |
| `file_insight` | `open_commit` | Aller au commit de la ligne | `Enter` |
| `history` | `show_in_log` | Afficher dans le Log | `Enter` |
| `history` | `copy_sha` | Copier le SHA du commit | `y` |
| `history` | `copy_summary` | Copier le résumé | `Shift+Y` |
| `history` | `show_diff` | Afficher le diff de ce commit | `d`, `Alt+1` |
| `history` | `open_file` | Ouvrir le fichier à ce commit | `o`, `Alt+2` |
| `history` | `blame` | Blame à ce commit | `b`, `Alt+3` |
| `history` | `compare_working_tree` | Comparer avec l'arbre de travail | `c` |
| `history` | `restore` | Restaurer cette version | `Shift+R` |
| `history` | `range_up` | Étendre la sélection vers le haut | `Shift+Up` |
| `history` | `range_down` | Étendre la sélection vers le bas | `Shift+Down` |
| `history` | `switch_focus` | Basculer entre la liste et le diff | `Tab` |
| `history` | `widen_list` | Élargir la liste | `Alt+Right` |
| `history` | `narrow_list` | Rétrécir la liste | `Alt+Left` |
| `history` | `reset_split` | Réinitialiser la largeur de la liste | `Alt+0` |
| `commit_column` | `scroll_left` | Faire défiler la colonne à gauche | `Shift+Left` |
| `commit_column` | `scroll_right` | Faire défiler la colonne à droite | `Shift+Right` |
| `commit_column` | `next_part` | Dimensionner la partie suivante | `Tab` |
| `commit_column` | `move_left` | Déplacer la partie à gauche | `<` |
| `commit_column` | `move_right` | Déplacer la partie à droite | `>` |
| `commit_column` | `wider_part` | Élargir la partie | `]` |
| `commit_column` | `narrower_part` | Rétrécir la partie | `[` |
| `commit_column` | `reset_widths` | Réinitialiser les largeurs des parties | `0` |
| `commit_column` | `fit` | Ajuster la colonne à son contenu | `=` |
| `commit_column` | `fit_all` | Ajuster toutes les colonnes à leur contenu | `+` |
| `file_insight` | `blame_card` | Blame : Afficher ou masquer la fiche de détail | `i` |
| `file_insight` | `blame_next` | Blame : Hunk suivant | `n` |
| `file_insight` | `blame_prev` | Blame : Hunk précédent | `p` |
| `file_insight` | `blame_next_same` | Blame : Hunk suivant du même commit | `Shift+N` |
| `file_insight` | `blame_prev_same` | Blame : Hunk précédent du même commit | `Shift+P` |
| `file_insight` | `blame_goto` | Blame : Aller à la ligne | `g` |
| `file_insight` | `blame_copy_sha` | Blame : Copier le SHA du commit | `Shift+Y` |
| `file_insight` | `blame_diff` | Blame : Diff du commit pour ce fichier | `d` |
| `file_insight` | `blame_parent` | Blame : Blame au parent du commit | `b` |
| `file_insight` | `blame_history` | Blame : Ouvrir l'historique du fichier | `Shift+H` |
| `file_insight` | `blame_wider` | Blame : Élargir la colonne d'annotation | `>` |
| `file_insight` | `blame_narrower` | Blame : Réduire la colonne d'annotation | `<` |
| `image` | `zoom_in` | Zoomer l'image | `+`, `=` |
| `image` | `zoom_out` | Dézoomer l'image | `-` |
| `image` | `fit` | Ajuster l'image à la vue | `0` |
| `image` | `actual_size` | Afficher l'image en taille réelle | `1` |
| `image` | `flip_side` | Afficher l'autre côté (avant / après) | `Tab` |
| `image` | `widen_before` | Élargir le volet Avant | `Alt+Right` |
| `image` | `widen_after` | Élargir le volet Après | `Alt+Left` |
| `image` | `reset_split` | Rétablir la séparation avant / après | `Alt+0` |
| `image` | `edit_in` | Ouvrir avec... | `Shift+O` |
| `image` | `edit_default` | Ouvrir dans l'application par défaut | `o` |
| `editor` | `save` | Enregistrer | `Ctrl+S` |
| `editor` | `undo` | Annuler | `Ctrl+Z` |
| `editor` | `redo` | Rétablir | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `editor` | `find` | Chercher | `Ctrl+F` |
| `editor` | `find_next` | Suivant (Shift : précédent) | `F3` |
| `editor` | `paste` | Coller | `Ctrl+V` |
| `editor` | `toggle_minimap` | Basculer la minimap | `Alt+M` |
| `editor_prompt` | `save` | Non enregistré : enregistrer et fermer | `s`, `Shift+S` |
| `editor_prompt` | `discard` | Non enregistré : abandonner et fermer | `d`, `Shift+D` |
| `editor_prompt` | `cancel` | Non enregistré : continuer l'édition | `c`, `Shift+C` |
| `merge` | `next_conflict` | Conflit suivant | `F7`, `Alt+Down` |
| `merge` | `prev_conflict` | Conflit précédent | `Shift+F7`, `Alt+Up` |
| `merge` | `next_file` | Fichier en conflit suivant | `Alt+PageDown` |
| `merge` | `prev_file` | Fichier en conflit précédent | `Alt+PageUp` |
| `merge` | `next_pane` | Aller au volet suivant | `F6` |
| `merge` | `prev_pane` | Aller au volet précédent | `Shift+F6` |
| `merge` | `toggle_panes` | Afficher/masquer les volets Refs et Status | `F9` |
| `merge` | `toggle_base` | Afficher ou masquer le volet Base | `Alt+D` |
| `merge` | `toggle_full_file` | Basculer le diff du fichier entier | `Alt+F` |
| `merge` | `undo` | Annuler (choix et saisie) | `Ctrl+Z` |
| `merge` | `redo` | Rétablir (choix et saisie) | `Ctrl+Y`, `Ctrl+Shift+Z` |
| `merge` | `widen_current` | Élargir le volet Courant (A) | `Alt+Right` |
| `merge` | `widen_incoming` | Élargir le volet Entrant (B) | `Alt+Left` |
| `merge` | `reset_panes` | Réinitialiser la taille des volets | `Alt+0` |
| `merge` | `save` | Enregistrer sans marquer comme résolu | `Alt+S` |
| `merge` | `save_resolve` | Enregistrer et marquer comme résolu | `Ctrl+S` |
| `merge` | `ai_suggest` | ✦ Suggérer une résolution (IA) | `Alt+I` |
| `merge` | `unpick` | Désélectionner ce conflit (non résolu) | `Alt+U` |
| `merge` | `reset_all` | Réinitialiser tout le fichier | `Alt+R` |
| `merge` | `focus_a` | Aller au volet Courant (A) | `Alt+1` |
| `merge` | `focus_b` | Aller au volet Entrant (B) | `Alt+2` |
| `merge` | `focus_output` | Aller au volet Résultat | `Alt+3` |
| `merge` | `focus_base` | Aller au volet Base | `Alt+4` |
| `merge_sides` | `toggle_line` | Choisir ou retirer la ligne | `Space` |
| `merge_sides` | `toggle_chunk` | Choisir ou retirer tout le bloc | `Enter` |
| `merge_sides` | `take_a` | Prendre seulement le courant (A) | `a` |
| `merge_sides` | `take_b` | Prendre seulement l'entrant (B) | `b` |
| `merge_sides` | `take_both` | Prendre les deux, A puis B | `c` |
| `merge_sides` | `take_both_reversed` | Prendre les deux, B puis A | `Shift+C` |
| `merge_sides` | `clear` | Ne rien prendre pour ce conflit | `x` |
| `merge_sides` | `unpick` | Désélectionner ce conflit (non résolu) | `u` |
| `merge_sides` | `reset_all` | Réinitialiser tout le fichier | `Shift+U` |
| `merge_sides` | `swap_order` | Inverser l'ordre des deux côtés | `s` |
| `merge_sides` | `next_conflict` | Conflit suivant | `n` |
| `merge_sides` | `prev_conflict` | Conflit précédent | `Shift+N` |
| `merge_sides` | `take_all_a` | Tout prendre du courant (A) | `Shift+A` |
| `merge_sides` | `take_all_b` | Tout prendre de l'entrant (B) | `Shift+B` |
| `merge_sides` | `toggle_base` | Afficher ou masquer le volet Base | `d` |
| `merge_sides` | `scroll_left` | Faire défiler les volets vers la gauche | `Shift+Left`, `Shift+H` |
| `merge_sides` | `scroll_right` | Faire défiler les volets vers la droite | `Shift+Right`, `Shift+L` |
| `merge_sides` | `toggle_full_file` | Basculer le diff du fichier entier | `f` |
| `merge_sides` | `cycle_height` | Changer la hauteur de la sortie | `z` |
| `merge_card` | `take_a` | Garder A (courant) tel quel | `a` |
| `merge_card` | `take_b` | Prendre B (entrant) tel quel | `b` |
| `merge_card` | `take_base` | Prendre la base (ancêtre commun) | `o` |
| `merge_card` | `keep` | Garder le fichier restant | `k` |
| `merge_card` | `delete` | Accepter la suppression | `Shift+D` |
| `merge_card` | `external_tool` | Ouvrir dans l'outil de merge externe | `m` |
| `rebase` | `pick` | Pick | `p` |
| `rebase` | `reword` | Reword | `r` |
| `rebase` | `squash` | Squash | `s` |
| `rebase` | `fixup` | Fixup | `f` |
| `rebase` | `drop` | Drop | `d` |
| `rebase` | `edit` | Edit (s'arrêter pour amender) | `e` |
| `rebase` | `autosquash` | Basculer l'autosquash | `a` |
| `rebase` | `start` | Lancer le plan | `Ctrl+S`, `Ctrl+Enter` |
| `rebase` | `reset` | Réinitialiser le plan | `Ctrl+R` |
| `rebase` | `split` | Scinder le commit de la ligne | `t` |
| `rebase` | `move_up` | Monter la ligne | `Ctrl+Up`, `Alt+Up` |
| `rebase` | `move_down` | Descendre la ligne | `Ctrl+Down`, `Alt+Down` |
| `compose` | `apply` | Créer les commits | `Ctrl+S`, `Ctrl+Enter`, `Alt+Enter` |
| `split` | `line_mode` | Basculer la sélection de lignes | `v` |
| `split` | `move_up` | Monter le commit | `Alt+Up`, `Shift+K` |
| `split` | `move_down` | Descendre le commit | `Alt+Down`, `Shift+J` |
| `split` | `reset` | Réinitialiser la scission | `Shift+R` |
| `rewrite` | `preview` | Prévisualiser la réécriture | `F5` |
| `rewrite` | `apply` | Appliquer la réécriture | `Ctrl+Enter`, `Ctrl+S` |
| `rewrite` | `cancel` | Annuler l'aperçu | `Ctrl+X` |
| `rewrite` | `add_rule` | Ajouter une règle | `a`, `Insert` |
| `rewrite` | `toggle_rule` | Activer ou désactiver la règle | `Space` |
| `rewrite` | `move_up` | Monter la règle | `Alt+Up` |
| `rewrite` | `move_down` | Descendre la règle | `Alt+Down` |
| `rewrite` | `delete_rule` | Supprimer la règle | `d`, `Delete` |
| `rewrite` | `restore_rule` | Rétablir la règle supprimée | `Ctrl+Z` |
| `rewrite` | `duplicate_rule` | Dupliquer la règle | `Ctrl+D` |
| `rewrite` | `next_tab` | Onglet d'aperçu suivant | `Alt+Right` |
| `rewrite` | `prev_tab` | Onglet d'aperçu précédent | `Alt+Left` |
| `rewrite` | `save_shared` | Enregistrer la recette dans le dépôt | `Alt+S` |
| `rewrite` | `save_mine` | Enregistrer la recette pour moi seul | `Alt+M` |
| `rewrite` | `load` | Charger une recette | `Alt+L` |
| `rewrite` | `name` | Renommer la recette | `F2` |
| `rewrite` | `protected_all` | Inclure toutes les branches protégées | `Alt+A` |
| `rewrite` | `protected_none` | Exclure toutes les branches protégées | `Alt+U` |
| `rewrite` | `check_tools` | Revérifier les outils | `F6` |
| `rewrite` | `analyze` | Analyser les tailles de l'historique | `F7` |
| `rewrite` | `host_howto` | Expliquer la protection de l'hébergeur | `Alt+H` |
| `rewrite` | `ai_rules` | Créer des règles de réécriture avec l'IA | `Alt+I` |
| `reword` | `apply` | Appliquer le nouveau message | `Ctrl+Enter`, `Ctrl+S` |
| `reword` | `deeper` | Modifier l'historique en profondeur | `Alt+H` |
| `composer` | `commit` | Commit depuis le compositeur | `Ctrl+Enter`, `Alt+Enter` |
| `composer` | `insert_issue_ref` | Insérer une référence de ticket | `Alt+R` |
| `md_editor` | `preview` | Éditeur Markdown : Écrire ou Aperçu | `Alt+P` |
| `md_editor` | `split` | Éditeur Markdown : vue côte à côte | `Alt+L` |
| `md_editor` | `submit` | Éditeur Markdown : envoyer | `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` |
| `md_editor` | `ai` | Éditeur Markdown : écrire avec l'IA | `Alt+E` |
| `dialog` | `browse` | Parcourir un dossier | `Ctrl+B` |
| `dialog` | `sources` | Aller aux sources de clonage | `Alt+S` |
| `dialog` | `submit` | Envoyer le formulaire de pull request | `Ctrl+S` |
| `dialog` | `web` | Ouvrir le formulaire web du fournisseur | `Alt+W` |
| `folder_picker` | `choose` | Choisir le dossier | `Ctrl+S`, `Ctrl+Enter` |
| `folder_picker` | `new_folder` | Nouveau dossier | `Alt+N` |
| `folder_picker` | `filter` | Filtrer les entrées du dossier | `Ctrl+F`, `/` |
| `folder_picker` | `toggle_siblings` | Afficher ou masquer les dossiers voisins | `Alt+S` |
| `folder_picker` | `toggle_hidden` | Afficher ou masquer les dossiers cachés | `Alt+H` |
| `preferences` | `search` | Chercher dans les paramètres | `Ctrl+F`, `/` |
| `preferences` | `list_add` | Ajouter un élément à la liste | `Insert` |
| `preferences` | `list_duplicate` | Dupliquer l'élément sélectionné | `Ctrl+D` |
| `preferences` | `list_remove` | Supprimer l'élément sélectionné | `Delete` |
| `preferences` | `list_move_up` | Monter l'élément sélectionné | `Alt+Up` |
| `preferences` | `list_move_down` | Descendre l'élément sélectionné | `Alt+Down` |
| `preferences` | `prev_tab` | Onglet précédent de la page de paramètres | `Alt+Left` |
| `preferences` | `next_tab` | Onglet suivant de la page de paramètres | `Alt+Right` |
| `ai_popup` | `copy` | Copier le résultat | `c` |
| `activity_log` | `scroll_left` | Faire défiler le journal à gauche | `Left` |
| `activity_log` | `scroll_right` | Faire défiler le journal à droite | `Right` |
| `integrations` | `add` | Ajouter un profil | `a` |
| `integrations` | `edit` | Modifier le profil sélectionné | `e` |
| `integrations` | `open_token` | Ouvrir la page du jeton | `o` |
| `integrations` | `remove` | Supprimer le profil sélectionné | `d`, `Delete` |
| `prompt` | `ai_stash_message` | IA : générer le message de stash | `Ctrl+G` |
<!-- keymap:end -->

### Touches que le terminal ne peut pas envoyer, et le protocole clavier kitty

Un terminal ancien ne peut pas envoyer certains enchaînements : `Ctrl+,`,
`Ctrl+Shift+N`, `Ctrl+Enter` et `Ctrl+<chiffre>`. Il n'envoie rien, ou une autre touche :
par exemple, `Ctrl+Shift+N` arrive comme `Ctrl+N`, `Ctrl+I` comme `Tab` et `Ctrl+M` comme
`Enter`. La page des Paramètres avertit d'une telle affectation (« ce terminal n'envoie pas
... »).

Les terminaux avec le **protocole clavier kitty** peuvent envoyer ces enchaînements.
Exemples : kitty, WezTerm, Ghostty, foot, Alacritty 0.13 et plus, et zellij (tmux n'y
répond pas : voir [Multiplexeurs](multiplexers.md#tmux)). GitPoulpi utilise le protocole par défaut :

- Au démarrage, GitPoulpi interroge le terminal et attend au plus environ 100 ms la
  réponse. Si le terminal ne répond pas, les touches classiques restent.
- Si le terminal répond, GitPoulpi active le protocole. Il le désactive à nouveau quand
  il quitte, échoue ou plante.
- Les avertissements de la page des Paramètres suivent le protocole : quand il est actif,
  les enchaînements ci-dessus sont envoyables.
- **Paramètres > Clavier et souris > Protocole clavier kitty** (`ui.kitty_keyboard`) réglé
  sur `off` ne change jamais le mode clavier du terminal. Utilisez-le si un terminal ou un
  multiplexeur se comporte mal. Un redémarrage est nécessaire.

Les consoles Windows signalent elles-mêmes les modificateurs et n'ont pas besoin du
protocole.

## Palette de commandes (`Ctrl+P`)

La palette liste **toutes** les commandes de l'application, si bien qu'elle atteint tout
sans raccourci. La liste complète est [plus bas](#toutes-les-commandes-de-la-palette).

| Touche | Action |
|---|---|
| `Ctrl+P` | Ouvrir la palette |
| *(taper)* | Filtrer la liste. Une correspondance exacte après le préfixe `Catégorie : ` vient d'abord, puis une correspondance n'importe où dans le libellé, puis une correspondance floue |
| `Up` / `Down`, `Home` / `End`, `PageUp` / `PageDown` | Déplacer la surbrillance (voir [Listes de popups filtrables](#listes-de-popups-filtrables)) |
| `Enter`, ou un double-clic | Lancer la commande en surbrillance |
| `Esc` | Fermer |

`j` et `k` sont tapés dans le filtre. Ils ne déplacent pas la surbrillance.

Chaque ligne montre son raccourci à droite. Une commande qui ne s'applique pas maintenant
(par exemple, aucune ref sélectionnée) reste dans la liste, grisée, et la raison s'affiche
sous la liste. Les libellés commencent par une catégorie, comme `Navigation :`, `Refs :`,
`Git :`, `Affichage :`, `Diff :` ou `Application :`. Tapez une catégorie pour voir tout son
groupe.

### Une touche pour chaque commande

Chaque ligne de la palette se termine par un bouton touche (`[k]`), et `Alt+K` fait la même chose sur la ligne en surbrillance : il ouvre **Paramètres > Clavier et souris** filtré sur le raccourci de cette commande, prêt à être réaffecté. Une commande sans raccourci propre en reçoit un ici : elle figure sous **Commandes de la palette**, sans touche par défaut, et la touche que vous lui donnez lance la commande exactement comme le fait son choix dans la palette (une commande qui ne peut pas s'exécuter maintenant dit pourquoi). La ligne affiche alors votre touche.

Dans `config.toml`, c'est la table `[keybindings.palette]`, nommée d'après la commande (`toggle_minimap = "Ctrl+Alt+M"`). Une touche en conflit avec une autre action s'affiche en rouge, et `c` sur sa ligne liste toutes les actions avec lesquelles elle est en conflit.

La bande d'aide sous la liste indique cette touche (`Alt+K` par défaut). C'est l'action `palette.configure_shortcut` : elle se réaffecte comme les autres.

La palette et les sélecteurs (thème, langue, onglet, recherche d'espace de travail) se déplacent et se redimensionnent comme une boîte de dialogue : faites glisser le bord supérieur, appuyez sur `Ctrl+N` puis `m` (comme sur toute boîte de dialogue), ou appuyez sur `Alt+G`. `Alt+G` est l'action `palette.arrange` (« Fenêtre : déplacer ou redimensionner la fenêtre au premier plan ») : un moyen de plus, valable sur toute boîte de dialogue et toute fenêtre contextuelle, de lancer le mode de disposition sur celle qui est au premier plan (les lignes `Dialogue : Déplacer/redimensionner` et `Dialogue : Réinitialiser la position et la taille` de la palette agissent sur la palette elle-même quand elle est ouverte). Les flèches la déplacent, `Shift`+flèches la redimensionnent, `0` la réinitialise, `Enter` ou `Esc` quittent le mode et la fenêtre reste ouverte. `ui.dialog_geometry` décide si la position est conservée.

`Ctrl+N` est aussi le préfixe de navigation ici : il affiche un badge sur le champ de filtre, la première ligne de la liste et la croix de fermeture, et la touche suivante actionne ce contrôle, ou `m` lance la disposition de la fenêtre. Il ne descend plus la surbrillance, et `Ctrl+P` ne la remonte pas : seuls les flèches, `PageUp` / `PageDown` et `Home` / `End` déplacent une liste qui a un champ de filtre.

### Toutes les commandes de la palette

Générée à partir de l'application (un test la garde à jour). Les commandes IA
n'apparaissent que lorsque l'IA est activée ; les autres sont grisées, avec la raison,
quand elles ne s'appliquent pas. Les touches sont celles par défaut.

<!-- palette:begin -->
| Commande | Touche par défaut |
|---|---|
| Navigation : Aller au volet Refs | `Ctrl+N <badge>` |
| Navigation : Aller au volet Log | `Ctrl+N <badge>` |
| Navigation : Aller au volet Status | `Ctrl+N <badge>` |
| Navigation : Aller au filtre des Refs | `Ctrl+N <badge>` |
| Navigation : Aller à Refs > Locales | `Ctrl+N <badge>` |
| Navigation : Aller à Refs > Distantes | `Ctrl+N <badge>` |
| Navigation : Aller à Refs > Worktrees | `Ctrl+N <badge>` |
| Navigation : Aller à Refs > Sous-modules | `Ctrl+N <badge>` |
| Navigation : Aller à Refs > Stashes | `Ctrl+N <badge>` |
| Navigation : Aller à Refs > Tags | `Ctrl+N <badge>` |
| Volet gauche : Afficher/masquer les sections... |  |
| Volet gauche : Réinitialiser les sections |  |
| Volet gauche : Utiliser ces sections pour tous les dépôts |  |
| Volet gauche : Afficher/masquer Locales |  |
| Volet gauche : Afficher/masquer Distants |  |
| Volet gauche : Afficher/masquer Worktrees |  |
| Barre latérale : Afficher la section Worktrees |  |
| Volet gauche : Afficher/masquer Stashes |  |
| Volet gauche : Afficher/masquer Tags |  |
| Volet gauche : Afficher/masquer Pull requests |  |
| Volet gauche : Afficher/masquer Tickets |  |
| Volet gauche : Afficher/masquer Sous-modules |  |
| Navigation : Aller à Status > unstaged | `Ctrl+N <badge>` |
| Navigation : Aller à Status > Staged | `Ctrl+N <badge>` |
| Navigation : Passer au volet suivant | `Tab` |
| Navigation : Passer au volet précédent | `Shift+Tab` |
| Navigation : Rechercher des commits... | `f` |
| Navigation : Replier ou déplier le volet Refs | `Ctrl+N H` |
| Navigation : Élargir le volet Refs | `Ctrl+N Right` |
| Navigation : Rétrécir le volet Refs | `Ctrl+N Left` |
| Navigation : Élargir le volet Status | `Ctrl+N Shift+Left` |
| Navigation : Rétrécir le volet Status | `Ctrl+N Shift+Right` |
| Navigation : Replier ou déplier le volet Status | `Ctrl+N L` |
| Navigation : Agrandir la section Refs | `Ctrl+N +` |
| Navigation : Réduire la section Refs | `Ctrl+N -` |
| Navigation : Réinitialiser la taille des sections Refs |  |
| Navigation : Réinitialiser la disposition |  |
| Détails du commit : Agrandir le message | `Ctrl+N ]` |
| Détails du commit : Réduire le message | `Ctrl+N [` |
| Détails du commit : Réinitialiser la taille du message |  |
| Détails du commit : Faire défiler le message vers le bas |  |
| Détails du commit : Faire défiler le message vers le haut |  |
| Détails du commit : Ouvrir un lien du message... |  |
| Fichiers : Inverser le tri des fichiers du commit (A-Z / Z-A) |  |
| Refs : Basculer sur la ref sélectionnée | `double-clic` |
| Refs : Copier le nom de la ref |  |
| Refs : Copier la ref complète (refs/heads/...) |  |
| Refs : Copier le SHA ciblé par la ref |  |
| Refs : Merge la branche sélectionnée dans la branche courante |  |
| Refs : Rebase la branche courante sur la sélection | `Ctrl+G r` |
| Refs : Déposer la ref sélectionnée sur la branche courante... |  |
| Refs : Marquer comme source de dépôt | `m` |
| Refs : Déposer ici la ref marquée... | `M` |
| Refs : Renommer la branche sélectionnée... |  |
| Refs : Avancer la branche sélectionnée jusqu'à la courante (fast-forward) |  |
| Refs : Avancer la branche courante jusqu'à la sélection (fast-forward) |  |
| Refs : Créer un worktree depuis la branche ou le tag sélectionné... |  |
| Refs : Reflog de la branche sélectionnée... |  |
| Refs : Ouvrir une pull request depuis la branche sélectionnée... |  |
| Refs : Ouvrir le worktree dans un onglet | `Enter / double-clic` |
| Refs : Ouvrir le worktree dans... |  |
| Refs : Merge de la branche du worktree dans... |  |
| Refs : Retirer le worktree... |  |
| Refs : Créer un worktree... | `+` |
| Refs : Créer un worktree depuis la branche courante |  |
| Refs : Créer un worktree en choisissant une branche |  |
| Refs : Mettre à jour le sous-module (sélection) |  |
| Refs : Initialiser le sous-module (sélection) |  |
| Refs : Ouvrir le sous-module dans un onglet | `Enter / double-clic` |
| Git : Mettre à jour tous les sous-modules |  |
| Git : Créer un worktree sur le commit sélectionné... |  |
| Git : Créer un patch depuis le commit sélectionné... |  |
| Git : Créer un patch du commit sélectionné à HEAD... |  |
| Git : Créer un patch depuis les modifications locales... |  |
| Réécriture : purger l'ancien historique... |  |
| Git : Appliquer un patch... |  |
| Réécriture : ouvrir l'atelier | `Ctrl+G H` |
| Réécriture : remplacer un auteur... |  |
| Réécriture : anonymiser les auteurs... |  |
| Réécriture : appliquer .mailmap à l'historique... |  |
| Réécriture : merge des identités (.mailmap)... |  |
| Réécriture : décaler les dates des commits... |  |
| Réécriture : changer le fuseau horaire des commits... |  |
| Réécriture : garder les commits dans des plages horaires... |  |
| Réécriture : date du committer = date de l'auteur |  |
| Réécriture : rechercher et remplacer dans les messages... |  |
| Réécriture : supprimer des lignes de message... |  |
| Réécriture : ajouter ou retirer des trailers... |  |
| Réécriture : retirer les signatures... |  |
| Réécriture : corriger l'encodage des messages... |  |
| Réécriture : rechercher et remplacer dans les fichiers... |  |
| Réécriture : retirer des fichiers de l'historique... |  |
| Réécriture : retirer les gros fichiers de l'historique... |  |
| Réécriture : déplacer ou renommer des chemins... |  |
| Réécriture : migrer des fichiers vers LFS... |  |
| Réécriture : ramener des fichiers depuis LFS... |  |
| Réécriture : convertir les fins de ligne... |  |
| Réécriture : retirer le bit exécutable... |  |
| Réécriture : convertir l'encodage des fichiers... |  |
| Réécriture : exécuter un script... |  |
| ✦ Réécriture : écrire un script avec l'IA... |  |
| ✦ Réécriture : créer des règles avec l'IA... |  |
| Réécriture : du commit sélectionné à HEAD... |  |
| Réécriture : commits sélectionnés... |  |
| LFS : Initialiser pour ce dépôt |  |
| LFS : Suivre un motif... |  |
| LFS : Ne plus suivre un motif... |  |
| LFS : Fetch des objets |  |
| LFS : Ouvrir le menu | `Ctrl+G l` |
| LFS : Fetch de tous les objets | `Ctrl+G F` |
| LFS : Checkout de tous les fichiers | `Ctrl+G K` |
| LFS : Push de tous les objets | `Ctrl+G J` |
| LFS : Pull des objets | `Ctrl+G L` |
| LFS : Push des objets |  |
| LFS : Élaguer les objets locaux | `Ctrl+G V` |
| LFS : Afficher les verrous |  |
| LFS : Ouvrir la page d'installation |  |
| LFS : Basculer le pointeur brut dans le diff | `r` |
| LFS : Suivre le fichier sélectionné |  |
| LFS : Suivre l'extension du fichier sélectionné |  |
| LFS : Verrouiller le fichier sélectionné |  |
| LFS : Déverrouiller le fichier sélectionné |  |
| LFS : Forcer le déverrouillage du fichier sélectionné |  |
| Refs : Supprimer la branche ou le tag sélectionné... | `Ctrl+G d` |
| Refs : Supprimer la branche sélectionnée de son distant... |  |
| Refs : Supprimer la branche sélectionnée en local et sur son distant... |  |
| Refs : Copier le lien de la branche sélectionnée sur le distant |  |
| Git : Copier le lien vers le commit sélectionné sur le distant |  |
| Git : Tag annoté sur le commit sélectionné... |  |
| Refs : Push du tag sélectionné vers le distant par défaut |  |
| Refs : Supprimer le tag sélectionné de son distant... |  |
| Git : Fetch le tag sélectionné |  |
| Refs : Annoter le tag sélectionné... |  |
| Refs : Masquer ou afficher dans le graphe |  |
| Refs : Révéler dans le graphe | `g` |
| Refs : Activer ou désactiver le solo sur la ref sélectionnée | `o` |
| Refs : Arrêter le solo (retire toutes les refs en solo) | `O` |
| Refs : Afficher toutes les refs masquées | `H` |
| Refs : Masquer ou afficher le dossier ou distant sélectionné |  |
| Refs : Solo sur le dossier ou distant sélectionné | `o` |
| Refs : Filtrer par nom... | `/` |
| Refs : Effacer le filtre par nom | `Esc` |
| Refs : Déplier tous les dossiers | `=` |
| Refs : Replier tous les dossiers | `-` |
| Refs : Déplier le dossier sélectionné et ses sous-dossiers | `*` |
| Refs : Replier le dossier sélectionné et ses sous-dossiers | `_` |
| Refs : Section suivante | `l` |
| Refs : Section précédente | `h` |
| Refs : Déplier ou replier la section active | `Enter` |
| Refs : Basculer l'arborescence des branches | `y` |
| Git : Stage du fichier sélectionné | `Ctrl+G s` |
| Git : Unstage du fichier sélectionné | `Ctrl+G u` |
| Git : Stage de toutes les modifications | `Ctrl+G a` |
| Modifications locales : Abandonner le fichier sélectionné... |  |
| Modifications locales : Ignorer le fichier sélectionné |  |
| Modifications locales : Ignorer l'extension du fichier sélectionné |  |
| Modifications locales : Traiter le fichier sélectionné comme binaire... |  |
| Modifications locales : Traiter le fichier sélectionné comme texte... |  |
| Modifications locales : Traiter l'extension du fichier sélectionné comme binaire... |  |
| Modifications locales : Traiter l'extension du fichier sélectionné comme texte... |  |
| Modifications locales : Stash du fichier sélectionné |  |
| Modifications locales : Historique du fichier sélectionné |  |
| Modifications locales : Blame du fichier sélectionné |  |
| Modifications locales : Ouvrir le fichier sélectionné dans l'outil de diff externe |  |
| Modifications locales : Ouvrir le fichier sélectionné dans l'éditeur externe |  |
| Modifications locales : Ouvrir le fichier sélectionné avec le programme par défaut |  |
| Modifications locales : Afficher le fichier sélectionné dans son dossier |  |
| Modifications locales : Copier le chemin du fichier sélectionné |  |
| Modifications locales : Créer un patch depuis le fichier sélectionné... |  |
| Modifications locales : Modifier le fichier sélectionné |  |
| Modifications locales : Supprimer le fichier sélectionné... |  |
| Modifications locales : Ouvrir le menu du dossier sélectionné |  |
| Modifications locales : Créer un fichier |  |
| Modifications locales : Abandonner toutes les modifications... | `Ctrl+G x` |
| Git : Commit... | `Ctrl+G c` |
| Git : Commiter le stage maintenant | `Ctrl+G Enter` |
| Git : Modifier le message du commit WIP (dans le Log) | `c dans le Log` |
| Git : Fetch de tous les distants | `Ctrl+G f` |
| Git : Pull | `Ctrl+G p` |
| Git : Pull (fast-forward si possible) |  |
| Git : Pull (fast-forward uniquement) |  |
| Git : Pull (rebase) |  |
| Git : Pull de toutes les branches | `Ctrl+G Ctrl+P` |
| Git : Pull de toutes les branches (fast-forward si possible) |  |
| Git : Pull de toutes les branches (fast-forward uniquement) |  |
| Git : Pull de toutes les branches (rebase) |  |
| Git : Options de pull... |  |
| Git : Push | `Ctrl+G P` |
| Git : Push vers un distant... |  |
| Git : Push vers des distants choisis... |  |
| Git : Push vers toutes les cibles |  |
| Git : Pull depuis toutes les sources |  |
| Git : Pull depuis un seul distant... |  |
| Git : Afficher le Push groupé |  |
| Git : Continuer le Push groupé |  |
| Git : Abandonner le Push groupé |  |
| Git : Relancer les envois échoués |  |
| Git : Push forcé (avec lease)... |  |
| Git : Distants de la branche... |  |
| Distant : Ajouter un distant... |  |
| Distant : Supprimer un distant... |  |
| Distant : Modifier un distant... |  |
| Distant : Définir le distant par défaut... |  |
| Distant : Copier le nom du distant |  |
| Distant : Copier le lien du distant |  |
| Distant : Fetch un distant... |  |
| Distant : Actions du distant sélectionné... | `.` |
| Dépôt : Extraction partielle... |  |
| Dépôt : Authentification SSH et identifiants... |  |
| Dépôt : Tester la connexion |  |
| Dépôt : Utiliser les identifiants par défaut |  |
| Flow : Ouvrir le menu | `Ctrl+G W` |
| Flow : Démarrer une branche... |  |
| Flow : Nouveau ticket... |  |
| Flow : Reprendre une branche de l'hôte... |  |
| Flow : Terminer la branche courante... |  |
| Flow : Synchroniser avec le parent... |  |
| Flow : Promouvoir... |  |
| Flow : Définir le parent... |  |
| Flow : Continuer ou abandonner l'action interrompue... |  |
| Flow : Basculer le groupement des branches |  |
| Flow : Paramètres du workflow... |  |
| Distant : Masquer le distant du graphe... |  |
| Distant : Afficher le distant dans le graphe... |  |
| Git : Nouvelle branche sur HEAD... | `Ctrl+G b` |
| Git : Nouveau tag sur HEAD... | `Ctrl+G t` |
| Git : Push tous les tags non envoyés | `Ctrl+G U` |
| Git : Stash de toutes les modifications | `Ctrl+G T` |
| Git : Stash de toutes les modifications avec message... | `Ctrl+G m` |
| Git : Stash : pop | `Ctrl+G o` |
| Git : Stash : apply |  |
| Git : Stash : drop... |  |
| Git : Cherry-pick du commit sélectionné sur HEAD | `Ctrl+G y` |
| Git : Basculer sur le commit sélectionné (HEAD détaché) |  |
| Git : Créer une branche sur le commit sélectionné... |  |
| Git : Créer un tag sur le commit sélectionné... |  |
| Commit : Copier le message |  |
| Commit : Copier le nom de l'auteur |  |
| Commit : Copier l'e-mail de l'auteur |  |
| Commit : Copier la date |  |
| Commit : Copier le SHA court |  |
| Commit : Copier le SHA complet |  |
| Git : Revert du commit sélectionné... | `Ctrl+G v` |
| Git : Modifier le message du commit... | `Ctrl+G e` |
| Commit : Modifier... | `Ctrl+G E` |
| Commit : Modifier l'auteur... | `Ctrl+G A` |
| Commit : Modifier le résumé... |  |
| Commit : Modifier la description... |  |
| Git : Scinder le commit sélectionné... |  |
| Git : Rebase interactif depuis le commit sélectionné... |  |
| Rebase : Continuer |  |
| Rebase : Passer |  |
| Rebase : Abandonner |  |
| git am : Continuer |  |
| git am : Ignorer ce patch |  |
| git am : Abandonner |  |
| Dépôt : Supprimer l'index.lock périmé |  |
| Bandeau : Restaurer l'auto-stash en attente (pop) |  |
| Bandeau : Garder le résultat de l'auto-stash |  |
| Bandeau : Afficher l'auto-stash en attente |  |
| Conflits : Afficher les conflits prévus |  |
| Bandeau : Supprimer l'auto-stash en attente... |  |
| Bandeau : Repartir de zéro... |  |
| Git : Réinitialiser sur le commit sélectionné (soft : garde index et fichiers) |  |
| Git : Réinitialiser sur le commit sélectionné (mixed : garde les fichiers) | `Ctrl+G R` |
| Git : Réinitialiser sur le commit sélectionné (hard : abandonne tout)... |  |
| Git : Résoudre le conflit avec Ours (cette branche) |  |
| Git : Résoudre le conflit avec Theirs (entrant) |  |
| Git : Résoudre le conflit avec Base (ancêtre commun) |  |
| Conflits : Ouvrir l'éditeur de merge | `Enter sur un fichier en conflit` |
| Merge : Conflit suivant | `F7` |
| Merge : Conflit précédent | `Shift+F7` |
| Merge : Fichier en conflit suivant | `Alt+PageDown` |
| Merge : Fichier en conflit précédent | `Alt+PageUp` |
| Merge : Prendre le bloc de A |  |
| Merge : Prendre le bloc de B |  |
| Merge : Prendre le courant pour ce conflit | `a` |
| Merge : Prendre l'entrant pour ce conflit | `b` |
| Merge : Prendre les deux, A puis B | `c` |
| Merge : Prendre les deux, B puis A | `C` |
| Merge : Prendre la base pour ce conflit |  |
| Merge : Vider ce conflit | `x` |
| Merge : Inverser l'ordre | `s` |
| Merge : Tout prendre du courant | `A` |
| Merge : Tout prendre de l'entrant | `B` |
| Merge : Modifier ce conflit dans le résultat |  |
| Merge : Afficher ou masquer les volets latéraux | `F9` |
| Merge : Faire défiler les volets vers la gauche | `Shift+Left` |
| Merge : Faire défiler les volets vers la droite | `Shift+Right` |
| Merge : Afficher ou masquer le volet de base | `Alt+D` |
| Merge : Basculer le diff du fichier entier | `Alt+F` |
| Merge : Annuler | `Ctrl+Z` |
| Merge : Rétablir | `Ctrl+Y` |
| Merge : Enregistrer sans marquer résolu | `Alt+S` |
| Merge : Enregistrer et marquer résolu | `Ctrl+S` |
| Merge : Réinitialiser le résultat au merge | `Alt+R` |
| Merge : Changer la hauteur du résultat | `z` |
| Merge : Élargir le volet Courant | `Alt+Right` |
| Merge : Élargir le volet Entrant | `Alt+Left` |
| Merge : Réinitialiser la taille des volets | `Alt+0` |
| Merge : Désélectionner ce conflit | `Alt+U` |
| Merge : Aller au volet Courant (A) | `Alt+1` |
| Merge : Aller au volet Entrant (B) | `Alt+2` |
| Merge : Aller au volet Résultat | `Alt+3` |
| Merge : Aller au volet Base | `Alt+4` |
| Merge : Volet de gauche (même conflit) |  |
| Merge : Volet de droite (même conflit) |  |
| Merge : Volet suivant | `F6` |
| Merge : Volet précédent | `Shift+F6` |
| Merge : Reprendre depuis le fichier sur disque |  |
| Merge : Recommencer depuis le merge |  |
| Merge : Rouvrir comme non résolu |  |
| Merge : Recharger le fichier |  |
| Merge : Garder le fichier courant tel quel | `a` |
| Merge : Garder le fichier entrant tel quel | `b` |
| Merge : Garder le fichier de base tel quel | `o` |
| Merge : Garder le fichier restant | `k` |
| Merge : Accepter la suppression | `D` |
| Merge : Ouvrir dans l'outil de merge externe | `m` |
| ✦ IA : Suggérer une résolution de conflit de merge | `Alt+I` |
| Merge : Fermer l'éditeur de merge |  |
| Git : Abandonner le merge, le cherry-pick ou le revert |  |
| Git : Passer le cherry-pick ou le revert |  |
| Dépôt : Ouvrir ou changer (gestion des dépôts)... | `Ctrl+O` |
| Dépôt : Parcourir pour trouver un dépôt... | `b (onglet de gestion)` |
| Dépôt : Créer un dépôt (init)... | `i (onglet de gestion)` |
| Dépôts : Ouvrir la sélection | `Enter` |
| Dépôts : Ajouter ou retirer la sélection des favoris | `f` |
| Dépôts : Fermer ou retirer la sélection | `x` |
| Dépôts : Menu du dépôt sélectionné... | `.` |
| Dépôts : Copier le chemin de la sélection | `y` |
| Dépôts : Ouvrir la sélection avec l'outil par défaut | `o` |
| Dépôts : Rechercher | `/ / Ctrl+F` |
| Dépôts : Replier toutes les sections | `-` |
| Dépôts : Déplier toutes les sections | `+` |
| Dépôts : Fermer tous les onglets... |  |
| Dépôts : Retirer la sélection de la liste |  |
| Dépôts : Retirer tous les récents |  |
| Espace de travail : Nouveau... | `n` |
| Espace de travail : Nouveau depuis les onglets ouverts... | `a` |
| Espace de travail : Changer... | `Alt+W` |
| Espace de travail : Modifier... | `e` |
| Espace de travail : Fermer (aucun espace de travail) |  |
| Espace de travail : Gérer... | `Alt+J` |
| Espace de travail : Fermer l'onglet |  |
| Espace de travail : Localiser le dépôt absent... | `l` |
| Espace de travail : Supprimer... |  |
| Espace de travail : Fetch All | `F` |
| Espace de travail : Pull All | `P` |
| Espace de travail : Cloner les absents | `m` |
| Espace de travail : Ajouter l'onglet à l'espace de travail |  |
| Espace de travail : Retirer l'onglet de l'espace de travail |  |
| Radar : Ouvrir | `Alt+R` |
| Radar : Actualiser | `F5` |
| Radar : Rechercher | `/` |
| Radar : Ouvrir dans le navigateur | `o` |
| Radar : Ouvrir le dépôt | `Enter` |
| Radar : Basculer sur la branche | `c` |
| Radar : Créer un worktree... | `t` |
| Radar : Afficher dans le dépôt | `s` |
| Radar : Commencer à travailler sur un ticket... | `n` |
| Radar : Épingler ou désépingler | `p` |
| Radar : Reporter... | `z` |
| Radar : Relire la pull request... | `e` |
| Radar : Ouvrir la conversation du ticket | `i` |
| Radar : Relire via l'API |  |
| Radar : Cloner et relire... |  |
| Radar : Modifier les assignés... |  |
| Radar : Modifier les relecteurs... |  |
| Radar : Modifier les étiquettes... |  |
| Radar : Définir le jalon... |  |
| Radar : Choisir le périmètre | `w` |
| Radar : Choisir la vue | `v` |
| Radar : Choisir le filtre | `f` |
| Radar : Afficher les demandes non clonées | `a` |
| Radar : Afficher les brouillons | `d` |
| Radar : Activer les relectures du compte... | `T` |
| Radar : Menu de la ligne... | `.` |
| Radar : Choisir une colonne | `C` |
| Radar : Élargir la colonne | `]` |
| Radar : Réduire la colonne | `[` |
| Radar : Réinitialiser les colonnes | `0` |
| Radar : Ajuster la colonne à son contenu | `=` |
| Radar : Ajuster toutes les colonnes à leur contenu | `+` |
| Radar : Retour à l'accueil | `Esc` |
| Radar : Fermer l'onglet |  |
| Onglet : Nouvel onglet | `Alt+T` |
| Onglet : Nouveau... |  |
| Onglet : Menu d'actions... | `Ctrl+T` |
| Onglet : Fermer les autres onglets... |  |
| Onglet : Fermer les onglets à droite... |  |
| Onglet : Ajouter aux favoris ou retirer |  |
| Onglet : Copier le chemin du dépôt |  |
| Onglet : Ouvrir dans le gestionnaire de fichiers |  |
| Onglet : Ouvrir dans un terminal |  |
| Onglet : Ouvrir dans... | `Alt+O` |
| Onglet : Ouvrir dans l'entrée par défaut | `Ctrl+Alt+O` |
| Dépôt : Changer de profil... |  |
| Aide : À propos de GitPoulpi | `Ctrl+N a` |
| Aide : Licences et crédits |  |
| Affichage : Journal d'activité | `Ctrl+N A` |
| Affichage : Journal d'activité global | `Ctrl+N G` |
| Aide : Notes de version |  |
| Mises à jour : Afficher la mise à jour | `Ctrl+N u` |
| Mises à jour : Ouvrir l'avis de sécurité |  |
| Mises à jour : Copier la commande de mise à jour |  |
| Mises à jour : Exécuter la commande de mise à jour dans un terminal |  |
| Mises à jour : Rechercher une mise à jour maintenant |  |
| Mises à jour : Télécharger |  |
| Mises à jour : Mettre à jour maintenant |  |
| Mises à jour : Redémarrer pour mettre à jour |  |
| Mises à jour : Revenir à la version précédente |  |
| Mises à jour : Ignorer cette version |  |
| Notes de version : Fermer l'onglet |  |
| Tâches : afficher les tâches en arrière-plan | `Ctrl+N J` |
| Tâches : annuler tout ce qui peut l'être |  |
| Affichage : Copier le diagnostic |  |
| Journal d'activité : Afficher ou masquer le niveau Erreur |  |
| Journal d'activité : Afficher ou masquer le niveau Avertissement |  |
| Journal d'activité : Afficher ou masquer le niveau Info |  |
| Journal d'activité : Afficher ou masquer le niveau Débogage |  |
| Journal d'activité : Afficher ou masquer le niveau Trace |  |
| Journal d'activité : Afficher tous les niveaux |  |
| Journal d'activité : N'afficher aucun niveau |  |
| Journal d'activité : Effacer le journal de l'application |  |
| Journal d'activité : Choisir le périmètre |  |
| Journal d'activité : Opérations Git |  |
| Journal d'activité : Journal de l'application |  |
| Journal d'activité : Afficher l'opération de cette ligne |  |
| Journal d'activité : Opération en échec suivante |  |
| Journal d'activité : Opération en échec précédente |  |
| Journal d'activité : Défiler à gauche | `Left` |
| Journal d'activité : Défiler à droite | `Right` |
| Journal d'activité : Copier la dernière erreur |  |
| Journal d'activité : Lancer l'action suggérée |  |
| Journal d'activité : Ouvrir la dernière erreur |  |
| Onglet : Quitter et faire cd ici |  |
| Dépôt : Cloner... | `Ctrl+Shift+N` |
| Dépôt : Onglet suivant | `Ctrl+PageDown / Ctrl+Tab` |
| Dépôt : Onglet précédent | `Ctrl+PageUp / Ctrl+Shift+Tab` |
| Onglet : Déplacer à gauche |  |
| Onglet : Déplacer à droite |  |
| Dépôt : Fermer l'onglet | `Ctrl+W` |
| Dépôt : Choisir un onglet... | `t` |
| Recherche : Espace de travail... | `Ctrl+Alt+F` |
| Édition : Annuler | `Ctrl+Z` |
| Édition : Rétablir | `Ctrl+Y` |
| Édition : Oublier l'annulation bloquée |  |
| Édition : Historique d'annulation... | `Alt+Z` |
| Édition : Reflog... |  |
| Édition : Retrouver les commits perdus... |  |
| Édition : Restaurer la branche là où l'action bloquée l'a laissée | `Alt+Shift+Z` |
| Revue : Afficher l'aperçu |  |
| Revue : Afficher les fichiers |  |
| Revue : Afficher les commits |  |
| Revue : Commenter la ligne... | `c` |
| Revue : Commenter la discussion... | `C` |
| Revue : Répondre au fil de discussion... | `r` |
| Revue : Résoudre ou rouvrir le fil de discussion | `x` |
| Revue : Retour au commentaire dans l'aperçu | `Esc` |
| Revue : Replier ou déplier le fil de discussion | `z` |
| Revue : Modifier le commentaire en attente... | `e` |
| Revue : Supprimer le commentaire en attente | `d` |
| Revue : Marquer le fichier comme vu | `v` |
| Revue : Fichier suivant | `]` |
| Revue : Fichier précédent | `[` |
| Revue : Terminer la revue... | `f` |
| Revue : Abandonner la revue en attente... |  |
| Revue : Actualiser | `F5` |
| Revue : Ouvrir dans le navigateur | `o` |
| Revue : Basculer sur la branche | `b` |
| Revue : Fermer | `Esc` |
| Revue : Modifier les assignés... | `a` |
| Revue : Modifier les relecteurs... | `p` |
| Revue : Modifier les étiquettes... | `l` |
| Revue : Définir le jalon... | `m` |
| Revue : Choisir le patch set... | `P` |
| Revue : Comparer des patch sets... | `D` |
| Revue : Soumettre le change... | `U` |
| Revue : Abandonner le change... | `Q` |
| Revue : Disposition unifiée ou côte à côte | `V` |
| Revue : Hunks seulement ou fichier entier | `H` |
| Revue : Afficher ou masquer les modifications d'espaces | `S` |
| Revue : Activer ou désactiver le retour à la ligne | `W` |
| Revue : Hunk suivant | `J` |
| Revue : Hunk précédent | `K` |
| Revue : Choisir l'encodage... | `E` |
| Revue : Afficher ou masquer la minimap | `M` |
| Revue : Défiler le diff vers la gauche | `Left` |
| Revue : Défiler le diff vers la droite | `Right` |
| Revue : Copier le texte sélectionné ou la ligne | `Ctrl+C` |
| Tableau : Déplacer la carte vers... | `m` |
| Tableau : Déplacer la carte à gauche |  |
| Tableau : Déplacer la carte à droite |  |
| Tableau : Ouvrir la carte | `Enter` |
| Tableau : Modifier les assignés... |  |
| Tableau : Modifier les étiquettes... |  |
| Tableau : Définir le jalon... |  |
| Tableau : Nouveau ticket dans cette colonne | `n` |
| Tableau : Filtrer par assigné... | `a` |
| Tableau : Filtrer par étiquette... | `l` |
| Tableau : Filtrer par jalon... | `M` |
| Tableau : Filtrer par texte | `/` |
| Tableau : Effacer les filtres | `x` |
| Tableau : Définir les étiquettes des colonnes... | `c` |
| Tableau : Actualiser | `r` |
| Tableau : Fermer | `Esc` |
| Conversation : Retour |  |
| Conversation : Actualiser | `F5` |
| Conversation : Charger les entrées plus anciennes | `m` |
| Conversation : Entrée précédente | `Up` |
| Conversation : Entrée suivante | `Down` |
| Conversation : Prochaine entrée non lue | `n` |
| Conversation : Répondre à l'entrée | `r` |
| Conversation : Citer l'entrée | `Q` |
| Conversation : Nouveau commentaire | `c` |
| Conversation : Liens de l'entrée... | `l` |
| Conversation : Ouvrir le diagramme de l'entrée | `g` |
| Conversation : Ouvrir le code du fil de discussion | `G` |
| Conversation : Copier le lien de l'entrée | `y` |
| Conversation : Copier le texte | `Ctrl+C` |
| Conversation : Ouvrir dans le navigateur | `o` |
| Conversation : Modifier le commentaire | `e` |
| Conversation : Supprimer le commentaire | `d` |
| Conversation : Afficher ou masquer les événements | `v` |
| Conversation : Basculer écriture et aperçu |  |
| Conversation : Basculer l'affichage côte à côte |  |
| Conversation : Envoyer le commentaire |  |
| Conversation : Basculer la note interne |  |
| Conversation : Fermer | `Esc` |
| Éditeur Markdown : écrire avec l'IA |  |
| Éditeur Markdown : actualiser le contexte de l'IA |  |
| Hébergeur : Ouvrir la conversation d'un ticket... |  |
| Revue : ✦ Rédiger une revue... | `s` |
| Revue : Accepter la suggestion | `y` |
| Revue : Modifier et accepter la suggestion... | `w` |
| Revue : Écarter la suggestion | `n` |
| Revue : Accepter toutes les suggestions | `A` |
| Revue : Écarter toutes les suggestions | `N` |
| Revue : Utiliser le résumé IA dans le texte de la revue | `i` |
| Revue : ✦ Utiliser le brouillon... | `u` |
| Revue : Modifier le texte du brouillon ✦... | `t` |
| Revue : Abandonner le brouillon ✦ | `N` |
| Historique d'annulation : Annuler jusqu'ici... | `u` |
| Historique d'annulation : Rétablir jusqu'ici... | `y` |
| Historique d'annulation : Restaurer seulement celle-ci... | `r` |
| Historique d'annulation : Restaurer la branche là où elle était... | `o` |
| Historique d'annulation : Actualiser |  |
| Historique d'annulation : Fermer | `Esc` |
| Reflog : Afficher dans le Log | `l` |
| Reflog : Créer une branche ici... | `b` |
| Reflog : Créer tag ici... | `t` |
| Reflog : Appliquer les changements à l'arbre de travail | `w` |
| Reflog : Enregistrer en stash | `s` |
| Reflog : Restaurer à cette position... | `r` |
| Reflog : Charger la suite | `m` |
| Reflog : Actualiser |  |
| Travail perdu : Créer une branche ici... | `b` |
| Travail perdu : Créer tag ici... | `t` |
| Travail perdu : Cherry-pick | `c` |
| Travail perdu : Appliquer les changements à l'arbre de travail | `w` |
| Travail perdu : Enregistrer en stash | `s` |
| Travail perdu : Relancer la recherche |  |
| Reflog / Travail perdu : Volet suivant | `Tab` |
| Reflog / Travail perdu : Liste des fichiers en chemins ou en arbre | `v` |
| Reflog / Travail perdu : Réinitialiser la taille des volets |  |
| Dialogue : Déplacer/redimensionner | `Ctrl+N m` |
| Dialogue : Réinitialiser la position et la taille |  |
| Affichage : Actualiser | `r` |
| Affichage : Actualiser (fetch, pull requests, tickets) | `F5` |
| Affichage : Afficher ou masquer l'aide | `?` |
| Affichage : Afficher ou masquer la colonne Ref |  |
| Affichage : Afficher ou masquer la colonne Graphe |  |
| Affichage : Afficher ou masquer la colonne Message |  |
| Affichage : Afficher ou masquer la colonne Auteur |  |
| Affichage : Afficher ou masquer la colonne Date |  |
| Affichage : Afficher ou masquer la colonne Modifs |  |
| Affichage : Afficher ou masquer la colonne SHA |  |
| Affichage : Colonne du Log : Choisir une colonne à déplacer ou à dimensionner | `C` |
| Affichage : Colonne du Log : Choisir la colonne suivante | `Tab` |
| Affichage : Colonne du Log : Choisir la colonne précédente | `Shift+BackTab` |
| Affichage : Colonne du Log : Déplacer la colonne à gauche | `<` |
| Affichage : Colonne du Log : Déplacer la colonne à droite | `>` |
| Affichage : Colonne du Log : Élargir la colonne | `]` |
| Affichage : Colonne du Log : Rétrécir la colonne | `[` |
| Affichage : Colonne du Log : Réinitialiser les largeurs des colonnes | `0` |
| Affichage : Colonne du Log : Ajuster la colonne à son contenu | `=` |
| Affichage : Colonne du Log : Ajuster toutes les colonnes à leur contenu | `+` |
| Affichage : Log : Faire défiler les colonnes vers la gauche | `Shift+Left` |
| Affichage : Log : Faire défiler les colonnes vers la droite | `Shift+Right` |
| Affichage : Log : Activer ou désactiver le défilement latéral des colonnes | `s` |
| Affichage : Log : Choisir les colonnes à afficher |  |
| Diff : Ouvrir le diff du fichier sélectionné | `Enter` |
| Diff : Fermer la vue diff | `Esc` |
| Diff : Hunk suivant | `n` |
| Diff : Hunk précédent | `p` |
| Diff : Stage du hunk sous le curseur | `Space` |
| Diff : Unstage du hunk sous le curseur | `Space` |
| Diff : Sélectionner des lignes (ou effacer la sélection) | `v` |
| Diff : Stage des lignes sélectionnées | `Enter` |
| Diff : Unstage des lignes sélectionnées | `Enter` |
| Diff : Abandonner le hunk sous le curseur... | `x` |
| Diff : Abandonner les lignes sélectionnées... | `X` |
| Copier : Texte sélectionné (sélectionnez-le d'abord à la souris) | `Ctrl+C` |
| Copier : Ligne sous le curseur (ou lignes sélectionnées) | `y` |
| Diff : Copier le hunk sous le curseur |  |
| Diff : Disposition en ligne (+/- sur une colonne) | `m` |
| Diff : Disposition côte à côte (avant et après) | `m` |
| Diff : Hunks seulement ou fichier entier | `H` |
| Fichiers : Basculer chemin / arborescence | `m` |
| Fichiers : Basculer l'affichage de tous les fichiers | `a` |
| Commit : Basculer Ignorer les hooks | `H` |
| Fichiers : Ouvrir le menu de la ligne ou de la sélection | `.` |
| Fichiers : Déplier tous les dossiers | `=` |
| Fichiers : Replier tous les dossiers | `-` |
| Fichiers : Déplier le dossier sélectionné et ses sous-dossiers | `*` |
| Fichiers : Replier le dossier sélectionné et ses sous-dossiers | `_` |
| Fichiers : Tout sélectionner |  |
| Fichiers : Effacer la sélection |  |
| Fichiers : Basculer la sélection de la ligne courante |  |
| Log : Copier en gitGraph Mermaid... | `Y` |
| Log : Comparer la sélection | `v` |
| Log : Inverser les extrémités de la comparaison | `x` |
| Log : Aller à la ligne comparée suivante | `n` |
| Diff : Basculer le retour à la ligne | `w` |
| Diff : Choisir l'encodage... | `C` |
| Affichage : Afficher ou masquer la minimap | `M` |
| Diff : Basculer la mise en évidence des mots modifiés |  |
| Affichage : Afficher ou masquer la colonne des commits | `Alt+C` |
| Affichage : Colonne des commits : Masquer |  |
| Affichage : Colonne des commits : Historique du fichier |  |
| Affichage : Colonne des commits : Tous les commits |  |
| Affichage : Placer le clavier sur la colonne des commits |  |
| Affichage : Élargir la colonne des commits |  |
| Affichage : Rétrécir la colonne des commits |  |
| Affichage : Colonne des commits : Faire défiler à gauche | `Shift+Left` |
| Affichage : Colonne des commits : Faire défiler à droite | `Shift+Right` |
| Affichage : Colonne des commits : Dimensionner la partie suivante | `Tab` |
| Affichage : Colonne des commits : Déplacer la partie à gauche | `<` |
| Affichage : Colonne des commits : Déplacer la partie à droite | `>` |
| Affichage : Colonne des commits : Élargir la partie | `]` |
| Affichage : Colonne des commits : Rétrécir la partie | `[` |
| Affichage : Colonne des commits : Réinitialiser les largeurs des parties | `0` |
| Affichage : Colonne des commits : Ajuster la colonne à son contenu | `=` |
| Affichage : Colonne des commits : Ajuster toutes les colonnes à leur contenu | `+` |
| Affichage : Colonne des commits : Menu de la ligne choisie | `.` |
| Affichage : Colonne des commits : Choisir les colonnes |  |
| Affichage : Colonne des commits : Afficher ou masquer Refs |  |
| Affichage : Colonne des commits : Afficher ou masquer Graphe |  |
| Affichage : Colonne des commits : Afficher ou masquer Auteur |  |
| Affichage : Colonne des commits : Afficher ou masquer Date |  |
| Affichage : Colonne des commits : Afficher ou masquer Modifs |  |
| Affichage : Colonne des commits : Afficher ou masquer SHA |  |
| Affichage : Colonne des commits : Afficher la ligne choisie dans le Log |  |
| Affichage : Colonne des commits : Modifications du fichier à la ligne choisie |  |
| Affichage : Colonne des commits : Fichier tel qu'il était à la ligne choisie |  |
| Affichage : Colonne des commits : Blame du fichier à la ligne choisie |  |
| Éditeur : Basculer le suivi de la sélection |  |
| Image : Zoom avant | `+` |
| Image : Zoom arrière | `-` |
| Image : Ajuster à la vue | `0` |
| Image : Taille réelle (1:1) | `1` |
| Image : Basculer avant / après | `Tab` |
| Image : Élargir le volet Avant | `Alt+Right` |
| Image : Élargir le volet Après | `Alt+Left` |
| Image : Rétablir les tailles des volets | `Alt+0` |
| Image : Éditer avec... | `O` |
| Image : Éditer avec l'éditeur par défaut | `o` |
| Historique du fichier : Afficher dans le Log | `Enter` |
| Historique du fichier : Copier le SHA du commit | `y` |
| Historique du fichier : Copier le résumé | `Y` |
| Historique du fichier : Afficher le diff de ce commit | `d` |
| Historique du fichier : Ouvrir le fichier à ce commit | `o` |
| Historique du fichier : Blame à ce commit | `b` |
| Historique du fichier : Comparer avec l'arbre de travail | `c` |
| Historique du fichier : Restaurer cette version | `R` |
| Historique du fichier : Étendre la sélection vers le haut | `Shift+Up` |
| Historique du fichier : Étendre la sélection vers le bas | `Shift+Down` |
| Historique du fichier : Basculer entre la liste et le diff | `Tab` |
| Historique du fichier : Élargir la liste | `Alt+Right` |
| Historique du fichier : Rétrécir la liste | `Alt+Left` |
| Historique du fichier : Réinitialiser la largeur de la liste | `Alt+0` |
| Fichier : Ouvrir avec... | `O` |
| Fichier : Ouvrir dans l'application par défaut | `o` |
| Diff : Basculer les modifications d'espaces seuls | `s` |
| ✦ IA : Générer le message de commit | `Ctrl+G i` |
| ✦ IA : Générer le message de stash |  |
| ✦ IA : Insérer une variable de prompt |  |
| ✦ IA : Modifier un profil... |  |
| ✦ IA : Ouvrir l'agent... |  |
| ✦ IA : Expliquer le commit sélectionné | `Ctrl+G w` |
| ✦ IA : Scinder le commit sélectionné... |  |
| ✦ IA : Expliquer les modifications locales | `Ctrl+G w` |
| ✦ IA : Composer des commits à partir des modifications locales... | `Ctrl+G C` |
| Fichiers sensibles : Afficher dans le fichier | `S` |
| Composer : Déplacer le fichier vers un commit... | `m` |
| Diff : Blame |  |
| Blame : Afficher le commit dans le Log | `Enter` |
| Blame : Copier le SHA du commit | `Y` |
| Blame : Diff du commit pour ce fichier | `d` |
| Blame : Blame au parent du commit | `b` |
| Blame : Ouvrir l'historique du fichier | `H` |
| Blame : Afficher ou masquer la fiche de détail | `i` |
| Blame : Hunk suivant | `n` |
| Blame : Hunk précédent | `p` |
| Blame : Hunk suivant du même commit | `N` |
| Blame : Hunk précédent du même commit | `P` |
| Blame : Aller à la ligne | `g` |
| Blame : Élargir la colonne d'annotation | `>` |
| Blame : Réduire la colonne d'annotation | `<` |
| Diff : Historique du fichier |  |
| Diff : Vue du fichier |  |
| Fichier : Modifier dans le dossier de travail | `e` |
| Éditeur : Changer l'encodage |  |
| Éditeur : Rouvrir avec l'encodage |  |
| Éditeur : Convertir vers l'encodage |  |
| Éditeur : Convertir les fins de ligne |  |
| Éditeur : Changer l'indentation |  |
| Application : Paramètres... | `Ctrl+, / Ctrl+N ,` |
| Application : Rechercher dans les Paramètres... | `/ dans les Paramètres` |
| Application : Raccourcis clavier... |  |
| Agent : Afficher la vue en attente | `Ctrl+N Enter` |
| Agent : Ignorer la vue en attente |  |
| Agent : Retour | `Ctrl+N Backspace` |
| Raccourcis : Tout réinitialiser... |  |
| Intégrations : Ouvrir la pull request liée | `Ctrl+G g` |
| Intégrations : Modifier la pull request liée sur l'hébergeur | `e (sur une ligne)` |
| Intégrations : Copier l'URL de la pull request liée |  |
| Intégrations : Actualiser les pull requests et les tickets |  |
| Intégrations : Basculer sur une pull request... |  |
| Intégrations : Ouvrir une référence de commit (#ticket / !demande)... |  |
| Intégrations : Ouvrir un ticket... |  |
| Hébergeur : Relire une pull request... |  |
| Hébergeur : Push pour relecture... |  |
| Hébergeur : Modifier les assignés... |  |
| Hébergeur : Modifier les relecteurs... |  |
| Hébergeur : Modifier les étiquettes... |  |
| Hébergeur : Définir le jalon... |  |
| Tickets locaux : Fermer un ticket... |  |
| Tickets locaux : Rouvrir un ticket... |  |
| Tickets locaux : Ouvrir dans Obsidian... |  |
| Tickets : Ouvrir le tableau |  |
| Tickets locaux : Créer une Base Obsidian |  |
| Pull requests : Créer une pull request... | `n` |
| Hébergeur : Nouveau ticket... | `n` |
| Pull requests : Rechercher | `Enter sur la ligne de recherche` |
| Pull requests : Choisir le filtre (ouvertes / mergées / toutes)... | `v` |
| Pull requests : Choisir l'hébergeur... | `Enter sur la ligne de l'hébergeur` |
| Pull requests : Choisir le dépôt... | `Enter sur la ligne du dépôt` |
| Pull requests : Délier la source (aucune) |  |
| Tickets : Choisir l'hébergeur... | `Enter sur la ligne de l'hébergeur` |
| Tickets : Délier la source (aucune) |  |
| Intégrations : Délier le profil (automatique) |  |
| Tickets : Choisir le dépôt... | `Enter sur la ligne du dépôt` |
| Tickets : Rechercher | `Enter sur la ligne de recherche` |
| Tickets : Choisir le filtre... | `v` |
| Tickets : Modifier la requête / les listes... |  |
| Commit : Insérer une référence de ticket... |  |
| Intégrations : Utiliser un profil pour ce dépôt... |  |
| Intégrations : Gérer le profil de ce dépôt |  |
| Intégrations : Connecter une intégration... |  |
| Paramètres : Options du Radar d'un profil... |  |
| Intégrations : Revoir le contrôle de sécurité |  |
| Application : Quitter | `q` |
| Application : Thème... |  |
| Conflits : Modifier les listes de branches de base... |  |
| Conflits : Branches de base de ce dépôt... |  |
| Application : Changer de thème... | `Ctrl+N t` |
| Application : Changer de langue... | `Ctrl+N l` |
| Lanceur : Créer le raccourci |  |
| Lanceur : Créer le raccourci du bureau |  |
| Lanceur : Supprimer les raccourcis |  |
| Lanceur : Créer l'alias de commande |  |
| Lanceur : Supprimer l'alias de commande |  |
| MCP : Suggérer un port libre |  |
| Lanceur : Ajouter au PATH |  |
| Lanceur : Retirer du PATH |  |
| Lanceur : Installer le menu Ouvrir dans GitPoulpi |  |
| Lanceur : Supprimer le menu Ouvrir dans GitPoulpi |  |
| Lanceur : Devenir l'installation principale |  |
| Lanceur : Supprimer les anciens lanceurs |  |
| Lanceur : Oublier les installations disparues |  |
| Application : Changer le thème de couleurs du graphe... | `Ctrl+N T` |
| Application : Basculer la teinte de ligne du graphe | `Ctrl+N Y` |
<!-- palette:end -->

## Se déplacer

| Touche | Action |
|---|---|
| `Tab` / `Shift+Tab` | Cibler le volet suivant / précédent |
| `Ctrl+N`, puis un badge | Cibler cette partie de la fenêtre, ou actionner ce contrôle de la boîte de dialogue ou de la fenêtre au premier plan (voir [Préfixe de navigation](#préfixe-de-navigation-ctrln)) |
| `j` / `k`, `Up` / `Down` | Déplacer la sélection, dans chaque volet |
| `PageUp` / `PageDown`, `Home` / `End` | Avancer d'une page ; aller à la première / dernière ligne |
| un nombre, puis `j` / `k` / `Up` / `Down` | Préfixe de nombre : `4j` avance de quatre lignes. Avec `PageUp` / `PageDown`, le nombre est un nombre de pages. La ligne d'état montre le nombre en attente (`4...`). `Esc` ou toute autre touche l'efface. |
| `h` / `l`, `Left` / `Right` | Volet Refs : changer de section. Log : quitter (`h`) ou entrer dans (`l`) la liste des fichiers modifiés d'un commit |
| `?` | Ouvrir ou fermer l'aide (voir [Aide dans l'application](#aide-dans-lapplication-)) |
| `r` | Actualiser |
| `F5` | Fetch de chaque distant, rechargement des pull requests et des tickets, et actualisation |
| `q` | Quitter |
| `Ctrl+Z` | **Annuler** la dernière opération. Voir [Annuler et rétablir](undo-redo.md) |
| `Ctrl+Y`, `Ctrl+Shift+Z` | **Rétablir**. `Ctrl+Shift+Z` demande le protocole clavier kitty ; `Ctrl+Y` marche toujours |
| `.`, `Menu`, `Shift+F10` | Ouvrir le menu contextuel de la ligne sélectionnée |

### Ctrl+Z / Ctrl+Y dans un champ de texte

Tant qu'un champ de texte a le clavier, `Ctrl+Z` et `Ctrl+Y` (`Ctrl+Shift+Z` aussi)
agissent sur le texte de ce champ, jamais sur le dépôt : un champ qui a le clavier gagne
toujours, si bien que `Ctrl+Z` dans le message de commit ne peut pas annuler un commit.
Partout ailleurs, ils gardent leur sens global (le journal d'annulation / rétablissement
ci-dessus). Cela vaut pour tous les champs de texte et éditeurs multilignes : le message de
commit, les champs et éditeurs de texte des Paramètres, les formulaires de pull request et
de ticket, les fenêtres de l'IA et chaque filtre ou zone de recherche.

- **La saisie** s'annule un mot à la fois (une suite de lettres, une suite d'espaces, une
  suite de ponctuation), et une saisie qui s'interrompt plus d'une seconde commence une
  nouvelle étape. Des retours arrière à la suite forment une seule étape. Un collage, un
  couper, `Ctrl+U` / `Ctrl+K`, un modèle inséré ou une complétion acceptée forment chacun
  une étape.
- Une annulation rend le texte, le curseur et la sélection. Une nouvelle modification après
  une annulation abandonne ce qui pouvait être rétabli.
- Chaque champ garde un historique borné (les 200 dernières étapes, environ 1 Mo de texte),
  et l'oublie quand le champ est remplacé de l'extérieur (un dossier choisi dans le
  sélecteur, un formulaire rechargé) ou qu'un autre champ prend le clavier.
- Quand il n'y a plus rien à annuler, `Ctrl+Z` ne fait rien : il ne retombe jamais sur
  l'annulation du dépôt tant qu'un champ a le clavier.
- L'**éditeur d'instructions** des paramètres de l'IA a aussi des boutons **Annuler** et
  **Rétablir**, à côté d'Enregistrer et Annuler, désactivés tant qu'il n'y a rien à
  parcourir.
- L'éditeur de code et l'éditeur de merge gardent leur propre annulation (`Édition : Annuler`,
  `Merge : Annuler`), qui couvre aussi leurs choix.

## Listes de popups filtrables

Toutes les listes de fenêtres contextuelles filtrables utilisent les mêmes touches : la
palette de commandes et ses sélecteurs (thème, langue, couleurs du graphe), le sélecteur
d'onglets, et chaque liste déroulante avec une ligne de recherche (choix des Paramètres,
formulaire de pull request, sélecteur de références de tickets, liste **Ouvrir dans**).
Tapez pour filtrer la liste. Ces touches déplacent la surbrillance :

| Touche | Action |
|---|---|
| `Up` / `Down`, `Ctrl+K` / `Ctrl+J` | Monter / descendre d'une ligne |
| `Home` / `End` | Première / dernière ligne |
| `PageUp` / `PageDown` | Avancer d'une page visible |
| `Enter` | Choisir la ligne en surbrillance |
| `Esc` | Fermer |

Dans ces fenêtres, `Home` et `End` déplacent la liste, pas le curseur du champ de filtre.
`Ctrl+K` / `Ctrl+J` déplacent aussi la liste, si bien que `Ctrl+K` ne supprime plus jusqu'à
la fin du filtre (`Ctrl+U` l'efface toujours ; dans un champ de texte ordinaire hors d'une
liste, `Ctrl+K` continue de supprimer jusqu'à la fin de la ligne). `Ctrl+A` et `Ctrl+E`
déplacent toujours le curseur au début et à la fin. `Ctrl+J` / `Ctrl+K` déplacent aussi
toutes les autres listes, avec ou sans champ de texte (`Ctrl+N` est le préfixe de
navigation et `Ctrl+P` la palette de commandes, ce ne sont donc pas des touches de liste).
Les simples `j` et `k` sont tapés dans le filtre ; dans les listes sans champ de texte,
`j` / `k` sont un moyen de plus de descendre / monter.

À la souris :

- La molette **fait défiler** la liste. Elle ne déplace pas la surbrillance.
- Un clic **sélectionne** une ligne. Un double-clic la **choisit**, comme `Enter`.
- Une liste plus haute que sa fenêtre montre une barre de défilement sur son bord droit.
  Vous pouvez la glisser.
- Une courte liste déroulante sans ligne de recherche (deux ou trois choix) choisit au
  premier clic.

**Aperçu en direct.** Les sélecteurs de thème, de thème de couleurs du graphe et de langue
appliquent aussitôt la ligne en surbrillance, derrière la fenêtre. `Enter` la garde. `Esc`,
ou un clic hors de la fenêtre, rétablit la valeur précédente.

**Filtre de thème.** Le sélecteur de thème a une ligne sous le champ de filtre : **Tous**,
**Sombres**, **Clairs** et **Colorés**. `Tab` / `Shift+Tab` vont au suivant / précédent.
`Alt+1` à `Alt+4` en choisissent un directement. Vous pouvez aussi cliquer dessus.

## Aide dans l'application (`?`)

`?`, `Ctrl+N ?` ou `Affichage : Afficher ou masquer l'aide` de la palette ouvre l'aide.
Elle liste les touches et les gestes de souris de chaque partie de l'application. Elle
s'ouvre par-dessus tout, l'écran des Paramètres compris.

| Touche | Action |
|---|---|
| `?` | Fermer l'aide (et effacer le filtre) |
| *(taper)* | Filtre : ne garde que les sections qui contiennent le texte. `j`, `k` et `q` sont tapés aussi |
| `Backspace`, `Ctrl+U` | Supprimer le dernier caractère / effacer le filtre |
| `Esc` | Effacer le filtre ; s'il est vide, fermer l'aide |
| `Up` / `Down` | Défiler d'une ligne |
| `PageUp` / `PageDown`, `Home` / `End` | Défiler d'une page ; aller en haut / en bas |
| `Ctrl+D`, ou un clic sur le lien | Ouvrir cette documentation sur GitHub dans le navigateur |

La molette fait défiler l'aide, et vous pouvez glisser sa barre de défilement. Un clic en
dehors de l'aide la ferme.

## Préfixe de navigation (`Ctrl+N`)

`Ctrl+N` attend une touche de plus. La ligne d'état montre les touches qu'il accepte. La
touche suivante lance une action, ou annule.

| Touche | Action |
|---|---|
| `Ctrl+N` puis un badge | Cibler la partie de la fenêtre qui affiche ce badge |
| `Ctrl+N Shift+H` / `Ctrl+N Shift+L` | Replier ou restaurer le volet Refs / le volet de droite |
| `Ctrl+N Left` / `Ctrl+N Right` | Déplacer d'un cran vers la gauche / la droite le séparateur entre le volet Refs et le Log (un volet Refs replié revient d'abord) |
| `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` | Déplacer d'un cran vers la gauche / la droite le séparateur entre le Log et le volet de droite |
| `Ctrl+N +` / `Ctrl+N -` | Agrandir / réduire la section Refs active d'une ligne |
| `Ctrl+N ]` / `Ctrl+N [` | Agrandir / réduire d'une ligne le bloc du message de commit du volet de droite |
| `Ctrl+N Tab` | Choisir un onglet (le sélecteur d'onglets ; `t` seul fait la même chose) |
| `Ctrl+N t` / `Ctrl+N l` / `Ctrl+N Shift+T` | Changer le thème / la langue / le thème de couleurs du graphe |
| `Ctrl+N Shift+Y` | Activer ou désactiver les teintes de ligne du graphe |
| `Ctrl+N Enter` / `Ctrl+N Backspace` | Afficher la vue qu'un agent a préparée / revenir en arrière (voir [MCP et agents](mcp-and-agents.md)) |
| `Ctrl+N ,` | Ouvrir les Paramètres |
| `Ctrl+N ?` | Ouvrir ou fermer l'aide |
| `Esc`, ou toute autre touche | Annuler |

### Badges

Pendant que le préfixe attend, chaque partie de la fenêtre qui peut prendre le focus
montre un badge d'un caractère sur sa bordure supérieure gauche. Appuyez sur ce caractère
pour la cibler. Les badges sont redistribués à chaque fois, dans l'ordre de l'écran : `1`
à `9` d'abord, puis `a`, `b`, `c`... Lisez donc le badge à l'écran : une partie n'a pas
toujours le même badge.

Parties avec un badge : le filtre des Refs et chaque section Refs visible, le Log (ou la
vue Diff), la vue Fichier quand elle est ouverte, les listes Unstaged et Staged, le champ
de résumé du compositeur de commits, et les contrôles de la liste des fichiers du commit.
Sur l'onglet Accueil, les badges sont la zone de recherche, les quatre sections et les boutons de la barre d'outils (le badge d'un bouton l'actionne).
Sur l'onglet Radar, ce sont le champ de filtre, les listes déroulantes Périmètre, Vue
et Afficher (le badge d'une liste déroulante l'ouvre), les boutons Actualiser et Retour,
et chaque titre de section à l'écran. Le badge d'une section place la sélection sur son
titre ; `Enter` ou `Space` la replie ensuite.

Dans le sélecteur de dossier Parcourir et dans les boîtes Cloner, Init, Créer un worktree, Espace de travail, Sparse checkout, Flow, Nouveau ticket et Pull request, les badges sont les champs, les cases à cocher et les boutons (le badge d'un bouton l'actionne, celui d'un champ y place le curseur ; dans une liste, un seul badge est sur sa première ligne ; le sélecteur de dossier Parcourir a un badge pour la liste Accès rapide et un pour la liste Dossiers, à côté de la première ligne de chacune, et appuyer sur l'un place le clavier dans cette liste sans choisir de ligne). Dans le Journal d'activité, ce sont les deux onglets, les zones Périmètre et Niveau, le filtre, les cases des niveaux, les boutons et la première entrée de la liste. Un badge fait ce que ferait un clic sur ce contrôle. Dans le sélecteur de dossier, `Ctrl+N` lance les badges : sa touche Nouveau dossier est donc `Alt+N`.

Dans la fenêtre Modifier (et Modifier le message du commit), les badges sont le résumé, la description, le nom, l'e-mail et la date de l'auteur, **Modifier l'historique en profondeur...**, **Annuler**, **Appliquer** et la croix de fermeture ; dans l'atelier de réécriture d'historique, ce sont la zone du périmètre, ses cases et son bouton de refs, le nom de la recette, chaque règle, sa case et ses boutons, le bouton **+ Ajouter une règle**, les champs de la règle sélectionnée, les onglets de l'aperçu, la zone de filtre, les lignes de branches protégées et leurs Tout cocher / Tout décocher, et les boutons Aperçu, Appliquer, Annuler, Enregistrer, Charger et Créer des règles avec l'IA (les pages Secrets, Identités, Taille et résultat ajoutent leurs propres boutons et lignes ; les lignes de l'onglet Commits, étant une liste, n'en ont pas). Un badge fait ce que ferait un clic. Rien n'est badgé tant qu'une liste déroulante, une question, un champ en cours de saisie, le sélecteur de dossier, l'écran de purge, l'éditeur de script, les boîtes IA, le diff de blob ou le guide d'hébergeur est ouvert, ni pendant que la fenêtre Modifier demande « abandonner ? » ; `Échap` retire les badges sans rien fermer.

Dans les Paramètres, les badges sont la zone de recherche, la liste des pages (un seul badge, sur la page courante), les sous-onglets, la liste et ses boutons, chaque champ de la page visible à l'écran et la croix de fermeture. Un badge fait ce que ferait un clic sur ce contrôle : un champ de texte s'ouvre en édition, une case se coche, une liste déroulante s'ouvre, un bouton s'exécute. `Ctrl+N` garde son sens propre pendant la saisie dans un champ ou quand une liste déroulante, un éditeur ou la recherche est ouvert.

La palette a les mêmes actions, par exemple
`Navigation : Aller à Refs > Locales` ou `Navigation : Aller à Status > Staged`.

## Dépôts et onglets

| Touche | Action |
|---|---|
| `Ctrl+O`, ou un clic sur `⌂` | Aller à l'onglet Accueil (sur l'onglet Accueil : cibler la zone de recherche ; avec le Radar ou les notes de version ouverts : le fermer d'abord) |
| `Ctrl+Shift+N` | Ouvrir l'onglet Accueil avec la boîte Cloner |
| `Ctrl+PageDown` / `Ctrl+PageUp`, `Ctrl+Tab` / `Ctrl+Shift+Tab` | Onglet suivant / précédent (la paire `Ctrl+Tab` seulement si le terminal l'envoie) |
| `t` | Choisir un onglet dans une liste filtrable |
| `Ctrl+W` | Fermer l'onglet courant (demande d'abord ; le dépôt n'est pas modifié) |
| `Ctrl+T`, ou un clic droit sur un onglet | Le menu de l'onglet (ses lignes sont listées dans [Barre d'onglets](interface-tour.md#barre-donglets)) |
| `Alt+O` / `Ctrl+Alt+O` | Ouvrir le dossier de l'onglet : choisir dans la liste **Ouvrir dans** / utiliser l'entrée par défaut. Voir [Ouvrir dans](open-in.md) |

### L'onglet Accueil

L'onglet `⌂` gère vos dépôts. Il a quatre sections : **Dépôts ouverts**, **Favoris**,
**Dépôts récents** et **Tous les dépôts**. Voir
[Visite de l'interface](interface-tour.md#accueil).

| Touche | Action |
|---|---|
| `j` / `k`, flèches, `PageUp` / `PageDown`, `Home` / `End` | Se déplacer |
| `Enter`, ou un double-clic | Ouvrir le dépôt (ou passer à son onglet). Sur un en-tête de section : la replier ou la déplier |
| `f` ou `*`, ou un clic sur l'étoile | Ajouter aux favoris / retirer des favoris |
| `x`, `Delete`, ou le bouton de la ligne | Fermer son onglet (Ouverts), le retirer de la liste (Récents), retirer l'étoile (Favoris), l'oublier (Tous). Rien n'est supprimé sur le disque |
| `.`, un clic droit, ou le `⋮` de la ligne | Le menu de la ligne |
| `/` ou `Ctrl+F` | Chercher dans toutes les sections par nom, branche ou chemin. `Enter` ouvre la première correspondance ; `Esc` efface la recherche, puis quitte le champ |
| `Space`, `Left` / `Right` | Replier / déplier la section courante |
| `-` / `+` | Tout replier / Tout déplier |
| `b` / `c` / `i` | Parcourir (ouvrir un dossier) / Cloner / Init |
| `Esc` | Effacer la recherche, ou revenir à l'onglet de dépôt actif |
| `q` | Quitter |

`Ctrl+P`, `Ctrl+,`, `?` et `Ctrl+N` fonctionnent sur l'onglet Accueil. `Ctrl+G`, `Ctrl+W`
et `Ctrl+T` demandent un onglet de dépôt : sur l'onglet Accueil ils n'affichent qu'un
message. Les commandes de palette qui demandent un dépôt sont grisées.

### Le sélecteur de dossiers

**Parcourir** (et le bouton **Parcourir...** des boîtes Init, Cloner et Créer un worktree)
ouvre le sélecteur de dossiers. Il a un champ de chemin en haut, les emplacements et
l'arbre des dossiers à gauche, et le contenu du dossier courant à droite.

| Touche | Action |
|---|---|
| `Tab` / `Shift+Tab` | Champ de chemin, puis la colonne de gauche, puis le contenu |
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Se déplacer dans la colonne ciblée. Dans le champ de chemin : parcourir les complétions |
| une lettre (contenu) | Aller au prochain dossier qui commence par cette lettre |
| `Enter` / `Right` | Ouvrir le dossier sélectionné. Dans le champ de chemin : aller au dossier tapé ; sur le dossier déjà affiché, le choisir |
| `Backspace` / `Left` / `Alt+Up` | Dossier parent (dans le champ de chemin, `Backspace` supprime un caractère) |
| `Tab` (champ de chemin) | Accepter la seule complétion, ou celle en surbrillance |
| `Alt+N` | Nouveau dossier (`Ctrl+N` lance les badges, voir [Badges](#badges)) |
| `Alt+H` | Afficher ou masquer les dossiers cachés |
| `Ctrl+F`, ou `/` hors du champ de chemin | Filtrer les entrées du dossier pendant la frappe (sans tenir compte de la casse ; `..` reste ; l'en-tête compte « x sur n » ; `Enter` va à la première correspondance, `Esc` vide le filtre ; il se vide quand vous changez de dossier). Aussi accessible avec `Tab` ou un clic |
| `Ctrl+S`, `Ctrl+Enter`, ou le bouton de confirmation | Choisir le sous-dossier sélectionné, sinon le dossier affiché |
| `Esc` | Fermer les complétions, ou annuler |

Quand le champ de chemin affiche un nouveau chemin, tout le chemin est sélectionné : la
première touche tapée le remplace.

### Boîtes Init et Cloner

| Touche | Action |
|---|---|
| `Tab` / `Shift+Tab`, `Up` / `Down` | Champ ou bouton suivant / précédent |
| `Enter` | Appuyer sur le bouton ciblé, ou créer / cloner |
| `Space` | Basculer la case à cocher ciblée, ou appuyer sur le bouton ciblé |
| `Ctrl+B` | Parcourir pour le dossier |
| `j` / `k`, `g` / `G` | Dans la liste des sources et la liste des dépôts de la boîte Cloner : bas / haut, premier / dernier (pas dans le champ de recherche) |
| `Esc` | Fermer. Pendant un clonage, le clonage continue en arrière-plan |

Dans la boîte Cloner, la colonne de gauche liste **Cloner avec une URL** et une source par
profil d'hébergeur connecté (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit). Choisissez une
source avec `Up` / `Down` ou un clic. Tapez ensuite dans le champ de recherche
**Dépôt**, déplacez-vous dans la liste avec les flèches, `PageUp` / `PageDown` ou la
molette, et choisissez une ligne avec `Enter` ou un clic. Voir [Premiers pas](getting-started.md)
pour les champs.

## Volet Refs

| Touche | Action |
|---|---|
| `Enter` / `Space` | Déplier ou replier la section active, ou le dossier sélectionné |
| `h` / `l`, `Left` / `Right` | Changer de section |
| `/` | Filtrer par nom (`Enter` applique, `Esc` annule) |
| `y` | Vue hiérarchique des branches activée ou non : des dossiers à partir des `/` des noms de branches (LOCALES et DISTANTES) |
| `o` | **Solo** sur la ref sélectionnée, ou sur chaque ref sous le dossier ou le distant sélectionné. Mettez plusieurs refs en solo pour voir leur union |
| `Shift+O` | **Arrêter le solo** : effacer chaque solo |
| `Shift+H` | **Tout afficher** : réafficher chaque ref masquée |
| `g` | **Révéler dans le graphe** : fait défiler le Log jusqu'au commit de la ref (ou du stash) sélectionnée et fait clignoter sa ligne ; la sélection ne bouge pas |
| `m` | **Marquer** la ref sélectionnée comme source de dépôt (`Échap`, ou `m` de nouveau dessus, efface la marque) |
| `Shift+M` | **Déposer la ref marquée** sur la ref sélectionnée : ouvre le menu qu'ouvre un glisser-déposer (Merge, Rebase, Push, Fast-forward...) |
| `+` (section DISTANTES) | Ajouter un distant |
| `.`, `Menu`, `Shift+F10`, ou un clic droit | Le menu de la ligne : une branche, un tag, le nom d'un distant, ou un dossier |
| `Enter` sur une ligne WORKTREES | Ouvrir ce worktree dans son propre onglet |
| `Ctrl+G d` | Supprimer la branche ou le tag sélectionné (demande d'abord). Sur une ligne DISTANTES : supprimer cette branche distante |

Pour ce que font les menus, voir [Travailler avec le Log](log-and-history.md).

### Pull requests et tickets

Avec une intégration correspondante, le volet Refs a une section de pull requests (GitLab :
merge requests). Avec un profil d'hébergeur connecté, il a aussi une section TICKETS. Voir
[Hébergeurs et suivi de tickets](pull-requests-and-issues.md#pull-requests-et-tickets-dans-le-panneau-latéral).

| Touche | Action |
|---|---|
| `Enter` | Ouvrir la ligne sélectionnée : un ticket (et une demande sur Azure DevOps ou Bitbucket) comme [conversation](conversations.md), une pull request dans la [revue](review.md) sur les hébergeurs qui en ont une, tout le reste dans le navigateur. Sur une ligne `Reconnect` : ouvrir les Paramètres sur ce profil |
| `Space` | Déplier ou replier la section |
| `v` | Ouvrir le filtre de la section. Pull requests : Ouvertes, Mergées / fermées, Toutes. Tickets : Ouverts, Qui me sont assignés, Fermés, Tous |
| `n` | Section de pull requests : créer une pull request. Section TICKETS : nouveau ticket. (Le bouton `+` de l'en-tête) |
| `e` | Section de pull requests : modifier la demande sélectionnée chez l'hébergeur (le crayon de la ligne) |
| `Enter` sur la ligne de recherche | Taper dans le champ de recherche. `Enter`, `Tab` ou `Esc` le quitte ; `Ctrl+U` l'efface |
| `Enter` sur un en-tête de groupe | Replier ou déplier le groupe (*Mes pull requests*, *Assignées à moi*, *En attente de ma relecture*, *Toutes les pull requests*) |
| `Enter` sur la ligne du filtre, du fournisseur ou du dépôt | Ouvrir cette liste déroulante. La liste des fournisseurs se termine par **Aucun (délier ce dépôt)** |
| `.`, ou un clic droit | Le menu de la ligne : ouvrir la conversation, ouvrir dans le navigateur, copier l'URL ou le numéro, extraire la branche source (pull requests) ; créer une branche depuis le ticket (tickets) |

### Le tableau des tickets

**Tableau** dans l'en-tête TICKETS (palette : **Tickets : Ouvrir le tableau**) ouvre le tableau Kanban de la source de tickets. Voir
[le tableau](pull-requests-and-issues.md#le-tableau-des-tickets) et [les tickets locaux](local-issues.md#le-tableau-kanban). Chaque touche
ci-dessous est aussi une commande de palette (**Tableau : ...**).

| Touche | Action |
|---|---|
| Flèches | Sélectionner une carte (`Up`, `Down`) ou la colonne voisine (`Left`, `Right`) ; `Home`, `End`, `PageUp`, `PageDown` sautent dans la colonne |
| `Shift+Left`, `Shift+Right` | Déplacer la carte sélectionnée vers la colonne précédente ou suivante |
| `m` | **Déplacer la carte vers...** : une liste déroulante des colonnes |
| `Enter` | Ouvrir la [conversation](conversations.md) du ticket (la page de l'hébergeur quand il n'a rien à lire) |
| `e`, `.` | Modifier les assignés, étiquettes ou jalon ; le menu de la carte |
| `n` | Nouveau ticket dans la colonne sélectionnée |
| `a`, `l`, `M`, `/` | Filtrer par assigné, étiquette, jalon, texte. `x` efface les filtres |
| `c` | Les étiquettes qui forment les colonnes d'un tableau GitHub / GitLab |
| `r` | Relire les tickets |
| `Esc` | Fermer le tableau |

Souris : glissez une carte sur une autre colonne pour la déplacer, cliquez sur une carte pour la sélectionner, double-cliquez pour l'ouvrir,
clic droit pour le menu de la carte, molette pour faire défiler une colonne, `Shift`+molette (ou la barre sous le tableau) pour faire défiler les colonnes.

### Flow (workflows de branches)

Quand un workflow est défini pour le dépôt (voir les [workflows de branches](branching-workflows.md)), la barre d'outils a un
bouton **Flow** et la barre latérale groupe LOCALES et DISTANTES par type de branche. Chaque ligne du menu est une commande de palette
(**Flow : ...** ; **Flow : Démarrer Feature...** et **Flow : Nouveau ticket Feature...** sont générées pour chaque type de branche du
workflow). Les lignes pilotées par l'hôte (*Nouveau ticket <Type>*, *Reprendre une branche créée par l'hôte*), leurs fenêtres, la carte d'attente et la
carte « une branche est apparue » sont décrites dans les [workflows de branches](branching-workflows.md#flux-pilotés-par-lhôte-dans-lapplication) ;
`Enter` lance le bouton qui a le focus (*Créer le ticket*, *Reprendre*), `Esc` arrête l'attente ou ferme.

| Touche | Action |
|---|---|
| `Ctrl+G Shift+W` | Ouvrir le menu Flow (le bouton de la barre d'outils) |
| `Enter`, `Tab`, `Shift+Tab` | Dans la fenêtre Démarrer / Terminer / Synchroniser / Promouvoir / Définir le parent : aperçu ou exécution, champ suivant, champ précédent |
| `Space` | Cocher une case (*Publier*, *Exécuter les hooks*, *Mettre aussi à jour les branches empilées dessus*) |
| `Esc` | Fermer une liste déroulante, revenir de l'aperçu au formulaire, fermer la fenêtre (pas pendant une exécution) |
| Survoler une branche | Son type, son parent, et de combien elle est en retard sur le parent |

### L'éditeur Markdown et la vue de conversation

Chaque zone dont le texte part vers un hébergeur (un commentaire, une réponse, un résumé de revue, la description d'un nouveau ticket ou d'une pull request) est le même éditeur. Ses touches sont dans le tableau `md_editor` plus haut et peuvent être réaffectées.

| Touche | Action |
|---|---|
| `Alt+P` | Alterner **Écrire** et **Aperçu** (depuis **Côte à côte**, on passe à Aperçu) |
| `Alt+L` | Activer **Côte à côte**, ou revenir à Écrire (demande une fenêtre de 100 colonnes) |
| `Ctrl+Enter`, `Ctrl+S`, `Alt+Enter` | Envoyer le texte (le bouton du formulaire fait la même chose) |
| `Alt+E` | **Écrire avec l'IA** : ouvre ou ferme le volet sous la barre de modes (IA activée seulement) |
| `Alt+R` | Le compositeur de commits seulement : insérer une référence de ticket |
| `Alt+W` | Les formulaires de nouveau ticket et de pull request : créer sur la page web de l'hébergeur |
| `Esc` | Dans le volet de **Écrire avec l'IA** : arrêter un brouillon en cours, sinon fermer le volet |

Dans la [vue de conversation](conversations.md) d'un ticket ou d'une demande (lignes de palette *Conversation : ...* ; celles qui ont une touche sont dans la liste de la palette plus haut) :

| Touche | Action |
|---|---|
| `r` | Répondre à l'entrée sélectionnée (dans son fil, ou en citant son texte sur un hébergeur sans fils) |
| `Q` | Citer l'entrée sélectionnée dans un nouveau commentaire |
| `c` | Nouveau commentaire |
| `e`, `d` | Modifier, supprimer un de vos commentaires |
| `Alt+I` | Dans la zone de commentaire : **Note interne** (GitLab, Redmine) |
| `l`, `y`, `o` | Liens de l'entrée, copier son adresse, l'ouvrir dans le navigateur |
| `n` | Entrée non lue suivante |
| `v`, `m`, `F5` | Afficher ou masquer les événements, charger les entrées plus anciennes, actualiser |
| `g` | Ouvrir le diagramme de l'entrée (un bloc `mermaid`) |
| `Alt+Left`, `Backspace` | Revenir en arrière dans la pile des éléments ouverts depuis des références |
| `Tab` | Chronologie, zone de commentaire, boutons de l'en-tête |
| `q`, `Esc` | Fermer |

Dans le Radar, `i` ouvre la conversation d'une ligne de ticket.

Sur une ligne de pull request ou de ticket des sections de la barre latérale, `Enter` ouvre la conversation et `o` ouvre la ligne dans le navigateur.

## Volet Log

| Touche | Action |
|---|---|
| `Enter` / `l` | Entrer dans la liste des fichiers modifiés du commit sélectionné |
| `Enter` / `l` sur une ligne de fichier | Ouvrir le diff du fichier au centre |
| `Esc` / `h` | Revenir aux commits |
| `Right` | (commit avec plus d'une ref) Ouvrir la liste de ses autres refs : `Up` / `Down` déplacent, `Enter` extrait, `Esc` ferme |
| `f` | Ouvrir la barre de recherche (voir [Recherche](#recherche)) ; ce n'a jamais été `/`, la recherche du Log est donc inchangée |
| `/` | Mettre le focus sur le champ de filtre des Refs (la même touche le fait quand le volet Refs a le focus) ; tapé comme un caractère quand un champ de texte a le focus |
| `c` | (avec des modifications non commitées) Sélectionner la ligne WIP et placer le curseur dans son champ de message |
| `Enter` sur la ligne WIP | Placer le curseur dans le champ de message de la ligne WIP |
| `.`, `Menu`, `Shift+F10` | Le menu du commit. `Up` / `Down` déplacent, `Right` / `l` / `Enter` ouvrent un sous-menu, `Left` / `h` / `Esc` reviennent |
| `m` / `a` | Fichiers du volet de droite : `Chemins` ou `Arbre` / `Tous les fichiers` |
| `Shift+C` | Choisir une colonne du Log à déplacer ou à dimensionner (voir ci-dessous) |

Avec une colonne choisie, `Tab` / `Shift+Tab` en choisissent une autre, `<` / `>` la déplacent, `]` / `[` l'élargissent ou la rétrécissent, `0` rend à chaque largeur son réglage automatique et `Échap` ou `Enter` quittent (contexte `log_columns` dans le tableau généré ci-dessus).

Le champ de message de la ligne WIP est le résumé du compositeur de commits. Tant qu'il a
le curseur, chaque touche est tapée, mais les enchaînements `Ctrl` fonctionnent encore.
`Enter` passe à la description du compositeur. `Esc` quitte le champ et garde le texte. Voir
[Stage et commit](staging-and-committing.md).

### Sélectionner plusieurs commits

| Touche ou geste | Action |
|---|---|
| `Space` | Ajouter le commit courant à la sélection, ou le retirer |
| `Shift+Up` / `Shift+Down`, `Alt+Up` / `Alt+Down` | Étendre la sélection (Alt pour les terminaux qui gardent Shift pour eux) |
| `Ctrl+clic` / `Shift+clic` ou `Alt+clic` | Basculer un commit / sélectionner une plage |
| `Esc` | Effacer la sélection |
| `Shift+Y` | [Copier la sélection en gitGraph Mermaid](log-and-history.md#copier-des-commits-en-gitgraph-mermaid) (une fenêtre avec des niveaux de détail et un aperçu en direct ; `Enter` copie) |

Avec plusieurs commits sélectionnés, faites un clic droit sur l'un d'eux et choisissez
**Cherry-pick de N commits sur `<branche>`...**. Cela ouvre la même vue qu'un rebase
interactif.

### Nommer une branche ou un tag

`Ctrl+G b`, `Ctrl+G t`, les boutons **Branche** et **Tag** de la barre d'outils et les
entrées **Créer une branche ici...** / **Créer un tag ici...** des menus demandent le nom
dans une zone de texte sur la ligne du commit. `Enter` crée la ref. `Esc` annule. Un nom
invalide ou existant s'affiche en rouge, et la zone reste ouverte.

Quand cette ligne n'est pas à l'écran (un diff est ouvert, le Log est isolé, la colonne Ref
est masquée), le nom se saisit à la place dans une petite boîte encadrée, avec une croix de
fermeture et les boutons **Annuler** et **Créer**. Renommer une branche se fait directement
sur sa ligne dans le volet Refs, ou dans la cellule Ref de sa ligne du Log ; créer un fichier
se fait dans la liste des modifications, sous le dossier. Les mêmes touches valent partout :
`Enter` valide, `Esc` annule.

Rien n'est jamais saisi dans la ligne d'état. Les invites qui demandent plus de place sont
des fenêtres avec de vrais boutons : le message de stash, le message d'un tag annoté
(`Ctrl+Enter` ou `Ctrl+S` valide, `Enter` ajoute une ligne), Définir l'amont (une liste
filtrable de branches distantes, avec un bouton Retirer l'amont) et les motifs Git LFS à
suivre ou à ne plus suivre. Un réglage numérique ou texte choisi dans la palette ouvre les
Paramètres sur sa ligne, prêt à modifier, et le filtre des Refs (`/`) se saisit dans la zone
en haut du volet Refs.

### Modifier un message de commit

**Modifier le message du commit...** dans le menu du commit, `Ctrl+G e`, ou la commande de
palette `Git : Modifier le message du commit...` ouvre la fenêtre **Reformuler le message du
commit**. Voir [Travailler avec le Log](log-and-history.md) pour ce que cela change.

| Touche | Action |
|---|---|
| `Enter` dans le résumé | Aller à la description |
| `Enter` dans la description | Nouvelle ligne. `Up` sur la première ligne revient au résumé |
| `Ctrl+Enter`, `Ctrl+S`, ou **Mettre à jour le message** | Enregistrer le nouveau message |
| `Tab` / `Shift+Tab` | Passer entre le résumé, la description et les boutons |
| `Esc`, ou **Annuler** | Fermer sans changement |

### Modifier un commit

`Ctrl+G Shift+E` (tous les champs) et `Ctrl+G Shift+A` (l'auteur) ouvrent la fenêtre de modification, tout comme le menu du commit, les lignes `Commit : Modifier...` de la palette et le volet de détail du commit ; chaque entrée est décrite dans [Modifier un commit](log-and-history.md#modifier-un-commit-message-auteur-dates). Les deux raccourcis sont réaffectables : `git_prefix.amend` et `git_prefix.edit_author`. Touches dans la fenêtre :

| Touche | Action |
|---|---|
| `Tab` / `Shift+Tab` | Passer d'un champ ou d'un bouton à l'autre |
| `Enter` dans un champ d'une seule ligne | Champ suivant ; après le dernier, **Appliquer** |
| `Up` / `Down` | Champ précédent / suivant (dans la description, depuis sa première / dernière ligne) |
| `Ctrl+Enter`, `Ctrl+S`, ou **Appliquer** | Appliquer les champs modifiés |
| `Esc`, ou **Annuler** | Fermer sans changement |
| `Alt+H`, ou **Modifier l'historique en profondeur...** | Fermer la fenêtre et ouvrir l'atelier de réécriture d'historique à partir de ce commit (builds avec la réécriture d'historique seulement) ; demande d'abord si un champ a été modifié (`Esc` confirme). Réaffectable : `reword.deeper` |

### Créer un worktree

**Créer un worktree depuis `<ref>`...** (menus de branche et de tag) et **Créer un
worktree ici...** (menu du commit) ouvrent une boîte : le dossier, une nouvelle branche
facultative, et **Ouvrir le nouveau worktree dans un onglet**. Touches : `Tab` /
`Shift+Tab` déplacent, `Enter` crée, `Space` bascule la case, `Ctrl+B` parcourt, `Esc`
annule.

### Rebase interactif

**Rebase interactif depuis ici...** dans le menu du commit, ou
`Git : Rebase interactif depuis le commit sélectionné...`, ouvre la vue du rebase. Un
cherry-pick de plusieurs commits utilise la même vue. Voir
[Travailler avec le Log](log-and-history.md).

| Touche | Action |
|---|---|
| `p` / `r` / `s` / `f` / `d` / `e` | Régler l'action de la ligne : Pick / Reword / Squash / Fixup / Drop / Edit (Fixup et Edit seulement dans un rebase) |
| `Space` | Ouvrir la liste d'actions de la ligne (`Up` / `Down` choisissent, `Enter` applique, `Esc` ferme la liste) |
| `Up` / `Down` | Déplacer la sélection |
| `Ctrl+Up` / `Ctrl+Down`, `Alt+Up` / `Alt+Down` | Monter / descendre la ligne sélectionnée |
| `a` | Aperçu de l'autosquash activé ou non |
| `Enter` sur une ligne Reword | Modifier le nouveau message de ce commit |
| `Ctrl+S`, `Ctrl+Enter`, ou **Lancer le rebase** / **Lancer le cherry-pick** | Lancer le plan |
| `Ctrl+R`, ou **Réinitialiser** | Restaurer le plan tel qu'il était à l'ouverture de la vue |
| `Esc`, ou **Annuler** | Fermer sans changement |

Quand un rebase s'arrête (sur un conflit ou une étape Edit), un bandeau au-dessus du Log
propose **Continuer**, **Passer** et **Annuler**. La palette a `Rebase : Continuer`,
`Rebase : Passer` et `Rebase : Abandonner`.

## Recherche

`f`, le bouton **Chercher** de la barre d'outils, ou `Navigation : Rechercher des commits...`
ouvre la barre de recherche au-dessus du Log. Elle cherche dans tout le message de commit,
le nom et l'e-mail de l'auteur, le SHA et les noms de refs. Elle ne masque pas de lignes :
les lignes qui ne correspondent pas sont estompées.

| Touche | Action |
|---|---|
| *(taper)* | Modifier le texte de recherche (chaque touche est tapée) |
| `Enter` / `Down`, ou le `↑` de la barre | Correspondance suivante |
| `Shift+Enter` / `Up`, ou le `↓` de la barre | Correspondance précédente |
| `Esc`, ou le `✕` de la barre | Fermer la barre de recherche |

## Vue diff

Un fichier modifié s'ouvre au centre, à la place du Log. Voir
[Stage et commit](staging-and-committing.md).

| Touche | Action |
|---|---|
| `Enter` / `l`, ou un double-clic (sur une ligne de fichier) | Ouvrir le diff du fichier |
| `Esc`, `q`, ou le `✕` du titre | Fermer le diff |
| `j` / `k`, `Up` / `Down` | Défiler |
| `PageUp` / `PageDown`, `Home` / `End` | Défiler d'une page ; aller au début / à la fin |
| `h` / `l`, `Left` / `Right` | Défiler latéralement (pas avec le retour à la ligne) |
| `n` / `p` | Hunk suivant / précédent |
| `m` | Disposition en ligne ou côte à côte |
| `Shift+H` | Hunks seulement ou fichier entier |
| `w` | Retour à la ligne activé ou non |
| `s` | Afficher ou masquer les modifications d'espaces seuls |
| `Shift+M` | Minimap activée ou non |
| `Shift+C` | Choisir l'encodage de lecture du fichier (une liste : Détection automatique d'abord) |
| `e` | Modifier le fichier dans l'éditeur intégré (voir [Éditeur](#éditeur)) |
| `r` | (fichier Git LFS) La carte LFS ou le pointeur brut. Voir [Git LFS](lfs.md) |
| `Shift+E` | (fichier en conflit) Ouvrir l'éditeur de merge. Voir [Résoudre les conflits de merge](merge-conflicts.md) |
| `y` / `Ctrl+C` | Copier le texte sélectionné. Sans sélection : la ligne sous le curseur, ou les lignes sélectionnées |
| `.`, ou un clic droit sur un hunk | Le menu du hunk |

### Stager et abandonner depuis le diff

Ces touches ne marchent que sur un diff des modifications locales (ouvert depuis la liste
Unstaged ou Staged). L'abandon ne marche que sur la liste Unstaged.

| Touche | Action |
|---|---|
| `Space` | Stager (Unstaged) / unstager (Staged) le hunk sous le curseur |
| `v` | Démarrer ou abandonner une sélection de lignes ; `j` / `k` l'étendent |
| `Enter` | Stager / unstager les lignes sélectionnées, ou la ligne sous le curseur |
| `x` | Abandonner le hunk sous le curseur (demande d'abord : `y` / `Enter` confirment, `n` / `Esc` annulent) |
| `Shift+X` | Abandonner les lignes sélectionnées, ou la ligne sous le curseur (demande d'abord) |

Seules les lignes modifiées comptent : les lignes de contexte d'une sélection sont
ignorées. Une sélection qui déborde sur un second hunk n'agit que sur le premier hunk.

## Vue image

Une image dans la vue Diff ou la vue Fichier. Les touches sont dans le contexte `image` du
tableau ci-dessus. Voir [Images](image-view.md).

## Vue Fichier, Blame et Historique du fichier

| Touche | Action |
|---|---|
| `Esc` / `q` | Fermer (avec une sélection de texte, `Esc` l'efface d'abord) |
| `j` / `k`, `Up` / `Down` | Déplacer la sélection |
| `PageUp` / `PageDown` | Défiler d'une page |
| `Home` / `End`, `Shift+G` | Première / dernière ligne |
| `h` / `l`, `Left` / `Right` | Faire défiler le texte latéralement |
| `Enter` | (Blame, Historique du fichier) Aller au commit de la ligne dans le Log |
| `e` | Modifier le fichier dans l'éditeur intégré |
| `y` / `Ctrl+C` | Copier le texte sélectionné. Sans sélection : la ligne sélectionnée (Blame : le code seulement ; Historique du fichier : le résumé) |

Le Blame et l'historique du fichier ont leurs propres touches, listées dans le tableau généré ci-dessus (contextes `file_insight` et `history`) et sous `Blame : ...` / `Historique du fichier : ...` dans la palette ; toutes peuvent être remappées. Détails dans [Blame](log-and-history.md#blame) et [Historique du fichier](log-and-history.md#historique-du-fichier). Dans la liste de l'historique, les lettres seules (`d` `o` `b` `c` `y`) agissent sur la ligne sélectionnée, mais un accord Ctrl (`Ctrl+C`, `Ctrl+P`, `Ctrl+G ...`) garde toujours son sens global.

## Colonne des commits

La colonne à gauche de la vue d'un fichier (voir [La colonne des commits](interface-tour.md#la-colonne-des-commits)). `Alt+C` l'affiche ou la masque depuis n'importe où ; les autres touches marchent une fois qu'elle a le clavier (`Ctrl+N` puis son badge, ou `Affichage : Placer le clavier sur la colonne des commits`).

| Touche | Action |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Choisir un commit |
| `Enter` / `Right` | Passer à la liste des fichiers du commit choisi |
| `Esc` | Rendre le clavier à la vue |
| `.`, `Menu`, `Shift+F10` | Le menu de la ligne choisie |

Ses autres touches (`Shift+Left` / `Shift+Right` pour défiler sur le côté, `Tab`, `]`, `[` et `0` pour dimensionner les parties) sont dans le tableau généré plus haut, contexte `commit_column`.

## Éditeur

**Éditer**, ou `e` dans un diff ou une vue Fichier, ouvre l'éditeur intégré sur la copie
du dossier de travail du fichier. Voir [Stage et commit](staging-and-committing.md).

| Touche | Action |
|---|---|
| *(taper)* | Insérer au niveau du curseur (remplace la sélection) |
| `Enter` | Nouvelle ligne, avec l'indentation de la ligne courante |
| `Tab` / `Shift+Tab` | Indenter / désindenter (les lignes sélectionnées, s'il y en a plusieurs) |
| flèches, `Home` / `End` | Se déplacer. `Home` va au premier caractère non blanc, puis à la colonne 0. `Shift` étend la sélection |
| `Ctrl+Left` / `Ctrl+Right` | Avancer d'un mot |
| `Ctrl+Home` / `Ctrl+End` | Début / fin du fichier |
| `PageUp` / `PageDown` | Une page vers le haut / le bas |
| `Ctrl+A` | Tout sélectionner |
| `Ctrl+C` / `Ctrl+X` / `Ctrl+V` | Copier / couper / coller |
| `Ctrl+U` / `Ctrl+K` | Supprimer jusqu'au début / à la fin de la ligne |
| `Ctrl+Z` | Annuler |
| `Ctrl+Y` / `Ctrl+Shift+Z` | Rétablir |
| `Ctrl+S` | Enregistrer |
| `Ctrl+F` / `F3` | Chercher. Dans le champ de recherche : `Enter` / `Down` / `F3` suivant, `Shift+Enter` / `Up` précédent, `Esc` ferme |
| `Alt+M` | Minimap activée ou non (**Map**) |
| `Esc` | Fermer. Avec des modifications non enregistrées, il demande : **Enregistrer** (`S` / `Enter`), **Abandonner** (`D`), **Annuler** (`C` / `Esc`) |

`Tab` indente dans l'éditeur, il ne déplace donc pas le focus. Les autres enchaînements
`Ctrl` (`Ctrl+P`, `Ctrl+N`) fonctionnent encore.

L'**encodage**, les **fins de ligne** et l'**indentation** de la ligne d'état sont des
boutons : chacun ouvre une liste déroulante (flèches ou molette, taper pour filtrer,
`Enter` pour choisir, `Esc` pour fermer). La palette (`Ctrl+P`) ouvre les mêmes
menus : *Éditeur : Changer l'encodage*, *Éditeur : Rouvrir avec l'encodage*, *Éditeur : Convertir vers l'encodage*, *Éditeur : Convertir les fins de ligne*, *Éditeur : Changer l'indentation*. Détails dans [Stage et commit](staging-and-committing.md).

## Actions git (`Ctrl+G`)

`Ctrl+G` attend une touche de plus :

| Touche | Action |
|---|---|
| `s` / `u` | Stager / unstager le fichier sélectionné |
| `a` | Stager toutes les modifications |
| `x` | Abandonner toutes les modifications (demande d'abord) |
| `c` | Ouvrir le compositeur de commits |
| `Enter` | Commiter tout de suite les modifications stagées, avec le message du formulaire, d'où que vous soyez (voir [Stage et commit](staging-and-committing.md#écrire-le-commit)) |
| `m` | Stasher toutes les modifications avec un message, saisi dans une petite fenêtre (vide autorisé). Avec l'IA activée, son bouton IA (`Ctrl+G` dans la fenêtre) écrit le message |
| `Shift+T` | Stasher toutes les modifications (fichiers non suivis compris), sans message |
| `o` | Pop d'un stash : la ligne de stash sélectionnée dans le Log, sinon le dernier stash |
| `p` | Pull, comme le bouton Pull est réglé (`git.pull_mode`) |
| `Shift+P` | Push vers le distant par défaut (définit l'amont au premier push), ou demander le distant |
| `Shift+U` | Pousser tous les tags |
| `f` | Fetch de chaque distant, avec prune |
| `g` | Ouvrir dans le navigateur la pull request de la branche extraite |
| `b` / `t` | Nouvelle branche / nouveau tag sur HEAD (voir [Nommer une branche ou un tag](#nommer-une-branche-ou-un-tag)) |
| `d` | Supprimer la branche ou le tag sélectionné (demande d'abord) |
| `e` | Modifier le message du commit sélectionné |
| `Shift+E` / `Shift+A` | Modifier le commit sélectionné (tous les champs) / son auteur (voir [Modifier un commit](#modifier-un-commit)) |
| `y` | Cherry-pick du commit sélectionné |
| `v` | Revert du commit sélectionné |
| `Shift+R` | Reset (mixed) au commit sélectionné |
| `r` | Rebase sur la branche sélectionnée |
| `Shift+W` | Ouvrir le menu Flow (dans un dépôt qui a un workflow de branches) |
| `Shift+H` | Ouvrir l'atelier de réécriture d'historique (builds avec la réécriture d'historique seulement) |
| `l` | Ouvrir le menu LFS. `Shift+F` / `Shift+K` / `Shift+L` / `Shift+J` / `Shift+V` : LFS fetch de tout / checkout de tout / pull / push de tout / prune. Voir [Git LFS](lfs.md) |
| `i` | ✦ IA : écrire le message de commit pour les modifications stagées |
| `w` | ✦ IA : expliquer le commit sélectionné, ou les modifications locales |
| `Shift+C` | ✦ IA : proposer comment répartir les modifications locales en commits |

Les touches IA ne marchent que lorsque l'IA est activée (voir [Assistance IA](ai.md)).
`Esc` annule une requête IA en cours.

## Volet de droite (modifications locales et détails du commit)

Avec la ligne WIP sélectionnée, le volet de droite montre les listes **Unstaged** et
**Staged** et le compositeur de commits. Avec un commit sélectionné, il montre les détails
du commit et ses fichiers.

| Touche | Action |
|---|---|
| `j` / `k`, `Up` / `Down` | Se déplacer. À la fin d'une liste, la sélection passe dans l'autre |
| `h` / `l`, `Left` / `Right` | Aller à la liste Unstaged / Staged |
| `Enter` | Replier ou déplier la liste active |
| `Space` / `Insert` | Ajouter le fichier courant à la sélection, ou l'en retirer |
| `Ctrl+A` | Sélectionner tous les fichiers de la liste |
| `Ctrl+Space` | Stager / unstager le fichier sélectionné, ou replier une ligne de dossier |
| `d` | Ouvrir le diff du fichier sélectionné |
| `m` | `Chemins` (liste plate) ou `Arbre` (dossiers) |
| `a` | `Tous les fichiers` : tout l'arbre, pas seulement les fichiers modifiés |
| `z` | Replier tous les dossiers, ou les déplier à nouveau |
| `/` | Filtrer par nom les fichiers modifiés d'un commit |
| `.`, `Menu`, `Shift+F10` | Le menu du fichier |
| `c` | Cibler le champ de résumé du compositeur de commits |
| `Shift+A` | **Amender le commit précédent** activé ou non |
| `Ctrl+N ]` / `Ctrl+N [` | Agrandir / réduire le bloc du message de commit |

Dans le compositeur de commits :

| Touche | Action |
|---|---|
| `Enter` | Résumé : aller à la description. Description : nouvelle ligne |
| `Ctrl+Enter`, `Alt+Enter`, `Ctrl+J` | Commiter (`Ctrl+G Enter` le fait depuis n'importe où) |
| `Alt+R` | Insérer une référence de ticket |
| `Esc` | Quitter le compositeur |

Le menu du fichier contient : Stager ou Unstager, Abandonner les modifications, Ignorer, Stash
du fichier, Historique du fichier, Blame du fichier, Ouvrir dans l'outil de diff externe,
Ouvrir dans l'éditeur externe, Ouvrir le fichier dans le programme par défaut, Afficher dans
le dossier, Copier le chemin du fichier, Créer un patch depuis les modifications du fichier,
Modifier le fichier et Supprimer le fichier. Sur un fichier en conflit, il commence par
**Ouvrir dans l'éditeur de merge** et les trois lignes **Résoudre avec**. Avec Git LFS activé,
il a des lignes LFS. **Abandonner les modifications** et **Supprimer le fichier** demandent
d'abord.

## Boutons de la fenêtre

Chaque bouton a son jumeau au clavier :

| Bouton | Touche |
|---|---|
| <img src="../../../assets/logo/gitpoulpi.png" alt="Icône GitPoulpi" height="16"> à gauche de la barre de titre (À propos) | `Aide : À propos de GitPoulpi` dans la palette |
| **Ouvrir dans** dans la barre de titre | `Alt+O`, `Ctrl+Alt+O` |
| `×` à droite de la barre de titre (quitter) | `q` |
| La version dans la barre de titre (Notes de version) | `Aide : Notes de version` dans la palette |
| `⌕ Recherche globale` dans la barre de titre, à gauche du `⚙` | `Ctrl+Alt+F` |
| `⚙` dans la barre de titre (Paramètres) | `Ctrl+,` |
| La pastille d'espace de travail dans la barre d'onglets | `Alt+W` |
| `+` après le dernier onglet | `Alt+T` |
| L'onglet **Radar** | `Alt+R` |
| L'onglet **Espaces de travail** | `Alt+J` |
| **Activité** dans la ligne d'état | `Ctrl+N Shift+A` |
| Barre d'outils : **Annuler**, **Rétablir** | `Ctrl+Z`, `Ctrl+Y` |
| Barre d'outils : **Pull** / son `▾` | `Ctrl+G p` / `Git : Options de pull...` |
| **Pull de toutes les branches** (entrée du `▾` de Pull) | `Ctrl+G Ctrl+P` |
| Barre d'outils : **Push** / son `▾` | `Ctrl+G Shift+P` / `Git : Push vers un distant...` |
| Barre d'outils : **Branche**, **Tag** | `Ctrl+G b`, `Ctrl+G t` |
| Barre d'outils : **Stash**, **Pop** | `Ctrl+G Shift+T`, `Ctrl+G o` |
| Barre d'outils : **LFS** (quand LFS est activé) | `Ctrl+G l` |
| Barre d'outils : **Flow** (quand le dépôt a un workflow) | `Ctrl+G Shift+W` |
| Barre d'outils : **Chercher** | `f` |
| Barre d'outils : la pull request liée (`GH #12`, `GL !53`) | `Ctrl+G g` |
| Barre d'outils : la liste de profils (avec deux profils ou plus) | `Dépôt : changer de profil...` |

La barre d'outils est décrite dans la [Visite de l'interface](interface-tour.md#barre-doutils).

## Paramètres

`Ctrl+,`, `Ctrl+N ,`, l'engrenage `⚙`, ou `Application : Paramètres...` ouvre
l'écran des Paramètres.

| Touche | Action |
|---|---|
| `Tab` / `Shift+Tab`, `h` / `l`, `Left` / `Right` | Passer de la liste des pages aux options |
| `j` / `k`, `Up` / `Down` | Déplacer la sélection |
| `Enter` / `Space` | Ouvrir une page, basculer une case, modifier un champ, ouvrir une liste déroulante, ou appuyer sur un bouton |
| `/`, `Ctrl+F` | Chercher dans toutes les options |
| `Alt+Left` / `Alt+Right` | Changer de sous-onglet (Général / la liste) |
| `Esc` / `q` | Fermer les Paramètres. Pendant la modification d'un champ, `Esc` n'annule que la modification |
| `?` | Ouvrir l'aide par-dessus l'écran des Paramètres |

Voir [Paramètres](settings.md).

## Champs de texte

Tous les champs de texte partagent les mêmes touches : filtres, champs de recherche, palette
de commandes, saisies, compositeur de commits, champ de la ligne WIP, champs de boîtes de
dialogue et champs des Paramètres. Tant qu'un champ a le clavier, ces touches l'emportent sur
les raccourcis du volet.

| Touche | Action |
|---|---|
| un caractère | Le taper au niveau du curseur (remplace la sélection) |
| `Left` / `Right` | Avancer d'un caractère (avec une sélection : aller à son début / sa fin) |
| `Ctrl+Left` / `Ctrl+Right` | Avancer d'un mot |
| `Home` / `End` | Début / fin de la ligne |
| `Shift` + un déplacement ci-dessus | Étendre ou réduire la sélection |
| `Up` / `Down` (description seulement) | Ligne précédente / suivante |
| `Ctrl+Home` / `Ctrl+End` (description seulement) | Début / fin du texte |
| `Ctrl+A` | Tout sélectionner |
| `Backspace` / `Delete` | Supprimer le caractère avant / après le curseur, ou la sélection |
| `Ctrl+U` | Supprimer du curseur au début de la ligne (ou la sélection) |
| `Ctrl+K` | Supprimer du curseur à la fin de la ligne (ou la sélection) |
| `Ctrl+C` / `Ctrl+X` | Copier / couper la sélection (pas dans un champ de mot de passe) |
| coller (le collage propre au terminal) | Insérer au niveau du curseur, en remplaçant la sélection |

Dans la description du commit, `Enter` tape une nouvelle ligne et `Ctrl+Enter` commite.
Les terminaux n'envoient pas tous `Ctrl+Enter` de la même façon, donc `Alt+Enter` et
`Ctrl+J` commitent aussi.

Certaines consoles envoient `Ctrl+lettre` comme un caractère de contrôle (`Ctrl+A` comme
`0x01`). GitPoulpi comprend les deux formes. Les caractères `AltGr` (`@`, `#`, `€` sur les
claviers suisses ou français) sont tapés.

## Infobulles

Une infobulle apparaît quand le pointeur reste environ 0,6 s sur une cible (une cellule du
Log, un bouton de la barre d'outils, la pull request liée). Un appui de touche, un clic, un
défilement, un redimensionnement, une fenêtre ou un menu ouvert, ou le pointeur qui quitte
la cible la masque. Ensuite, elle ne revient que lorsque le pointeur bouge à nouveau.
