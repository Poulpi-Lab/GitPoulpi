[English](../log-and-history.md) · [Čeština](../cs/log-and-history.md) · [Deutsch](../de/log-and-history.md) · [Español](../es/log-and-history.md) · **Français** · [Italiano](../it/log-and-history.md) · [日本語](../ja/log-and-history.md) · [한국어](../ko/log-and-history.md) · [Nederlands](../nl/log-and-history.md) · [Polski](../pl/log-and-history.md) · [Português (Brasil)](../pt-BR/log-and-history.md) · [Português (Portugal)](../pt-PT/log-and-history.md) · [Русский](../ru/log-and-history.md) · [Svenska](../sv/log-and-history.md) · [Türkçe](../tr/log-and-history.md) · [Українська](../uk/log-and-history.md) · [简体中文](../zh-CN/log-and-history.md)

# Travailler avec le Log

Le Log est le centre de GitPoulpi : tout l'historique sous forme de graphe, avec
chaque branche, tag et stash dessus. Cette page couvre sa lecture, la recherche
dedans, et tout ce que vous pouvez faire aux commits et aux refs. Les zones sont
nommées dans la [visite de l'interface](interface-tour.md) ; les touches exactes
sont dans [Raccourcis clavier](keybindings.md) et les gestes de souris dans
[Souris et gestes](mouse-and-gestures.md).

![Le Log avec un commit de merge sélectionné : ses détails et ses fichiers à droite](../images/log-graph.png)

## Lire le graphe

Chaque ligne est un commit, du plus récent au plus ancien. Un point se place sur
la voie du commit ; des lignes relient un commit à ses parents ; un merge montre
deux lignes qui se rejoignent. Les pastilles à gauche nomment les branches et les
tags de ce commit, dans la couleur de la voie, avec un trait fin jusqu'au point.
Une icône d'ordinateur portable marque une branche locale, un globe une branche
distante, et `✓` la branche que vous avez extraite. `+N` signifie que d'autres
refs pointent sur ce commit ; survolez-le pour les lister, ou utilisez le menu
ci-dessous.

La première ligne est celle des **modifications locales** (un cercle en pointillés :
votre travail non commité, relié au commit sur lequel il repose). Les stashes
apparaissent comme de petits carrés sur une voie à côté du commit sur lequel ils
ont été faits.

### Couleurs, teinte de ligne et avatars

Les couleurs des voies viennent du thème. *Paramètres > Apparence > Thème de
couleurs du graphe* combine n'importe quel thème avec n'importe quelle palette de
voies, et le sélecteur de la palette *Application : Changer le thème de couleurs du graphe...*
(`Ctrl+N Shift+T`) prévisualise chaque palette en direct ; voir
[Thèmes](themes.md). *Fond des voies du graphe* réglé sur **Teinte de ligne**
peint une bande légère de la couleur de la voie derrière la colonne du graphe et
une fine barre à son bord. *Force de la teinte* (**Forte**, **Moyenne** (par défaut),
**Subtile**) règle à quel point la bande est estompée.

| Palette de voies Serie avec teinte de ligne | Palette de voies Material avec teinte de ligne |
|---|---|
| ![Le graphe du Log dans le thème de couleurs Serie avec teinte de ligne](../images/theme-serie.png) | ![Le graphe du Log dans le thème de couleurs Material avec teinte de ligne](../images/theme-material.png) |

Les *Avatars des auteurs* (une image dans le point du graphe ou dans la colonne
Auteur) sont désactivés par défaut. Les activer envoie un hash de l'adresse e-mail
de chaque auteur à Gravatar, et interroge votre hébergeur. *Sources d'avatars* et
*Avatar de repli* sur la même page choisissent d'où viennent les images et ce
qui est dessiné quand il n'y en a pas.

Si votre terminal ne peut pas afficher d'images, le graphe est dessiné avec des
caractères. *Rendu du graphe* et *Protocole d'image* en forcent l'un ou l'autre ;
voir [Dépannage](troubleshooting.md).

La surbrillance de la ligne sélectionnée ou survolée commence au cercle du
commit. Les voies à sa gauche gardent le fond normal.

### Colonnes

Faites un clic droit sur l'en-tête pour choisir les colonnes : **Ref**, **Graph**,
**Message** (toujours active), **Auteur**, **Date**, **Modifs** et **SHA**. La
colonne facultative **Modifs** montre, pour chaque commit, `n files +ajoutées
-supprimées`. Glissez la bordure entre deux en-têtes pour redimensionner une
colonne ; le choix est mémorisé, en cellules de terminal. Avec la valeur par défaut `ui.column_resize = "line"`, la bordure est l'espace d'une cellule à la fin d'une colonne : un petit repère y repose sur la ligne d'en-tête, et quand le pointeur est sur cet espace (sur l'en-tête ou n'importe où le long du tableau) un trait fin d'accent est tracé sur toute la hauteur du tableau et un glissement depuis n'importe quel point du trait redimensionne (le pointeur devient un pointeur de redimensionnement dans les terminaux qui le gèrent). Avec `"header"`, il n'y a ni repère ni trait : le premier caractère de la cellule d'en-tête après la bordure s'allume au survol et le glissement part de là, sur la ligne d'en-tête seulement (Paramètres > Journal et graphe > Redimensionnement des colonnes). Les lignes sont identiques dans les deux modes : l'espace fait partie de la colonne à sa gauche, si bien que la sélection et les teintes de ligne le traversent. Ref, Graphe et les autres colonnes
gardent leur largeur quand la fenêtre est redimensionnée ou agrandie : seule **Message** s'étire
ou se rétrécit (dans une fenêtre très étroite, les autres colonnes cèdent temporairement de la
place, Graphe en premier, et retrouvent vos largeurs quand la fenêtre s'agrandit). Glissez la colonne Graphe jusqu'à sa largeur minimale, un point de commit, et le graphe devient compact : chaque commit est sur une seule voie, les uns sous les autres, sans lignes de branche ni de merge. Une cellule de plus et l'arbre revient. Avec les images des auteurs dans les ronds de commit (Paramètres > Apparence > Avatars : Graphe), le rond fait deux cellules de large : la colonne s'arrête donc une cellule plus tôt (trois cellules) et ne coupe jamais une image.

Glisser un en-tête montre un fantôme de son nom qui suit le pointeur et une barre d'accent là où la colonne atterrirait ; l'ordre change au relâchement, et `Échap` annule. Le sélecteur ouvert par le clic droit est une liste de cases (`[x]` / `[ ]`) avec un ✕ dans son titre ; on l'atteint aussi depuis la palette (`Affichage : Log : Choisir les colonnes à afficher`), on s'y déplace avec les flèches et on coche avec `Space` / `Enter`.

*Ajuster au contenu.* Un **double-clic** sur la bordure d'une colonne ajuste la colonne à sa gauche à son contenu visible le plus large et à son en-tête (les caractères larges et les emoji sont comptés selon les cellules qu'ils occupent). Dans la disposition ajustée, le Log remplit toujours son volet : une colonne s'arrête donc à ce que le volet peut lui donner une fois que **Message** garde sa part et chaque autre colonne sa largeur minimale ; la ligne d'état dit alors `Auteur demande 34 cellules, 20 tiennent.` Même chose au clavier : avec une colonne choisie (`Shift+C`, `Tab`), `=` l'ajuste et `+` ajuste toutes les colonnes ; la palette a `Affichage : Colonne du Log : Ajuster la colonne à son contenu` et `... Ajuster toutes les colonnes à leur contenu` ; le sélecteur du clic droit propose `Ajuster <colonne> à son contenu` (la colonne sous le pointeur), `Ajuster toutes les colonnes à leur contenu` et `Réinitialiser les largeurs des colonnes` sous ses cases. **Message** est la colonne flexible et n'a pas de largeur propre à ajuster : elle prend la place que les autres laissent.

*Défilement latéral.* Par défaut, chaque colonne tient dans le volet. Dans la disposition **défilement** (la case `Défilement latéral (les colonnes gardent leur largeur)` du sélecteur, Paramètres > Journal et graphe > Disposition des colonnes, la commande `Affichage : Log : Activer ou désactiver le défilement latéral des colonnes` de la palette, ou `s` quand une colonne est choisie) les colonnes gardent leur largeur, **Message** comprise (elle a alors une largeur propre, et l'ajuster prend son résumé le plus large), rien n'est sacrifié quand la fenêtre est étroite, et le tableau peut être plus large que le volet. Une barre sur la bordure inférieure le fait alors défiler : glissez le curseur, appuyez sur la piste vide pour amener le curseur sous le pointeur, ou utilisez ses flèches d'extrémité. Au clavier, `Shift+Left` / `Shift+Right` font défiler de quatre cellules (la palette a `Affichage : Log : Faire défiler les colonnes vers la gauche` / `vers la droite`) ; la molette horizontale ou `Shift`+molette sur le tableau font de même, et choisir une colonne (`Shift+C`, `Tab`) la fait défiler dans la vue. Un tableau plus étroit que le volet laisse un espace vide à droite, et la bande d'une ligne sélectionnée va jusqu'à la bordure. Chaque disposition garde ses propres largeurs (`ui.log_column_widths` et `ui.log_column_widths_scroll`) ; le premier glissement, la première touche ou le premier ajustement dans la disposition défilement enregistre chaque colonne à la largeur qu'elle a, si bien que seule celle que vous changez bouge. Dans la disposition ajustée, un ajustement qui ne tient pas dans le volet le dit et renvoie à cette disposition.

*La poignée de la dernière colonne.* La dernière colonne a aussi une poignée, sur sa propre dernière cellule (pas la bordure du volet, qui redimensionne toujours les volets) : un glissement la déplace de la distance parcourue par le pointeur, **Message** absorbant la différence. **SHA** fait 8 cellules par défaut (les 7 chiffres et cette cellule de poignée).

Au clavier, `Shift+C` (ou `Affichage : Colonne du Log : Choisir une colonne à déplacer ou à dimensionner` dans la palette) choisit une colonne, dont la cellule d'en-tête s'allume. Ensuite `Tab` / `Shift+Tab` en choisissent une autre, `<` / `>` la déplacent d'une place vers la gauche ou la droite parmi les colonnes affichées (l'équivalent de faire glisser son en-tête), `]` / `[` l'élargissent ou la rétrécissent (l'équivalent de faire glisser sa limite ; la dernière colonne déplace son bord gauche), `0` rend toutes les largeurs automatiques, et `Échap` ou `Enter` quittent. Les flèches continuent de parcourir les commits entre-temps. L'ordre et les largeurs sont mémorisés comme après un glissement, et appartiennent au Log seul : la colonne des commits à côté d'un diff garde sa propre liste.

