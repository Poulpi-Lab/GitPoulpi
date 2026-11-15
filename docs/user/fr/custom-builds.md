[English](../custom-builds.md) · [Čeština](../cs/custom-builds.md) · [Deutsch](../de/custom-builds.md) · [Español](../es/custom-builds.md) · **Français** · [Italiano](../it/custom-builds.md) · [日本語](../ja/custom-builds.md) · [한국어](../ko/custom-builds.md) · [Nederlands](../nl/custom-builds.md) · [Polski](../pl/custom-builds.md) · [Português (Brasil)](../pt-BR/custom-builds.md) · [Português (Portugal)](../pt-PT/custom-builds.md) · [Русский](../ru/custom-builds.md) · [Svenska](../sv/custom-builds.md) · [Türkçe](../tr/custom-builds.md) · [Українська](../uk/custom-builds.md) · [简体中文](../zh-CN/custom-builds.md)

# Builds personnalisés

Pour qui veut un binaire plus petit : un script interactif demande quelles
langues, intégrations d'hébergeurs, thèmes, couleurs de logo et fonctions
inclure, puis ne compile que cela avec le profil optimisé. Le build normal et
la release officielle ne changent pas (tout est activé).

```
./build-custom.sh                # Linux / macOS (scripts/build-custom.sh)
build-custom                     # Windows, cmd ou PowerShell (scripts/build-custom.ps1)
```

Le résultat arrive dans `target/custom/`, jamais par-dessus le
`target/dist/` officiel. Le script affiche la taille de chaque binaire et la
différence avec `target/dist/` quand ce build existe. Il écrit aussi
`BUILD-FEATURES.txt` à côté des binaires.

## Ce que vous choisissez

