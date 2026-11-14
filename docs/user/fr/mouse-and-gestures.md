[English](../mouse-and-gestures.md) · [Čeština](../cs/mouse-and-gestures.md) · [Deutsch](../de/mouse-and-gestures.md) · [Español](../es/mouse-and-gestures.md) · **Français** · [Italiano](../it/mouse-and-gestures.md) · [日本語](../ja/mouse-and-gestures.md) · [한국어](../ko/mouse-and-gestures.md) · [Nederlands](../nl/mouse-and-gestures.md) · [Polski](../pl/mouse-and-gestures.md) · [Português (Brasil)](../pt-BR/mouse-and-gestures.md) · [Português (Portugal)](../pt-PT/mouse-and-gestures.md) · [Русский](../ru/mouse-and-gestures.md) · [Svenska](../sv/mouse-and-gestures.md) · [Türkçe](../tr/mouse-and-gestures.md) · [Українська](../uk/mouse-and-gestures.md) · [简体中文](../zh-CN/mouse-and-gestures.md)

# Souris et gestes

Tout ce que vous pouvez faire à la souris a aussi un équivalent clavier. Voir
[Raccourcis clavier](keybindings.md). Les noms des zones sont dans la
[visite de l'interface](interface-tour.md).

## Règles générales

- **Cliquez** sur une ligne pour la sélectionner. Cela marche dans chaque volet,
  liste, menu et boîte de dialogue.
- **Double-cliquez** sur une ligne pour l'ouvrir ou la lancer. Un double-clic sur une
  branche locale, un tag ou une pastille de ref l'extrait. Un double-clic sur une
  ligne de dépôt ouvre le dépôt.
- **Clic droit** sur une ligne pour son menu. `.`, `Shift+F10` ou la touche Menu
  ouvre le même menu pour la ligne sélectionnée.
- **La molette** fait défiler la zone sous le pointeur. Dans une liste, elle ne
  déplace que la vue. La sélection reste où elle est, même quand elle sort de la
  vue. Seul un clic ou une touche déplace la sélection.
- **Shift+molette** fait défiler latéralement, dans le diff, la vue Fichier et
  l'éditeur quand le retour à la ligne est désactivé.
- **Survolez** un bouton, une ligne ou le curseur d'une barre de défilement pour
  l'allumer. Beaucoup de terminaux ne peuvent pas changer le pointeur de la souris,
  cette lumière est donc souvent le seul signe que quelque chose est cliquable.
  Laissez le pointeur sur un bouton ou une cellule du Log pour une infobulle.
- **Cliquez en dehors** d'un champ de texte pour le quitter. Ce que vous avez tapé
  est conservé. Le filtre des Refs applique ce que vous avez tapé, comme `Enter`. La
  barre de recherche du Log se ferme, comme `Esc`.

## Fermer les fenêtres et les boîtes de dialogue

Chaque fenêtre ou boîte de dialogue encadrée a un `✕` en haut à droite de sa bordure, et les mêmes règles
valent partout :

- **Cliquez sur `✕`** pour la fermer. `Esc` fait pareil, et ferme d'abord l'élément le plus interne (une liste
  déroulante ouverte, un champ de texte, une confirmation) avant la fenêtre.
- **Cliquez à l'extérieur** d'une boîte de dialogue, d'un panneau d'aide ou d'information ou d'une fenêtre
  contextuelle pour la fermer. Un clic à l'extérieur ne ferme jamais une vue plein écran (Paramètres, vue Diff,
  vue Fichier, éditeurs, Revue, Tableau) : elles n'ont pas d'extérieur.
- Un clic **à l'intérieur** d'un panneau ne le ferme jamais.
- Un formulaire qui contient du texte saisi (un nouveau ticket, une pull ou merge request) ignore un clic à
  l'extérieur, pour qu'un clic égaré ne fasse rien perdre. `✕` et `Esc` demandent d'abord **Abandonner les
  modifications ?** ; **Continuer** revient au formulaire. Un formulaire où rien n'a été saisi se ferme
  aussitôt, tout comme une boîte de dialogue à une seule ligne.
- La palette de commandes, les sélecteurs (onglet, thème, langue), les petites boîtes de revue, de tableau,
  de fusion et du compositeur, ainsi que la confirmation portent aussi le `✕`. Les menus contextuels et les
  listes déroulantes n'en ont pas : ils se ferment avec `Esc` ou un clic à l'extérieur.

## Listes dans les Paramètres

Les listes des Paramètres (Profils, profils IA, Workflow, Ouvrir dans, Images) répondent à la souris ainsi. **Cliquez sur un sous-onglet** (Général ou la liste) pour y passer ; `Alt+Left` / `Alt+Right` fait de même.

- **Cliquez** sur une ligne pour la sélectionner, **double-cliquez** pour passer à ses champs, utilisez la
  **molette** pour défiler sans sélectionner.
- **Appuyez sur une ligne et faites-la glisser** vers le haut ou le bas pour réordonner : une barre montre où
  elle atterrira, le relâchement la dépose, `Esc` annule. L'équivalent clavier est `Alt+Up` / `Alt+Down`.