![Le Log avec la colonne Modifs affichée : chaque commit indique son nombre de fichiers et de lignes ajoutées et supprimées](../images/log-changes-column.png)

### Auteurs et `.mailmap`

Quand le dépôt a un `.mailmap` (le fichier de git qui donne un seul nom et une
seule adresse à une personne qui a commité sous plusieurs identités), la colonne
**Auteur** du Log, les détails du commit, le Blame et l'Historique d'un fichier
affichent l'identité **canonique**, comme `git log`. Les commits eux-mêmes ne
changent pas. Le réglage `log.use_mailmap` (Paramètres > Journal et graphe,
activé par défaut) le désactive pour afficher les noms enregistrés. La
recherche par auteur porte sur le nom et l'adresse tels qu'affichés : chercher
le nom canonique trouve donc tous les commits de la personne. L'onglet
**Identités** de l'atelier d'historique liste les noms et adresses de
l'historique et écrit les lignes de `.mailmap` à votre place, voir
`history-rewrite.md` ; depuis un terminal : `gitpoulpi mailmap list|check|add`.

## Chercher

- **Recherche** (`f`, le bouton **Chercher** de la barre d'outils, ou
  *Navigation : Rechercher des commits...* dans la palette) : cherche dans tout le message de commit, le
  nom et l'e-mail de l'auteur, le hash et les noms de refs. Elle estompe ce qui
  ne correspond pas et garde la forme du graphe. `Enter` / `Shift+Enter` sautent à
  la correspondance suivante / précédente.
- **Filtre d'auteur** (l'entonnoir dans l'en-tête **Auteur**, `Shift+A` dans le Log,
  ou *Journal : Filtrer par auteur...* dans la palette) : préfiltre le Log selon
  l'auteur des commits. Une fenêtre contextuelle s'accroche sous l'en-tête avec un
  champ de recherche, une ligne **Effacer le filtre** (✕) et une ligne par auteur de
  l'historique chargé : un avatar (quand `ui.avatars` n'est pas désactivé : la photo de l'auteur sur les
  terminaux qui affichent des images, une fois la photo prête, le badge d'initiales
  d'ici là et sur les autres terminaux), le nom puis l'e-mail en sourdine, les plus gros contributeurs d'abord
  puis par nom. Taper restreint la liste avec la correspondance floue de la
  palette ; `Enter` ou un clic coche l'auteur en surbrillance (une coche ; choisissez-en
  autant que vous voulez), `Esc` ou un clic à côté ferme la fenêtre et garde le
  filtre. Le filtre se comporte exactement comme la recherche : les commits des
  auteurs choisis restent éclairés, toutes les autres lignes sont estompées
  et le graphe garde sa forme, rien n'est retiré ni rechargé. Avec la barre de
  recherche ouverte aussi, les deux s'appliquent (un commit doit être d'un auteur choisi
  **et** correspondre au texte), et `Enter` / `Shift+Enter` parcourent ces résultats.
  Tant que le filtre est actif, l'entonnoir devient un `▼` plein de la couleur d'accent,
  son infobulle liste les auteurs choisis, et quand la colonne Auteur est masquée ou
  trop étroite pour l'entonnoir, la bordure supérieure du Log affiche plutôt
  *Auteur : Alice Example +1* (la touche et la palette fonctionnent toujours). Pour
  l'enlever : la ligne **Effacer le filtre**, un clic droit sur l'entonnoir allumé,
  *Journal : Effacer le filtre d'auteur* dans la palette, ou `Esc` dans le Log (après une
  sélection, que `Esc` abandonne en premier ; dans la fenêtre, `Esc` vide d'abord le champ de
  recherche, puis ferme la fenêtre et garde le filtre).
  Tant qu'il est actif, Haut/Bas (et PageUp/PageDown, Début/Fin) déplacent la sélection uniquement entre les commits éclairés, en sautant les estompés ; un clic sélectionne toujours n'importe quelle ligne.
  Les auteurs sont ceux des commits que le Log a chargés (mailmap appliqué, comme
  dans la colonne), et le filtre est mémorisé par dépôt, d'un changement d'onglet à
  l'autre et d'un redémarrage à l'autre. Une recherche de texte n'est pas mémorisée,
  le filtre l'est.
- **Filtre des refs** (`/` dans le panneau latéral) : restreint les listes de
  branches, de tags et de distants par nom.
- **Solo** (la touche `o`, ou le menu de la ref, sur une ref, un dossier ou un
  distant) : n'affiche que les commits des refs en solo ; mettez-en plusieurs en
  solo pour voir leur union. `Shift+O` arrête le solo.