| Choix | Si vous le désactivez |
|---|---|
| Langues | Seulement l'anglais (toujours compilé) plus celles que vous cochez. Le sélecteur des Paramètres ne liste que les langues compilées. |
| Thèmes | GitPoulpi Dark, GitPoulpi Light et Terminal sont toujours compilés (`system` a besoin de Dark et Light). Tous les autres thèmes se choisissent par famille (`catppuccin`, `gruvbox`, `nightfox`, ... ; `--list-themes` les affiche). Les thèmes non sélectionnés ne sont pas compilés : ils manquent dans la liste des Paramètres et dans `gitpoulpi theme list`, et une configuration qui en nomme un retombe comme pour un thème inconnu, avec l'avertissement habituel. |
| Couleurs du logo | La pieuvre en pixels de la fenêtre À propos existe en 20 coloris (`--list-logo-colors`). Choisissez ceux à compiler (au moins un), celui par défaut (À propos l'affiche en premier ; c'est aussi la valeur de repli d'un `ui.logo_color` enregistré qui n'est pas compilé ; il doit faire partie des coloris compilés ; défaut officiel : lavande) et la couleur de l'icône de l'exécutable Windows (n'importe quel coloris, défaut : celui du logo par défaut). Avec un seul coloris, le logo tourne toujours au clic mais garde cette couleur, et la liste des couleurs (`c`, clic droit) n'est pas proposée. |
| Intégrations (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit, Jira, Trello, Linear, YouTrack, Redmine) | Sans aucune : clonage par URL uniquement ; pas de tickets, pas de pull/merge requests. |
| IA | Pas de boutons ni de commandes IA, pas d'outils `ai` en CLI/MCP. |
| MCP | Pas de binaire `gitpoulpi-mcp` ni de commande `gitpoulpi mcp`. Les commandes d'agent `ui` restent. |
| LFS, Worktree, Sous-modules | Les écrans et commandes de gestion correspondants disparaissent ; une réponse typée « not built into this binary » est renvoyée. |
| Téléchargement des avatars | Seulement des pastilles d'initiales, aucun accès réseau. |
| Aperçu des images | Les fichiers image affichent l'avis binaire simple ; **Éditer avec** fonctionne toujours. |
| Radar | Pas d'onglet Radar, ni de `Alt+R` ou de commande de palette ; le Radar ne s'ouvre jamais. Nécessite une intégration (sans intégration, il est de toute façon inerte). |
| Mermaid | Pas de « Copier en gitGraph Mermaid » (menu du Log, `Shift+Y`, palette) ni de `log --format mermaid` en CLI/MCP. Pas non plus de bouton **Ouvrir le diagramme** sous un bloc `mermaid` d'une [conversation](hosting-integration.md#5c-conversations) (le bloc reste du code) ni de ligne de palette *Conversation : Ouvrir le diagramme de l'entrée*. |
| Board | Pas de vue tableau Kanban (bouton d'en-tête TICKETS, palette) ni de commande `hosting board`. Nécessite une intégration. |
| Review | Pas de vue de revue de code intégrée (menu de ligne, `Enter` sur une ligne de pull request, palette, `Review` du Radar) ni d'assistance IA à la revue. Les commandes CLI/MCP `hosting review` restent (Gerrit et les agents les utilisent). Nécessite une intégration. |
| Conversation | Pas de vue de [conversation](hosting-integration.md#5c-conversations) d'un ticket ou d'une demande dans l'application : `Enter` sur une ligne de ticket, une carte du tableau ou une ligne du Radar ouvre le navigateur comme avant, il n'y a ni ligne *Ouvrir la conversation* ni commande de palette *Conversation : ...*, ni **Écrire avec l'IA** dans une conversation (les compositeurs de la revue, du formulaire de ticket et du formulaire de pull request gardent le leur). Les commandes de conversation du CLI et du MCP (`hosting issue comments`, `comment`, `comment-edit`, `comment-delete`, `hosting review conversation`) et `ai hosting-reply` restent. Nécessite une intégration. |
| Atelier de réécriture de l'historique (réécriture avancée : auteurs, dates, messages, contenu) | Pas d'atelier de réécriture : pas de commandes de palette `Réécriture : ...`, pas d'accord, de touches ni de section d'aide de l'atelier, pas de lignes « Réécrire l'historique » dans les menus du Log, pas de bouton « Modifier l'historique en profondeur... » dans la fenêtre Amender, pas de « Créer des règles avec l'IA » ni d'IA pour les scripts, pas de limites de script dans la page Synchronisation des Paramètres et pas de blocs IA de réécriture d'historique dans les Paramètres IA. Les commandes CLI/MCP `rewrite`, `ai rewrite-script`, `ai rewrite-rules` et `ui propose rewrite` disparaissent de l'arbre des commandes. Les identités `.mailmap` (`mailmap`, `log.use_mailmap`) n'en font pas partie et restent. |
| Recherche globale | Pas de bouton Recherche globale dans la barre de titre, pas de `Ctrl+Alt+F`, pas de commande de palette « Rechercher dans l'espace de travail » et pas de commande CLI/MCP `gitpoulpi search`. La barre de recherche du Log (`ui show search`) est une autre fonction et reste. |
| Espaces de travail | La fenêtre fonctionne avec la seule liste d'onglets par défaut (« Aucun espace de travail ») : pas de pastille d'espace de travail, pas d'onglet Espaces de travail, pas de `Alt+W` / `Alt+J`, pas de commandes de palette `Espace de travail : ...`, pas de lignes « Ajouter à un espace de travail » dans les onglets, pas de commandes CLI/MCP `gitpoulpi workspace`, pas d'option `--workspace` sur `commit` et `search`. Les onglets et les sessions fonctionnent comme d'habitude. Un `workspaces.toml` écrit par un autre build n'est ni lu ni modifié ; le profil par défaut et les instructions IA d'un espace de travail ne s'appliquent pas. Le Radar et la Recherche globale couvrent les onglets ouverts. |
| Scripts de réécriture (Lua intégré) | Les règles Script d'une réécriture d'historique sont refusées par une réponse typée « non intégré à ce binaire », et aucun Lua n'est lié. L'atelier de réécriture de l'historique lui-même reste (il a son propre choix). Sans objet, et signalé par un avertissement, quand l'atelier est désactivé. |

Vos fichiers de configuration sont partagés entre les builds : un paramètre pour
une fonction non compilée est conservé et ignoré.

## Interactif

Le script Bash utilise des listes à cocher `whiptail` ou `dialog` (trois
écrans, puis un récapitulatif), ou des questions numérotées si aucun n'existe.
Le script PowerShell utilise `Out-GridView` s'il est disponible, sinon des
questions. Vos choix précédents sont pré-cochés.

## Non interactif

Tout argument saute les questions et passe directement à
`cargo run -p xtask -- build-custom` :

```
./build-custom.sh --langs fr,de --integrations github --no-ai --no-mcp
./build-custom.sh --last --dry-run         # previous choices, print the cargo command only
.\build-custom -Langs fr -Integrations none -NoAi -NoLfs -NoWorktree -NoSubmodules
```

| Option | Signification |
|---|---|
| `--langs fr,de` / `all` / `en` | Langues en plus de l'anglais. Les codes sont les noms des fichiers de langue (`pt-br`, `zh-cn`...). |
| `--integrations github,gitlab,azure-devops,bitbucket,jira,trello,gitea,linear,youtrack,redmine,gerrit,local-issues` / `all` / `none` | Fournisseurs : GitHub, GitLab (y compris auto-hébergé), Azure DevOps (Services et Server), Bitbucket (Cloud et Data Center) (hébergeurs), Gitea et Forgejo, Codeberg compris (`gitea`) (hébergeurs), Jira, Trello, Linear, YouTrack (Cloud et Server), Redmine (suivi de tickets), Gerrit Code Review (`gerrit`, revue seulement), tickets Markdown locaux (`local-issues`, un dossier ou un coffre Obsidian). `azure-devops`, `bitbucket`, `gerrit`, `gitea`, `jira`, `linear`, `local-issues`, `redmine`, `trello` et `youtrack` sont chacun une feature cargo à part ; tailles dans [binary-size](../../developer/binary-size.md). |
| `--themes all` / `none` / `nord,catppuccin` | Familles de thèmes à compiler (défaut `all`). `--list-themes` affiche les familles. |
| `--logo-colors all` / `violet,teal` | Coloris du logo à compiler (défaut `all`, gardés dans l'ordre du tableau, qui est l'ordre de rotation). `--list-logo-colors` les affiche (`*` = défaut officiel). |
| `--logo-default teal` | Le coloris par défaut ; doit faire partie de ceux compilés (défaut : lavande, sinon le premier compilé). |
| `--icon-color sun` | Couleur de l'icône de l'exécutable Windows (défaut : celui du logo par défaut). |
| `--[no-]ai`, `--[no-]mcp`, `--[no-]lfs`, `--[no-]worktree`, `--[no-]submodules`, `--[no-]avatar-fetch`, `--[no-]image-view`, `--[no-]radar`, `--[no-]mermaid`, `--[no-]board`, `--[no-]review`, `--[no-]conversation` | Chacune est activée par défaut. |
| `--[no-]history-rewrite` | L'atelier de réécriture de l'historique (réécriture avancée : auteurs, dates, messages, contenu). Activé par défaut. |
| `--[no-]global-search` | Recherche globale (bouton de la barre de titre, `Ctrl+Alt+F`, `gitpoulpi search`). Activée par défaut. |
| `--[no-]workspaces` | Espaces de travail (groupes nommés de dépôts). Activés par défaut ; désactivés, il ne reste que la liste d'onglets par défaut. |
| `--[no-]rewrite-scripts` | Lua intégré pour les règles Script d'une réécriture. Activé par défaut ; ignoré avec un avertissement quand l'atelier de réécriture est désactivé. |
| `--save <name\|file>` | Enregistre les choix résolus comme sélection enregistrée (un nom, ou un chemin de fichier). |
| `--preset <name\|file>` | Part d'une sélection enregistrée (un nom, ou un chemin de fichier) ; les autres options la remplacent, où qu'elles se trouvent dans la ligne. |
| `--list-presets`, `--delete-preset <name>` | Affiche les sélections enregistrées et leur dossier / supprime l'une d'elles par son nom. |
| `--last` | Réutilise `target/custom-build/last-choices.env` ; les autres options le remplacent. |
| `--full` | Aucune sélection (tout est activé). |
| `--profile custom\|dist\|release` | Défaut `custom` (les paramètres de `dist`, dossier de sortie séparé). |
| `--dry-run` | Affiche la commande cargo exacte et s'arrête. |
| `-- <cargo args>` | Transmis à cargo, par exemple `--target x86_64-unknown-linux-musl`. |

Noms PowerShell : `-Langs -Integrations -Themes -LogoColors -LogoDefault -IconColor -NoAi -NoMcp -NoLfs -NoWorktree
-NoSubmodules -NoAvatarFetch -NoImageView -NoRadar -NoMermaid -NoBoard -NoReview -NoConversation -NoHistoryRewrite -NoGlobalSearch -NoWorkspaces -NoRewriteScripts -Last -Preset -Save -ListPresets -DeletePreset -Full -DryRun -Profile -CargoArgs`.

La sélection des thèmes arrive dans le build via la variable d'environnement
`GITPOULPI_THEMES` (`all`, `none` ou une liste de familles séparées par des
virgules), lue par le script de build de `gitpoulpi-config`, qui retire les
autres palettes à la compilation ; définissez-la vous-même pour un simple
`cargo build` si vous préférez. `gitpoulpi
--version` affiche une ligne `themes:` avec l'ensemble compilé.

Les choix de logo sont les variables d'environnement
`GITPOULPI_LOGO_COLORS` (`all` ou une liste de slugs séparés par des
virgules), `GITPOULPI_LOGO_DEFAULT` et `GITPOULPI_ICON_COLOR`, lues par les
scripts de build de `gitpoulpi-tui` (il filtre `assets/logo/colourways.json`
pour produire le tableau qu'embarque la fenêtre À propos) et des trois crates
binaires (l'icône). La source des couleurs reste
`assets/logo/colourways.json`.

### Icône de l'exécutable Windows

Quand la cible est Windows, les exécutables `gitpoulpi`, `gitpoulpi-tui` et
`gitpoulpi-mcp` reçoivent la pieuvre comme icône, générée à la compilation
(sans Python ni ImageMagick) dans la couleur d'icône choisie, en tailles de 16
à 128 px. Elle est embarquée avec la crate `winresource` et demande le
compilateur de ressources de la plateforme (`rc.exe` du SDK Windows avec MSVC,
`windres` avec GNU). S'il manque, le build n'affiche qu'un avertissement et
l'exécutable garde l'icône par défaut. Avec la chaîne d'outils GNU, seul
`gitpoulpi` porte l'icône (la ressource serait sinon liée deux fois). Les
builds Linux et macOS ignorent tout cela, sauf le tableau des couleurs.

## Sélections enregistrées

Pour recompiler une nouvelle version sans refaire toute la sélection, enregistrez-la sous un nom et repartez-en plus tard. Les options données avec `--preset` remplacent toujours les valeurs enregistrées.

```
./build-custom.sh --no-ai --no-rewrite-scripts --save no-ai    # keep these choices as "no-ai"
./build-custom.sh --preset no-ai --dry-run                     # start from them later
./build-custom.sh --preset ~/dotfiles/gitpoulpi.env --langs fr # a file path works too
./build-custom.sh --list-presets
./build-custom.sh --delete-preset no-ai
.\build-custom -Preset no-ai -Save no-ai-v2                    # PowerShell
```

Les sélections enregistrées sont de simples fichiers `CLÉ=valeur` (le format de `last-choices.env`, lisible par un shell). Elles se trouvent dans le dossier `custom-builds` du répertoire de configuration de GitPoulpi, hors de `target/`, donc `cargo clean` ne les supprime pas :

- Linux : `$XDG_CONFIG_HOME/gitpoulpi/custom-builds/` (`~/.config/gitpoulpi/custom-builds/` par défaut)
- Windows : `%APPDATA%\GitPoulpi\config\custom-builds\`
- macOS : `~/Library/Application Support/GitPoulpi/custom-builds/`

`GITPOULPI_CONFIG_DIR` déplace ce dossier avec la configuration. `--preset` et `--save` acceptent aussi un chemin de fichier (une valeur contenant `/` ou `\`, ou finissant par `.env`), pour garder une sélection dans un dépôt de dotfiles.

Une sélection écrite par une autre version de GitPoulpi se charge quand même. Une clé inconnue, ou une langue, une intégration, un thème ou une couleur qui n'existe plus, est ignorée avec un avertissement. Une clé absente du fichier (une fonctionnalité ajoutée depuis) prend sa valeur par défaut, et une intégration ajoutée depuis est incluse ; les deux sont signalées sur la sortie d'erreur.

Quand une sélection enregistrée existe, le mode interactif propose d'abord « Partir de » (les choix précédents, une sélection enregistrée, ou tout activé), et demande à la fin s'il faut enregistrer la sélection actuelle sous un nom.

## Ce que ça économise

Mesuré sous Linux le 2026-10-29 au commit `65999ef5c`, à partir du build complet, écart par rapport à celui-ci entre parenthèses
(détails dans `docs/developer/binary-size.md`) :

| Build (à partir de `full` sauf indication) | `gitpoulpi` (Mio) | `gitpoulpi-tui` (Mio) | `gitpoulpi-mcp` (Mio) |
|---|---|---|---|
| Complet (tout activé) | 64.6 | 50.5 | 31.7 |
| Anglais seulement | 52.0 (-12.6) | 40.2 (-10.3) | 31.7 (0.0) |
| MCP désactivé | 61.6 (-3.0) | 49.6 (-0.8) | non compilé |
| IA désactivée | 62.6 (-2.0) | 48.6 (-1.8) | 30.5 (-1.3) |
| Toutes les intégrations désactivées | 56.3 (-8.2) | 43.7 (-6.7) | 26.7 (-5.0) |
| IA + intégrations + téléchargement des avatars désactivés | 52.2 (-12.3) | 39.9 (-10.6) | 23.0 (-8.8) |
| Atelier de réécriture de l'historique désactivé (`--no-history-rewrite`) | 61.6 (-3.0) | 48.9 (-1.5) | 28.5 (-3.2) |
| Scripts de réécriture désactivés | 63.1 (-1.5) | 49.0 (-1.5) | 31.1 (-0.7) |
| Recherche globale désactivée | 63.5 (-1.1) | 49.5 (-1.0) | 31.6 (-0.1) |
| Espaces de travail désactivés | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.2) |
| Radar désactivé | 63.4 (-1.2) | 49.3 (-1.2) | 31.7 (0.0) |
| Board désactivé | 63.6 (-1.0) | 49.5 (-1.0) | 31.7 (0.0) |
| Review désactivé | 63.2 (-1.4) | 49.0 (-1.4) | 31.7 (0.0) |
| Mermaid désactivé | 63.6 (-1.0) | 49.5 (-0.9) | 31.6 (-0.1) |
| LFS désactivé | 63.5 (-1.1) | 49.5 (-0.9) | 31.5 (-0.3) |
| Worktree désactivé | 63.7 (-0.9) | 49.6 (-0.8) | 31.7 (0.0) |
| Sous-modules désactivés | 63.7 (-0.9) | 49.6 (-0.9) | 31.6 (-0.1) |
| Téléchargement des avatars désactivé | 63.7 (-0.9) | 49.6 (-0.9) | 31.7 (0.0) |
| Aperçu des images désactivé | 63.3 (-1.2) | 49.2 (-1.3) | 31.7 (0.0) |
| Tout désactivé, anglais seulement (LFS conservé) | 35.2 (-29.4) | 28.9 (-21.5) | non compilé |
| **Tout désactivé, anglais seulement** | **34.9 (-29.7)** | **28.8 (-21.6)** | non compilé |

## Notes

- Un build personnalisé est lent comme tout build `dist` (LTO complet, quelques
  minutes).
- Il tourne sans sccache, comme `build-opti`.
- Les codes de langue sont vérifiés par rapport aux langues livrées ; un code
  inconnu est refusé avec la liste des codes valides.