- **Cliquez sur `+ Ajouter`, `Dupliquer`, `Supprimer`, `↑`, `↓`** sous la liste. Touches : `Insert` ou
  `Ctrl+N`, `Ctrl+D`, `Delete`, `Alt+Up` / `Alt+Down` ; voir [Paramètres](settings.md#listes).

## Listes dans les fenêtres contextuelles

Ces règles s'appliquent à chaque liste d'une fenêtre contextuelle qui a un champ de
filtre : la palette de commandes, ses sélecteurs (thème, langue, couleurs du graphe),
le sélecteur d'onglets, les listes déroulantes filtrables, la liste des références de
tickets et l'aide.

- **Cliquez** sur une ligne pour la sélectionner.
- **Double-cliquez** sur une ligne pour la lancer ou la choisir, comme `Enter`.
- **La molette** fait défiler la liste. Elle ne déplace pas la sélection.
- Quand la liste est plus longue que la fenêtre, une **barre de défilement**
  apparaît sur son bord droit. Glissez le curseur ou cliquez sur la piste.

Une courte liste déroulante sans champ de filtre choisit la ligne au premier clic.
Un menu contextuel lance la ligne au premier clic. Les touches de ces listes sont
dans [Raccourcis clavier](keybindings.md#listes-de-popups-filtrables).

## Barres de défilement

Chaque zone qui peut défiler montre une fine barre de défilement sur sa bordure
quand son contenu ne tient pas : le Log, les sections Refs, les listes de fichiers,
le diff, la vue Fichier, l'éditeur et les listes des fenêtres contextuelles.

- **Glissez le curseur** pour défiler.
- **Cliquez sur la piste** pour avancer d'une page vers le clic. Dans le Log et les
  listes de fichiers, la vue saute au clic.
- **Maintenez le bouton sur une flèche d'extrémité** pour continuer à défiler. Cela
  démarre après un court délai. Sortez de la flèche pour faire une pause.
- Une barre de défilement ne déplace que la vue. Elle ne déplace jamais la
  sélection.

Dans le diff et l'éditeur, la barre de défilement verticale montre aussi tout le
fichier en petites marques : vert pour les lignes ajoutées, rouge pour les lignes
supprimées, ambre pour les lignes modifiées, `◆` pour la ligne du curseur et `▌` pour
chaque autre endroit où le texte sélectionné apparaît. L'éditeur de merge marque
chaque conflit sur ses barres de défilement. Cliquez sur une marque pour y aller.

## Échecs et Journal d'activité

- **Cliquez sur l'erreur rouge** de la ligne d'état pour ouvrir le Journal d'activité sur son entrée ; **cliquez sur Copier / le bouton suggéré** (Réessayer, Pull d'abord, Stash puis réessayer, Ouvrir les Intégrations) de l'entrée.
- **Cliquez sur Activité** à l'extrémité droite de la ligne d'état pour ouvrir le Journal d'activité. Cliquez sur un onglet pour changer, sur une case de niveau (ou sur **Tout** / **Rien**) pour afficher ou masquer ces lignes, sur une entrée de l'onglet Opérations Git pour ouvrir ses détails, sur **Effacer le journal** dans l'onglet Application (une confirmation est demandée d'abord). Faites défiler les longues lignes latéralement avec `Shift` + molette ou la barre de défilement horizontale. Voir [Le Journal d'activité](troubleshooting.md#le-journal-dactivité).
- **Cliquez sur `[x]`** de l'indicateur de tâches à droite de la ligne d'état pour annuler la tâche en cours, **cliquez sur son libellé** pour ouvrir le panneau des tâches. Le bouton **Activité** en bas à droite de l'onglet Accueil (ou `Ctrl+N Shift+G`) ouvre le Journal d'activité sur Tous les dépôts (sa zone Périmètre, une liste déroulante sous les onglets, choisit un dépôt ou un espace de travail).

## Redimensionner

- **Glissez un séparateur** entre le panneau Refs, le Log et le volet de droite. Le clavier les déplace d'un cran à la fois : `Ctrl+N Left` / `Ctrl+N Right` déplacent le séparateur Refs | Log, `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` le séparateur Log | volet de droite (le séparateur va dans le sens de la flèche).
  Glissez-le au-delà de la largeur minimale pour replier ce volet. Les équivalents au clavier sont `Ctrl+N Left` / `Ctrl+N Right` (le séparateur entre le volet Refs et le Log) et `Ctrl+N Shift+Left` / `Ctrl+N Shift+Right` (le séparateur entre le Log et le volet de droite), d'un cran chacun, le séparateur allant dans le sens de la flèche ; la palette les propose sous les noms `Navigation : Élargir le volet Refs`, `Navigation : Rétrécir le volet Refs`, `Navigation : Élargir le volet Status` et `Navigation : Rétrécir le volet Status`, ainsi que `Navigation : Replier ou déplier le volet Refs` et `Navigation : Replier ou déplier le volet Status`.
- **Glissez la bordure inférieure** d'une section Refs pour changer sa hauteur.
- **Glissez la bordure inférieure** du bloc du message de commit dans le volet de
  droite.
- **Glissez la bordure entre deux en-têtes du Log** (par défaut l'espace, depuis l'en-tête ou n'importe où le long du tableau ; Paramètres > Journal et graphe > Redimensionnement des colonnes) pour redimensionner une colonne (au clavier : `Shift+C` pour choisir une colonne, puis `]` / `[`).
- **Double-cliquez la bordure entre deux en-têtes du Log** pour ajuster la colonne à sa gauche à son contenu (au clavier : `Shift+C` pour choisir une colonne, puis `=` ; `+` ajuste toutes les colonnes). La dernière colonne a une poignée sur sa propre dernière cellule : glissez-la pour la redimensionner, la colonne Message absorbant la différence.
- **Faites défiler le Log latéralement** (la disposition de colonnes `scroll` ; Paramètres > Journal et graphe > Disposition des colonnes, ou la case `Défilement latéral` du sélecteur) : la molette horizontale ou `Shift`+molette sur le tableau, la barre de la bordure inférieure (glissez son curseur, appuyez sur la piste vide, cliquez ses flèches d'extrémité), ou `Shift+Left` / `Shift+Right`. Dans la disposition `fit` par défaut, les colonnes remplissent le volet et il n'y a rien à faire défiler.
- **Glissez un en-tête du Log** pour déplacer la colonne : un fantôme de son nom suit le pointeur, l'en-tête quitté est estompé et une barre d'accent marque l'emplacement où elle atterrirait. Les colonnes bougent au relâchement ; `Échap` annule, et le pointeur montre la main ouverte pendant que vous la portez (au clavier : `Shift+C` pour choisir une colonne, puis `<` / `>`).

GitPoulpi mémorise chaque taille. Voir
[Les tailles sont mémorisées](interface-tour.md#les-tailles-sont-mémorisées).

## Barre de titre, barre d'onglets et barre d'outils

- **Cliquez sur 🐙** dans la barre de titre pour ouvrir **À propos**. Dans À propos,
  les pupilles de la pieuvre suivent le pointeur ; cliquez sur le logo pour le faire
  tourner vers la couleur suivante, clic droit dessus pour la liste des couleurs,
  cliquez sur un lien pour l'ouvrir.
- **Cliquez sur Ouvrir dans ▾** dans la barre de titre pour la liste des programmes.
  Voir [Ouvrir dans](open-in.md).
- **Cliquez sur la version** (`v1.0.0`, ou `vX.Y.Z disponible`) dans la barre de titre pour ouvrir l'onglet Notes de version.
- **Cliquez sur la loupe et `Recherche globale`** dans la barre de titre, juste à gauche de l'engrenage, pour la [recherche globale](search.md).
- **Cliquez sur l'engrenage** dans la barre de titre, juste à gauche de la ×, pour les
  Paramètres.
- **Cliquez sur ×** à l'extrémité droite de la barre de titre pour quitter.
- **Cliquez sur un onglet** pour y passer. **Cliquez sur son `x`** (affiché sur l'onglet
  actif et sur celui qui est sous le pointeur) pour le fermer. **Faites glisser un onglet**
  sur le côté pour le réordonner. **Clic droit sur un onglet** pour le menu de l'onglet.
- **Clic du milieu sur un onglet** pour le fermer, comme son `x`.
- **Cliquez sur `‹` ou `›`** aux extrémités des onglets (affichés quand ils ne tiennent pas tous) pour faire défiler la ligne d'un onglet ; maintenez le bouton pour continuer, ou utilisez la molette sur la ligne. Faire glisser un onglet sur une flèche fait aussi défiler la ligne. La ligne défile aussi toute seule pour montrer l'onglet actif chaque fois qu'il change.
- **Cliquez sur le `+`** après le dernier onglet pour un menu : Nouvel onglet (`Alt+T`) et, tant que leurs onglets sont fermés, Radar et Espaces de travail (`Alt+J`).
- **Cliquez sur l'icône de maison** pour l'onglet Accueil.
- **Cliquez sur l'onglet Radar ou Espaces de travail** pour l'ouvrir ; son `x`, un clic du milieu ou `Ctrl+W` le ferme. **Cliquez sur la pastille d'espace de travail** (`Alt+W`) pour changer d'espace de travail.
- **Cliquez sur Quitter & cd ici** (affiché à droite des onglets quand vous avez lancé GitPoulpi par le wrapper shell `gp`) pour quitter en laissant votre shell dans le dossier de l'onglet actif. Voir [Suivre le dossier de l'onglet actif](following-the-tab-folder.md).
- **Cliquez sur un bouton de la barre d'outils** pour le lancer. **Cliquez sur `▾`** à
  côté de Pull, Push ou LFS pour sa liste. **Cliquez sur `⋯`** pour les boutons qui
  n'ont pas tenu.
- **Cliquez sur la liste déroulante de profils** à l'extrémité droite de la barre
  d'outils pour choisir l'identité de commit de ce dépôt.

## Onglet Accueil

- **Cliquez** sur une ligne pour la sélectionner. **Double-cliquez** dessus pour
  ouvrir le dépôt.
- **Cliquez sur l'étoile** pour ajouter ou retirer un favori.
- **Cliquez sur Ouvrir, Aller, Fermer ou Retirer** sur une ligne, ou sur **`⋮`** pour
  le menu de la ligne. **Clic droit** sur une ligne pour le même menu.
- **Cliquez sur un titre de section** pour la replier.
- **Cliquez sur Parcourir, Cloner, Init, Tout replier ou Tout déplier** en haut, et
  sur **Fermer tous les onglets** ou **Tout retirer** sur un titre de section.

Dans le sélecteur de dossiers **Parcourir** : cliquez sur un emplacement ou un dossier
pour y aller, double-cliquez sur un dossier pour l'ouvrir, cliquez sur le champ du
chemin pour taper un chemin. Cochez
**Afficher les dossiers voisins** (`Alt+S`, mémorisé) et l'arbre de gauche liste aussi les
autres dossiers de chaque niveau ouvert : les dossiers du chemin ouvert montrent `▾`,
les autres `▸`, et un clic sur l'un d'eux y va. Dans **Cloner** et **Init** : cliquez sur un champ pour y
taper, et cliquez sur un bouton ou une case à cocher pour l'utiliser.

## Panneau latéral Refs

- **Cliquez** sur une ligne pour la sélectionner. **Double-cliquez** sur une branche
  locale ou un tag pour l'extraire. **Double-cliquez** sur un worktree ou un
  sous-module pour l'ouvrir dans un onglet. Le **+** de l'en-tête WORKTREES crée
  un worktree ; clic droit sur un worktree pour Ouvrir dans un nouvel onglet,
  Ouvrir dans, Merge et Retirer ([Worktrees](worktrees.md)).
- **Survolez** une ligne pour afficher l'icône à sa gauche. **Cliquez** dessus pour
  masquer la ref dans le Log, ou la réafficher. Quand **Solo** est actif, le même
  endroit met la ref en solo. Sur une ligne de dossier, l'icône agit sur chaque ref du
  dossier.
- **Cliquez sur Arrêter le solo** ou **Tout afficher** dans le titre du volet
  pour effacer chaque solo ou chaque ref masquée.
- **Clic droit** sur une ligne pour son menu : checkout, merge, fast-forward, rebase,
  créer un worktree, renommer, supprimer, masquer, solo, et plus. La ligne propre d'un
  remote a **Fetch**, **Modifier**, **Retirer** et d'autres actions de remote. Un stash a
  **Pop**, **Appliquer** et **Supprimer**. Une ligne grisée dit pourquoi elle ne peut
  pas s'exécuter.
- **Cliquez sur `+`** dans le titre DISTANTES pour ajouter un distant.
- **Cliquez sur une ligne de dossier** pour la replier ou la déplier.

### Pull requests et tickets

| Geste | Action |
|---|---|
| Clic sur une ligne | La sélectionner. |
| Double-clic sur une ligne | L'ouvrir dans le navigateur. |
| Clic droit sur une ligne | Son menu : ouvrir, copier l'URL ou le numéro, extraire la branche source, créer une branche depuis un ticket. |
| Clic sur `+` dans le titre de la section | Ouvrir le formulaire **Créer une pull request** ou **Nouveau ticket**. |
| Clic sur la ligne de filtre (`Ouvertes ▾`) | Choisir quelles demandes ou quels tickets lister. |
| Clic sur un titre de groupe | Replier ou déplier le groupe. |
| Clic sur la ligne du fournisseur ou du dépôt (TICKETS) | Choisir la source des tickets. **Aucun (délier ce dépôt)** la retire. |
| Clic sur la ligne de requête (TICKETS) | Saisir une requête JQL ou WIQL (Jira, Azure DevOps), ou choisir des listes Trello. |
| Clic sur la pastille de la barre d'outils (`GH #12`, `GL !53`, ...) | Ouvrir la pull request dans le navigateur. Clic droit dessus pour **Ouvrir dans le navigateur**, **Copier l'URL**, **Actualiser** et **Changer de profil**. |
| Clic sur la pastille estompée `+ PR` | Démarrer une pull request pour la branche poussée. |

Voir [Hébergeurs et suivi de tickets](hosting-integration.md).

## Le Log

- **Cliquez** sur un commit pour le sélectionner. Ses détails apparaissent dans le
  volet de droite.
- **Ctrl+clic** ajoute un commit à la sélection ou le retire. **Shift+clic**
  sélectionne une plage (**Alt+clic** dans les terminaux qui gardent Shift+clic pour leur
  propre sélection de texte, comme kitty et Windows Terminal). Clic droit sur un commit
  sélectionné pour tous les cherry-picker. La sélection peut inclure la ligne WIP et les
  stashes ; deux lignes ou plus transforment le volet de droite en comparaison, voir
  [Comparer des commits](log-and-history.md#comparer-des-commits).
- **Cliquez sur une carte** du volet de comparaison pour amener le curseur du Log sur cette
  ligne ; **cliquez sur Inverser** pour échanger les extrémités De et À. Un simple clic
  sur une ligne quitte la comparaison.
- **Cliquez sur la ligne WIP** pour les modifications locales. **Cliquez sur son texte
  `// WIP`** pour y taper le résumé du commit.
- **Survolez** une cellule pour le texte complet : le message, l'auteur et l'e-mail, la
  date complète, le hash complet.
- **Survolez un badge `+N`** pour les autres refs du commit. Entrez dans la fenêtre
  pour cliquer sur une ref. Double-cliquez sur une ref de la fenêtre pour l'extraire.
- **Double-cliquez sur une pastille de ref** pour l'extraire.
- **Clic droit sur un commit** pour le menu du commit. Il a un sous-menu pour chaque
  ref du commit.
- **Clic droit sur une pastille de ref** pour un menu avec les actions de la ref et
  celles du commit.
- **Clic droit sur l'en-tête** pour afficher ou masquer des colonnes : une liste de cases (`[x]` / `[ ]`) avec un ✕ dans son titre ; cliquez sur une ligne ou utilisez les flèches et `Space` / `Enter`, puis fermez avec le ✕, `Échap` ou un clic ailleurs (la palette : `Affichage : Log : Choisir les colonnes à afficher`). Sous les cases : `Ajuster <colonne> à son contenu` (la colonne cliquée), `Ajuster toutes les colonnes à leur contenu` et `Réinitialiser les largeurs des colonnes`. L'équivalent clavier du clic droit : `Shift+C` pour choisir une colonne, puis `.` (ou `Shift+F10`) ouvre le même menu sur elle.
- **Les colonnes du Radar** fonctionnent de la même façon : glissez le séparateur après la colonne du dépôt, `#`, de l'auteur ou de l'âge pour la dimensionner, **double-cliquez dessus** pour ajuster la colonne à son entrée la plus large (`C` choisit une colonne, puis `=` ; `+` ajuste toutes les colonnes), et **clic droit sur le titre d'une colonne** (ou `C`, puis `.`) pour `Ajuster <colonne> à son contenu` et `Ajuster toutes les colonnes à leur contenu`.
- **Cliquez sur `↑`, `↓` ou `✕`** dans la barre de recherche pour aller à la
  correspondance précédente ou suivante, ou pour fermer la barre.
- **Cliquez sur `#12`, `ABC-12` ou `AB#12`** dans un message de commit, dans le volet
  de droite, pour ouvrir ce ticket ou cette pull request dans le navigateur.

### Sections du volet gauche

**Les dossiers** (les branches et tags nommés `a/b/c`, les fichiers du volet de droite en vue arbre) se replient et se
déplient d'un coup : **clic droit sur l'en-tête ou l'espace vide d'une section** qui a des dossiers pour **Tout déplier** /
**Tout replier** (et **Afficher/masquer les sections...**, qui mène à la liste de cases ci-dessous), et **clic droit sur une
ligne de dossier** pour **Déplier tous les sous-dossiers** / **Replier tous les sous-dossiers** (le dossier et tout ce qu'il
contient). Touches, dans le volet Refs, les listes de fichiers stagés et non stagés et les fichiers modifiés d'un commit : `=` déplie tout,
`-` replie tout, `*` (ou `Shift+Right`) déplie le dossier sous le curseur et tout ce qu'il contient, `_` (ou `Shift+Left`) le
replie. Palette : *Refs : Déplier tous les dossiers*, *Refs : Replier tous les dossiers*, *Refs : Déplier le dossier
sélectionné et ses sous-dossiers*, *Refs : Replier le dossier sélectionné et ses sous-dossiers* et les quatre jumelles
*Fichiers :* pour la liste de fichiers qui a le focus. Tant qu'un filtre de nom est actif (le filtre Refs, *Filtrer les fichiers modifiés* d'un commit), chaque dossier qui contient un résultat s'affiche ouvert, quoi que vous ayez replié, et vos propres
pliages ne sont ni modifiés ni enregistrés : effacez le filtre et ils reviennent. Un pliage fait pendant le filtre agit sur ce
qui est affiché et s'oublie avec le filtre.

**Clic droit sur l'en-tête d'une section** (une section sans dossiers, ou la partie vide du volet gauche) pour une liste de cases à cocher
de ses sections : Locales, Distantes, Worktrees, Stashes, Tags, Pull requests, Tickets et Sous-modules. Un clic
décoche ou coche une section ; le menu reste ouvert. Une section masquée n'est pas dessinée et le clavier la saute.
La dernière section visible ne peut pas être décochée. Le choix est propre à chaque dépôt et conservé d'un lancement à l'autre ; un dépôt sans choix propre suit le réglage par défaut (le `ui.sidebar_sections_hidden` global). *Utiliser pour tous les dépôts* fait du choix actuel le réglage par défaut, et *Réinitialiser* abandonne le choix propre à ce dépôt (palette : *Volet gauche : Utiliser ces sections pour tous les dépôts*). Palette : *Volet gauche : Afficher/masquer les sections...*, une commande
*Afficher/masquer* par section, *Volet gauche : Réinitialiser les sections*.

### Glisser des refs

Appuyez sur une branche dans le panneau Refs, ou sur une pastille de ref dans le Log,
et glissez-la sur une autre branche. La ligne d'état dit ce que vous portez. Une cible
qui peut recevoir le dépôt s'allume. Relâchez pour obtenir un menu. `Esc` annule le
glissement.

| Glissement | Menu |
|---|---|
| Branche locale X sur la branche locale Y | **Merge X dans Y**, **Rebase X sur Y**, **Rebase interactif X sur Y...**, **Fast-forward Y vers X** |
| Branche locale sur une branche distante | **Push X** vers ce remote |
| Branche distante sur la branche locale Y | **Merge**, **Rebase Y sur elle**, **Fast-forward Y vers elle**, et **Pull** quand c'est l'amont de Y |

Quand l'action change une branche qui n'est pas extraite, GitPoulpi l'extrait
d'abord. Si git refuse le checkout, rien ne change. Les tags ne peuvent être ni
glissés ni recevoir un dépôt.

Clavier : « marquer puis déposer » fait ce que fait un glissement, pour n'importe quelle paire de
refs. Dans le volet Refs, appuyez sur `m` sur une branche (palette : `Refs : Marquer comme source de dépôt`).
La ref affiche un `◆` et est soulignée, y compris sur ses pastilles dans le Log, et la ligne d'état
indique où aller. Sélectionnez une autre ref, dans n'importe quelle section, et appuyez sur
`Shift+M` (palette : `Refs : Déposer ici la ref marquée...`) : le même menu qu'un glissement
s'ouvre, et ses lignes s'exécutent exactement comme après un glissement. La marque est consommée
dès que le menu est ouvert ; `Échap`, ou `m` sur la ref marquée, l'efface avant. La commande de
palette `Refs : Déposer la ref sélectionnée sur la branche courante...` ouvre le menu pour la ref
sélectionnée et la branche courante.

## Le volet de droite

### Changements locaux

- **Cliquez** sur un fichier pour ouvrir son diff. **Cliquez à nouveau sur le même
  fichier** pour revenir au Log. Si le fichier est ouvert dans l'éditeur avec des
  modifications non enregistrées, GitPoulpi demande d'abord.
- **Survolez** une ligne pour afficher son bouton **Stage fichier** ou **Unstage
  fichier**. Une ligne de dossier a **Stage dossier** ou **Unstage dossier**.
- **Cliquez sur Stage de tout** ou **Unstage de tout** sur un titre de section.
- **Cliquez sur un titre de section** (`Fichiers unstaged`, `Fichiers staged`) pour la
  replier.
- **Cliquez sur Chemins ou Arbre** pour choisir une liste plate ou des dossiers.
  **Cliquez sur Tout replier** ou **Tout déplier** pour replier chaque dossier.
- **Cliquez sur `✖`** à gauche du titre pour abandonner toutes les modifications
  (demande d'abord). **Cliquez sur `✦`** à droite pour expliquer les modifications
  avec l'IA.
- **Clic droit sur un fichier** pour son menu : stage, abandon, ignorer, LFS, stasher
  le fichier, historique, blame, ouvrir dans un programme externe, copier le chemin,
  créer un patch, modifier, supprimer. Un fichier en conflit commence par les lignes
  **Résoudre avec ...**.
- **Clic droit sur une ligne de dossier** pour le menu du dossier : stage, abandon,
  ignorer, stash, créer un patch, ouvrir le dossier, y créer un fichier.
- **Clic droit sur la partie vide** de Fichiers unstaged pour **Créer un fichier**.
- **Ignorer**, **Traiter comme binaire** et **Traiter comme texte** (menus de fichier et de dossier) ouvrent d'abord la liste des fichiers `.gitignore` ou `.gitattributes` où la règle peut aller. Cliquez sur une ligne puis appuyez sur `Enter` (ou utilisez les flèches) ; `Esc` ou un clic à l'extérieur annule sans rien écrire.
- **Ctrl+clic** sur une ligne de fichier ou de dossier l'ajoute à la sélection ou la retire, **Shift+clic** (**Alt+clic**) prend une plage ; pareil dans les fichiers modifiés d'un commit et dans Voir tous les fichiers. Un clic simple recommence. **Clic droit sur une ligne sélectionnée** : le menu de tous les fichiers sélectionnés.
- **Clic droit sur la partie vide** d'une liste propose aussi **Ouvrir le dossier du dépôt** ; Unstaged ajoute **Stage de tout** et **Abandonner toutes les modifications...**, Staged ajoute **Unstage de tout**.
- **Convertir** (menus de fichier, de dossier et de sélection) réécrit l'encodage, les fins de ligne ou l'indentation des fichiers sur le disque ; un `Ctrl+Z` annule le lot.

### Le formulaire de commit

- **Cliquez** sur le Résumé ou la Description pour y taper.
- **Cliquez sur Amender le commit précédent** pour activer ou désactiver l'amend.
- **Cliquez sur Commiter N fichiers** pour commiter. Quand le bouton est grisé, un clic
  dit pourquoi (par exemple *Stagez des modifications pour commiter*).
- **Cliquez sur `✦`** à droite du Résumé pour écrire le message avec l'IA.
- **Cliquez sur `#▾`** à la fin du Résumé pour la liste des tickets. Cliquez sur l'un
  pour insérer sa référence.
- **Cliquez sur Composer les commits avec l'IA** pour ouvrir le compositeur de commits.

### Détails du commit

- **Cliquez** sur un fichier modifié pour ouvrir son diff. **Cliquez à nouveau** pour
  revenir au Log.
- **Glissez** la bordure inférieure du message pour le redimensionner. La molette sur
  le message fait défiler une longue description.
- **Clic droit** sur le résumé, la description ou le bloc de l'auteur pour *Modifier
  le résumé...*, *Modifier la description...* ou *Modifier l'auteur...* ; **double-cliquez** sur l'un d'eux (ou sur la date d'auteur) pour
  ouvrir la fenêtre de modification complète, le curseur dans ce champ. Les équivalents clavier sont `Ctrl+G Shift+E` (tous les champs) et `Ctrl+G Shift+A` (l'auteur) ; le résumé et la description sont dans la palette.
- **Clic droit** sur un fichier modifié, une ligne de dossier, la partie vide de la liste ou une sélection pour son menu (ouvrir, historique, blame, ouvrir dans un programme, copier les chemins, ignorer, traiter comme binaire ou texte, convertir, restaurer depuis le commit, supprimer). **Ctrl+clic** et **Shift+clic** sélectionnent plusieurs fichiers.

Voir [Stage et commit](staging-and-committing.md).

## Le diff et la vue Fichier

- **Cliquez sur un bouton** de la barre au-dessus du diff : Vue fichier, Vue diff,
  Blame, le sélecteur de colonne `Fichier │ Tout`, Éditer, `↑` `↓`, En ligne, Côte à côte, Hunks, Esp., Retour, Carte, Encodage. Un
  bouton activé est mis en évidence. Un clic sur `Fichier` ou `Tout` affiche cette liste dans la colonne
  des commits ; un clic sur le segment déjà sélectionné la masque (voir
  [La colonne des commits](interface-tour.md#la-colonne-des-commits)).
- **Cliquez sur `✕`** dans le titre pour fermer le fichier.
- **Survolez un hunk** pour afficher **Stage du hunk** ou **Unstage du hunk**, et
  **Abandonner le hunk**.
- **Survolez une ligne modifiée** pour afficher ses propres boutons **+ Stage**, **−
  Unstage** et **✖ Abandonner** à l'extrémité droite de la ligne.
- **Clic droit sur une ligne** pour le menu du hunk : stager, unstager ou abandonner
  le hunk ou les lignes sélectionnées, historique du fichier, blame, copier.
- **Cliquez sur la minimap** pour déplacer la vue à cette ligne. Maintenez le bouton
  enfoncé et déplacez pour défiler.

Chaque abandon demande d'abord confirmation. Quand un abandon supprime des fichiers non suivis, les dossiers qui deviennent entièrement vides disparaissent avec eux (jamais un dossier qui contient encore un fichier suivi, ignoré ou caché, ni un dossier déjà vide) ; Annuler restitue les fichiers avec leurs dossiers.

### Blame et historique du fichier

- **Blame.** Laissez le pointeur sur une annotation pour voir la fiche du hunk ; un clic sélectionne un hunk, un clic droit ouvre son menu. Faites glisser la bordure de la colonne d'annotation pour la redimensionner (enregistré). Cliquez ou faites glisser sur la minimap pour défiler.
- **Historique du fichier.** Un clic sélectionne un commit ; `Shift+clic` sélectionne une plage à comparer ; un double-clic affiche le commit dans le Log ; un clic droit ouvre le menu de la ligne ; faites glisser la bordure entre la liste et le diff pour la redimensionner ; le ✕ de la liste la masque (cliquez la bande `›` pour la ramener) ; cliquez sur `Vue fichier` / `Vue diff` / `Blame` dans la barre pour changer ce que montre le côté droit, et sur le sélecteur `Fichier │ Tout` pour choisir la liste de la colonne. La molette défile la moitié sous le pointeur. Chacun a son équivalent clavier ([Blame](log-and-history.md#blame), [Historique du fichier](log-and-history.md#historique-du-fichier)).

### La colonne des commits (liste Tout)

- **Survolez une ligne** pour l'infobulle du Log (le message entier, le nom et l'e-mail de l'auteur, la date complète, selon la partie sous le pointeur).
- **Clic droit sur une ligne** pour son menu : l'afficher dans le Log, le diff du commit pour ce fichier, le fichier à ce commit, son blame, puis le menu du Log pour ce commit. La touche menu fait de même au clavier.
- **Faites glisser un séparateur de l'en-tête** (`ui.column_resize` : par défaut la cellule de séparation d'une cellule entre deux parties, marquée par un `│` estompé dans l'en-tête et tracée en trait d'accent le long de la colonne tant que le pointeur est dessus, un glissement depuis n'importe quel point du trait redimensionnant ; avec `header`, la première lettre du nom suivant, éclairée quand le pointeur est dessus) pour dimensionner la partie à sa gauche : voies, message, auteur, date, refs, modifs ou SHA. La cellule juste après la dernière partie est le séparateur de la dernière partie.
- **Faites glisser un nom de l'en-tête** sur un autre pour y déplacer la partie : le même fantôme, la même source estompée et la même barre d'accent que l'en-tête du Log, et la partie bouge au relâchement (`Échap` annule ; les touches `<` / `>` du clavier, après `Tab`, font de même, d'une place à la fois).
- **Double-cliquez une bordure de l'en-tête** pour ajuster la partie à sa gauche à son contenu, comme les bordures du Log : la largeur qui montre sa ligne la plus large en entier, jamais plus large que la colonne (au clavier : cliquez une ligne ou `Ctrl+N` pour donner les touches à la colonne, `Tab` pour choisir la partie, puis `=` ; `+` ajuste toutes les parties ; la palette a `Ajuster la colonne à son contenu` et `Ajuster toutes les colonnes à leur contenu`). Le message garde ce que les autres parties laissent.
- **Clic droit sur l'en-tête** pour choisir les parties affichées (Ref, Graph, Auteur, Date, Modifs, SHA ; Message est toujours affiché) ; la liste de cases a un ✕ dans son titre. Dessous : `Ajuster <partie> à son contenu` (la partie cliquée) et `Ajuster toutes les colonnes à leur contenu`. La palette a `Choisir les colonnes` (le menu au clavier, sur la partie choisie par `Tab`) et une bascule par partie.
- **Shift+molette** ou une molette horizontale fait défiler les lignes sur le côté quand elles sont plus larges que la colonne.
- **Faites glisser la bordure droite** de la colonne pour la redimensionner. **Cliquez sur la bande `›`** (ou sur le ✕ de la colonne, ou **cliquez sur le segment sélectionné** du sélecteur `Fichier │ Tout` de la barre) pour la ramener ou la masquer.

### Sélectionner et copier du texte

Vous pouvez sélectionner et copier le texte du diff, de la vue Fichier, du Blame et
de l'Historique du fichier.

- **Appuyez et glissez** pour sélectionner du texte. La sélection reste sur le même
  texte quand vous faites défiler.
- **Double-cliquez** pour sélectionner un mot.
- **Un simple clic** ne sélectionne pas de texte. Il déplace le curseur ou sélectionne
  la ligne, et efface la sélection de texte.
- **`y` ou `Ctrl+C`** copie la sélection. Sans sélection, ils copient la ligne
  courante.
- **Clic droit** dans le diff pour **Copier la sélection**, **Copier la ligne** et
  **Copier le hunk**.

GitPoulpi ne copie que le texte du fichier. Les numéros de ligne, les marques `+` et
`-`, les en-têtes de hunk et les colonnes du Blame ne sont pas copiés. En mode côte à
côte, une sélection reste du côté où vous avez commencé le glissement.

Dans le diff, un glissement sélectionne aussi les lignes qu'il traverse, comme `v`.
`Enter` stage alors ces lignes. `Esc` efface d'abord la sélection de texte, puis la
sélection de lignes, puis ferme le diff.

Sélectionner du texte teinte aussi chaque autre endroit où ce texte apparaît dans le
diff et l'éditeur.

## Champs de texte

Cela s'applique à chaque champ de texte : filtres, barre de recherche, palette,
Résumé et Description, champs de boîtes de dialogue et valeurs des Paramètres.

- **Cliquez** pour y placer le curseur.
- **Appuyez et glissez** pour sélectionner du texte.
- **Double-cliquez** pour sélectionner un mot.
- Tapez pour remplacer la sélection. `Backspace` ou `Delete` la supprime.

Les touches des champs de texte sont dans [Raccourcis clavier](keybindings.md).

## L'éditeur

- **Cliquez** pour y placer le curseur. **Shift+clic** étend la sélection.
- **Glissez** pour sélectionner. Glissez au-delà du bord supérieur ou inférieur pour
  faire défiler.
- **Double-cliquez** pour sélectionner un mot.
- **La molette** fait défiler sans déplacer le curseur.
- **Cliquez sur Enregistrer, Annuler, Rétablir ou Chercher** dans la barre. En mode
  recherche, **`↑`** et **`↓`** vont à la correspondance précédente et suivante.
- **Cliquez sur `✕`** pour fermer. GitPoulpi demande d'abord quand il y a des
  modifications non enregistrées.

## Images

Voir [Images](image-view.md) pour la souris dans la vue image.

## Déplacer et redimensionner les boîtes

Les boîtes peuvent être déplacées, et les grandes fenêtres redimensionnées :

- **Glissez la barre de titre** pour déplacer une boîte (la croix de fermeture garde son clic). La palette de commandes et les sélecteurs n'ont pas de barre de titre à part : glissez la bordure du haut.
- **Glissez la bordure gauche, droite ou basse**, ou un coin du bas (la poignée `◢`), pour redimensionner une fenêtre. La boîte reste à l'écran et ne devient jamais plus petite que son minimum (60 x 16 pour l'Historique d'annulation, moins pour les autres). Une fenêtre agrandie utilise la place : zones de texte et listes grandissent, le texte se replie à la nouvelle largeur.
- Le glisser continue quand le pointeur sort de la boîte, et relâcher à l'extérieur ne la ferme jamais.
- **Glissez les séparateurs** dans la vue Reflog / Travail perdu de l'Historique d'annulation ; double-cliquez-en un pour le réinitialiser.
- Les équivalents clavier sont `Ctrl+N` puis `m` (les flèches déplacent, `Shift`+flèches redimensionnent, `0` réinitialise, `Enter` ou `Esc` quittent) et `Alt`+flèches pour les séparateurs ; la palette a `Dialogue : Déplacer/redimensionner` et `Dialogue : Réinitialiser la position et la taille`. Ils agissent sur la boîte au premier plan. `Ctrl+N` fonctionne de la même façon sur toutes les surfaces : il affiche un badge sur les contrôles de la fenêtre, de la boîte de dialogue ou de la fenêtre contextuelle au premier plan (champs, listes, onglets, boutons et croix de fermeture), la touche suivante actionne ce contrôle, `m` lance le mode de disposition et `Esc` annule ; la ligne d'état nomme `<badge>`, `m=disposer` et `Esc`. Cela vaut aussi pour la palette de commandes, les sélecteurs, la recherche d'espace de travail et l'aide, où `Ctrl+N` ne descend plus dans la liste. `Alt+G` (`palette.arrange`) est un moyen de plus, valable sur toute boîte de dialogue et toute fenêtre contextuelle, de disposer celle qui est au premier plan sans le préfixe.
- **Un seul réglage** (`ui.dialog_geometry`, Paramètres > Apparence > Disposition) couvre toutes les boîtes : chacune mémorise sa propre place et sa propre taille pour cette session, sur disque pour le prochain lancement, ou jamais.

| Boîte | Déplacer | Redimensionner |
|---|---|---|
| Historique d'annulation / Reflog / Travail perdu, Journal d'activité, panneau des tâches, sélecteur de dossier | oui | oui |
| Palette de commandes, sélecteurs d'onglet / de thème / de langue, recherche dans l'espace de travail | oui | oui (la liste grandit avec la hauteur, les lignes avec la largeur) |
| Clone (la liste des dépôts), Pull request et Nouveau ticket (la description), « Ce qui est envoyé » (le volet), Mermaid (l'aperçu), options de revue (les champs, largeur seulement) | oui | oui |
| Éditeurs de texte long des Paramètres (modèles, hooks, prompts IA), la zone IA du formulaire d'espace de travail | oui | oui (la zone de texte) |
| Réécriture de l'historique : les deux boîtes d'aide IA | oui | oui (la description, le volet envoyé) |
| À propos, panneau de mise à jour, contrôle de sécurité, conflits prévus, « Limite IA dépassée », suppression de tag, fichiers sensibles | oui | non (la taille suit le texte) |
| Init, Worktree, Sparse checkout, boîte Flow, boîtes du flux de tickets, formulaire du message de commit (reword), formulaire d'espace de travail, sélecteurs de label / jalon | oui | non |
| Confirmations (chaque « Êtes-vous sûr ? ») | oui | non |
| Boîtes de revue et de tableau : leurs petites boîtes (commentaire, terminer, choisir, colonnes) | oui | non |
| Fenêtres de la page des raccourcis (confirmation de réinitialisation, capture de touche), fenêtre de réponse IA | oui | non |
| Paramètres : les résultats et la confirmation d'authentification du dépôt | oui | non |
| Réécriture de l'historique : boîtes de purge, d'identité et de question | oui | non |
| Fenêtre de déplacement de fichier du compositeur de commit, avis de fusion | oui | non |

Non déplaçables : les écrans pleine page (Paramètres, Aide, pages Revue et Tableau, qui remplissent déjà la fenêtre), les menus contextuels et les listes déroulantes (ils pendent à ce qui les a ouverts), les infobulles, et les petites fenêtres à l'intérieur d'une page (le formulaire des remotes, l'éditeur de scission de commit).

Le pointeur prend une forme de déplacement sur la barre de titre et une forme de redimensionnement sur les bordures et les coins (terminaux avec `OSC 22`).

## Forme du pointeur de la souris

Certains terminaux laissent GitPoulpi changer le pointeur de la souris : un pointeur
de redimensionnement sur les séparateurs, sur une frontière de colonne du Log, du Radar et de la colonne des commits (aussi pendant qu'on la glisse), une main sur les éléments cliquables. Cela
marche dans kitty, foot, Ghostty, mintty et une version de WezTerm avec la PR amont #8092 (la séquence
`OSC 22`). Elle n'est envoyée que quand la forme change, dans tmux seulement enveloppée
pour le passthrough (`allow-passthrough on` ; kitty et foot l'affichent, Ghostty non), et
jamais dans screen, zellij ou herdr. Le pointeur par défaut est rétabli quand GitPoulpi
se ferme. Dans les autres terminaux (Rio, Alacritty, Tabby, iTerm2, Terminal.app, Windows
Terminal, xterm), le pointeur ne change pas ; voir
[Choisir un terminal](terminals.md#forme-du-pointeur-de-la-souris-osc-22). Le séparateur
ou le bouton sous le pointeur s'allume dans tous les terminaux.

Le même type de composant donne le même pointeur partout dans l'application.

- **Les boutons sont de vrais boutons**, dans les Paramètres (Intégrations : Modifier,
  Déconnecter, Supprimer, Parcourir, Effacer, les boutons de modèles de l'éditeur
  d'instructions...) comme dans les fenêtres : des libellés avec marge sur un fond plein
  qui s'éclaircit sous le pointeur, atténués quand ils sont désactivés, jamais un texte
  entre `[ ]`. Les seuls crochets restants sont les marques de cases `[x]` / `[ ]`, qui
  sont des cases à cocher et prennent la main comme tout autre contrôle.

- **La main** est sur les contrôles : boutons, cases à cocher et interrupteurs, listes
  déroulantes (dans les Paramètres, les volets des tickets et des pull requests, le
  Radar, les fenêtres), onglets et leurs croix, boutons `+`, liens et références
  (`#12`, `!45`), en-têtes de groupe et de dossier avec leur flèche de dépliage,
  pastilles de ref, barres de défilement, et les boutons Activité, tâches et version de
  la barre d'état. Une case ou un bouton à l'intérieur d'une ligne garde la main.
- **La flèche** est sur les lignes. Une ligne s'éclaire en entier quand le pointeur est
  dessus (la même lumière dans toutes les listes) et n'est pas un contrôle en elle-même :
  les lignes des volets Refs, Worktrees, Stashes et Submodules, le Log et la colonne des
  commits (seule une pastille de ref prend la main), les lignes d'un diff, les fichiers
  modifiés, les éléments de pull request, de ticket et du Radar, et les lignes des listes
  des fenêtres et des panneaux (cloner, sélecteur de dossiers, Workspaces, cartes du
  tableau, panneau des tâches, sélecteurs d'assigné, d'étiquette et de jalon, étapes de
  rebase, fichiers et commits de revue, choix des menus et des listes déroulantes).
- **Le curseur de texte (I-beam)** est sur les zones de saisie, sur toute la zone : les
  champs de recherche (recherche globale, Paramètres, palette de commandes, Home), les
  filtres (Refs, fichiers, Radar, Journal d'activité, recherche du volet des tickets),
  les lignes de texte, de nombre et de zone des Paramètres (aussi pendant que vous les
  modifiez), les champs des fenêtres, les zones de commentaire de revue, le compositeur
  de commits, le texte de l'éditeur de fichier et le résultat modifiable de l'éditeur de
  merge (ses volets en lecture seule gardent la flèche). Dans les détails du commit, le
  message et la description sont modifiables : l'I-beam s'y affiche et tout le bloc
  s'éclaire légèrement.
- **Dans les détails du commit**, les lignes d'information sont des contrôles : cliquez
  sur `commit: <id>` pour copier l'id court, double-cliquez sur la ligne de l'auteur ou de
  la date pour la modifier (la main et une légère marque les signalent). La ligne de
  statistiques informe seulement et garde la flèche.

Les pointeurs de redimensionnement restent sur les séparateurs et les frontières de
colonne, et la main ouverte ne s'affiche que pendant que vous portez une ref.

## Copier dans le presse-papiers

Les actions de copie placent le texte dans le presse-papiers du système de la machine où
tourne GitPoulpi et, là où le terminal est connu pour la respecter, l'envoient aussi au
terminal (OSC 52). Par SSH, ou sans affichage, seul OSC 52 est envoyé : si votre terminal
ne le prend pas en charge (iTerm2 tant que sa permission d'accès au presse-papiers n'est pas
activée, Terminal.app, GNU screen), rien n'est copié et la ligne d'état dit que le texte a
seulement été envoyé au terminal. Voir
[Choisir un terminal](terminals.md#presse-papiers-actions-de-copie).