- **Masquer** une ref (son menu) pour la sortir du graphe : sa pastille disparaît,
  et les commits que seule elle atteint aussi (le commit extrait reste toujours,
  avec la ligne de l'arbre de travail). *Masquées n* dans le titre de Refs les
  compte, **Tout afficher** (`Shift+H`) les ramène. Le total de la ligne d'état
  compte l'historique visible.

## Branches, tags et le panneau Refs

Double-cliquez sur une branche locale ou un tag pour l'extraire. Faites un clic
droit sur une ref (dans le panneau latéral, ou sa pastille dans le Log) pour son
menu. Les entrées les plus utiles :

| Entrée | Fait |
|---|---|
| Révéler dans le graphe | Fait défiler le Log jusqu'au commit de la ref (en chargeant d'abord l'historique plus ancien si besoin) et fait clignoter la ligne ; la sélection reste. Un simple clic sur une ref fait la même chose, sauf si *Un clic sur une ref la révèle dans le graphe* est désactivé dans les Paramètres. Une ref masquée du graphe (ou par Solo) le dit dans la ligne d'état à la place |
| Checkout | Passe sur la branche ou se détache sur le tag |
| Merge, Rebase sur | Fait un merge de cette ref dans la branche extraite, ou un rebase de la branche extraite sur elle |
| Fast-forward `<branche>` vers `<courante>` / `<courante>` vers `<branche>` | Avance une branche quand c'est un fast-forward ; sinon grisé avec la raison (`diverged, not a fast-forward`) |
| Créer un worktree depuis... | Ouvre un worktree lié dans son propre onglet |
| Pull, Push, Fetch, Distants de la branche... | Opérations réseau sur cette branche (Distants de la branche... ouvre « Distants de <branche> » : l'amont plus une case Push et une case Pull aussi par distant) |
| Masquer / Afficher dans le graphe, Solo | Affichage seulement ; masquent ou mettent en solo une ref (masquer retire aussi les commits que seule cette ref atteint) sans toucher au dépôt |
| Renommer, Supprimer | Supprimer demande d'abord ; son jumeau distant peut aussi être supprimé |
| Copier le nom de la branche, Copier le lien de la branche sur le distant | Vers le presse-papiers |

**Déplacer un tag.** Glissez un tag sur une autre ref (voir *Glisser des refs* dans
[Souris et gestes](./mouse-and-gestures.md)) ou utilisez *Refs : Déplacer le tag
sélectionné vers le commit sélectionné...*. Il garde son type et son annotation,
seul votre tag local bouge (un distant garde l'ancienne position jusqu'à un push), et
une annulation le remet en place. En ligne de commande : `gitpoulpi tag move <name> <target> [--force-non-ff]`,
`gitpoulpi tag merge-branch <name> <branch> [-m <message>]`.

**Où se trouve un tag.** Un tag peut être sur cet ordinateur, sur un distant, ou les deux, et
les lignes TAGS du panneau latéral (et la pastille du tag dans le Log) l'indiquent par un marqueur
après le nom. Survolez un tag pour le même renseignement en toutes lettres.

| Marqueur | Signification |
|---|---|
| flèche vers le haut | Ici seulement : aucun distant ne l'a. Son menu propose *Push* |
| flèche gauche-droite | Ici et sur un distant, au même commit |
| flèche vers le bas (ligne estompée) | Sur un distant seulement : pas dans ce dépôt (jamais récupéré, ou supprimé ici). Son menu propose *Fetch `<tag>` depuis `<remote>`* et *Supprimer `<tag>` sur `<remote>`...* ; `Git : Fetch le tag sélectionné` dans la palette |
| flèche haut-bas | Aussi sur un distant, mais à un autre commit (un push simple serait refusé). Son menu a deux lignes par distant de ce type : *Utiliser `<tag>` de `<remote>`* (le tag local se déplace vers la version du distant, d'après le dernier fetch, sans réseau ; un tag annoté devient ce même objet tag) et, en rouge, *Push `<tag>` vers `<remote>` (forcé)* (demande confirmation ; avec un bail sur ce que le dernier fetch a vu, donc un distant qui a bougé depuis est refusé avec « fetch first » et rien n'est écrasé). Chacune est une entrée d'annulation. *Git : Utiliser la version distante du tag sélectionné* et *Git : Push forcé du tag sélectionné vers son distant* dans la palette (le distant par défaut quand plusieurs diffèrent) ; en ligne de commande : `tag take-remote`, `tag force-push` |

git ne garde aucune ref de suivi distant pour les tags : GitPoulpi apprend ce que chaque distant
possède par votre dernier **fetch** (n'importe lequel, l'automatique compris) et par ses propres
pushes et suppressions de tags. Rien n'interroge le réseau pendant que vous naviguez ; si quelqu'un
d'autre a modifié les tags d'un distant, faites un fetch pour le voir. Les mêmes informations se
trouvent dans `gitpoulpi tag list` (champs `local`, `remote`, `differs_on`, `where`).

**Supprimer un tag.** Quand un distant a le tag, le bouton rouge **Supprimer** ouvre une boîte
avec trois lignes numérotées (`1`-`9`, ou Haut/Bas et `Enter` ; `Esc` annule), et la ligne
en surbrillance s'explique juste dessous :

1. **Supprimer en local** : le retire de ce dépôt seulement. Cela s'exécute aussitôt, et
   le message dit que le distant l'a encore. **Le prochain fetch le rapporte** (git suit
   les tags du distant), sauf si vous le supprimez aussi sur le distant.
2. **Supprimer sur `<remote>`** : le retire de là-bas pour tout le monde (`git push <remote>
   --delete`). Une confirmation qui nomme le distant est demandée. Votre tag local
   reste. Cette action est irréversible.
3. **Supprimer en local et sur `<remote>`** : les deux, après la même confirmation, de sorte
   qu'un fetch ne puisse plus le rapporter. **Annuler ne restaure que le tag local** (le même
   objet tag) ; il redevient alors local seulement et *Push* le remet sur le distant.

![La boîte de dialogue Supprimer le tag avec ses trois lignes, Supprimer en local, Supprimer sur origin et Supprimer en local et sur origin, au-dessus de la section Tags du panneau des refs où chaque tag porte un marqueur : local seulement, ou aussi sur origin](../images/tag-delete.png)

Avec plusieurs distants, chacun a ses propres lignes. Un tag présent ici seulement garde la
confirmation simple ; un tag présent sur un seul distant passe directement à la confirmation de ce distant.
CLI : `gitpoulpi tag delete <name> [--local | --remote <r> | --both [--remote <r>]]`.

**Fetch et tags.** Un fetch rafraîchit aussi ce que chaque distant possède (les marqueurs), et
le suivi des tags propre à git s'applique toujours : un tag supprimé seulement ici est récupéré de nouveau.
Un tag que quelqu'un a **supprimé sur le distant** disparaît du côté distant du
marqueur après n'importe quel fetch, mais reste ici comme votre tag local (désormais *ici seulement*),
parce que git n'élague pas les tags par défaut. **Paramètres > Synchronisation > Élagage automatique des
tags** (désactivé par défaut ; `gitpoulpi fetch --prune-tags`) supprime aussi un tel tag local,
mais seulement s'il avait été sur le distant : un tag que vous n'avez jamais poussé, ou que vous avez
déplacé, n'est jamais touché. Demande git 2.41 ou plus récent pour savoir ce qui a été élagué.

**Nouvelle branche ou nouveau tag.** Les boutons **Branche** et **Tag** de la barre
d'outils, `Ctrl+G b` et `Ctrl+G t`, ou *Créer une branche ici...* d'un commit
demandent le nom dans une zone en ligne dans la colonne Ref, sur la ligne du
commit visé. `Enter` le crée, `Esc` annule, et un nom que git refuserait est
signalé en rouge sans fermer la zone.

Sur un commit sans ref, survoler sa ligne affiche aussi, dans la colonne Ref, une pastille estompée avec le nom de la branche à laquelle il appartient (sinon sa branche distante), déduit de l'historique ; un tag n'est jamais montré en fantôme. Une branche supprimée après une fusion s'affiche quand même, barrée, sous le nom du sujet du commit de fusion (`Merge branch 'x'`, `Merge pull request #N from owner/x`...) ; les squash, rebases et fast-forwards ne laissent pas cette trace. Survoler la pastille indique son type. Elle ne crée rien : utilisez la commande de création de branche ci-dessus. **Afficher une branche fantôme sur le commit survolé** dans Paramètres > Journal et graphe la désactive ; rien n'est dessiné là où la colonne Ref est trop étroite.

