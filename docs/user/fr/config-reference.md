[English](../config-reference.md) · [Čeština](../cs/config-reference.md) · [Deutsch](../de/config-reference.md) · [Español](../es/config-reference.md) · **Français** · [Italiano](../it/config-reference.md) · [日本語](../ja/config-reference.md) · [한국어](../ko/config-reference.md) · [Nederlands](../nl/config-reference.md) · [Polski](../pl/config-reference.md) · [Português (Brasil)](../pt-BR/config-reference.md) · [Português (Portugal)](../pt-PT/config-reference.md) · [Русский](../ru/config-reference.md) · [Svenska](../sv/config-reference.md) · [Türkçe](../tr/config-reference.md) · [Українська](../uk/config-reference.md) · [简体中文](../zh-CN/config-reference.md)

# Référence de configuration

Chaque clé de `config.toml`, sa valeur par défaut et ce qu'elle fait. Vous avez rarement
besoin de modifier le fichier à la main : l'[écran des Paramètres](settings.md) (`Ctrl+,`)
l'écrit au fur et à mesure de vos changements, et `gitpoulpi config set` aussi.

## Où il se trouve, et comment une valeur est choisie

| | |
|---|---|
| Fichier | `%APPDATA%\GitPoulpi\config.toml` (Windows), `~/.config/gitpoulpi/config.toml` (Linux), `~/Library/Application Support/GitPoulpi/config.toml` (macOS). `gitpoulpi config path` l'affiche. |
| Un autre dossier | `GITPOULPI_CONFIG_DIR=<dir>` déplace tout le dossier de configuration (le fichier, `sessions/`, les fichiers de port du démon) : une installation portable, un test, ou un agent tenu à l'écart de vos propres paramètres. |
| Priorité | valeur par défaut intégrée, puis `config.toml`, puis la variable `GITPOULPI_*`, puis une option de ligne de commande. La plus haute l'emporte. Une valeur erronée (un choix mal orthographié dans le fichier, une variable incorrecte) est une erreur, jamais ignorée en silence. |
| Variables | Mettez la clé en majuscules et remplacez les points par des tirets bas : `general.auto_prune` devient `GITPOULPI_GENERAL_AUTO_PRUNE`. Seulement pour les clés que GitPoulpi lit réellement (pas celles marquées *non utilisées* plus bas, ni les listes comme `ui.log_columns_hidden`). |
| Options | `--backend git2\|system` remplace `general.default_backend` pour un démon que cette commande démarre. |
| Jetons et clés | Jamais dans ce fichier. Les jetons d'hébergeurs et les clés d'API de l'IA vont dans l'assistant d'identifiants de votre système (Git Credential Manager, libsecret / `pass`, Keychain) ; le fichier ne contient qu'une référence (`token_ref`). Sans assistant, l'enregistrement d'un jeton échoue avec un message et le jeton ne dure que pour la session. |

```
gitpoulpi config path                          # the file, and whether it exists yet
gitpoulpi config get general.auto_stash        # the value, the layer it came from (default/file/env/flag), and whether anything reads it
gitpoulpi --json config get ui                 # a whole section
gitpoulpi config set general.auto_stash always # checked against the schema, then written
```

`config set` garde le fichier clairsemé (seulement les clés que quelqu'un a réglées) et
conserve les clés dont il n'a pas été question, mais supprime les commentaires. Il refuse
les listes et les tables entières, sauf `keybindings.*` (plusieurs enchaînements séparés
par des espaces ; vide dissocie). Modifiez `[[profiles]]`, `[[integrations]]` et
`[[ai.profiles]]` dans les Paramètres ou à la main.

Dans les tableaux, **Redémarrage** signifie que la valeur est lue au démarrage ; tout le
reste s'applique aussitôt (ou à la prochaine actualisation, au prochain fetch ou à la
fermeture des Paramètres).

