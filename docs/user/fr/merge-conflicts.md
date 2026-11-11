[English](../merge-conflicts.md) · [Čeština](../cs/merge-conflicts.md) · [Deutsch](../de/merge-conflicts.md) · [Español](../es/merge-conflicts.md) · **Français** · [Italiano](../it/merge-conflicts.md) · [日本語](../ja/merge-conflicts.md) · [한국어](../ko/merge-conflicts.md) · [Nederlands](../nl/merge-conflicts.md) · [Polski](../pl/merge-conflicts.md) · [Português (Brasil)](../pt-BR/merge-conflicts.md) · [Português (Portugal)](../pt-PT/merge-conflicts.md) · [Русский](../ru/merge-conflicts.md) · [Svenska](../sv/merge-conflicts.md) · [Türkçe](../tr/merge-conflicts.md) · [Українська](../uk/merge-conflicts.md) · [简体中文](../zh-CN/merge-conflicts.md)

# Résoudre les conflits de merge

Quand un merge, un cherry-pick, un revert ou un rebase s'arrête sur des
conflits, le Log affiche un bandeau (*Merge en cours · feature dans main · 2
fichiers en conflit*) et les fichiers en conflit apparaissent dans le volet
Status avec un `!` rouge. Ouvrez-en un dans l'**éditeur de merge** : A (le côté
courant) et B (le côté entrant) côte à côte, avec le **Résultat**, le fichier
que vous écrivez, en dessous. Cette page est la version courte ; la liste
complète des touches est dans [Raccourcis clavier](keybindings.md).

## L'ouvrir

- `Enter`, ou un double-clic, sur une ligne `!` du volet Status ; son menu clic
  droit commence par **Ouvrir dans l'éditeur de merge**.
- Le bouton **Résoudre les conflits...** du bandeau (il ouvre le premier fichier
  en conflit).
- La vue Diff d'un fichier en conflit : le bouton **Éditeur de merge**
  (`Shift+E`).
- La palette : **Conflits : Ouvrir l'éditeur de merge**.

Rien ne s'ouvre tout seul : une opération arrêtée n'affiche que le bandeau et
les lignes `!`.

![Le Log d'un dépôt arrêté sur un merge en conflit : le bandeau Merge en cours avec Résoudre les conflits et Annuler le merge, et deux fichiers en conflit marqués d'un ! rouge](../images/conflict-banner.png)

L'éditeur prend toute la largeur de la fenêtre : le volet
Refs et le volet de droite sont masqués tant qu'il est ouvert et
reviennent quand vous le fermez (`F9` les affiche ou les masque entre-temps). A
et B sont côte à côte dès que l'éditeur fait au moins
`merge.side_by_side_min_width` colonnes de large (100 par défaut, Paramètres >
Diff et éditeur ; `0` signifie toujours) ; plus étroit, vous avez un côté à la
fois derrière des onglets. Les longues lignes défilent latéralement, A et B
ensemble (`Shift+←` / `Shift+→`, Shift+molette ou une molette latérale), elles ne
sont jamais repliées.