**Distants.** Le `+` dans le titre DISTANTES en ajoute un ; clic droit sur le nom
d'un distant pour le fetcher, copier son lien web, en faire le distant par défaut,
le masquer du graphe, modifier ses URL ou le retirer. **Masquer un distant** (le ⊙
de sa ligne, l'entrée de menu *Masquer le distant du graphe*, ou `Distant : Masquer
le distant du graphe...` dans la palette) masque toutes les branches qu'il a ou
aura, et les commits que seul lui atteint quittent le Log : le moyen de parcourir
un historique sans celui d'un distant d'archive sans rapport, et de le ramener avec
*Afficher le distant dans le graphe*. Afficher un distant affiche aussi ses branches
qui avaient été masquées une par une. Une branche d'un distant masqué le dit sur la
ligne d'état si l'on clique son ⊙ : affichez d'abord le distant.

**Stashes.** **Stash** fait un stash de tout (fichiers non suivis compris) ;
`Ctrl+G m` demande d'abord un message (avec l'IA activée, `Ctrl+G` dans cette
saisie en suggère un). Clic droit sur une ligne de stash, dans le Log ou la
section STASHES, pour **Pop**, **Appliquer** (garde le stash) ou **Supprimer**.

**Worktrees.** *Créer un worktree depuis...* (sur une branche, un tag ou un
commit) ouvre une boîte de dialogue qui propose un dossier à côté du dépôt, un nom
facultatif de nouvelle branche et « Ouvrir le nouveau worktree dans un onglet ».
Les worktrees apparaissent dans la section WORKTREES ; double-cliquez sur l'un
pour l'ouvrir, et son onglet se ferme tout seul quand le worktree est supprimé. La fonctionnalité complète (ouvrir dans un autre outil, merger la branche d'un worktree, retirer) est dans [Worktrees](worktrees.md) ; la section SUBMODULES du même volet est dans [Sous-modules](submodules.md).

## Glisser des refs sur des commits

Vous pouvez appuyer sur une branche (dans le panneau
latéral ou sa pastille dans le Log) et la glisser. Une étiquette fantôme suit le
pointeur et les cibles valides s'allument ; relâchez pour obtenir un menu :

- une branche locale **sur une autre branche locale** : Merge, Rebase, Rebase
  interactif, Fast-forward ;
- une branche locale **sur une branche distante** : Push vers ce distant ;
- une branche distante **sur une branche locale** : Merge, Rebase, Fast-forward,
  Pull.

`Esc` annule un glissement. L'équivalent au clavier est la commande de palette
*Refs : Déposer la ref sélectionnée sur la branche courante...*.

**Merge ou fast-forward.** Quand la cible n'a aucun commit à elle (la branche glissée est juste en avance), un merge pourrait simplement faire avancer la cible. Par défaut (Paramètres > Git > **Merge fast-forward** = *never*, ou le choix propre à un dépôt sur sa page Options de merge), la ligne du menu affiche *Merge x dans y (commit de merge)* et crée un vrai commit de merge, pour que l'historique garde la branche ; utilisez la ligne séparée *Fast-forward de y vers x* pour un simple déplacement (pour une ligne droite, utilisez plutôt *Rebase*). Avec *allowed*, la ligne Merge affiche *(fast-forward)*, et avec *only* *(fast-forward seul)*, grisée avec la raison quand les branches ont divergé. Les mêmes lignes et libellés se trouvent dans le menu d'une branche, et `gitpoulpi merge --ff | --no-ff | --ff-only` choisit pour chaque merge.

## Le menu du commit

Faites un clic droit sur un commit (ou appuyez sur `.` dessus) pour son menu :

![Le menu du commit : cherry-pick, revert, reset, modifier le message, expliquer avec l'IA, rebase interactif, créer une branche, tag, checkout, worktree, patches et actions de copie](../images/context-menu.png)

| Entrée | Fait |
|---|---|
| **Cherry-pick sur `<branche>`** | Rejoue le commit sur la branche extraite |
| **Revert du commit...** | Crée un commit qui l'annule. Sur un commit de merge, une liste déroulante demande d'abord quel parent est la ligne principale (le côté qui reste ; le parent 1 est la branche sur laquelle le merge a été fait), puis la confirmation habituelle. Annuler restaure l'état précédent |
| **Reset de `<branche>` à ce commit** | Un sous-menu pour le type de reset ; un reset hard demande confirmation |
| **Modifier le message du commit...** | Voir plus bas |
| **Modifier...** | Modifie le message, le nom, l'e-mail et la date de l'auteur du commit dans une seule fenêtre ; voir [Modifier un commit](#modifier-un-commit-message-auteur-dates) |
| **Scinder le commit...** | Découpe le commit en plusieurs ; voir [Scinder un commit](#scinder-un-commit) |
| **Scinder le commit (IA)...** | Pareil, avec un regroupement proposé par le modèle ; seulement quand l'[IA](ai.md) est activée |
| **Expliquer les modifications (IA)** | Une fenêtre avec une explication de l'IA ; seulement quand l'[IA](ai.md) est activée |
| **Rebase interactif depuis ici...** | Voir plus bas |
| **Créer une branche ici...**, **Créer un tag ici...**, **Créer un tag annoté ici...** | Zone de nom en ligne |
| **Checkout de ce commit**, **Créer un worktree ici...** | HEAD détaché, ou un worktree |
| **Créer un patch depuis ce commit...**, **d'ici jusqu'à `<branche>`...** | Écrit des fichiers `.patch` ; *Appliquer un patch* est dans le menu d'une ref |
| **Copier le lien du commit sur le distant**, **Copier** > | Vers le presse-papiers (hash, message et plus) |

## Modifier un message de commit (reword)

*Modifier le message du commit...* (ou `Ctrl+G e`) ouvre le message complet : un
résumé d'une ligne avec un compteur, une description dessous, **Annuler** et
**Mettre à jour le message**. N'importe quel commit de la branche extraite peut
être reformulé (HEAD ou un ancêtre) ; les commits au-dessus sont recréés avec
leurs propres arbres, auteurs et dates, les merges restent des merges, et vos
modifications stagées et non stagées sont laissés tranquilles. Si le commit a déjà
été poussé, la fenêtre avertit en orange qu'il faudra un force push (avec lease),
et demande une seconde confirmation. `Ctrl+Z` (Annuler de la barre d'outils)
reprend le reword ; voir [Annuler et rétablir](undo-redo.md). La ligne d'état
nomme aussi l'ancien sommet.

Un commit qui n'est pas sur la branche extraite (il vit sur une autre branche) ne peut pas être reformulé ni modifié
là où vous êtes : *Modifier le message du commit...* et *Modifier...* ouvrent donc **Modifier un commit d'une autre branche**,
une liste de choix (`Up` / `Down` ou `Ctrl+J` / `Ctrl+K` se déplacent, `Enter` ou un clic choisit, `Esc`, la croix ou
**Annuler (A)** la ferme) :

- **Basculer sur `<branche>` et modifier (1)** : une ligne par branche locale dont l'historique contient le commit
  (`1` à `8`, les huit premières, les autres sont comptées). La branche est extraite de la façon habituelle, puis la
  modification s'ouvre dessus. L'annulation est celle des deux actions : `Ctrl+Z` annule la modification, le basculement
  reste un basculement.
- **Modifier avec la réécriture avancée (R)** : ouvre l'[atelier de réécriture d'historique](history-rewrite.md) sur ce commit,
  dans les versions qui l'ont.
- **Créer une branche ici... (B)** : seulement quand aucune branche locale ne contient le commit
  (il n'est que sur une branche distante, par exemple) ; ouvre la zone de nom d'une branche sur lui.

## Modifier un commit (message, auteur, dates)

*Modifier...* dans le menu du commit (ou `Ctrl+G Shift+E`, ou *Commit : Modifier...* dans la palette) ouvre une seule fenêtre pour les propriétés du commit : le résumé, la description, le nom, l'e-mail et la date de l'auteur, chacun prérempli avec les valeurs du commit. Changez ce que vous voulez puis appuyez sur **Appliquer** ; seuls les champs modifiés sont envoyés. Une date s'écrit `YYYY-MM-DD HH:MM:SS +HH:MM` et s'affiche en UTC tant que vous ne la modifiez pas. Le committer, c'est vous, maintenant (comme pour un reword). Tout commit de la branche extraite peut être modifié, avec les mêmes règles, le même avertissement pour un commit déjà poussé et la même seconde confirmation que *Modifier le message du commit...* ; les commits au-dessus sont recréés avec leurs propres auteurs et dates, et vos modifications, indexées ou non, restent telles quelles. `Ctrl+Z` (Annuler dans la barre d'outils) annule l'opération. Dans les builds avec la réécriture d'historique, **Modifier l'historique en profondeur...** (`Alt+H`), en bas à gauche de la fenêtre, la ferme sans rien appliquer et ouvre l'[atelier de réécriture d'historique](history-rewrite.md) sur ce commit et tous les plus récents ; si vous avez modifié un champ, il demande d'abord (`Esc` confirme). `Ctrl+N` affiche un badge sur chaque champ et chaque bouton de la fenêtre ; appuyez dessus pour placer le curseur dans ce champ ou actionner ce bouton (voir [Badges](keybindings.md#badges)).

Chaque entrée ouvre la fenêtre complète, tous les champs affichés et modifiables ; seul le curseur change de place. Dans le volet de détail du commit (à droite), clic droit sur le résumé, la description ou le bloc de l'auteur, puis *Modifier le résumé...*, *Modifier la description...* ou *Modifier l'auteur...* ; ou double-clic sur le résumé, la description, l'auteur ou la date d'auteur : la fenêtre s'ouvre avec le curseur dans ce champ. Les équivalents clavier marchent depuis le Log et depuis le volet de détail : `Ctrl+G Shift+E` (tous les champs), `Ctrl+G Shift+A` (l'auteur), et dans la palette *Commit : Modifier...*, *Commit : Modifier l'auteur...*, *Commit : Modifier le résumé...* et *Commit : Modifier la description...*. Les deux raccourcis sont réaffectables (`git_prefix.amend`, `git_prefix.edit_author` ; voir [Raccourcis clavier](keybindings.md)). Depuis un script ou un agent : `gitpoulpi amend <commit>` avec `--message` (ou `--summary` / `--description`), `--author-name`, `--author-email`, `--author-date` et les options `--committer-*` correspondantes (l'outil MCP s'appelle `amend`).

## Scinder un commit

*Scinder le commit...* dans le menu du commit (ou la palette : *Git : Scinder le commit sélectionné...*)
découpe un commit en deux ou plusieurs, du plus ancien au plus récent. Tout commit de la branche extraite
qui n'est pas un merge peut être scindé (HEAD ou un ancêtre, un commit racine aussi). L'éditeur de scission
a deux volets : les **Nouveaux commits** à gauche (un message chacun) et les **Modifications** du commit à
droite, une ligne par fichier et par hunk, chacune avec un champ `[N v]` qui désigne le nouveau commit qui
la reçoit.

| Touche | Souris | Action |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | cliquer une ligne | parcourir les modifications (`Shift+Up` / `Shift+Down` étend la sélection) |
| `1`-`9` | le champ `[N v]` de la ligne, puis un choix | donner la modification sélectionnée à ce nouveau commit ; `0` la laisse non assignée |
| `Enter` / `m` | le champ `[N v]` | ouvrir la liste des commits pour la modification sélectionnée |
| `n` | `+ Nouveau commit` | placer la modification sélectionnée dans un nouveau commit |
| `v` | | **mode ligne** : scinder un hunk ligne par ligne (glissez sur des lignes pour en sélectionner une plage) |
| `Tab` | cliquer un volet | passer des commits aux modifications |
| `e` / `Enter` (dans le volet des commits) | le crayon | modifier le message de ce commit |
| `d` / `Delete` | la croix | supprimer un nouveau commit (ses modifications deviennent non assignées) |
| `Alt+Up` / `Alt+Down` | les flèches | monter ou descendre un nouveau commit |
| `s` | la case à cocher | signer les nouveaux commits |
| `Shift+R` | `Réinitialiser` | retour au plan de départ |
| `Ctrl+S` | `Scinder en N` | scinder |
| `Esc` | la croix, `Annuler` | quitter ; avec des modifications en attente, une seconde pression confirme |

Chaque modification doit aller dans un commit et un seul ; l'éditeur compte ce qui reste *Non assigné* et le
bouton reste refusé tant que ce nombre n'est pas zéro. Un fichier binaire, qui ne change que son mode, qui est
nouveau ou supprimé dans le commit, un sous-module ou un fichier LFS ne peut être donné qu'en entier. Le
premier commit reprend le message d'origine ; donnez un message à chacun des autres. Un nouveau commit garde
l'auteur et la date d'origine ; le committer, c'est vous, maintenant.

**Rien ne change dans votre dépôt, sauf la branche.** Le dernier nouveau commit a exactement le contenu du
commit d'origine : les commits au-dessus sont donc recréés avec les mêmes arbres, auteurs et messages (les
merges restent des merges), et vos modifications stagées et non stagées sont laissées tranquilles : un arbre
de travail modifié ne gêne pas et aucun conflit n'est possible. Les hooks ne s'exécutent pas. Une scission
est bloquée tant qu'un auto-stash est en attente.

**Signature.** La case *Signer les commits* part de `commit.gpgSign` du dépôt. Décochée, les signatures des
commits réécrits (les nouveaux et ceux au-dessus) sont perdues, et l'éditeur le dit sous la case. Cochée,
tous sont signés ; si la signature échoue, rien n'est modifié.

**Annuler.** Un seul `Ctrl+Z` (Annuler de la barre d'outils) remet le commit d'origine en place, même avec un
arbre de travail modifié ; voir [Annuler et rétablir](undo-redo.md). Un commit déjà poussé demande ensuite un
force push (avec lease).

**Depuis le rebase interactif.** Chaque ligne de la revue du rebase a un petit bouton **scinder...** (`t` sur
la ligne) : il ouvre le même éditeur sur ce commit, et quand la scission est faite (ou annulée) vous revenez
à l'écran de revue, sur le nouvel historique. Si vous aviez modifié le plan, appuyez une seconde fois sur
scinder pour confirmer que ces modifications sont perdues.

Pour les scripts et les agents : `split-source` et `split-commit` dans la
[référence de la CLI](cli-reference.md) ; un agent peut aussi ouvrir cet éditeur prérempli pour que vous le
relisiez (`ui propose split`, voir [MCP et agents](mcp-and-agents.md)).

## Rebase interactif

Choisissez **Rebase interactif depuis ici...** sur un commit qui est un ancêtre
de la branche extraite, ou glissez une branche sur une autre et choisissez *Rebase
interactif*. Le Log est remplacé par un écran de revue : une ligne par commit, le
plus récent en haut, chacune avec une poignée de glissement et une liste
déroulante d'actions.

![L'écran du rebase interactif : quatre commits, le deuxième réglé sur Squash et le troisième sur Drop, avec les boutons Réinitialiser, Annuler le rebase et Lancer le rebase](../images/rebase.png)

| Action | Touche | Effet |
|---|---|---|
| Pick | `p` | Garder le commit |
| Reword | `r` | Le garder avec un nouveau message |
| Squash | `s` | Le fondre dans le commit du dessous, en combinant les messages |
| Fixup | `f` | Le fondre dans le commit du dessous, en abandonnant son message |
| Drop | `d` | Le laisser de côté |
| Edit | `e` | S'arrêter là pour que vous puissiez l'amender |

Glissez une ligne (ou `Ctrl+Up` / `Ctrl+Down`) pour réordonner. `a` prévisualise
l'autosquash (les commits `squash!` et `fixup!` vont à côté de leurs cibles). Rien
ne se passe tant que vous n'appuyez pas sur **Lancer le rebase** (`Ctrl+S`) ;
**Réinitialiser** remet le plan en place et **Annuler le rebase** (`Esc`) sort sans
rien changer. Quand une étape s'arrête sur un conflit ou un `edit`, un bandeau
au-dessus du Log propose **Continuer**, **Passer** et **Annuler**. Une plage qui
contient un commit de merge ne peut pas être rebasée interactivement ; l'entrée
de menu dit pourquoi.

## Cherry-picker plusieurs commits

`Ctrl+clic` bascule un commit, `Shift+clic` (ou `Alt+clic`) sélectionne une plage, `Space` bascule
le courant, `Esc` efface. Clic droit sur un commit sélectionné pour **Cherry-pick
de N commits sur `<branche>`...** : le même écran de revue s'ouvre, avec les
mêmes actions et le même réordonnancement, et **Lancer le cherry-pick** les rejoue
dans l'ordre.

## Copier des commits en gitGraph Mermaid

![La fenêtre Copier en gitGraph Mermaid sur trois commits sélectionnés : les choix de portée, de détail et de direction, l'option de clôture et l'aperçu en direct](../images/log-mermaid.png)

Sélectionnez des commits dans le Log (`Ctrl+clic`, `Shift+clic`, `Space` ; ou laissez simplement le curseur sur un seul)
et ouvrez **Copier en gitGraph Mermaid...** : `Shift+Y` dans le Log, le sous-menu **Copier** du menu du commit,
ou la palette avec `Log : Copier en gitGraph Mermaid...`. Une fenêtre montre le script
[Mermaid](https://mermaid.js.org/syntax/gitgraph.html) généré, en direct et défilable ; `Enter`
(ou le bouton **Copier**) met exactement ce texte dans le presse-papiers. Collez-le dans un README, la description d'une pull
request ou [mermaid.live](https://mermaid.live).

La fenêtre a trois combobox, une case à cocher et l'aperçu (`Tab` passe de l'un à l'autre ; chaque
combobox s'ouvre avec `Enter`, `Space` ou `Down`) :

- **Commits** : la sélection, ou - quand un seul commit est sélectionné et que HEAD en descend - **De la
  sélection à HEAD**, la suite de commits entre eux (calculée à partir de la partie chargée du Log).
- **Détail** : `structure` (commits, branches et merges ; les ids sont des SHA courts), `refs` (ajoute les vrais
  noms de branches comme branches Mermaid, les tags comme `tag:` et le commit HEAD comme `type: HIGHLIGHT`),
  `subjects` (l'id du commit devient `<sha> <sujet>`, échappé et coupé à `ui.mermaid_subject_width`
  caractères) et `full` (ajoute l'auteur et la date UTC à l'id).
- **Direction** : de gauche à droite (`gitGraph LR:`) ou de haut en bas (`gitGraph TB:`).
- **Inclure la clôture ```mermaid** entoure le script d'un bloc de code Markdown.

Le détail, la direction et la clôture sont mémorisés (`ui.mermaid_detail`, `ui.mermaid_orientation`,
`ui.mermaid_fence`). Le même générateur s'exécute en ligne de commande :
`gitpoulpi log --format mermaid [--detail structure|refs|subjects|full] [--orientation lr|tb] [--fence]
[--range A..B | --limit N]` (voir la [référence CLI](cli-reference.md)) ; le serveur MCP l'expose comme
l'argument `format` de l'outil `log`.

Le `gitGraph` de Mermaid est un langage de rejeu (commit, branch, checkout, merge), pas une description de
graphe : le générateur fait donc quelques choix qu'il vaut mieux connaître :

- Un commit continue la branche de son premier parent sélectionné ; les autres enfants ouvrent une nouvelle
  branche, nommée d'après une vraie branche qui s'y trouve (`b1`, `b2`... en `structure`). La première branche est
  `main` (ou le vrai nom, via un en-tête `mainBranchName`) et suit la chaîne des premiers parents de HEAD.
- Un merge rejoint la branche de l'autre parent. Quand cette branche a avancé après le parent, le générateur crée
  une petite branche pointeur (`<branche>-<sha>`) au niveau du parent et merge celle-ci, de sorte que le merge
  pointe toujours sur le bon commit.
- Un parent hors de la sélection est abandonné : un commit qui n'a plus de parent sélectionné commence une ligne
  à part, donc les trous d'une sélection apparaissent comme des lignes déconnectées.
- Un merge octopus (trois parents ou plus) devient une suite de merges ; les intermédiaires reçoivent le suffixe
  ` (octopus 1/2)`, le dernier porte le vrai commit.
- Les cherry-picks ne sont pas détectés ; les guillemets doubles dans les sujets deviennent des apostrophes
  (Mermaid ne sait pas les échapper).

## Comparer des commits

Sélectionnez deux lignes du Log ou plus : `Ctrl+clic` bascule une ligne, `Shift+clic` (ou `Alt+clic`, `Shift+Up/Down`, `Alt+Up/Down`) prend une plage, `Space` bascule la ligne courante. Les commits, la ligne **WIP** et les lignes de **stash** peuvent tous être sélectionnés. Le volet de droite devient un volet **Comparaison** :

- l'en-tête a un bouton **Inverser**, et dessous chaque ligne sélectionnée est listée dans l'ordre du Log (la plus récente d'abord) sous forme de **carte** sur deux lignes : l'avatar de l'auteur (selon les paramètres d'avatars du Log : une image sur les terminaux qui affichent des images, les initiales sinon), son nom, l'ancienneté, l'id court et le résumé. La ligne **WIP** est « Arbre de travail / Modifications non validées », un stash affiche le glyphe de stash, `stash@{n}` et son message. L'extrémité la plus ancienne porte une étiquette **De** et la plus récente une étiquette **À** (Inverser les échange) ; avec trois lignes ou plus, une note dit que les modifications sont combinées. La liste de cartes prend au plus un tiers du volet, puis défile (molette, barre de défilement, ou glissez le séparateur dessous pour la redimensionner) ;
- cliquez sur une carte pour amener le curseur du Log sur cette ligne et la faire clignoter dans le graphe ; la sélection n'est pas touchée, donc `Space` retire ensuite cette ligne de la sélection ;
- dessous, la liste habituelle des fichiers modifiés (vue chemin ou arbre, filtre, tri) liste les fichiers qui diffèrent entre elles. Cliquez sur un fichier, ou appuyez sur `Enter`, pour ouvrir son diff au centre, dans les mêmes dispositions unifiée ou côte à côte, avec le Blame, l'Historique du fichier et la vue du fichier sur l'extrémité la plus récente.

![Trois lignes du Log sélectionnées : le volet Comparaison les liste sous forme de cartes avec avatars et étiquettes De / À, au-dessus des fichiers modifiés](../images/compare-cards.png)

Ce qui est comparé dépend de la sélection :

| Sélection | Comparé |
|---|---|
| deux commits (ou un stash et un commit) | le plus ancien avec le plus récent, dans l'ordre du Log, quel que soit l'ordre des clics |
| la ligne WIP et un commit ou un stash | ce commit avec l'arbre de travail : modifications stagées et non stagées des fichiers suivis (les fichiers non suivis ne font pas partie d'un diff) |
| trois commits ou plus | leurs modifications combinées : de juste avant le plus ancien jusqu'au plus récent (les merges suivent leur premier parent) |