## `[general]`

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `general.default_backend` | `"git2"` | `git2` (libgit2, avec repli sur le programme `git` pour le stage, le merge, le rebase, le stash et le réseau) ou `system` (le `git` installé seulement). Un démon en cours garde son moteur : **redémarrez-le** (`gitpoulpi daemon restart`). |
| `general.auto_stash` | `"ask"` | Quand des modifications locales bloquent un checkout, un pull, un merge ou un rebase : `ask` (une boîte de dialogue dans la TUI ; la CLI et MCP échouent et nomment `--autostash`), `always` (stash, exécution, pop) ou `never` (refuser). Voir [Auto-stash](log-and-history.md#auto-stash). |
| `general.include_untracked_in_autostash` | `false` | L'auto-stash prend aussi les fichiers non suivis. |
| `general.initial_commits_in_graph` | `5000` | Commits que le Log charge d'abord (au moins 500). Plus élevé coûte de la mémoire et du temps de dessin sur les énormes dépôts. |
| `general.lazy_load_commits_in_graph` | `true` | Charge les commits plus anciens quand vous défilez près de la fin. Désactivé : le Log s'arrête au nombre initial. |
| `general.commits_per_lazy_page` | `1000` | Commits par page en arrière-plan (au moins 50). |
| `general.max_commits_in_graph` | `100000` | Où s'arrête le chargement progressif (environ 300 octets par commit) ; `0` = aucune limite. |
| `general.auto_fetch_interval_minutes` | `5` | Fetch All en arrière-plan toutes les N minutes, 0 à 60 ; `0` = désactivé. **Redémarrage.** |
| `general.auto_fetch_interval_unit` | `"auto"` | L'unité dans laquelle les Paramètres affichent l'intervalle : `minutes`, `hours` ou `auto`. Affichage seulement : l'intervalle est toujours de `auto_fetch_interval_minutes` minutes. `auto` (et une configuration sans cette clé) prend la plus grande unité qui divise exactement les minutes (60 s'affiche « 1 heure », 45 « 45 minutes ») ; une unité qui ne convient pas est ignorée. Les Paramètres écrivent les deux clés ; un GitPoulpi plus ancien ignore l'unité et lit toujours les minutes. |
| `general.fetch_on_tab_open` | `true` | Fetch All à l'ouverture d'un onglet (sauté si GitPoulpi a fait un fetch de ce dépôt il y a moins de 60 s). |
| `general.auto_prune` | `true` | Les fetchs élaguent aussi les branches de suivi distant disparues. |
| `general.auto_prune_tags` | `false` | Un fetch supprime aussi les tags locaux que le remote a supprimés (seulement ceux qui y étaient ; un tag que vous n'avez jamais poussé est conservé). Git 2.41+. |
| `general.hosting_refresh_minutes` | `5` | Actualisation des listes de pull requests / tickets et de la pastille de la barre d'outils, 0 à 120 ; `0` = seulement à la demande et sur événement. |
| `general.hosting_refresh_unit` | `"auto"` | L'unité dans laquelle les Paramètres affichent l'actualisation : `minutes`, `hours` ou `auto`, lue comme `general.auto_fetch_interval_unit`. Affichage seulement. |
| `general.keep_submodules_up_to_date` | `false` | Après un checkout ou un pull, `git submodule update --init --recursive` quand un sous-module n'est plus aligné. Voir [Sous-modules](submodules.md). |
| `general.default_branch_name` | `"main"` | Première branche d'un dépôt créé avec Init ; vide laisse git décider. Ne renomme rien. |
| `general.delete_orig_files_after_merge` | `false` | Après un merge abouti, supprime les fichiers de sauvegarde `*.orig`. |
| `general.remember_tabs` | `true` | Rouvre les onglets de la dernière session. Désactivé, un lancement dans un dépôt n'ouvre que ce dépôt, et un lancement hors d'un dépôt n'ouvre que l'Accueil. **Redémarrage.** |
| `general.radar_enabled` | `true` | Le [Radar](radar.md) : son onglet, ses commandes de palette, `Alt+R` et sa récupération en arrière-plan. Désactivé, tout cela est masqué et rien n'est récupéré pour lui. S'applique immédiatement ; absent d'un build sans la fonctionnalité `radar`. |
| `general.startup_tab_in_repo` | `"current-repo"` | L'onglet sur lequel s'ouvre un démarrage lorsque le dossier de lancement se trouve dans un dépôt: `"current-repo"` (l'onglet de ce dépôt, ouvert si besoin), `"last"` (l'onglet actif à la dernière fermeture), `"home"` ou `"radar"`. Un dépôt donné avec `--repo` ouvre toujours son propre onglet. `"last"` vaut Accueil avec `remember_tabs` désactivé ou si rien n'est mémorisé ; `"radar"` vaut Accueil si le Radar est désactivé ou absent du build. Lu une seule fois au démarrage ; demande un redémarrage. |
| `general.startup_tab_outside_repo` | `"home"` | L'onglet sur lequel s'ouvre un démarrage lorsque le dossier de lancement n'est pas un dépôt (un dossier personnel, par exemple): `"home"`, `"last"` ou `"radar"`. `"last"` vaut Accueil avec `remember_tabs` désactivé ou si rien n'est mémorisé ; `"radar"` vaut Accueil si le Radar est désactivé ou absent du build. Lu une seule fois au démarrage ; demande un redémarrage. |
| `general.startup_tab_launcher` | `"last"` | L'onglet sur lequel s'ouvre un démarrage lorsque GitPoulpi est lancé depuis le raccourci du bureau ou du menu Démarrer qu'il a créé (il passe `--launcher`), quel que soit le dossier: `"home"`, `"last"` ou `"radar"`. Prioritaire sur les deux réglages ci-dessus. `"last"` vaut Accueil avec `remember_tabs` désactivé ou si rien n'est mémorisé ; `"radar"` vaut Accueil si le Radar est désactivé ou absent du build. Lu une seule fois au démarrage ; demande un redémarrage. |
| `general.group_worktree_tabs` | `true` | Garde les onglets des worktrees liés d'un dépôt juste après l'onglet de ce dépôt : à l'ouverture d'un worktree, quand un onglet de worktree est déplacé ailleurs (il revient se coller à son groupe ; l'onglet du dépôt est la tête du groupe et ses worktrees le suivent) et quand le paramètre est activé. Désactivé, les onglets restent où vous les mettez. |
| `general.undo_depth` | `100` | Combien d'opérations l'annulation peut remonter ; `0` = illimité. |
| `general.conflict_detection` | `true` | Détection proactive des conflits : un badge sur le Log quand fusionner la branche courante dans la branche dont elle est partie provoquerait un conflit, calculé en arrière-plan (un merge en mémoire : rien ne change dans vos fichiers, l'index ni les refs). Un dépôt peut la désactiver pour lui-même (`[preferences] conflict_detection`). Les branches de base qu'elle compare ont leurs propres listes : [`[conflict_prevention]`](#conflict_prevention-listes-de-branches-de-base). Voir [Résoudre les conflits de merge](merge-conflicts.md#prévention-des-conflits). |
| `general.dev_mode` | absent (automatique) | Pour qui compile GitPoulpi depuis un dépôt cloné ; jamais affiché dans les Paramètres. Absent : activé quand l'exécutable en cours se trouve dans `<checkout>/target/<profile>/` (`Cargo.toml` et `crates/gitpoulpi-tui` à côté de `target`). `true` / `false` le force. Le mode développeur ne débloque que `ui.launcher_build`. |
| `general.journal_outside` | `true` | Enregistre ce qu'un shell ou un agent IA fait aux branches, aux tags, aux stashes et à HEAD comme des entrées de l'historique d'annulation, marquées extérieures, pour que `Ctrl+Z` puisse les reprendre. Les modifications de fichiers ne sont jamais capturées. Nécessite `undo_enabled`. Voir [Annuler et rétablir](undo-redo.md#changements-faits-en-dehors-de-gitpoulpi). |
| `general.undo_enabled` | `true` | `false` : aucune comptabilité d'annulation (le plus rapide), l'historique d'annulation est effacé, les boutons Annuler/Rétablir disparaissent. Voir [Annuler et rétablir](undo-redo.md). |
| `general.active_profile` | `""` | Identifiant de l'identité `[[profiles]]` avec laquelle les commits sont faits. Vide, ou un profil dont le nom ou l'e-mail est vide : `user.name` / `user.email` de git. |
| `general.external_editor` | `""` | Commande de l'élément *Ouvrir dans l'éditeur* d'un fichier, p. ex. `code -r {file}`. Vide : VS Code s'il est installé, sinon le défaut du système. |
| `general.external_diff_tool` | `""` | Commande pour *Ouvrir dans l'outil de diff externe* : `{local}` (version commitée / HEAD), `{remote}` (version de l'arbre de travail / de l'index). Vide : `git difftool --no-prompt`. |
| `general.external_merge_tool` | `""` | Commande pour *Ouvrir dans l'outil de merge externe* sur une carte de conflit (un fichier binaire, LFS ou trop gros) : `{base}`, `{local}` (A, courant), `{remote}` (B, entrant) et `{merged}` (le fichier où l'outil écrit le résultat ; GitPoulpi le stage quand l'outil se termine avec succès). Vide : `git mergetool --no-prompt`. |
| `general.file_explorer` | selon l'OS | Commande pour *Afficher dans le dossier* : `{path}` (le fichier), `{dir}` (son dossier). Défauts : `explorer.exe /select,"{path}"`, `open -R "{path}"`, `xdg-open "{dir}"`. |

Les trois modèles de commande sont d'abord découpés en arguments (`"..."` regroupe un
mot), puis les marqueurs sont remplis, si bien que les chemins avec des espaces restent un
seul argument. Aucun shell n'intervient.

```toml
[[profiles]]                # commit identities (Settings > Profiles)
id = "work"
name = "Work"
author_name = "Jane Doe"
author_email = "jane@work.example"
```

## `[ui]`

**Log et graphe**

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `ui.graph_renderer` | `"auto"` | `auto` (une image là où le terminal peut en dessiner une, des caractères ailleurs), `image`, `text`. **Redémarrage.** |
| `ui.image_protocol` | `"auto"` | Comment les images (graphe, avatars) parviennent au terminal : `auto`, `kitty`, `iterm2`, `sixel`, `none`. `auto` = protocole Kitty dans kitty, Ghostty, iTerm2 3.7.7 ou plus récent et un herdr avec `kitty_graphics = true` ; protocole iTerm2 dans WezTerm et Rio (transparence mélangée), iTerm2 plus ancien et mintty (fond opaque) ; sixel (la colonne du graphe et les avatars) dans Windows Terminal, foot, Tabby, Konsole, mlterm, dans tmux quand le terminal derrière lui accepte sixel, et dans tout terminal dont la réponse au démarrage cite sixel (un xterm lancé avec `-ti vt340`, un terminal VS Code avec les images activées) ; des caractères dans zellij, screen et les terminaux inconnus. Dans tmux, les images kitty demandent `kitty` ici plus `allow-passthrough on` ([Multiplexeurs](multiplexers.md)). Là où la pseudo-console du terminal perd ou déplace les images (WezTerm 20240203, Rio et mintty sous Windows), utilisez `ui.graph_renderer = "text"` ([Choisir un terminal](terminals.md#windows)). **Redémarrage.** |
| `ui.theme` | `"system"` | Le thème de couleurs : un slug intégré (`gitpoulpi theme list` les liste tous), le slug d'un fichier de `<config>/themes/`, ou `system` (suit le fond clair ou sombre du terminal, détecté au démarrage ; l'ancienne valeur `default` signifie la même chose). `config set` rejette les noms inconnus. Voir [Thèmes](themes.md). |
| `ui.graph_color_theme` | `"theme"` | Palette de voies : `theme` (suivre le thème) ou n'importe quel nom de palette de voies : `gitpoulpi`, `light`, `material`, `serie`, et les palettes des autres thèmes. |
| `ui.graph_lane_background` | `"off"` | `off` ou `row_tint` (une bande légère de la couleur de la voie derrière le graphe, plus une barre à son bord). **Application : Basculer la teinte de ligne du graphe** (`Ctrl+N Shift+T`) le bascule ; **Application : Changer le thème de couleurs du graphe...** (`Ctrl+N Shift+Y`) choisit `ui.graph_color_theme`. |
| `ui.graph_row_tint_strength` | `"medium"` | `strong`, `medium`, `subtle` (seulement avec `row_tint`). |
| `ui.toolbar_position` | `"bottom"` | `top`, `bottom`, `left`, `right`. `left` et `right` dessinent une colonne ; une fenêtre de moins de 80 colonnes revient à `bottom`. |
| `ui.dialog_geometry` | `"session"` | `session`, `saved`, `never` : si un dialogue que vous avez déplacé ou redimensionné (glissez sa barre de titre ou ses bordures, ou `Ctrl+N m`) garde sa place. Un seul réglage pour tous les dialogues (chacun garde sa propre place et sa propre taille) : jusqu'à la fermeture de GitPoulpi (par défaut), sur disque pour le prochain lancement (tous les dialogues dans l'unique `dialog_geometry.txt` du dossier de données), ou pas du tout (chaque dialogue s'ouvre à sa place et à sa taille par défaut). |
| `ui.toolbar_align` | `"start"` | `start`, `center`, `end` : où se placent les boutons le long de la barre (gauche, milieu, droite d'une ligne ; haut, milieu, bas d'une colonne). |
| `ui.toolbar_pr_pinned` | `true` | Le bouton de pull request est épinglé à l'extrémité de la barre d'outils ; `false` le laisse suivre les autres boutons. |
| `ui.statusbar_position` | `"bottom"` | `top` (au-dessus des onglets de dépôt) ou `bottom`. |
| `ui.tabs_position` | `"top"` | `top`, `bottom` (la dernière ligne de la fenêtre, sous la ligne d'état) ou `left` (une colonne au bord gauche ; sous 80 colonnes elle reste en haut). |
| `ui.graph_show_stashes` | `true` | Une ligne par stash dans le Log. |
| `ui.graph_ghost_ref` | `true` | La colonne Ref du Log affiche une pastille estompée avec le nom de la branche (sinon la branche distante, jamais un tag ; une branche supprimée est déduite du sujet du commit de fusion et barrée) à laquelle appartient un commit survolé, quand ce commit n'a pas de ref propre et que la colonne a la place. Survoler la pastille indique son type. |
| `ui.ref_connector` | `true` | Un trait fin d'une pastille de ref à son commit. |
| `ui.column_resize` | `"line"` | Comment une frontière de colonne se saisit dans le Log, le Radar et la colonne des commits des vues de diff, de fichier et de blame. `line` : chaque frontière est un espace d'une cellule avec un petit repère dans l'en-tête ; quand le pointeur est sur la frontière, dans l'en-tête ou n'importe où le long du tableau, un trait fin d'accent est tracé sur toute la hauteur et un glissement depuis n'importe quel point du trait redimensionne. `header` : pas d'espace ni de repère ; le premier caractère de la cellule d'en-tête après la frontière s'allume et un glissement depuis là redimensionne (l'en-tête seulement). Les touches de dimensionnement et les lignes de la palette sont les mêmes dans les deux modes. |
| `ui.ref_tree` | `true` | Regroupe les branches et les tags sur `/` en dossiers dans le panneau Refs (`y` le bascule). |
| `ui.reveal_on_ref_click` | `true` | Un simple clic sur une ref (branche, tag, branche distante, stash) du volet Refs fait défiler le Log jusqu'à son commit et fait clignoter la ligne ; la sélection de commit ne bouge pas. |
| `ui.log_wip_counts_in_message` | `true` | Compteurs de modifications WIP dans le message WIP tant que la colonne Modifs est masquée. |
| `ui.log_columns_layout` | `"fit"` | Comment le Log dispose ses colonnes. `fit` : chaque colonne visible tient exactement dans le volet et seule `message` s'adapte ; quand le volet est étroit, des colonnes entières cèdent la place. `scroll` : les colonnes gardent leur largeur (`message` aussi), rien ne cède, et le tableau défile latéralement quand il est plus large que le volet. |
| `ui.log_column_widths_scroll` | vide | Les largeurs de colonnes de la disposition `scroll`, nom -> cellules (`message` permis, contrairement à `ui.log_column_widths`, qui reste celle de la disposition `fit` et ne contient jamais `message`). Écrit quand vous glissez, dimensionnez ou ajustez une colonne alors que la disposition est `scroll`. |
| `ui.log_columns_hidden`, `ui.log_columns_order`, `ui.log_column_widths` | vide | Les colonnes du Log (`ref`, `graph`, `message`, `author`, `date`, `sha`, `changes`), écrites quand vous masquez, glissez ou redimensionnez une colonne. `message` ne peut pas être masquée. |
| `ui.commit_column_columns_visible`, `ui.commit_column_columns_order`, `ui.commit_column_columns_widths` | vide | Les parties de la [colonne des commits](interface-tour.md#la-colonne-des-commits) à côté d'un diff, à part de celles du Log : les parties affichées par nom (`ref`, `graph`, `message`, `author`, `date`, `changes`, `sha` ; vide, ce sont les voies, le message, l'auteur et la date), leur ordre de gauche à droite par les mêmes noms (vide, c'est l'ordre qui vient d'être donné ; un nom omis suit dans cet ordre) et les largeurs glissées en cellules par partie (une partie omise est automatique), écrites quand vous choisissez, déplacez ou dimensionnez les parties. `message` ne peut pas être masquée. Un fichier de session écrit avant l'existence de ces clés est lu une fois dedans. |
| `ui.sidebar_sections_hidden` | vide | Les sections du volet gauche que vous avez masquées (`local`, `remote`, `worktrees`, `stashes`, `tags`, `pull_requests`, `issues`, `submodules`), écrit depuis le menu clic droit du volet ou la palette. Au moins une section existante reste toujours affichée. C'est la valeur par défaut : chaque dépôt peut masquer ses propres sections (conservées dans son fichier d'état), et cette clé s'applique à un dépôt sans choix propre ; *Utiliser pour tous les dépôts* l'écrit. |
| `ui.sidebar_sections_scope` | `"repository"` | À qui appartient le choix des sections du volet gauche : `repository` (chaque dépôt garde le sien, `ui.sidebar_sections_hidden` étant la valeur par défaut) ou `global` (tous les dépôts affichent `ui.sidebar_sections_hidden`, et un changement y est écrit). Basculer ne perd jamais aucun des deux jeux : les choix par dépôt restent dans leurs fichiers d'état tant que `global` est actif. Paramètres : *Sections du volet gauche*. |
| `ui.language` | `"auto"` | La langue de l'interface : `auto` (la locale du système, sinon l'anglais), `en`, `fr`, `es`, `pt-BR`, `pt-PT`, `it`, `de`, `zh-CN`, `ja`, `ko`, `ru`, `uk`, `pl`, `cs`, `nl`, `sv`, `tr`. **Direct.** `GITPOULPI_LANG` l'emporte. Une langue absente de ce build affiche l'anglais. **Application : Changer de langue...** (`Ctrl+N g`) la règle aussi, avec un aperçu en direct. La sortie de la CLI et de MCP reste en anglais. |
| `ui.translate_git_terms` | `false` | `false` : commit, push, pull, stash, tag... restent en anglais dans toutes les langues. `true` : le vocabulaire git de la langue choisie (français « remiser », « étiquette », « valider »...). **Direct.** |
| `ui.git_terms_english` | `false` | Seulement ja, ko et zh-CN : affiche les mots git anglais (stage, unstage, staged, unstaged...) au lieu de leur transcription. Sans effet tant que `ui.translate_git_terms` vaut `true`. **Direct.** |
| `ui.mouse_enabled` | `true` | `false` laisse la souris (et la sélection de texte) au terminal. **Redémarrage.** |
| `ui.mouse_scroll_lines` | `3` | Lignes du Log par cran de molette, 1 à 20. **Redémarrage.** |
| `ui.kitty_keyboard` | `"auto"` | `auto` demande au terminal le protocole clavier kitty (pour que `Ctrl+,`, `Ctrl+Enter`, `Ctrl+Shift+N` arrivent ; Unix seulement, tmux ne répond jamais : voir [Multiplexeurs](multiplexers.md#tmux)) ; `off` ne demande jamais. **Redémarrage.** |
| `ui.logo_color` | `""` | Coloris du logo dans la fenêtre À propos, un slug (`violet`, `orange`, `teal`, `ocean`, `magenta`, `candy`, `coral`, `green`, `cyan`, `indigo`, `lavender`, `sun`, `mint`, `slate`) ; vide ou inconnu signifie le défaut du build (`lavender`). Écrit quand vous cliquez sur le logo ou choisissez une couleur dans À propos. |

**Avatars** (désactivés par défaut ; les activer envoie un hash SHA-256 des e-mails
d'auteurs à Gravatar, et interroge votre compte GitHub/GitLab connecté)

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `ui.avatars` | `"off"` | `graph` (dans le cercle du commit du graphe en images), `author` (dans la colonne Auteur), `off`. L'ancien `ui.show_avatars` (`true`) se lit encore comme `author` tant que `ui.avatars` est absent. |
| `ui.avatar_sources` | `"gravatar_hosting"` | Ordre de recherche : `hosting_gravatar`, `gravatar_hosting`, `gravatar`, `hosting`. Un hôte GitLab n'est interrogé que via un profil connecté. |
| `ui.avatar_fallback` | `"identicon"` | `identicon` (généré localement) ou `initials`. Sur un terminal sans protocole d'images, la pastille d'initiales est toujours utilisée. |

**Affichage du diff et de l'éditeur**

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `ui.diff_layout` | `"inline"` | `inline` ou `split` (`m` dans un diff). |
| `ui.diff_hunks_only` | `true` | `false` montre tout le fichier avec les modifications en place (`H`). |
| `ui.diff_show_whitespace` | `true` | Montrer les modifications d'espaces seuls (`s`). |
| `ui.diff_word_wrap` | `false` | Replie les longues lignes du diff (`w` ; Paramètres > Diff et éditeur). |
| `ui.minimap` | `false` | Une minimap à côté de la barre de défilement du diff et de l'éditeur (`M`). |
| `ui.mermaid_detail` | `refs` | Détail du Copier en gitGraph Mermaid du Log : `structure`, `refs`, `subjects`, `full` (mémorisé depuis la fenêtre ; la valeur par défaut de `log --format mermaid`). |
| `ui.mermaid_orientation` | `lr` | Direction de ce graphe : `lr` (de gauche à droite) ou `tb` (de haut en bas). |
| `ui.mermaid_fence` | `false` | Entoure le script Mermaid copié d'une clôture de code `mermaid`. |
| `ui.mermaid_subject_width` | `40` | Les sujets des commits sont coupés à ce nombre de caractères dans les ids Mermaid (`subjects`, `full`). |
| `ui.text_cursor` | `"terminal"` | Où s'affiche le point d'insertion d'un champ de texte actif : `terminal` y place le curseur du terminal lui-même (une barre clignotante, si bien que les réglages de curseur du terminal et la fenêtre de candidats d'une méthode de saisie le suivent ; masqué quand aucun champ n'a le clavier et rétabli à votre propre style de curseur à la sortie), `drawn` dessine le curseur propre à GitPoulpi et garde le curseur du terminal masqué. Direct. |
| `ui.color_depth` | `"auto"` | `auto`, `truecolor`, `256` ou `16` : avec combien de couleurs dessiner. `auto` est truecolor sauf sur Apple Terminal (256), la console Linux (16) et les terminaux `*-256color` sans `COLORTERM` (256). Direct. Voir [Thèmes](themes.md). |
| `ui.input_color` | `"theme"` | Fond des champs de texte : `theme` (celui du thème), `gray`, `blue`, `slate`, `dark`, `custom`. |
| `ui.input_color_custom` | `"#2a2a2a"` | La couleur `custom`, `#rrggbb`. |
| `ui.composer_files` | `"expanded"` | Le compositeur de commits IA s'ouvre avec les fichiers de chaque proposition `expanded` ou `collapsed`. |
| `ui.blame_side` | `"right"` | Quel côté de la vue Blame porte la colonne d'annotation (avatar, auteur, ancienneté, résumé) : `right` (par défaut) ou `left`. |
| `ui.blame_width` | `46` | Largeur de la colonne d'annotation du Blame en cellules (24 à 90). Faites glisser sa bordure, ou appuyez sur `<` / `>` dans la vue ; la nouvelle largeur est enregistrée ici. |
| `ui.pr_terminology` | `"auto"` | `auto` (pull request sur GitHub, merge request sur GitLab), `pull_request`, `merge_request`. |
| `ui.hide_finished_pr_chip` | `true` | Masque la pastille de la barre d'outils qui montre la dernière pull request ou merge request de la branche extraite une fois celle-ci fermée ou mergée ; c'est la pastille atténuée « + PR » (créer une nouvelle demande) qui s'affiche alors, comme si la branche n'en avait aucune. `false` : la pastille montre toujours la dernière demande, quel que soit son état. |

Les bascules du diff que vous changez dans un diff sont enregistrées ici aussitôt et
utilisées par chaque diff ouvert ensuite.

**Dates** : `ui.date_format` (`"YYYY-MM-DD"`, la colonne de date du Blame / de l'Historique du fichier),
`ui.date_time_format` (`"YYYY-MM-DD HH:mm"`, la colonne Date du Log) et
`ui.date_verbose_format` (`"D MMMM YYYY HH:mm"`, les détails du commit, l'infobulle de
Date et *Copier > Date*). Jetons : `YYYY` `YY` `MMMM` `MMM` `MM` `M` `DD` `D` `dddd` `ddd`
(jour de la semaine, complet / abrégé) `HH` `mm` `ss` ; tout le reste est copié tel quel.
Les noms de mois et de jours suivent `ui.language`, et un mois écrit après un numéro de
jour prend le génitif quand la langue en a un (russe, ukrainien, polonais, tchèque). Une
séquence `D MMMM YYYY` s'écrit année d'abord en japonais, chinois et coréen
(`2024年3月5日`). `ui.date_locale` (`"system"`, le défaut, ou `"english"`) garde les noms
en anglais quelle que soit la langue.

**Agents, terminaux et dossier de l'onglet**

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `ui.allow_agent_view_control` | `true` | La fenêtre accepte ou non `ui show` / `ui propose` des agents (Paramètres > Agents et MCP > *Laisser les agents IA piloter la vue*). Désactivé, aucun agent ne l'atteint. Voir [MCP et agents](mcp-and-agents.md#montrer-des-choses-à-lutilisateur). |
| `ui.launch` | `"off"` | Si un agent peut ouvrir une fenêtre GitPoulpi quand aucune n'est ouverte : `off`, un préréglage (`kitty`, `foot`, `alacritty`, `wezterm`, `wezterm-tab`, `ghostty`, `gnome-terminal`, `xdg-terminal-exec`, `windows-terminal`, `windows-terminal-tab`, `tmux`, `zellij`, `cmux`) ou `custom`. |
| `ui.launch_command` | `""` | Le modèle `custom` : `{repo}`, `{session}`, `{gitpoulpi}`. Découpé en arguments, sans shell. |
| `ui.tab_dir_follow` | `"buttons"` | `buttons` (la barre d'onglets propose *Quitter & cd ici*) ou `auto` (chaque sortie change le dossier du shell, et un nouvel onglet lance l'entrée *Ouvrir dans* par défaut). Voir [Suivre le dossier de l'onglet](following-the-tab-folder.md). |
| `ui.terminal_launch` | `"auto"` | Hérité, voir plus bas et [Ouvrir dans](open-in.md). Comment l'ancien *Terminal ici* ouvrait un terminal : `auto` (le préréglage `ui.launch`, sinon `$TERMINAL`, `xdg-terminal-exec`, Windows Terminal, Terminal), un préréglage, ou `custom`. |
| `ui.terminal_launch_command` | `""` | Le modèle `custom` ; `{repo}` est le dossier. |
| `ui.open_in` | *(absent)* | La liste [Ouvrir dans](open-in.md), un tableau de tables `[[ui.open_in]]` avec `label` et `command`. Absent signifie « pas encore configuré » : la liste est calculée à partir des anciennes clés `ui.terminal_launch*` (une valeur autre que `auto` devient la première entrée), de l'explorateur de fichiers de l'OS, des multiplexeurs et terminaux détectés d'après l'environnement (herdr, tmux, zellij, WezTerm, kitty, Windows Terminal, ...) et des éditeurs trouvés dans le `PATH`. Le premier enregistrement depuis les Paramètres écrit la liste. `open_in = []` est une liste vide. Non réglable avec `config set` (modifiez le fichier). |
| `ui.open_in_default` | `""` | Libellé de l'entrée que lance « Ouvrir dans par défaut » ; vide ou inconnu signifie la première entrée. |
| `ui.launch_terminal` | `"auto"` | Le terminal dans lequel les raccourcis de bureau (Paramètres > Lanceur) et la relance sous Windows démarrent GitPoulpi. `auto` choisit parmi ce qui est installé (Windows : Windows Terminal, sinon WezTerm, Alacritty, la console simple ; Linux : `xdg-terminal-exec`, sinon le premier terminal trouvé ; macOS : iTerm2, Ghostty, WezTerm ou kitty s'il est installé, sinon Terminal), `default` est celui du système, sinon un identifiant de terminal comme `windows-terminal`, `wezterm`, `alacritty`, `console`, `kitty`, `foot`, `ghostty`, `gnome-terminal`, `konsole`, `iterm`, `apple-terminal`. |
| `ui.launcher_build` | `"current"` | Mode développeur seulement (voir `general.dev_mode` ; affiché dans Paramètres > Lanceur seulement alors, ignoré sinon). `"current"` : les raccourcis et l'alias de commande lancent la version qui les a créés. `"newest"` : ils lancent la version la plus récemment compilée entre `target/dist` et `target/release` du dépôt cloné, décidée à chaque clic, et lancer l'une ou l'autre ne les réécrit jamais. |
| `ui.command_alias` | `""` | Le nom de la commande qui démarre GitPoulpi depuis un terminal, par exemple `gp` (Paramètres > Lanceur, `gitpoulpi alias create`). Vide : aucun. Lettres, chiffres, `-`, `_` et `.` uniquement. C'est seulement le nom qu'utilise le bouton **Créer : alias de commande** ; rien n'existe tant que vous ne le créez pas. |
| `ui.command_alias_add` | `""` | Le nom de la seconde commande, par exemple `gpa`, qui exécute `gitpoulpi --add` : elle ouvre le dossier courant dans un nouvel onglet du GitPoulpi déjà lancé (Paramètres > Lanceur, `gitpoulpi alias create gpa --add`). Vide : aucune. Mêmes règles de nom ; créée avec `ui.command_alias` par le bouton **Créer**. |
| `ui.relaunch_in_terminal` | `true` | Windows uniquement : quand l'exe est lancé par un double-clic dans une console à lui, GitPoulpi se rouvre dans `ui.launch_terminal` et quitte. Un terminal qui lance l'exe directement (`wezterm-gui start -- gitpoulpi.exe`, `wt.exe gitpoulpi.exe`, Rio, Tabby, Alacritty `-e`) est aussi une console à lui, mais pas une fenêtre de console classique : il n'est jamais relancé. Désactivé : il reste dans cette console. |
| `ui.reuse_window` | `true` | Un clic sur un raccourci GitPoulpi (`gitpoulpi launch`) affiche la fenêtre d'un GitPoulpi déjà ouvert au lieu d'en démarrer un autre. Désactivé : chaque clic démarre une nouvelle fenêtre. |

Les **commandes `ui.open_in`** sont d'abord découpées en arguments (espaces, `'...'`, `"..."`), puis les variables sont substituées dans chaque argument, si bien qu'un dossier avec des espaces reste un seul argument ; aucun shell n'intervient sauf si la commande en lance un (`sh -c '... "$GITPOULPI_CWD"'`). Variables : `${cwd}` (le dossier de l'onglet), `${repo}` (racine du dépôt), `${name}` (nom de base du dossier), `${branch}` (vide en HEAD détaché), `${web_url}` (la page web du remote `origin` quand c'est un projet GitHub ou GitLab, sinon vide ; une entrée qui l'utilise refuse de s'exécuter quand elle est vide), `${env:NAME}` (vide si non définie). L'ancien `{repo}` signifie encore `${cwd}`. Tout autre `${...}` est une erreur qui le nomme. Exemple :

```toml
[ui]
open_in_default = "Herdr: new tab"

[[ui.open_in]]
label = "Herdr: new tab"
command = "herdr tab create --workspace ${env:HERDR_WORKSPACE_ID} --cwd ${cwd} --focus"

[[ui.open_in]]
label = "VS Code"
command = "code ${cwd}"
```

Hérité : `ui.terminal_launch` et `ui.terminal_launch_command` sont encore lus (ils amorcent la liste tant que `ui.open_in` est absente : `custom` devient « Terminal (custom) » avec `{repo}` réécrit en `${cwd}`, un identifiant de préréglage devient ce préréglage) mais sont remplacés par `ui.open_in`.

### Images

| Clé | Défaut | Signification |
|---|---|---|
| `ui.image_view` | `true` | Affiche un fichier binaire qui est une image (PNG, JPEG, GIF, WebP, BMP, ...) comme une image dans la vue Diff et la vue Fichier. Désactivé : l'avis binaire simple. |
| `ui.image_view_max_mib` | `32` | Les images plus grandes (en Mio) ne sont pas décodées ; une fiche d'information s'affiche à la place. |
| `ui.image_view_max_unit` | `"auto"` | L'unité dans laquelle les Paramètres affichent la limite : `mib`, `gib` (1024 Mio) ou `auto`. Affichage seulement : la limite est toujours de `image_view_max_mib` Mio, de 1 à 1024. |
| `ui.image_view_text_fallback` | `"halfblock"` | Sur un terminal sans protocole d'image : `halfblock` (caractères demi-blocs colorés) ou `card` (fiche d'information seule). |
| `ui.image_view_pixelated_from` | `"2x"` | Zoom à partir duquel l'agrandissement montre des pixels carrés au lieu de lisser : `1x`, `2x`, `never`. |
| `ui.image_view_sync` | `true` | Les volets avant et après partagent un zoom et une position de défilement. |
| `ui.image_editors` | *(absent)* | Les éditeurs d'images « Éditer avec », un tableau de tables `[[ui.image_editors]]` avec `label` et `command`. Absent signifie « pas encore configuré » : la liste est celle des éditeurs détectés sur cette machine (programmes dans le `PATH`, installations Flatpak, dossiers d'installation connus sous Windows, `/Applications` sous macOS), puis l'application par défaut de l'OS. Le premier enregistrement depuis les Paramètres écrit la liste. `image_editors = []` est une liste vide. Non réglable avec `config set` (modifiez le fichier). |
| `ui.image_editor_default` | `""` | Libellé de l'éditeur que lance « Éditer avec par défaut » ; vide ou inconnu signifie la première entrée. |

Les **commandes `ui.image_editors`** sont d'abord découpées en arguments (espaces, `'...'`, `"..."`), puis les variables sont substituées dans chaque argument, si bien qu'un chemin avec des espaces reste un seul argument ; aucun shell n'intervient. Variables : `${file}` (le chemin absolu de l'image), `${dir}` (son dossier), `${name}` (son nom de fichier), `${repo}` (la racine du dépôt, vide pour une copie temporaire), `${env:NAME}` (vide si non définie). Tout autre `${...}` est une erreur qui le nomme. Une image qui n'est pas le fichier de l'arbre de travail (la version d'un commit, l'index, un fichier supprimé) est ouverte comme une **copie temporaire en lecture seule**, si bien qu'un éditeur ne peut pas toucher le dépôt. Les éditeurs sans ligne de commande (Affinity Photo du Microsoft Store, certaines installations de Photoshop) ne sont pas dans les préréglages : écrivez une entrée personnalisée. Exemple :

```toml
[ui]
image_editor_default = "GIMP"

[[ui.image_editors]]
label = "GIMP"
command = "gimp ${file}"

[[ui.image_editors]]
label = "Paint.NET"
command = "\"C:\\Program Files\\paint.net\\paintdotnet.exe\" ${file}"
```

**Pas encore utilisées** (acceptées, enregistrées, mais rien ne les lit) :
`ui.graph_colorization` (retiré des Paramètres ; encore
accepté au chargement), `ui.date_word_format` (retiré des Paramètres ; encore accepté au
chargement). L'ancien
`ui.diff_view_mode` est lu une fois comme `diff_layout` / `diff_hunks_only`.

## `[editor]`

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `editor.tab_size` | `4` | Colonnes par taquet de tabulation, 1 à 8 (éditeur et diffs). |
| `editor.eol` | `"auto"` | Fin de ligne que l'éditeur intégré écrit : `auto` (garder celle du fichier), `lf`, `crlf`. |
| `editor.syntax_highlighting` | `true` | Couleurs de langage dans l'éditeur. |
| `editor.line_numbers` | `true` | Numéros de ligne dans l'éditeur et les diffs. |
| `editor.follow_selection` | `true` | Tant que l'éditeur est ouvert, sélectionner un autre fichier dans le volet de droite l'ouvre aussi dans l'éditeur, après avoir proposé d'enregistrer les modifications non sauvegardées. Un fichier qui ne peut pas être édité s'ouvre dans sa vue habituelle. Seulement pour les modifications de travail : un fichier choisi dans un commit quitte l'éditeur pour le diff de ce commit. |

## `[merge]`

L'éditeur de conflits de merge ([Résoudre les conflits de merge](merge-conflicts.md)). Paramètres > Diff et éditeur le montre.

| Clé | Défaut | Signification |
|---|---|---|
| `merge.side_by_side_min_width` | `100` | La largeur, en colonnes, de la zone de l'éditeur à partir de laquelle A et B sont montrés côte à côte (les longues lignes défilent latéralement : Shift+molette, Shift+flèches). Plus étroit, il montre un côté à la fois derrière des onglets. `40` est la valeur minimale, `0` signifie toujours côte à côte (l'avis sous 40 colonnes reste). L'éditeur masque les panneaux Refs et Status tant qu'il est ouvert, sa zone est donc le plus souvent toute la fenêtre. |

## `[git]`, `[credentials]`, `[ssh]`

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `git.pull_mode` | `"merge"` | Ce que fait Pull (bouton, `Ctrl+G p`, `Git : Pull`) : `merge` (fast-forward quand possible), `ff_only`, `rebase`. Toujours passé explicitement à git. L'ancien `fetch_all` se lit comme `merge`. |
| `git.push_behavior` | `"direct"` | `direct` pousse vers le remote par défaut ; `ask` montre d'abord le menu des remotes. |
| `git.push_group_preflight` | `true` | Avant de pousser une branche vers plusieurs distants, vérifie d'abord chaque cible (`ls-remote`, plus un fetch de ce distant seulement quand il diffère), pour qu'un problème sur un distant suivant soit trouvé avant le premier push. |
| `git.merge_fast_forward` | `"never"` | Ce que fait un merge que vous lancez (menu de la branche, glisser-déposer, palette, `gitpoulpi merge`) quand la branche pourrait avancer en fast-forward : `never` crée toujours un commit de merge (`--no-ff`), `allowed` fait un fast-forward quand c'est possible (le comportement par défaut de git, `merge.ff` s'applique), `only` fait un fast-forward ou refuse. Le choix propre à un dépôt (Paramètres > Options de merge, `merge_fast_forward` dans son `repo-state.toml`) l'emporte ; les options `--ff`, `--no-ff` et `--ff-only` de la CLI l'emportent sur les deux. Appliqué à chaud. |
| `git.default_encoding` | `"utf-8"` | Encodage du texte qui **n'est pas** de l'UTF-8 valide (contenus de fichiers, blame, diffs, messages de commit sans en-tête `encoding`) : tout libellé WHATWG (`windows-1252`, `shift_jis`, ...). Affichage seulement ; le stage ne réencode jamais. Un dépôt peut le remplacer. |
| `git.lfs_enabled` | `true` | Afficher et piloter Git LFS. Un dépôt peut le remplacer. Voir [Git LFS](lfs.md). |
| `git.lfs_autofix_filters` | `true` | Avant un pull ou un push d'un dépôt qui utilise LFS, installe ses filtres LFS / hook pre-push (`git lfs install --local`, ce dépôt seulement) quand ils manquent. Voir [Git LFS](lfs.md). |
| `git.autocrlf`, `git.eol` | `"unset"`, `"native"` | *Non utilisées* : les `core.autocrlf` / `core.eol` d'un dépôt sont des paramètres propres à git, modifiés dans Paramètres > Git et encodage (ou `gitpoulpi repo-config`). |
| `credentials.https_mode` | `"credential_helper"` | Authentification HTTPS pour fetch, pull, push, clone : `credential_helper` (celui de git), `prompt` (ignore les assistants ; git demande dans un terminal, il échoue donc dans le démon d'arrière-plan), `none` (remotes publics seulement). Les anciennes valeurs `encrypted_file` se lisent comme `credential_helper`. |
| `ssh.key_type` | `"ed25519"` | Type créé par *Générer une paire de clés* : `ed25519`, `rsa` (4096 bits) ou `ecdsa` (NIST P-256) ; le fichier est `gitpoulpi_<type>`. |
| `ssh.private_key_path`, `ssh.public_key_path` | non défini | Enregistrés par *Générer une paire de clés*. Quand `ssh.private_key_path` est défini, fetch, pull, push et clone lancent git avec `GIT_SSH_COMMAND=ssh -i "<key>" -o IdentitiesOnly=yes` ; `ssh.public_key_path` fait la même chose (ssh choisit la clé correspondante de l'agent) quand le chemin privé est vide. Un `GIT_SSH_COMMAND`/`GIT_SSH` dans l'environnement ou un `core.sshCommand` dans la configuration git (l'épingle par dépôt, `gitpoulpi ssh-command set --key`) l'emporte. Lus avant chaque transfert, si bien que les modifications s'appliquent au suivant. |

## `[[integrations]]` (hébergeurs et suivi de tickets)

Écrit par Paramètres > Intégrations ou `gitpoulpi hosting integrations add`. Chaque type a
un client (jetons personnels seulement, pas d'OAuth).

```toml
[[integrations]]
id = "corp"                         # what --id takes; the token is stored under integration:corp
kind = "gitlab_self_managed"        # github | github_enterprise | gitlab | gitlab_self_managed | ...
url = "https://gitlab.example.com"  # the API root is derived from it
token_ref = "integration:corp"      # a reference, never the token
name = "Corp GitLab"                # optional display name
account = "me@example.com"          # optional, see below
enabled = true                      # false: ignored for host matching
```

Un serveur auto-hébergé peut être en http simple : `url = "http://localhost:3000"` est gardé
tel quel (le schéma et le port servent à chaque requête, et un distant sur cet hôte
correspond au profil). Un jeton envoyé en http simple circule en clair : n'utilisez
http que sur un réseau local ou de confiance.

`kind` accepte aussi `azure_devops`, `azure_devops_server`, `bitbucket`,
`bitbucket_data_center`, `jira_cloud`, `jira_data_center`, `trello`, `gitea`, `forgejo`, `linear`, `youtrack`, `redmine` et `gerrit`
(Gitea et Forgejo partagent un seul client ; un profil Forgejo vaut par défaut `https://codeberg.org`). Jira,
Trello, Linear, YouTrack et Redmine ne sont que des outils de suivi de tickets ; Gerrit ne fait que la revue (ses changes
sont les pull requests, il n'a pas de tickets). Bitbucket Cloud n'a pas de suivi de tickets.

`account` est l'identité non secrète à côté du jeton, vide et omise pour la plupart des
types : l'e-mail du compte Atlassian pour `jira_cloud` et `bitbucket` (un compte Bitbucket
vide signifie un jeton d'accès d'espace de travail ou de dépôt, envoyé comme Bearer), la
clé d'API pour `trello`, le nom d'utilisateur pour `gerrit` (le jeton est son mot de passe HTTP,
envoyé en HTTP basic). Le jeton lui-même reste le seul secret. Pour `azure_devops`,
l'`url` est `https://dev.azure.com/<organization>` ; une URL
`https://<organization>.visualstudio.com` est lue comme la même organisation. Pour
`azure_devops_server`, c'est la racine de la collection, comme
`https://tfs.example.com/tfs/DefaultCollection`.

`user` (le compte connecté) et `ssh` (le choix de clé du profil) sont écrits pour vous.
Le jeton vient de `GITPOULPI_TOKEN_<ID>` (id en majuscules, non-alphanumériques en `_`),
sinon du jeton stocké, sinon de votre assistant d'identifiants pour cet hôte quand un seul
profil l'utilise (profils GitHub et GitLab seulement : les autres types ne lisent jamais
l'assistant d'identifiants). Détails : [Intégrations d'hébergeurs et de suivi de tickets](hosting-integration.md).
L'ancienne table `[[hosting]]` est *non utilisée*.

Pour `local_markdown` (un dossier de notes Markdown comme source de tickets), `url` est le
dossier, relatif à chaque dépôt ou absolu, et il n'y a pas de jeton. Sa table facultative
`[integrations.local]` définit `statuses` (l'ordre des colonnes du tableau, par défaut `todo`,
`doing`, `done`), `done_status` (par défaut `done`), `reference_prefix` (`LOC` affiche `LOC-12`) et
`me` (ce que « assigné à moi » désigne). Voir [Tickets locaux et Obsidian](local-issues.md).

### `[integrations.radar]` (listes à l'échelle du compte)

Table facultative d'un profil d'hébergeur de code (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket Data Center, Gerrit ; pas Bitbucket Cloud). Absente,
toutes les clés sont désactivées. Écrite par `gitpoulpi hosting integrations radar` ; lue par
`gitpoulpi hosting radar` et le Radar, et modifiée dans Paramètres > Intégrations (le bloc Radar d'un profil connecté).

```toml
[integrations.radar]
review_requests = true    # demandes en attente de ma relecture, tous dépôts confondus
open_requests = false     # toutes les demandes ouvertes de mes espaces de noms
include_drafts = false    # garder les demandes en brouillon dans ces listes
assigned_issues = true    # tickets ouverts qui me sont assignés, tous dépôts confondus
namespaces = ["acme", "ptm/produits"]  # une entrée par ligne dans Paramètres ; vide = pas de limite
namespaces_regex = false  # true : chaque entrée est une expression régulière sur le chemin complet du dépôt
review_mode = "ask"       # ask | clone | api
```

| Clé | Par défaut | Signification |
|---|---|---|
| `review_requests` | `false` | Demandes ouvertes qui demandent votre relecture, que le dépôt soit cloné ou non |
| `open_requests` | `false` | Toutes les demandes ouvertes de votre espace de noms et de chaque organisation ou groupe dont vous êtes membre (ou de `namespaces`), jamais « tout ce que voit le jeton » |
| `include_drafts` | `false` | Les demandes en brouillon sont omises de ces listes sauf si cette clé est activée |
| `assigned_issues` | `false` | Tickets ouverts qui vous sont assignés, tous dépôts confondus |
| `namespaces` | `[]` | Les dépôts que gardent toutes les listes (demandes de relecture, tickets assignés, demandes ouvertes) ; vide signifie pas de limite, et `open_requests` cherche alors votre espace de noms plus toutes vos appartenances (GitHub : 12 au plus sont interrogés, GitLab : 10 groupes et 10 projets possédés, Gitea, Forgejo et Gerrit : 10). Une entrée correspond au chemin complet d'un dépôt (`groupe/sous/projet`, Azure DevOps `Projet/dépôt`, Bitbucket Data Center `CLÉ/dépôt`) quand elle lui est égale ou en est un préfixe qui s'arrête à un `/` : `ptm` garde `ptm/produits/x`, pas `ptmx/y` ; la casse est ignorée. Paramètres affiche une entrée par ligne ; un ancien fichier peut contenir une seule chaîne séparée par des virgules (`"ptm, formation"`), qui se charge encore et est réenregistrée comme liste |
| `namespaces_regex` | `false` | Lit chaque entrée de `namespaces` comme une expression régulière comparée au chemin complet du dépôt, tel que vous l'écrivez (donc `^ptm/(produits\|infra)/` s'ancre au début ; sans `^` ni `$` elle correspond n'importe où dans le chemin ; ajoutez `(?i)` pour ignorer la casse). `open_requests` cherche alors vos appartenances et les expressions filtrent les lignes. Une expression invalide s'affiche sous le champ dans Paramètres et n'est pas enregistrée ; dans un fichier modifié à la main, elle laisse les listes vides avec une note |
| `review_mode` | `"ask"` | Ce que fait la relecture d'une demande dont le dépôt n'est pas cloné : `ask` (demander), `clone` (le cloner, puis relire) ou `api` (relire via l'API de l'hébergeur) |

Les réponses sont mises en cache par profil dans `radar-account.json`, à côté de
`config.toml`, pendant cinq minutes ; le fichier est un cache, sans risque à supprimer.

### Le magasin de confiance (pas dans `config.toml`)

Ce à quoi chaque intégration et chaque profil d'IA à clé d'API a été approuvé (son
adresse, l'identifiant du compte, une clé de certificat épinglée, si le http
simple a été autorisé) vit dans `trust.toml`, dans le dossier de données privé
réservé à l'utilisateur (`<data>/private/trust.toml` : `%APPDATA%\GitPoulpi\private\`
sous Windows, `~/.local/share/gitpoulpi/private/` sous Linux). Il ne fait
volontairement pas partie de `config.toml` : qui peut modifier la configuration
ne doit pas pouvoir élargir la confiance en la modifiant. Changez-le avec
`gitpoulpi hosting integrations trust` et `gitpoulpi ai trust` (voir
[le contrôle de sécurité](hosting-integration.md#le-contrôle-de-sécurité)) ; le
fichier est réécrit par GitPoulpi, pas à la main.

## `[mcp]` (ce que les agents IA peuvent faire)

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `mcp.allow_mutations` | `false` | Désactivé : les outils MCP qui changent quoi que ce soit sont refusés ; les lectures, `ui_*` et les propositions continuent de marcher. `true` laisse les agents modifier le dépôt. |
| `mcp.mutation_allowlist` | `[]` | Non vide : seuls ces noms d'outils peuvent changer des choses (`["stage", "commit"]`). |
| `mcp.allow_ui` | `true` | `false` : refuse les outils `ui_*` qui déplacent votre fenêtre (pas `ui_state` / `ui_viewers`). |
| `mcp.ui_control` | `"ask"` | `ask` (la fenêtre montre une pastille que vous confirmez), `auto` (appliqué sauf si vous êtes occupé), `off`. `--ui-control` sur la commande du serveur le remplace par agent. S'applique aussi à `gitpoulpi ui show`. |
| `mcp.allow_ui_propose` | `false` | Autoriser `ui_propose_*` / `gitpoulpi ui propose` : des boîtes préremplies que vous confirmez vous-même. |
| `mcp.allow_history_purge` | `false` | Seulement dans `config.toml` (pas de ligne dans les Paramètres). `true` expose aux agents l'outil `rewrite_purge` (qui exige quand même `allow_mutations`). Désactivé, un agent ne voit pas l'outil du tout : une purge supprime définitivement l'ancien historique d'une réécriture d'historique, son annulation et ses reflogs. La CLI et la TUI ne sont pas concernées par ce réglage. |
| `mcp.enabled` | `true` | `false` : `gitpoulpi mcp serve` et `gitpoulpi-mcp` refusent de démarrer, avec un message qui nomme cette clé. Paramètres > Agents et MCP > *Serveur MCP*. Lu au démarrage du serveur par un agent. |
| `mcp.bind` | `"127.0.0.1:8808"` | L'adresse sur laquelle `--http` écoute quand vous n'en donnez pas (`--http ADDR` / `--bind` l'emportent). Loopback seulement sauf `--allow-remote`. Vide ou l'ancien `"stdio"` signifient le défaut. Paramètres > Agents et MCP > *Port du serveur MCP* modifie le port et conserve l'hôte. |
| `mcp.http_auth` | `true` | `--http` ne répond qu'à une requête portant `Authorization: Bearer <token>` (le jeton de `gitpoulpi mcp token`). `false` supprime la vérification ; le serveur avertit alors sur stderr, et `--allow-remote` est refusé. Paramètres > Agents et MCP > *Authentification HTTP*. |
| `mcp.tls` | `"off"` | `off`, `self_signed` (un certificat généré une fois dans le dossier de données privé, son empreinte SHA-256 affichée au démarrage) ou `files` (`tls_cert` et `tls_key`). Avec TLS, le serveur termine lui-même le TLS et aucun écouteur en clair n'existe. Paramètres > Agents et MCP > *TLS*. |
| `mcp.tls_cert` | `""` | Chaîne de certificats PEM utilisée avec `tls = "files"`. |
| `mcp.tls_key` | `""` | Clé privée PEM utilisée avec `tls = "files"`. |
| `mcp.behind_proxy` | `false` | Vous déclarez qu'un tunnel ou un proxy inverse placé devant le point d'accès assure le chiffrement. Cela ne peut pas être vérifié depuis le serveur : `--allow-remote` sans TLS exige donc cette clé. Le tronçon entre le proxy et le serveur circule en clair sur ce réseau. Paramètres > Agents et MCP > *Derrière un tunnel ou un proxy inverse*. |
| `mcp.allow_sensitive_staging` | `false` | Seulement dans `config.toml`. Désactivé, les outils MCP de stage n'offrent pas `allow_sensitive`, et un stage qui ajouterait un fichier signalé (voir `[privacy]`) est refusé avec un message qui dit à l'agent de vous demander. Activé, un agent peut passer `allow_sensitive`. La CLI et le TUI n'en dépendent pas : ils vous demandent. |

`allow_mutations`, `mutation_allowlist` et `allow_ui` ne contrôlent que le serveur MCP ;
la CLI est votre propre shell et n'est pas contrôlée par eux. Voir
[MCP et agents](mcp-and-agents.md#permissions).

## `[ai]` (IA intégrée, désactivée par défaut)

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `ai.enabled` | `false` | L'interrupteur. Désactivé, rien d'IA ne s'affiche nulle part et rien n'est envoyé. |
| `ai.default_profile` | `"default"` | Identifiant du profil que les actions utilisent quand elles n'en nomment aucun. |
| `ai.global_instructions` | `""` | Texte ajouté au prompt de chaque action. Peut utiliser `${repo}`, `${uilang}`, ... (voir [IA > Variables de prompt](ai.md#variables-de-prompt)) ; idem pour les instructions par action et du dépôt. |
| `ai.actions.<action>.profile` | `""` | Profil pour une action ; vide = le défaut. Actions : `commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`. Un espace de travail (`workspaces.toml`, `[workspace.ai_instructions]` et `[workspace.ai_overrides.<action>]`) et un dépôt peuvent ajouter à ces instructions ou les remplacer : voir [IA](ai.md#instructions). |
| `ai.actions.<action>.instructions` | `""` | Texte ajouté pour cette action seulement. |
| `ai.review.run_in_repository` | `"profile"` | `profile`, `yes` ou `no` : si l'agent local d'un brouillon de revue de code travaille dans le dépôt pour cette exécution, en passant outre le `workdir` du profil IA. Un dépôt peut avoir le sien (`gitpoulpi ai review-context --repo`). Voir [IA > Contexte de la revue](ai.md#contexte-de-la-revue). |
| `ai.review.include_claude_md` | `true` | Donne à un brouillon de revue de code le `CLAUDE.md` du dépôt (à la racine, lu sur la branche visée par la demande ; coupé à 8 Ko). |
| `ai.review.include_agents_md` | `true` | Pareil pour `AGENTS.md`. |
| `ai.review.extra_file` | `""` | Un fichier de plus pour un brouillon de revue de code : un chemin relatif à la racine du dépôt, un chemin absolu ou un lien vers un fichier d'un hébergeur (lu via le profil d'intégration correspondant, sinon anonymement). L'ensemble des fichiers est coupé à 16 Ko. |
| `ai.privacy.redact_secrets` | `true` | Remplace tout secret que l'analyseur reconnaît dans une requête (un jeton, un bloc de clé privée, un mot de passe dans une URL, une valeur `password = ...` qui paraît aléatoire) par `[REDACTED:<rule>]` avant son départ. Paramètres > IA affiche un avertissement rouge tant qu'il est désactivé. |
| `ai.privacy.exclude_sensitive_files` | `true` | Exclut de ce qui est envoyé le diff de chaque fichier correspondant à `privacy.sensitive_patterns`. Un agent local qui travaille dans le dossier du dépôt peut quand même lire les fichiers lui-même : voir [IA > Confidentialité](ai.md#confidentialité). |
| `ai.limits.conflict_bytes` | `40000` | Nombre maximal d'octets d'un conflit de merge (ses blocs base, courant et entrant ensemble) quand « Suggérer une résolution » l'envoie. Au-delà, GitPoulpi n'échoue pas : il prévient et propose *Envoyer quand même* (cette requête seulement), *Ouvrir le paramètre* et *Annuler* ; un conflit n'est jamais coupé. La limite est une protection propre à GitPoulpi, pas celle du modèle : augmentez-la quand votre modèle a la place. Bornée entre 1000 et 50000000 ; réglable dans Paramètres > IA > Limites. Les autres plafonds ci-dessous fonctionnent de la même façon et sont tous dans ce groupe. |
| `ai.limits.diff_bytes` | `60000` | Nombre maximal d'octets du diff que peut porter un message de commit, une explication, un nom de stash, une description de pull request, un plan de commits ou une scission de commit, et du diff entier d'une revue de code. Au-delà, rien n'est coupé en silence : dans la TUI, *Limite de l'IA dépassée* propose *Tout envoyer* (cette requête seulement, rien n'est coupé), *Envoyer tronqué* (la fin est écartée, comme avant), *Ouvrir le paramètre* et *Annuler* ; la CLI et MCP tronquent et le signalent dans `truncated` (`--send-all` envoie tout). Borné entre 1000 et 50000000. |
| `ai.limits.review_file_bytes` | `12000` | Nombre maximal d'octets du diff d'un fichier dans une revue de code. Au-delà, la même question et les deux mêmes réponses ; *Envoyer tronqué* raccourcit ce fichier avec une note visible. Même plage. |
| `ai.limits.review_text_bytes` | `4000` | Nombre maximal d'octets de la description de la pull request relue. Au-delà, *Envoyer tronqué* coupe la description avec une note `[text cut]`. Même plage. |
| `ai.limits.review_threads_bytes` | `6000` | Nombre maximal d'octets de la liste des commentaires de revue déjà présents (donnée pour que le modèle ne les répète pas). Au-delà, *Envoyer tronqué* coupe la liste. Même plage. |
| `ai.limits.review_context_file_bytes` | `8192` | Nombre maximal d'octets d'un fichier de contexte du projet (`CLAUDE.md`, `AGENTS.md`, le fichier supplémentaire) donné à une revue de code. Au-delà, *Envoyer tronqué* raccourcit le fichier avec une note. Même plage. |
| `ai.limits.review_context_bytes` | `16384` | Nombre maximal d'octets de tous les fichiers de contexte ensemble. Au-delà, *Envoyer tronqué* raccourcit le fichier qui franchit la limite et écarte ceux qui suivent. Même plage. |
| `ai.sent_preview.<action>` | `true` | Une clé par action (`commit_message`, `explain_changes`, `stash_message`, `pull_request_description`, `commit_composer`, `conflict_resolve`, `code_review`, `split_commit`, `rewrite_script`, `rewrite_rules`, `worktree_name`) : affiche « Ce qui est envoyé » avant le départ de la requête. « Ne plus afficher pour ... » de la boîte écrit `false` ; Paramètres > IA > *Afficher ce qui est envoyé avant...* les réactive. `rewrite_script` et `rewrite_rules` sont acceptées mais sans effet : les boîtes de réécriture d'historique montrent toujours leur propre volet. `worktree_name` est acceptée mais sans effet non plus : elle n'envoie que ce que vous avez tapé dans la boîte Créer un worktree et les noms des branches locales. |

La table `[preferences.ai_review]` propre à un dépôt (dans son `repo-state.toml`, ou l'entrée d'un dépôt non cloné dans
`remote-reviews.toml`) accepte les mêmes clés, toutes facultatives, plus `no_extra_file = true` : aucun fichier
supplémentaire pour ce dépôt, quoi que dise `ai.review.extra_file`. Une clé non définie, un `extra_file` non défini et
un `no_extra_file` absent héritent de la valeur globale ; définir `extra_file` sur un fichier efface l'interrupteur, et inversement.

```toml
[[ai.profiles]]
id = "default"
name = "Default"
kind = "open_ai"        # open_ai | anthropic | open_router | open_ai_compatible | local_agent
model = "gpt-5"         # empty = the kind's default
base_url = ""           # empty = the kind's default (Ollama: http://localhost:11434/v1)
token_ref = "ai:default"  # where the API key is stored; never the key
command = ""            # local_agent only: the exact program ("claude", an alias, a wrapper, a path)
harness = ""            # local_agent only: claude | codex | gemini | opencode | cursor-agent | copilot | pi | qwen | aider | goose | crush | amp | other; empty = inferred from `command`
args = []               # local_agent only, e.g. ["-p", "--model", "haiku"]
run_mode = "headless"   # local_agent only: headless (default) | terminal (the agent's interactive screen in a virtual terminal)
terminal_args = []      # local_agent only: the arguments of the terminal mode (args keeps the headless set)
workdir = "default"     # local_agent only: default | dedicated | repository (default = dedicated folder in terminal mode, GitPoulpi's folder in headless mode)
timeout_secs = 0        # local_agent only: seconds before the agent is stopped; 0 = 120 headless, 180 terminal

[ai.profiles.env]       # local_agent only, optional: variables set for that process only
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"   # ${env:NAME} = one of GitPoulpi's own variables
```

`harness` choisit les suggestions que propose Paramètres > IA (arguments sans interface,
options, option et liste de modèles, variables d'environnement) indépendamment de
`command` ; les deux clés sont omises du fichier quand elles sont vides. Les valeurs
d'environnement ne sont jamais journalisées et sont masquées dans les Paramètres quand elles
ressemblent à des secrets.

Paramètres > IA propose des préréglages pour `harness`/`command`/`args`/`env` (les CLI
d'agents courantes, avec leurs options sans interface, leur option de modèle et leurs
variables d'environnement utiles) et pour `kind`/`base_url`/`model` (les fournisseurs HTTP
hébergés et locaux) ; voir [IA > Configurer l'IA](ai.md#configurer-lia).

Un profil nommé *Default* est créé quand il n'y en a aucun. Les anciens fichiers
`[[ai.presets]]` / `[ai.defaults]` sont migrés au chargement et conservés dans
`[[ai.legacy_presets]]`. Tout le reste : [Assistance IA](ai.md).

## `[repos]`, `[session]`

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `repos.clone_root` | `""` | Où la boîte Cloner démarre ; vide = `%USERPROFILE%\source\repos` ou `~/src`. |
| `repos.clone_start_source` | `"last"` | La source sur laquelle s'ouvre la boîte Cloner : `last` (par défaut, ou vide) = celle qu'elle affichait à sa dernière fermeture, `url` = toujours Cloner par URL, ou l'identifiant d'un profil d'intégration (`id` dans son entrée `[[integrations]]`). Un profil supprimé, désactivé ou non connecté revient à Cloner par URL. La dernière source elle-même est un état, conservé dans `clone-source.toml` dans le dossier de configuration, pas ici. |
| `repos.browse_root` | `""` | Où Parcourir s'ouvre ; vide = le dernier dossier choisi, puis le dossier personnel. |
| `repos.show_hidden` | `false` | Le sélecteur de dossiers montre les dossiers cachés (`Alt+H` le bascule et l'enregistre). |
| `repos.show_siblings` | `false` | L'arbre de gauche du sélecteur de dossiers liste aussi les autres dossiers de chaque niveau ouvert, comme une vraie hiérarchie (`Alt+S` le bascule et l'enregistre). |
| `session.per_repo_state_dir` | `"in_git_dir"` | Où GitPoulpi garde ce qu'il retient d'un dépôt (filtres de refs, état des plis, la surcharge personnelle du workflow, le cache de l'hébergeur) : `in_git_dir` = `<dossier git>/gitpoulpi` (partagé par tous les worktrees, n'apparaît jamais dans `git status`), `in_worktree` = `<dépôt>/.gitpoulpi` (le seul endroit où une équipe peut commiter `workflow.toml`), `user_data` = `<dossier de données de l'OS>/repos/<nom>-<id>` (`%APPDATA%\GitPoulpi\data` sous Windows, `$XDG_DATA_HOME/gitpoulpi` sous Linux, `~/Library/Application Support/GitPoulpi` sous macOS). L'ancienne valeur `".gitpoulpi"` signifie toujours `in_worktree`. Le changer ne rapatrie rien : les données trouvées dans un ancien `<dépôt>/.gitpoulpi` sont déplacées vers l'endroit choisi la première fois qu'elles servent. |

## `[log]` (le journal de l'application)

Ce qu'affiche l'onglet « Application » du Journal d'activité et ce que cite « Copier le diagnostic ». Détails : [Quand quelque chose échoue](troubleshooting.md#quand-quelque-chose-échoue).

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `log.level` | `"info"` | La ligne la plus bavarde qui est enregistrée : `error`, `warn`, `info`, `debug` ou `trace`. La zone de niveau du Journal d'activité la change (et l'enregistre ici). |
| `log.retention_days` | `7` | Nombre de jours de fichiers journaux conservés, aujourd'hui compris : de 1 à 3650, soit dix ans (une valeur plus grande ou plus petite est ramenée dans cet intervalle). Toujours un nombre de jours, quelle que soit l'unité dans laquelle les Paramètres l'affichent. Les fichiers plus anciens sont supprimés au démarrage de GitPoulpi. Le fichier d'une journée est aussi plafonné par `log.max_file_mb` : le pire cas est donc d'environ `2 x max_file_mb x retention_days`. |
| `log.retention_unit` | `"auto"` | L'unité dans laquelle les Paramètres affichent `log.retention_days` : `days`, `weeks`, `months`, `years` ou `auto`. Affichage seulement, cela ne change jamais la durée de conservation des fichiers. Une semaine fait 7 jours, un mois 30 jours (pas un mois calendaire : le même nombre de jours quel que soit le moment) et une année 365 jours. `auto`, et une configuration écrite avant l'existence de cette clé, prend la plus grande unité qui divise exactement les jours (14 jours s'affichent « 2 semaines », 45 jours « 45 jours »). Une unité qui ne divise pas les jours (`retention_days = 10` avec `weeks`) est ignorée et les jours sont affichés. Les Paramètres écrivent les deux clés ; un GitPoulpi plus ancien ignore l'unité et lit toujours les jours. |
| `log.max_file_mb` | `5` | Le fichier d'une journée est renouvelé en une seule sauvegarde `.old` à cette taille ; une journée ne prend donc jamais plus du double. |
| `log.max_file_unit` | `"auto"` | L'unité dans laquelle les Paramètres affichent la taille : `mib`, `gib` (1024 Mio) ou `auto`. Affichage seulement : la taille est toujours de `max_file_mb` Mio, de 1 à 1024. |
| `log.timings` | `true` | L'onglet Opérations Git du Journal d'activité montre la durée de chaque opération (`Push - 123 ms` : ouverture d'un dépôt, actualisation, fetch, pull, push, commit, checkout, merge, rebase, stash, clone). Les durées sont mesurées par la fenêtre, de la requête à la réponse du démon, et incluent donc l'aller-retour IPC. Désactivé, il masque les durées et les entrées des opérations réussies. Les échecs sont toujours listés. |
| `log.extended` | `false` | « Journalisation étendue » : l'onglet Opérations Git liste aussi les opérations en arrière-plan (l'actualisation périodique qui interroge le statut, l'auto-fetch) avec leur durée. Elles sont toujours écrites dans le journal de l'application au niveau `debug`. |
| `log.commit_count` | `"system-git"` | Comment le `chargés / total` de la ligne d'état obtient le total. `"system-git"` lance `git rev-list --count` (un processus, le plus rapide sur une machine normale : git lit le fichier commit-graph). `"libgit2"` compte dans le processus avec un revwalk libgit2 et ne lance rien : à choisir quand chaque processus en ligne de commande démarre lentement, typiquement parce qu'un antivirus avec analyse en temps réel inspecte chacun ; sur 50 000 commits, cela prend environ 200 ms en arrière-plan au lieu de 12 ms, mais aucun processus n'est lancé. `"off"` ne compte jamais : la ligne d'état n'affiche que les commits chargés. S'applique immédiatement. Un comptage filtré par chemin utilise toujours git système. |
| `log.use_mailmap` | `true` | Affiche chaque auteur sous le nom que le `.mailmap` du dépôt (et `mailmap.file`) lui donne, dans le Log, le blame et l'historique d'un fichier : le daemon demande `%aN`/`%aE` à git (backend système) ou applique lui-même les mêmes fichiers (backend libgit2), si bien que la réponse est identique. Les commits enregistrés ne changent pas et rien n'est réécrit. La recherche par auteur dans le Log porte sur les noms tels qu'affichés. S'applique immédiatement ; le Log relit dans le délai de son sondage de 2 s. |

Les fichiers s'appellent `gitpoulpi-YYYY-MM-DD.log` et se trouvent dans le dossier d'état de l'OS (`~/.local/state/gitpoulpi/logs` sous Linux ; `<cache>\logs` sous Windows et macOS ; `<config>/cache/logs` avec `GITPOULPI_CONFIG_DIR`). Les jetons, mots de passe et en-têtes `Authorization` sont masqués avant toute écriture.

## `[search]` (la recherche dans l'espace de travail)

Ce que parcourt `gitpoulpi search` (et son outil MCP). Détails : [Espaces de travail et recherche en CLI](cli-reference.md#espaces-de-travail-et-recherche).

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `search.commit_limit` | `200000` | Commits (du plus récent au plus ancien, sur toutes les branches, branches distantes et tags) dont l'index de recherche garde les messages en mémoire par dépôt, environ 100 octets plus le message chacun. Un historique plus long est coupé là et la réponse indique qu'elle est `partial`. `0` désactive la recherche de commits (les branches, tags, stashes et worktrees restent cherchés). Lu en direct : la recherche suivante tient compte d'un changement. |

## `[rewrite]` (réécriture d'historique)

Ce que `gitpoulpi rewrite` et l'atelier de réécriture d'historique protègent. Détails : [Réécriture d'historique](history-rewrite.md).

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `rewrite.protected_branches` | `["main", "master", "develop", "release/*"]` | Branches (noms, ou motifs où `*` remplace n'importe quel texte) qu'une réécriture laisse de côté quand elles sont déjà poussées, sauf si l'exécution les liste (`--include-protected BRANCHE` ; `--include-all-protected` pour le push ; la case à cocher de chaque branche dans l'atelier). S'ajoutent aux bases que le `workflow.toml` d'un dépôt marque comme `protected`. Le push d'une réécriture les ignore de la même façon. C'est la protection propre à GitPoulpi : c'est toujours la protection de branche de l'hébergeur qui décide si le push forcé est accepté. Lu en direct : le prochain aperçu tient compte d'un changement. |
| `rewrite.external_scripts` | `["shell"]` | Les langages de script externe que la règle Script peut utiliser : `"shell"` (bash ou sh ; pwsh ou powershell pour les scripts PowerShell), `"python"`, `"lua"` (un Lua installé ; le Lua intégré est toujours actif et n'a pas besoin d'entrée), en toute combinaison. **Un script externe n'est pas isolé : il s'exécute avec vos droits.** Un langage absent de la liste est grisé dans l'atelier et refusé (`scripts_disabled`) par la CLI, MCP et le daemon. Même listé, un script externe qui vient du dépôt ou d'une recette partagée ne s'exécute qu'après votre approbation (`gitpoulpi rewrite script trust FICHIER`, ou la question posée dans l'atelier) ; l'approbation est liée au hachage du contenu du script, donc un script modifié redemande. Pour activer Python, modifiez `config.toml` (une liste ne se règle pas avec `config set`) : `[rewrite]` puis `external_scripts = ["shell", "python"]`. Lu en direct. |
| `rewrite.script_timeout_ms` | `5000` | Le temps dont dispose un script externe pour répondre pour un commit. En cas de dépassement, tout l'arbre de processus est tué et la réécriture échoue (`script_failed`, avec le commit concerné) ; rien n'a bougé. `0` vaut le défaut. Lu en direct. |
| `rewrite.script_timeout_unit` | `"auto"` | L'unité dans laquelle les Paramètres affichent la limite de temps : `milliseconds`, `seconds` ou `auto`. Affichage seulement : la limite est toujours de `script_timeout_ms` millisecondes. Les Paramètres acceptent de 1 à 600000 (dix minutes) ; `0` vaut toujours le défaut. Lu en direct. |
| `rewrite.script_total_timeout_s` | `3600` | Le temps total que peut prendre une passe d'un script externe. `0` vaut le défaut. Lu en direct. |
| `rewrite.script_total_timeout_unit` | `"auto"` | L'unité dans laquelle les Paramètres affichent la limite de temps : `seconds`, `minutes`, `hours` ou `auto`. Affichage seulement : la limite est toujours de `script_total_timeout_s` secondes. Les Paramètres acceptent de 1 à 86400 (un jour) ; `0` vaut toujours le défaut. Lu en direct. |

## `[updates]` (la recherche d'une version plus récente)

Ce que fait la pastille de version de la barre de titre au sujet des nouvelles versions. Détails : [Mises à jour](updates.md).

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `updates.mode` | `"notify"` | `"notify"` demande à GitHub une fois par jour, en arrière-plan, s'il existe une version plus récente, et transforme la pastille de version en `vX.Y.Z disponible`. `"auto"` télécharge et prépare aussi la mise à jour tout seul quand ce build vient d'une archive de version (il demande quand même avant de redémarrer) ; pour tout autre build, il se comporte comme `"notify"`. `"off"` ne fait jamais de requête. S'applique immédiatement. |
| `updates.channel` | `"stable"` | `"stable"` ne considère que les versions complètes ; `"prerelease"` propose aussi les versions candidates (`-rc.1`). S'applique immédiatement. |
| `updates.repo` | `"Poulpi-Lab/GitPoulpi"` | Le `propriétaire/nom` sur github.com dont les versions sont vérifiées. Tant que ce dépôt est privé, GitHub répond 404 sauf si un jeton est enregistré pour github.com (Paramètres > Intégrations) ; sans jeton, la vérification échoue discrètement et écrit une ligne dans le journal de l'application. |

Un administrateur peut définir `mode`, `channel` (sous le nom `release_channel`) et `repo`, plus quelques autres options de mise à jour, dans un fichier de politique valable pour toute la machine ou dans le registre Windows. Une clé définie par la politique l'emporte sur `config.toml` et s'affiche dans les Paramètres comme **Défini par votre administrateur**. `mode` dans la politique est un plafond : vous pouvez choisir une valeur plus basse, jamais plus haute. Voir [Distribution en entreprise](enterprise-distribution.md#le-fichier-de-politique-et-le-registre).

## `[privacy]` (fichiers sensibles)

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `privacy.sensitive_patterns` | `[".env", ".env.*", "!.env.example", "!.env.sample", "!.env.template", "*.pem", "*.key", "id_rsa*", "id_ed25519*", "id_ecdsa*", "*.p12", "*.pfx", "*.jks", "*.keystore", "*.crt", "*.cer", "*.der", "*.kdbx"]` | Motifs de chemin au style gitignore. Un motif sans `/` correspond au nom de fichier à toute profondeur, un motif avec `/` au chemin entier (un `/` initial l'ancre à la racine) ; `*` et `?` restent dans un seul dossier, `**` traverse les dossiers ; un `!` autorise de nouveau ce qu'un motif précédent avait retenu et la dernière correspondance l'emporte ; la casse est ignorée. Utilisé par la garde de stage et par `ai.privacy.exclude_sensitive_files`. |
| `privacy.staging_guard` | `true` | Stager un fichier qui correspond à un motif, ou dont le contenu renferme un secret reconnu, demande d'abord confirmation (TUI), est refusé sauf avec `--allow-sensitive` (CLI) ou sauf si `mcp.allow_sensitive_staging` laisse l'agent passer `allow_sensitive` (MCP). Les fichiers texte jusqu'à 1 Mio chacun et 32 Mio au total sont lus ; les fichiers binaires ne sont repérés que par leur nom. Voir [Stage et commit](staging-and-committing.md#fichiers-sensibles). |

## `[conflict_prevention]` (listes de branches de base)

Les branches que la détection proactive des conflits peut prendre pour base de la branche courante forment une liste de motifs (voir [Résoudre les conflits de merge](merge-conflicts.md#prévention-des-conflits)). Les listes peuvent être nommées et partagées ; Paramètres > Prévention des conflits les modifie.

| Clé | Défaut | Valeurs et effet |
|---|---|---|
| `conflict_prevention.default_list` | `""` | Le nom de la liste qu'utilise un dépôt qui n'a pas la sienne. Vide, ou un nom qu'aucune liste ne porte, signifie `**` intégré (toutes les branches). Relue à chaque vérification : un changement s'applique aux dépôts ouverts sans redémarrage. |
| `conflict_prevention.lists` | aucune | Les listes nommées, une table chacune : `[conflict_prevention.lists.release]` avec `patterns = ["origin/release/**", "!origin/release/old"]`. Un `patterns` vide autorise toutes les branches. Modifier une liste s'applique à chaque dépôt qui la suit. |

La liste qu'un dépôt suit se trouve dans son `repo-state.toml` (ci-dessous). Priorité : les motifs propres au dépôt, puis la liste nommée qu'il suit, puis `default_list`, puis `**`.

## `[keybindings]`

Réaffecte les raccourcis de la TUI ; Paramètres > Clavier et souris la modifie.

```toml
[keybindings.global]
git_prefix = "Ctrl+B"            # default Ctrl+G
nav_prefix = "Alt+n"             # default Ctrl+N
settings   = ["Ctrl+,", "F2"]    # a list = alternatives
tab_menu   = []                  # unbound

[keybindings.git_prefix]         # the key pressed after the git prefix
stage_file = "z"
```

`[keybindings.<contexte>] <action> = "<enchaînement>"` ; une valeur remplace les défauts
de cette action, supprimer la ligne les restaure. Enchaînements : `Ctrl+Shift+N`, `Alt+n`,
`F2`, `PageDown`, `Space`, `,`, `P` (= `Shift+p`). Une mauvaise entrée ne bloque jamais
rien : la TUI l'ignore, garde le défaut et le dit sur la ligne d'état. Contextes,
identifiants d'actions, défauts et règles de conflit : [Raccourcis clavier](keybindings.md).

## Paramètres par dépôt : `.gitpoulpi/repo-state.toml`

Conservé à l'intérieur du dépôt ; le dossier s'ignore lui-même (un `.gitignore` de `*`),
si bien qu'il n'apparaît jamais comme une modification. Écrit par la TUI et la CLI ; vous le
modifiez rarement.

| Clé | Ce que c'est | Se règle avec |
|---|---|---|
| `[preferences] encoding` | Remplace `git.default_encoding` ici | Paramètres > Git et encodage |
| `[preferences] lfs_enabled` | Remplace `git.lfs_enabled` ici | Paramètres > Git et encodage |
| `[preferences] profile` | L'identifiant `[[profiles]]` qui signe les commits ici ; absent signifie `general.active_profile` | la liste déroulante de profils de la barre d'outils, **Dépôt : changer de profil...** |
| `[preferences] autocrlf`, `eol` | *Pas encore utilisés* | Paramètres > Git et encodage |
| `[preferences] push_after_commit` | Pousser la branche vers son amont après chaque commit (désactivé si absent) | Paramètres > Paramètres du dépôt > Commit |
| `[preferences] skip_hooks_by_default` | La case Ignorer les hooks du formulaire de commit est cochée au départ (désactivé si absent) | Paramètres > Paramètres du dépôt > Commit |
| `[preferences] squash_merge` | Le merge d'une branche locale se fait en squash : indexe, sans commit de merge (désactivé si absent) | Paramètres > Paramètres du dépôt > Options de merge |
| `[preferences] template_apply`, `template_strip_comments` | Préremplir un formulaire de commit vide avec le modèle ; retirer les lignes de commentaire `#` au moment du commit (tous deux désactivés si absents) | Paramètres > Paramètres du dépôt > Modèle de message |
| `[preferences] conflict_detection` | Détection proactive des conflits pour ce dépôt ; absent, suit `general.conflict_detection` | Paramètres > Paramètres du dépôt > Prévention des conflits |
| `[preferences] conflict_base_branches` | Les motifs propres au dépôt pour les branches de base, un par ligne (toujours utilisés comme **Liste propre**) | Paramètres > Paramètres du dépôt > Prévention des conflits |
| `[preferences] conflict_base_source` | D'où viennent les branches de base : `default` (la liste globale par défaut), `list` (`conflict_base_list`) ou `own` (`conflict_base_branches`). Absent : les motifs propres s'il y en a, sinon le défaut | Paramètres > Paramètres du dépôt > Prévention des conflits |
| `[preferences] conflict_base_list` | La `[conflict_prevention.lists.<nom>]` que ce dépôt suit quand la source est `list` ; une liste supprimée revient au défaut | Paramètres > Paramètres du dépôt > Prévention des conflits |
| `commit-template.txt` (un fichier à côté de `repo-state.toml`) | Le modèle de commit ; le `commit.template` de git y pointe | Paramètres > Paramètres du dépôt > Modèle de message |
| `[preferences.ai_instructions]` | `text`, `mode = "add" \| "replace"` : les instructions IA de ce dépôt | Paramètres > Paramètres du dépôt > IA |
| `[preferences.ai_overrides.<action>]` | `profile`, `instructions` pour une action ici | Paramètres > Paramètres du dépôt > IA |
| `default_remote` | Le remote que Push/Pull/Fetch en un clic utilisent | menu des remotes, `gitpoulpi remote default <name>` |
| `hosting_integration` | Le profil `[[integrations]]` épinglé ici | `gitpoulpi hosting use --id` |
| `pulls_source`, `issues_source` | Le projet du fournisseur que lisent les listes de PR / tickets, sous la forme `{ integration, project, filter }`. `filter` est facultatif : une clause supplémentaire native du fournisseur (un fragment JQL Jira, une condition WIQL Azure, des noms de listes Trello) combinée en ET à la requête de la liste ; GitHub et GitLab l'ignorent. | sections du panneau latéral, `gitpoulpi hosting link` |
| `ref_visibility` | Refs masquées / en solo | panneau Refs, `gitpoulpi refs hide\|show\|solo` |
| `hidden_remotes` | Remotes masqués du graphe en entier (toutes les branches, y compris celles récupérées plus tard) | le ⊙ et le menu du remote, `gitpoulpi refs hide --remote` |
| `folds`, `collapsed_*_ref_folders` | Sections du panneau latéral et dossiers de branches et de tags repliés. `[folds]` contient aussi `status_folders` (dossiers repliés des arbres de fichiers Unstaged / Staged) et `commit_folders` (dossiers repliés de l'arbre des fichiers modifiés d'un commit, le même pour chaque commit), chacun plafonné à 500 chemins, les plus anciens supprimés d'abord | le panneau latéral, les arbres de fichiers |
| `pending_auto_stash` | Un stash que GitPoulpi a fait et n'a pas remis | écrit par le démon ; voir [Auto-stash](log-and-history.md#auto-stash) |

## Autres variables d'environnement

| Variable | Effet |
|---|---|
| `GITPOULPI_CONFIG_DIR` | Le dossier de configuration (voir plus haut). |
| `GITPOULPI_TOKEN_<ID>` | Un jeton d'hébergeur pour le profil `<ID>`, prioritaire sur celui qui est stocké (CI, usage ponctuel). |
| `GITPOULPI_GIT` | L'exécutable `git` à lancer (défaut : `git` dans le `PATH` ; sous Windows le vrai `mingw64\bin\git.exe` derrière le shim `cmd\git.exe` de Git for Windows, ce qui économise un processus par appel). |
| `GITPOULPI_LANG=fr` | La langue de l'interface, prioritaire sur `ui.language` (n'importe quelle valeur de `ui.language` ; `qps` est une pseudo-locale pour les traducteurs). Une valeur inconnue ou vide est ignorée. |
| `GITPOULPI_TERMINAL_IMAGE=off` | Aucune image du tout, quoi que dise la configuration. |
| `GITPOULPI_TERMINAL_CELL_PX=9x21`, `GITPOULPI_TERMINAL_SUPERSAMPLE=1..4` | Indications de résolution des images (taille de cellule en pixels ; suréchantillonnage, 2 par défaut, `1` économise du CPU). |
| `GITPOULPI_NO_BROWSER=1` | Ne jamais lancer de navigateur web (l'URL est quand même affichée). |
| `GITPOULPI_TIMING=1` | Écrit les durées des requêtes dans `gitpoulpi-timing.log` dans le dossier temporaire (rapports de performance). |
| `GITPOULPI_NO_EXIT_PAUSE=1` | N'attend pas Enter après la sortie de la TUI sur une erreur. |