![L'éditeur de merge : A (courant) et B (entrant) côte à côte au-dessus du Résultat, conflit 1 pris dans A et conflit 2 dans B, conflit 3 restant à choisir](../images/merge-editor.png)

## Choisir

Chaque conflit est un bloc encadré dans A et dans B, avec une case à cocher sur
son en-tête et un `+` / `✓` devant chaque ligne.

| Pour... | Touche | Souris |
|---|---|---|
| prendre la ligne sous le curseur (côté ciblé) | `Space` | cliquer sur la ligne |
| prendre tout le bloc du côté ciblé | `Enter` | cliquer sur le `[ ]` de son en-tête |
| prendre A seulement / B seulement pour ce conflit | `a` / `b` | clic droit sur le bloc |
| prendre les deux, A puis B / B puis A | `c` / `Shift+C` | clic droit sur le bloc |
| ne rien prendre | `x` | clic droit sur le bloc |
| échanger l'ordre de deux côtés choisis | `s` | cliquer sur la pastille `1st` / `2nd` |
| tout prendre de A / tout prendre de B (chaque conflit) | `Shift+A` / `Shift+B` | barre d'outils |
| désélectionner ce conflit (retour à non résolu) | `u` dans A / B, `Alt+U` partout | clic droit sur le bloc, **Désélectionner (non résolu)** |
| réinitialiser tout le fichier à ses conflits d'origine (demande d'abord si vous avez tapé dans le Résultat ; `Ctrl+Z` le ramène) | `Shift+U` dans A / B, `Alt+R` partout | **Réinitialiser** à côté de Tout prendre |
| conflit suivant / précédent | `F7` / `Shift+F7`, `Alt+Down` / `Alt+Up` (`n` / `Shift+N` dans A / B) | `▼` / `▲`, ou cliquer sur une marque d'une barre de défilement |
| un nombre répète un mouvement | `3j`, `2k`, `2n`, `3N` (dans A / B) | |
| fichier en conflit suivant / précédent | `Alt+PageDown` / `Alt+PageUp` | `›` / `‹` dans l'en-tête |
| afficher ou masquer le volet Base (ancêtre commun) | `Alt+D` (`d` dans A / B) | `Base` |
| afficher le fichier entier dans A et B (**Fichier entier**) | `Alt+F` (`f` dans A / B) | `Fichier entier` |
| donner plus de largeur à A ou B / revenir à des largeurs égales | `Alt+Right` / `Alt+Left`, `Alt+0` | glisser le séparateur ; double-clic dessus |
| faire défiler A / B / Base latéralement | `Shift+←` / `Shift+→` (`Shift+H` / `Shift+L`) | Shift+molette, molette latérale |
| afficher ou masquer les volets Refs et Status | `F9` | |
| cibler A, B, le Résultat | `F6` / `Shift+F6` | clic |
| aller à un volet, sur le conflit où vous êtes | `Alt+1` A, `Alt+2` B, `Alt+3` Résultat, `Alt+4` Base ; ou `Ctrl+N` puis `1` / `2` / `3` | |
| passer au volet voisin (A, Base, B, Résultat), même conflit | `h` / `l` ou `←` / `→` dans A / B | |
| faire défiler la hauteur du Résultat | `z` | glisser son titre |
| annuler / rétablir (choix et saisie, une seule pile) | `Ctrl+Z` / `Ctrl+Y` | |
| écrire le fichier, le garder en conflit | `Alt+S` | **Enregistrer** |
| écrire le fichier et le marquer résolu | `Ctrl+S` | **Enregistrer et marquer résolu** |
| fermer | `Esc` | `✕` |

Chaque barre de défilement (A / B / Base, et le Résultat) porte une marque par
conflit : orange tant qu'il demande un choix, vert une fois traité, bleu pour le
conflit courant. Un clic sur une marque saute à ce conflit (une marque sous le
curseur de la barre de défilement est déjà à l'écran, le clic y fait donc
glisser le curseur).

**Fichier entier** montre tout le fichier dans A et B avec chaque différence colorée, pas seulement les conflits :

![Le mode Fichier entier : tout le fichier dans A et B, avec les modifications colorées](../images/merge-editor-full-file.png)

Le Résultat est un vrai éditeur : cliquez dedans et tapez, sélectionnez,
coupez et collez, cherchez avec `Ctrl+F`. Un choix sur un conflit que vous avez
modifié à la main remplace votre modification (`Ctrl+Z` la ramène).
Enregistrer avec un conflit où rien n'est choisi, ou avec une ligne qui
ressemble encore à un marqueur de conflit, demande d'abord confirmation.

### ✦ Suggérer une résolution (IA)

Avec l'IA activée (Paramètres > IA ; voir [Assistance IA](ai.md)), la barre
d'outils a un bouton **✦ Suggérer**, le menu clic droit d'un conflit a *✦
Suggérer une résolution (IA)*, et `Alt+I` et la palette (`IA : Suggérer une résolution de conflit de merge`) font de même. Cela
ne travaille que sur le conflit courant et ne se lance que sur demande. La
réponse remplace la partie du Résultat de ce conflit en une seule modification,
marquée modifiée (`✎`), de sorte que `Ctrl+Z` remet le conflit tel qu'il était.
Elle n'est jamais enregistrée ni stagée à votre place. Pendant qu'elle tourne,
la ligne d'aide affiche le temps écoulé ; `Esc` annule, et un échec s'y affiche
aussi. Avec l'IA désactivée, aucun de ces contrôles n'existe.