Un fichier renommé ou déplacé apparaît comme un seul fichier, avec l'ancien nom et la similarité en haut de son diff. **Inverser** (`x`) échange les extrémités d'une comparaison de deux lignes ; avec la ligne WIP, qui est toujours l'extrémité la plus récente, ou une plage de trois lignes ou plus, il n'y a rien à échanger. Une comparaison avec la ligne WIP suit l'arbre de travail au fil de vos enregistrements.

Au clavier : `v` (ou **Log : Comparer la sélection** dans la palette de commandes) place le focus dans la liste des fichiers de la comparaison ; `x` (ou **Log : Inverser les extrémités de la comparaison**) les échange ; `n` (ou **Log : Aller à la ligne comparée suivante**) est le jumeau clavier du clic sur une carte : le curseur du Log va à la ligne sélectionnée suivante, en bouclant. `Esc` ou un simple clic sur une ligne revient au commit seul. L'entrée **Cherry-pick** ne compte que les commits de la sélection ; la ligne WIP et les stashes y sont ignorés.

Les mêmes listes sont disponibles pour les scripts et les agents : `gitpoulpi diff --from <rev> [--to <rev|worktree>] [--name-only]` (la CLI et l'outil MCP `diff` ; `--to` vaut par défaut l'arbre de travail).

