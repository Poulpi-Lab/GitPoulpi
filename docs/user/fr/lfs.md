[English](../lfs.md) · [Čeština](../cs/lfs.md) · [Deutsch](../de/lfs.md) · [Español](../es/lfs.md) · **Français** · [Italiano](../it/lfs.md) · [日本語](../ja/lfs.md) · [한국어](../ko/lfs.md) · [Nederlands](../nl/lfs.md) · [Polski](../pl/lfs.md) · [Português (Brasil)](../pt-BR/lfs.md) · [Português (Portugal)](../pt-PT/lfs.md) · [Русский](../ru/lfs.md) · [Svenska](../sv/lfs.md) · [Türkçe](../tr/lfs.md) · [Українська](../uk/lfs.md) · [简体中文](../zh-CN/lfs.md)

# Git LFS

Git LFS (« Large File Storage ») garde les gros fichiers (fichiers de design, vidéos, jeux de données) hors de l'historique d'un dépôt : git stocke à la place un petit *pointeur* texte de trois lignes, et le vrai contenu vit sur le serveur LFS et dans un cache local sous `.git/lfs`.

GitPoulpi n'a besoin d'aucun compte pour tout cela. LFS utilise les identifiants que votre distant utilise déjà (votre assistant d'identifiants git, ou ssh-agent).

## Ce qui demande le programme `git-lfs`, et ce qui n'en a pas besoin

GitPoulpi lance le programme standard `git-lfs` pour tout ce qui convertit ou déplace des fichiers (install, track, fetch, pull, push, verrous).

Lire l'*état* LFS ne demande aucun programme : si un dépôt utilise LFS, quels fichiers sont des pointeurs, leur taille, et si le vrai objet est déjà stocké localement.

| | demande `git-lfs` |
|---|---|
| `gitpoulpi lfs status` (signale le programme comme absent au lieu d'échouer) | non |
| `gitpoulpi lfs files` | non |
| install / track / untrack / prune | oui |
| fetch / pull / push | oui |
| verrous | oui |

## Installer `git-lfs`

- **Windows** : il est fourni avec Git for Windows ; ou `winget install GitHub.GitLFS`.
- **Debian / Ubuntu** : `apt install git-lfs`.
- **macOS** : `brew install git-lfs`.
- Tout autre système : https://git-lfs.com

La version 3.0 ou plus récente est requise. Après l'installation, GitPoulpi s'en aperçoit en moins d'une minute (ou immédiatement avec `lfs status`).

## Quand `git-lfs` manque

GitPoulpi ne plante ni ne quitte jamais à cause de cela :

- `gitpoulpi lfs status` réussit et affiche `git-lfs: not installed`, avec le lien d'installation.
- Une opération qui demande le programme échoue avec un message clair qui se termine par le lien d'installation (`https://git-lfs.com`) et un code de sortie non nul ; via MCP, l'outil renvoie ce message.
- Un dépôt qui utilise LFS s'ouvre quand même ; ses gros fichiers apparaissent simplement comme des pointeurs tant que le programme n'est pas installé.

## Regarder les fichiers LFS

```
gitpoulpi lfs status            # program, patterns, filters, hook
gitpoulpi lfs files             # pointer files in the index, with size and local/not downloaded
gitpoulpi lfs files --at v1.0   # ... as of a revision
gitpoulpi show big.bin --at HEAD  # reports "Git LFS object, 12.3 MB, sha256:..." instead of pointer text
```

## Configurer un dépôt : install, track, untrack

```
gitpoulpi lfs install                 # filters + hooks for THIS repository (git lfs install --local)
gitpoulpi lfs install --user          # into your global git config instead (never done silently)
gitpoulpi lfs track '*.psd'           # route *.psd through LFS
gitpoulpi lfs track '*.psd' --lockable   # ... and make them read-only until locked
gitpoulpi lfs track '*.bin' --renormalize  # ... and re-stage already-committed *.bin as pointers
gitpoulpi lfs untrack '*.psd'
gitpoulpi lfs prune                   # delete old, already-pushed objects from the local store
```

- `install` écrit par défaut dans le `.git/config` et les hooks du dépôt lui-même, de sorte que votre configuration git globale n'est jamais touchée sauf si vous passez `--user`.
- `track` et `untrack` modifient `.gitattributes` et le laissent **non stagé** : il apparaît comme une modification ordinaire de l'arbre de travail que vous commitez vous-même. Les fichiers ajoutés *après* le suivi sont stockés comme pointeurs par leur prochain commit.
- Les fichiers **déjà commités** comme contenu ordinaire restent ordinaires dans l'historique. `--renormalize` les re-stage pour que votre prochain commit les remplace par des pointeurs. Pour les déplacer aussi dans chaque commit passé, utilisez la règle de réécriture d'historique *Migrer des fichiers vers LFS* (voir [Réécriture d'historique](history-rewrite.md#déplacer-des-fichiers-vers-ou-hors-de-git-lfs)) : elle est propre à GitPoulpi, ce n'est pas `git lfs migrate`.
- Mettez le motif entre guillemets pour que votre shell ne l'étende pas.
- Via MCP, ce sont des mutations ordinaires (`lfs_install`, `lfs_track`, `lfs_untrack`, `lfs_prune`), soumises au contrôle `[mcp]` de `config.toml`.

## Déplacer les objets : fetch, pull, push

```
gitpoulpi lfs fetch [--remote origin] [--all]   # download objects into the local store; working tree untouched
gitpoulpi lfs pull  [--remote origin]           # fetch, then replace pointer files with the real content
gitpoulpi lfs push  [--remote origin] [--all]   # upload the current branch's objects (or every ref with --all)
```

- Le distant par défaut est celui du dépôt, comme pour `fetch` et `push`.
- Les identifiants sont ceux habituels de votre distant (assistant d'identifiants, ssh-agent). GitPoulpi ne demande jamais de compte propre.
- `pull` installe les filtres LFS de ce dépôt s'ils manquent : sans eux, git-lfs télécharge les objets mais ne remplace pas les pointeurs, et un checkout ultérieur remettrait les pointeurs.
- MCP traite `lfs push` comme `push` : un outil destructif (il modifie le distant).
- La fenêtre affiche la progression d'un transfert (fichiers et octets) sur la ligne d'état. La CLI attend le résultat.

### Récupérer le contenu : `lfs checkout`

```
gitpoulpi lfs checkout [<path>...]   # offline: write real content over the pointer files, from the local store
```

Il ne télécharge jamais (c'est `lfs pull`) ; les objets absents du stockage local restent des pointeurs. Comme `pull`, il installe d'abord les filtres LFS du dépôt quand aucun n'est configuré. Comme toute opération LFS, il n'est pas annulable ; voir [Annuler et rétablir](undo-redo.md). Via MCP, c'est `lfs_checkout` (une écriture, non destructive).

### Le bouton LFS de la barre d'outils, et pull / push

La barre d'outils a un bouton **`LFS ▾`** après **Pop** (masqué quand `git.lfs_enabled` est désactivé, et dans un dépôt où LFS n'est ni initialisé ni utilisé par son `.gitattributes` : voir [Paramètres > LFS](settings.md#propre-au-dépôt--lfs)). Un clic ouvre un menu déroulant : **Fetch tous les fichiers LFS** (`lfs fetch --all`), **Checkout de tous les fichiers LFS** (`lfs checkout`), **Pull tous les fichiers LFS** (`lfs pull`), **Push tous les fichiers LFS** (`lfs push --all`, les objets de chaque ref) et **Élaguer le stockage LFS local** (`lfs prune`). Chaque ligne est aussi une commande de la palette (`LFS : Fetch de tous les objets`, `LFS : Checkout de tous les fichiers`, `LFS : Pull des objets`, `LFS : Push de tous les objets`, `LFS : Élaguer les objets locaux`). `LFS : Ouvrir le menu` ouvre le menu déroulant. Chacune a un enchaînement `Ctrl+G` : `l` pour le menu, puis `Shift+F`, `Shift+K`, `Shift+L`, `Shift+J` et `Shift+V` pour les lignes. Vous pouvez les changer dans *Paramètres > Clavier et souris*. Quand `git-lfs` manque, chaque ligne mène à la boîte d'installation.

![Le menu LFS de la barre d'outils : fetch, checkout, pull et push de tous les fichiers LFS, et élagage du stockage local](../images/lfs-menu.png)

**Pull et push gèrent LFS tout seuls.** Dans un dépôt qui utilise LFS, avec `git-lfs` installé, un simple `pull` vérifie d'abord que les filtres LFS du dépôt sont installés, et un simple `push` vérifie son hook `pre-push`. Quand l'un manque, GitPoulpi lance `git lfs install --local` (ce dépôt seulement) avant le transfert, et la ligne d'état affiche « filtres LFS installés ». Sans le hook, un push n'enverrait aucun objet LFS. Désactivez cela avec `git.lfs_autofix_filters = false` (*Paramètres > Git et encodage > Installer les filtres LFS avant pull et push*). Un simple `push` n'envoie toujours que les objets de la branche poussée, jamais ceux de chaque ref ; l'élagage n'est jamais implicite.

## Verrous de fichiers

Certains fichiers ne se fusionnent pas (un fichier Photoshop, un niveau de jeu, une ressource binaire). Les *verrous* Git LFS permettent à une personne de dire « je suis en train de le modifier » : personne d'autre ne peut le verrouiller ni y pousser de modifications tant qu'il n'est pas déverrouillé.

```
gitpoulpi lfs locks  [--remote origin]              # who holds which file (yours are marked)
gitpoulpi lfs lock   assets/hero.psd [--remote origin]
gitpoulpi lfs unlock assets/hero.psd [--remote origin] [--force]
```

- Le verrouillage est une fonction du **serveur LFS** (GitHub, GitLab et Gitea l'ont). On l'atteint avec les identifiants propres du distant ; GitPoulpi n'a pas de compte propre.
- Un serveur sans verrouillage fait échouer `lock` avec un message qui le dit (le `LfsLockingUnsupported` typé). `locks` sur un tel serveur ne liste rien, ce que git-lfs lui-même signale.
- Suivre un motif avec `--lockable` (`lfs track '*.psd' --lockable`) rend ces fichiers en lecture seule tant que vous ne les verrouillez pas.
- `unlock` du verrou d'un autre est refusé ; `--force` le brise, c'est pourquoi MCP traite `lfs_unlock` comme un outil destructif. `lfs_locks` est en lecture seule.

## Cloner un dépôt LFS

`gitpoulpi clone <url>` (et la boîte Cloner de la TUI) extrait le dépôt *sans* lancer les filtres LFS, puis, quand le dépôt utilise LFS et que `git-lfs` est installé, télécharge les objets avec `git lfs pull` (en installant les filtres de ce dépôt si aucun n'est configuré) et l'affiche comme une phase à part, *Téléchargement des objets LFS*. `--no-lfs` saute cette étape et laisse des pointeurs.

Un clonage **n'échoue jamais à cause de LFS** : avec `git-lfs` absent, ou un téléchargement qui échoue, le clonage réussit quand même et les gros fichiers restent des pointeurs ; l'état LFS du dépôt en donne la raison, et `gitpoulpi lfs pull` termine le travail une fois le problème réglé.

## Dans l'interface du terminal

- **L'étiquette `LFS`.** Un fichier stocké avec Git LFS porte une petite étiquette `LFS` après son nom dans les listes Unstaged et Staged du volet Status et dans la liste des fichiers modifiés d'un commit. **LFS uniquement** (`o` avec le volet Status ou le Log au focus, la case de la barre d'outils des listes de fichiers, ou *Fichiers : Basculer LFS uniquement* dans la palette) restreint ces listes aux fichiers LFS. La case se trouve à l'extrémité droite de la ligne de filtre (volet Status : dans la barre d'outils, ou dans son menu de débordement quand il est étroit) ; dans la liste des fichiers d'un commit, elle se réduit à `[ ] LFS` sur un volet étroit au lieu de disparaître. Elle n'apparaît que si les fichiers listés en contiennent. Dans le volet Status et dans les fichiers modifiés d'un commit, un fichier compte comme LFS quand son chemin est un pointeur dans l'index (un seul appel `git` groupé par rafraîchissement, rien pour un dépôt sans attributs `filter=lfs`). Avec **Tous les fichiers** activé dans le Log, l'arbre entier à la révision sélectionnée est listé et les fichiers LFS sont ceux de cette révision, lus dans ses propres blobs (mis en cache par révision, hors du thread de rendu, sans programme `git-lfs`), de sorte qu'un fichier LFS à cette révision mais absent de l'index est étiqueté et conservé par le filtre.
- **La carte de diff.** Ouvrir le diff d'un fichier LFS affiche une carte au lieu du texte du pointeur : `Git LFS object`, `1.2 MB -> 1.4 MB`, les oids court ancien et nouveau, et si le nouvel objet est stocké localement ou `not downloaded`. Appuyez sur `r` pour le diff brut du pointeur, `r` encore pour revenir. Seul le stage au niveau du fichier s'applique à un fichier LFS (il n'y a ni hunks ni lignes à stager). Sous la carte, **Ouvrir par défaut** et **Ouvrir dans** (`o`, `Shift+O`, ou la palette) ouvrent le vrai fichier, jamais le pointeur : voir [Ouvrir dans](open-in.md#fichiers-git-lfs).
- **La vue Fichier** d'un fichier LFS affiche `Git LFS object, 12.3 MB, sha256:ab12cd34...` et où récupérer le contenu.
- **Le bandeau.** Pour un dépôt qui utilise LFS alors que `git-lfs` est absent ou antérieur à 3.0, un bandeau du volet Status le dit. Cliquez dessus (ou utilisez l'entrée de palette *LFS : Ouvrir la page d'installation*) pour ouvrir https://git-lfs.com. Rien d'autre ne change : la fenêtre s'ouvre, les diffs fonctionnent, les gros fichiers apparaissent simplement comme des pointeurs.
- **Le menu du fichier et la palette.** Faites un clic droit sur un fichier modifié dans le volet Status : *Suivre avec LFS : ce fichier* / *cette extension* pour un fichier ordinaire ; *Verrouiller le fichier (LFS)*, *Déverrouiller le fichier (LFS)* ou *Forcer le déverrouillage* pour un fichier LFS, selon qui détient son verrou. La ligne d'un fichier verrouillé dit `locked by you` / `locked by <name>`. Tout est aussi une commande de la palette (`Ctrl+P`, tapez `LFS`) : initialiser, suivre ou ne plus suivre un motif, fetch, pull ou push des objets, élaguer, afficher les verrous, ouvrir la page d'installation.
- **Une action qui échoue s'explique.** Si une action LFS échoue parce que `git-lfs` manque, une boîte de dialogue porte le message et un bouton *Ouvrir la page d'installation*.
- **Paramètres.** *Paramètres > LFS* (par dépôt) indique si LFS est initialisé, liste les motifs suivis et propose **Configurer LFS ici**. *Git et encodage* (par dépôt) contient l'interrupteur **Git LFS**, une ligne en lecture seule **Programme git-lfs** et un bouton **Initialiser LFS pour ce dépôt**.
- **Git LFS** désactivé (`git.lfs_enabled = false`, voir la [référence de configuration](config-reference.md)) masque tout cela. Les filtres LFS propres à git continuent de tourner.
- L'état LFS est lu peu après l'ouverture du dépôt, après chaque changement, lors d'un rafraîchissement manuel, quand `.gitattributes` change, et toutes les dix secondes pour un dépôt qui utilise LFS.

## Limites connues

- Un dépôt dont les motifs LFS ne se trouvent que dans un `.gitattributes` *imbriqué* (ni celui de la racine, ni `.git/info/attributes`) n'est détecté comme utilisant LFS qu'une fois qu'il possède un stockage d'objets LFS (`.git/lfs`), pas avant.
- `git lfs migrate` lui-même n'est pas utilisé. Déplacer des fichiers vers ou hors de LFS sur tout l'historique se fait avec les règles de réécriture d'historique *Migrer des fichiers vers LFS* / *Ramener des fichiers depuis LFS* : elles travaillent sur le clone local, seul le `.gitattributes` racine est écrit, et les objets doivent ensuite encore être poussés (*Envoyer les objets LFS* dans le panneau de résultat, ou `lfs push --all`).