Après **Enregistrer et marquer résolu**, le fichier est stagé et la liste des
fichiers en conflit revient ; ouvrez vous-même le fichier
suivant. `Ctrl+Z` (ou le bouton Annuler) reprend le dernier « marquer résolu ».
Quand il ne reste aucun fichier, le bandeau propose **Commiter le merge** (ou
**Continuer** pour un cherry-pick, un revert ou un rebase) et **Annuler le
merge**, qui demande d'abord confirmation : il remet HEAD, l'index et l'arbre de
travail là où ils étaient.

Un merge dont le résultat est égal à HEAD (vous avez pris notre côté partout) n'a
rien de stagé et aucun fichier modifié, mais il peut quand même être conclu :
**Commiter le merge** ouvre la saisie du commit avec le message de merge, et le
commit a ses deux parents. De même, le bouton Commit du compositeur est actif
sans rien de stagé tant qu'un merge est arrêté.

Un cherry-pick ou un revert vide (la modification est déjà appliquée) ne peut pas
être commité : git le refuse, et **Continuer** le dit et renvoie vers **Passer**.
**Passer** (bouton du bandeau, entrée de palette *Git : Passer le cherry-pick ou le revert*, CLI
`conflicts skip`) demande d'abord confirmation, puis termine l'opération sans
rien enregistrer pour elle ; HEAD, l'index et l'arbre de travail reviennent à
HEAD. Passer n'est pas annulable (rien n'est perdu qui n'ait été abandonné
exprès) ; un merge n'a rien à sauter.

## Les conflits qui ne sont pas du texte

Un fichier binaire, un pointeur Git LFS, un fichier de plus de 8 Mio, un fichier
supprimé d'un côté, un reste de renommage, un lien symbolique, un changement de
mode ou un sous-module n'ont pas de lignes à choisir. L'éditeur affiche une
**carte** avec ce que contient chaque côté et un bouton par choix ; choisir
résout et stage le fichier aussitôt.

| Carte | Choix |
|---|---|
| Fichier binaire, pointeur LFS, trop gros | **Garder A** (`a`), **Prendre B** (`b`), **Prendre la base** (`o`, quand il y en a une), **Ouvrir dans l'outil de merge** (`m`) |
| Supprimé d'un côté, modifié de l'autre | **Garder le fichier** (`k`), **Accepter la suppression** (`Shift+D`) |
| Reste de renommage (le chemin n'existe que d'un côté) | **Garder le fichier** (`k`), **Supprimer le fichier** (`Shift+D`) |
| Lien symbolique, sous-module, mode du fichier | **Garder A**, **Prendre B**, **Prendre la base** |