## Conflits et autres bandeaux

Quand un merge, un rebase, un cherry-pick ou un revert s'arrête sur un conflit,
un bandeau apparaît au-dessus du Log et les fichiers en conflit portent un `!`
rouge dans le volet Status. Résolvez-les dans l'éditeur de merge, puis utilisez
**Continuer** ou **Commiter le merge** du bandeau, ou **Annuler** pour reculer.
Voir [Résoudre les conflits de merge](merge-conflicts.md).

Autres bandeaux au-dessus du Log :

| Bandeau | Boutons | Palette |
|---|---|---|
| *Rebase de `<branche>` sur `<commit>` · étape/total · arrêté à ...* | **Continuer**, **Passer**, **Annuler** | *Rebase : Continuer / Passer / Abandonner* |
| *Applying patches (git am) · patch n sur m* (un `git am` arrêté) | **Résoudre les conflits...** (quand un patch est en conflit), **Continuer**, **Sauter**, **Annuler** (demande d'abord) | *git am : Continuer / Sauter ce patch / Annuler* |
| *Un index.lock oublié (n s) bloque toute modification de ce dépôt* (une commande git est morte et a laissé `.git/index.lock`) | **Supprimer index.lock** (demande d'abord ; proposé seulement quand le fichier a au moins 10 secondes) | *Dépôt : Supprimer l'index.lock périmé* |
| *Auto-stash en attente : ...* | Voir [Auto-stash](#auto-stash) | *Bandeau : ...* |

## Blame

Le Blame montre tout le fichier avec numéros de ligne et couleurs syntaxiques, et une colonne d'annotation à **droite** du code (`ui.blame_side = "left"` la met à gauche). Ouvrez-le depuis la barre d'un fichier (`Blame`), le menu du fichier, la palette, ou depuis une ligne de l'historique (`b`). Depuis l'arbre de travail, les lignes pas encore commitées sont montrées à part (voir Bandes).

- **Hunks.** Une suite de lignes modifiées en dernier par un même commit forme un hunk. Sa première ligne montre l'avatar de l'auteur (une image si le terminal sait en afficher, sinon le badge d'initiales, rien si *Avatars des auteurs* est désactivé), l'auteur, l'ancienneté et le résumé du commit ; les autres lignes ne montrent que la barre de couleur de l'auteur. Un trait fin sépare les hunks. Quand un hunk commence au-dessus de l'écran, son annotation est répétée sur la première ligne visible.
- **Bandes.** À côté de l'annotation, la barre de couleur de l'auteur et une **bande de chaleur d'âge** : chaud pour les lignes récentes, froid pour les anciennes, relativement à la ligne la plus ancienne et la plus récente du fichier (la légende est dans le titre). Les lignes pas encore commitées sont pointillées et disent « Not committed yet ».
- **Fiche.** Laissez le pointeur sur une annotation un instant, ou appuyez sur `i`, pour la fiche de détail du hunk : ID complet du commit, auteur et e-mail, date, résumé et ses actions. Le hunk sélectionné est teinté.

  ![Le Blame avec la minimap activée et la fiche ouverte sur le hunk sélectionné](../images/blame-card.png)
- **Actions** (lignes de la fiche, menu clic droit sur un hunk, touches, et `Blame : ...` dans la palette) : afficher le commit dans le Log (`Enter`), copier son ID (`Shift+Y`), afficher le diff du commit pour ce fichier (`d`), Blame au parent du commit (`b`, en suivant les renommages), ouvrir l'Historique du fichier (`Shift+H`).
- **Se déplacer.** `j` / `k`, PageUp/PageDown, Home, `Shift+G` et la molette comme partout ; `n` / `p` hunk suivant / précédent, `Shift+N` / `Shift+P` hunk suivant / précédent du même commit, `g` aller à un numéro de ligne.
- **Hunks hauts.** Sous le résumé, un hunk de trois lignes ou plus montre l'ID court du commit et la date complète (atténués), pour que la colonne ne reste pas vide.
- **Minimap.** Avec `ui.minimap` activé (le bouton `Carte`, `M`), la minimap du fichier est à côté du code, colorée par la même chaleur d'âge que la bande ; cliquez ou faites glisser le long de la minimap pour défiler.
- **Largeur.** Faites glisser la bordure de la colonne d'annotation, ou appuyez sur `>` / `<` ; la largeur est enregistrée dans `ui.blame_width`.

## Historique du fichier

Ouvrez-le depuis le segment `Fichier` du sélecteur `Fichier │ Tout` de la barre du diff, le menu d'un fichier ou la palette (`Historique du fichier : ...`). Le centre se divise en deux : à gauche les commits du fichier, du plus récent au plus ancien, en suivant les renommages ; à droite le diff du fichier pour le commit sélectionné, avec tous les modes de la vue Diff (hunks / fichier entier, en ligne / côte à côte, diff par mot, minimap, retour à la ligne).

- **La liste.** Deux lignes par commit. La première : la lettre du changement (A ajouté, M modifié, D supprimé, R renommé), le résumé et les `+ajoutées -supprimées` en couleur. La seconde, atténuée : l'avatar de l'auteur (exactement ce que la colonne Auteur du Log montre selon *Avatars des auteurs*), l'auteur, la date relative, l'ID court du commit, et `ancien -> nouveau` sur le commit qui a renommé le fichier. Une liste étroite abandonne d'abord le renommage, puis l'ID, puis la fin du nom de l'auteur, mais jamais l'auteur ni la date. Les pages de 100 se chargent en approchant de la fin ; une ligne `chargement de la suite...` s'affiche entre-temps.
- **Se déplacer.** `j` / `k`, flèches, PageUp/PageDown, Home / End, la molette et un clic ; le diff suit aussitôt (les diffs sont lus en arrière-plan, mis en cache par ligne, et une touche maintenue n'en demande qu'un). Une ligne antérieure à un renommage est comparée sous le nom que le fichier avait alors.
- **Diff / Fichier / Blame.** Les boutons **Vue fichier**, **Vue diff** et **Blame** de la barre de vue choisissent ce que le côté droit montre pour le commit sélectionné : sa **vue Diff**, la **vue Fichier** (le fichier tel qu'à ce commit) ou son **Blame**, le tout dans la division, la liste restant à gauche et le segment `Fichier` du sélecteur de la colonne allumé. Le mode choisi est conservé quand vous changez de ligne ; le côté droit suit la sélection. Touches `Alt+1` / `Alt+2` / `Alt+3` (aussi `d` / `o` / `b` dans la liste), depuis l'une ou l'autre moitié, et les entrées `Historique du fichier : ...` de la palette font de même ; `Esc` revient au Diff.
- **La colonne.** La liste est la *colonne des commits* qui montre sa liste **Fichier** : masquer, afficher, tailles et sélecteur `Fichier │ Tout` sont décrits une seule fois dans [La colonne des commits](interface-tour.md#la-colonne-des-commits). Un clic sur `Tout` quitte l'Historique du fichier et garde la vue sur le commit sélectionné.

  ![L'historique du fichier avec un commit du milieu sélectionné : son diff à droite](../images/file-history.png)

  ![L'historique du fichier avec le côté droit en mode Blame, le blame du fichier au commit sélectionné](../images/file-history-blame.png)
- **Comparer deux commits.** `Shift+Up` / `Shift+Down` ou `Shift+clic` sélectionnent une plage ; le côté droit montre le diff du fichier entre les deux extrémités, chacune avec son chemin. `Esc` abandonne la plage.
- **Focus.** `Tab` (ou un clic) donne le clavier au diff et le rend ; le diff garde ses propres touches.
- **Actions** (menu clic droit, touches, et `Historique du fichier : ...` dans la palette) : afficher dans le Log (`Enter`, double-clic ; le Log se charge aussi loin que nécessaire), copier l'ID (`y`) ou le résumé (`Shift+Y`), afficher le diff (`d`), le fichier à ce commit (`o`) ou son blame (`b`) à droite, comparer avec l'arbre de travail (`c`, de nouveau pour quitter), restaurer le fichier à cette version (`Shift+R`, confirmé).
- **Annuler.** Restaurer une version est une étape journalisée : Annuler (`Ctrl+Z`) remet les anciens octets du fichier.
- **Séparateur.** Faites glisser la bordure entre les moitiés, ou `Alt+Right` / `Alt+Left`, `Alt+0` pour réinitialiser ; la largeur est mémorisée par session.

## Fetch, pull et push

- **Pull** (`Ctrl+G p`) fait le pull comme le dit *Paramètres > Git et encodage >
  Bouton Pull* (`git.pull_mode` : merge, fast-forward seul, rebase ; un paramètre
  par dépôt). Son `▾` lance *Fetch All*, *Pull (fast-forward si possible)*, *Pull
  (fast-forward seul)* ou *Pull (rebase)* une fois, sans changer le bouton. Le mode
  est toujours passé explicitement à git, si bien que le `pull.rebase` de votre
  configuration git ne peut pas vous surprendre.
- **Pull de toutes les branches** (`Ctrl+G Ctrl+P`, une entrée du `▾` de Pull) télécharge
  depuis les distants que vos branches suivent, puis amène d'un coup chaque branche
  locale au niveau de son amont. Une branche non extraite est seulement avancée en
  fast-forward (aucun fichier n'est touché) ; une branche qui a divergé est laissée telle
  quelle et signalée. La branche extraite est tirée comme le bouton Pull le prévoit, et
  le `▾` propose aussi l'entrée de chaque autre mode (*Pull de toutes les branches
  (fast-forward seul)*, *(rebase)*, *(fast-forward si possible)*). Quand une branche en
  retard est extraite dans un autre worktree, une fenêtre demande d'abord s'il faut
  mettre à jour ces worktrees aussi, les ignorer ou annuler. La ligne d'état résume la
  passe et le Journal d'activité a une ligne par branche ; Annuler remet toutes les
  branches comme avant en une seule étape. `Ctrl+G p` reste un pull ordinaire.
- **Push** (`Ctrl+G Shift+P`) envoie la branche extraite vers le distant par défaut,
  en définissant l'amont la première fois. Son `▾` pousse une fois vers un autre
  distant de votre choix. Si le distant a des commits que vous n'avez pas, une boîte
  de dialogue propose **Pull (merge), puis push**, **Force push (avec lease)...**
  (jamais un `--force` nu) ou **Annuler**.
- `Ctrl+G f` fait un fetch de tous les distants. Dans *Paramètres > Synchronisation*,
  *Auto-prune* décide si les fetchs élaguent, et *Intervalle d'auto-fetch
  (minutes)* fait un fetch en arrière-plan.

Tout cela est aussi disponible sous la forme `gitpoulpi fetch`, `pull` et `push`
dans la [CLI](cli-reference.md).

## Auto-stash

Quand des modifications non commitées bloquent un checkout, un pull, un merge ou un
rebase, GitPoulpi peut les mettre de côté pour vous : il les stashe, lance
l'opération, puis fait un pop du stash. *Paramètres > Synchronisation > Auto-stash
avant un checkout, un pull, un merge ou un rebase* (`general.auto_stash`) :

- **demander** (par défaut) : une boîte de dialogue propose *Stash des modifications et continuer*, *Toujours faire le stash automatiquement* (pareil, et cela règle `general.auto_stash = "always"` pour ne plus vous le demander ; Paramètres > Synchronisation le remet) ou *Annuler*.
  La CLI et MCP échouent à la place et nomment `--autostash`.
- **toujours** : fait sans demander. **jamais** : l'opération est refusée.

*Inclure les fichiers non suivis dans l'auto-stash* prend aussi les fichiers non
suivis. Un arbre propre ne voit jamais de stash. Le stash, l'opération et le pop
sont chacun annulables.

Si le stash ne peut pas être remis (il entre en conflit avec le résultat, ou un
merge ou un rebase s'est arrêté sur son propre conflit), il reste dans la liste
des stashes. Un **bandeau au-dessus du Log** dit *Auto-stash en attente : vos
modifications d'avant le checkout sont dans stash@{n}*. Ses boutons :

| Bouton | Fait |
|---|---|
| **Résoudre les conflits...** | Ouvre l'éditeur de merge (seulement après un pop en conflit) |
| **Garder le résultat** | Garde l'arbre de travail tel quel et abandonne la copie du stash (seulement après un pop en conflit, une fois qu'aucun fichier n'est en conflit) |
| **Pop maintenant** (**Réessayer le pop** après un pop échoué) | Remet le stash |
| **Voir le stash** | Montre les modifications du stash |
| **Repartir de zéro** | Voir plus bas |
| **Abandonner** | Abandonne le stash (demande d'abord) |

Chaque bouton a une commande de palette (*Bandeau : Restaurer l'auto-stash en attente (pop)*, *Bandeau :
Repartir de zéro...*, ...). Rien n'est réessayé ni abandonné à votre place.

**Repartir de zéro** demande d'abord confirmation. Puis il :

1. garde une copie de l'état actuel des fichiers stashés sous
   `refs/gitpoulpi-backup/autostash-start-over-<heure>` ;
2. abandonne les modifications sur ces fichiers seulement (il refuse, en les
   nommant, si d'autres fichiers ont des modifications locales qui pourraient être
   perdus) ;
3. annule l'opération qui a fait l'auto-stash, via le journal d'annulation (cette
   opération doit donc être la dernière) ;
4. refait le pop du stash sur le commit d'origine et efface le bandeau.

Si une étape échoue, il s'arrête avec le stash intact et dit où. Tant que le
bandeau est affiché et que la dernière opération est celle qui a fait l'auto-stash,
**Annuler** (`Ctrl+Z`, `gitpoulpi undo`) fait la même chose, après confirmation. Les
scripts utilisent `gitpoulpi autostash start-over [--dry-run] [--yes]` (code de
sortie 3 sans `--yes`).

**Tant qu'un auto-stash est en attente, les opérations qui déplacent l'historique
sont refusées :** checkout d'une branche ou d'un commit, merge, rebase (interactif
aussi), pull, fast-forward, cherry-pick, revert, reword, composition de commits par
l'IA et rétablir. La fenêtre grise ces boutons et ces entrées de palette et donne
la raison. La CLI et MCP reçoivent une erreur `PendingAutoStash` (code de sortie
1). Tout le reste reste permis : lectures, stage, abandon, commit, branches, tags,
distants, fetch, push, continuer / sauter / annuler un rebase, résolution de
conflits, `stash push`, `reset`, et `stash pop` / `apply` / `drop`. Annuler est
permis aussi.

Quand l'auto-stash attend un merge ou un rebase arrêté sur des conflits, il est
restauré tout seul dès que cette opération se termine (dans GitPoulpi, ou avec git
simple suivi d'un rafraîchissement).