Quand le fichier est une image (PNG, JPEG, GIF, WebP, BMP, ICO), les cases
montrent aussi les images A, B et Base, ajustées et en lecture seule, dans une
fenêtre assez large pour elles ; voir [Images](image-view.md#dans-léditeur-de-merge).

![La carte d'un conflit binaire : ce que contiennent A, B et la base, et les boutons Garder A, Prendre B, Prendre la base et Ouvrir dans l'outil de merge](../images/merge-editor-card.png)

`Esc` ferme la carte sans choisir. L'outil de merge externe est
`general.external_merge_tool` dans Paramètres > Diff et éditeur (`{base}`, `{local}` = A,
`{remote}` = B, `{merged}` ; vide signifie `git mergetool`) ; le fichier est
stagé quand l'outil se termine avec succès.

## Prévention des conflits

GitPoulpi peut vous prévenir *avant* un merge. Quand la branche sur laquelle vous êtes entrerait en conflit avec la
branche dont elle est partie (sa **base**), une bande apparaît sur la ligne de bordure supérieure du Log :
**En conflit avec origin/main dans 2 fichiers**, avec un bouton **Afficher les fichiers**. Elle prend la prochaine
ligne de bordure libre après les bandeaux des opérations en cours : quand l'un d'eux est affiché, elle peut attendre
d'avoir la place. Rien n'est fusionné : le démon fait un merge en mémoire des deux commits (libgit2, ou
`git merge-tree --write-tree` avec git 2.38 ou plus récent ; un git plus ancien désactive la fonctionnalité avec une
ligne dans le journal), vos fichiers, l'index et les refs restent donc intacts.

- **La base** est la branche cible de la pull request de la branche quand GitPoulpi la connaît déjà (le cache que
  tient la section Pull requests ; aucun appel réseau n'est fait pour cela), sinon la branche, parmi les motifs des
  **Branches de base** (Paramètres > Prévention des conflits), qui a le moins de commits depuis la bifurcation. En cas
  d'égalité, la branche dont la pointe est la plus proche de la bifurcation l'emporte, puis une branche de suivi
  distant sur une branche locale, puis le nom. La branche elle-même et sa propre copie distante ne sont jamais la base.
- **La fenêtre** (cliquez sur la bande, ou la commande de palette **Conflits : Afficher les conflits prévus**, utilisable
  au clavier) liste les fichiers, avec **Ouvrir l'aperçu du merge** et **Rebase sur la base**. Ce sont les propres
  lignes de menu de la branche de base (*Merge <base> dans <branche>*, *Rebase <branche> sur <base>*) lancées pour
  vous : journalisées, donc `Ctrl+Z` les annule, et un merge qui entre bien en conflit s'arrête dans l'éditeur de
  merge comme n'importe quel autre. Gauche et Droite passent d'un bouton à l'autre, Enter en appuie un, Esc ferme.
- **Quand cela se lance** : en arrière-plan, environ 2 secondes après que le dépôt s'est calmé (un commit, un checkout,
  un fetch qui a déplacé une branche, un rebase), une fois par paire `(HEAD, base)`. La bande se masque dès que HEAD
  bouge et revient, ou non, avec la nouvelle réponse. Un HEAD détaché n'est pas vérifié.
- **Listes de branches de base** : l'origine des branches de base d'un dépôt se choisit sur sa page **Prévention des conflits**. **Par défaut** suit la liste globale par défaut (le `**` intégré, toutes les branches, tant que vous n'en choisissez pas une autre dans Paramètres > Prévention des conflits) ; **Liste : <nom>** suit l'une de vos listes nommées ; **Liste propre** est une liste à lui. Les listes sont partagées : modifiez-en une et chaque dépôt qui la suit applique le changement à la prochaine vérification, sans redémarrage. **Enregistrer comme liste...** transforme les motifs propres d'un dépôt en liste nommée, **Revenir au défaut** abandonne le choix du dépôt. Supprimer une liste qu'un dépôt suit lui fait utiliser le défaut.
- **Désactivation** : **Activer la détection proactive des conflits** dans Paramètres > Prévention des conflits, ou pour un seul dépôt
  sur sa page **Prévention des conflits**. Depuis un script, `gitpoulpi conflicts check` affiche la même réponse en JSON
  ([référence CLI](cli-reference.md)).

## Agents

Un agent peut vous montrer un conflit (`gitpoulpi ui show conflict <path> [--conflict
N]` : l'éditeur s'ouvre dessus, rien n'est choisi) ou **proposer** une résolution
(`gitpoulpi ui propose resolution <path> --pick 1=a,2=ba` ou `--content-file F`,
ce qui demande *Laisser les agents IA proposer des modifications à confirmer*). Une proposition ouvre
l'éditeur avec les choix (ou tout le texte) déjà dans le Résultat et un bandeau
violet *✦ Proposed by claude-code*. Relisez-la, changez ce que vous voulez et
appuyez sur **Ctrl+S** : seul cela écrit ou stage quelque chose. Fermer
l'éditeur rejette la proposition. Voir [MCP et agents](mcp-and-agents.md).

Depuis un script, les mêmes opérations sont `gitpoulpi conflicts list | show | resolve |
unresolve | state | abort` ([Référence CLI](cli-reference.md)) ; `conflicts check` les prédit avant un merge.
