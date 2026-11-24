[English](../undo-redo.md) · [Čeština](../cs/undo-redo.md) · [Deutsch](../de/undo-redo.md) · [Español](../es/undo-redo.md) · **Français** · [Italiano](../it/undo-redo.md) · [日本語](../ja/undo-redo.md) · [한국어](../ko/undo-redo.md) · [Nederlands](../nl/undo-redo.md) · [Polski](../pl/undo-redo.md) · [Português (Brasil)](../pt-BR/undo-redo.md) · [Português (Portugal)](../pt-PT/undo-redo.md) · [Русский](../ru/undo-redo.md) · [Svenska](../sv/undo-redo.md) · [Türkçe](../tr/undo-redo.md) · [Українська](../uk/undo-redo.md) · [简体中文](../zh-CN/undo-redo.md)

# Annuler et rétablir

**Vous pouvez toujours revenir en arrière.** Chaque modification que GitPoulpi
apporte à votre dépôt est enregistrée (jusqu'à 100 actions par défaut,
`general.undo_depth`), tout comme chaque position qu'une branche a eue, y compris
les déplacements faits par d'autres programmes. Si vous faites une erreur, l'un
de ces moyens ramène votre travail :

| Vous voulez | Utilisez |
|---|---|
| reprendre la dernière chose | `Ctrl+Z` (rétablir : `Ctrl+Y` ou `Ctrl+Shift+Z`), ou les boutons **Annuler** et **Rétablir** de la barre d'outils |
| reprendre plusieurs choses d'un coup, ou une chose d'il y a longtemps | l'**Historique d'annulation** (`Alt+Z`, la flèche à côté d'Annuler, `Édition : Historique d'annulation...`) |
| récupérer une branche, un tag, un stash, un fichier ou un hunk supprimé il y a plusieurs actions, **sans annuler ce que vous avez fait depuis** | **Restaurer seulement celle-ci** dans l'Historique d'annulation |
| remettre une branche là où elle était, même si un autre programme l'a déplacée | le **Reflog** de la branche (menu de la ref : `Reflog...`) |
| retrouver des commits qu'aucune branche n'atteint plus | **Travail perdu** (`Édition : Retrouver les commits perdus...`) |
| reprendre ce qu'un shell ou un **agent IA** a fait aux branches, aux tags, aux stashes ou à HEAD | le même `Ctrl+Z` : ces changements sont aussi dans l'historique, marqués **extérieurs** (voir [Changements faits en dehors de GitPoulpi](#changements-faits-en-dehors-de-gitpoulpi)) |

Après chaque action qui détruit quelque chose et peut être annulée (abandon,
suppression d'un fichier, suppression d'une branche ou d'un tag, reset hard,
suppression d'un stash, retrait d'un distant), la ligne d'état le dit :
« abandon des modifications : fait - Ctrl+Z pour annuler », avec la touche telle
que vous l'avez configurée. L'infobulle d'Annuler dit combien d'actions vous
pouvez remonter.

`Ctrl+Z` reprend la dernière chose que GitPoulpi a faite à votre dépôt. `Ctrl+Y`
(ou `Ctrl+Shift+Z`) la remet. La barre d'outils a des boutons **Annuler** et **Rétablir**,
la palette a `Édition : Annuler` et `Édition : Rétablir`, et l'infobulle de chaque
bouton dit ce qu'il reprendrait (« Annuler Commit "fix typo" (a81b9e4) »).

![La barre d'outils après Stage de tout : Annuler est allumé et son infobulle nomme l'opération qu'il reprendrait](../images/undo-toolbar.png)

Rétablir ne relance jamais l'opération. Il restaure l'état qui a été enregistré,
si bien qu'un commit rétabli est exactement le même commit, avec le même id et
le même horodatage.

`Ctrl+Z` est inactif quand un champ de texte a le clavier (le message de
commit, la zone de filtre, l'éditeur, qui garde sa propre annulation de texte).

## L'Historique d'annulation

`Alt+Z` (réaffectable), la flèche à côté du bouton Annuler ou `Édition : Historique d'annulation...`
ouvre un panneau avec chaque action enregistrée, la plus récente d'abord : une
icône pour le genre d'action (commit, branche, tag, stash, reset, merge et
compagnie, abandon, stage, configuration, changement extérieur), ce qu'elle a
fait dans votre langue, il y a combien de temps, et son état (**restaurable**,
**bloquée** avec la raison, ou **annulée** et prête à être rétablie). Quand les
entrées s'étalent sur plusieurs jours, un fin intertitre (Aujourd'hui, Hier, puis
la date) ouvre chaque jour. L'en-tête les compte (« 12 actions peuvent être
annulées (jusqu'à 100) »).

Les actions que vous avez annulées se trouvent **au-dessus** de l'action la plus
récente, sous un séparateur (« annulées - Rétablir les ramène »), atténuées, celle
que Rétablir ramène en premier étant la plus proche du séparateur. La sélection
commence sur l'action la plus récente, pas sur elles. Les flèches se déplacent
(les intertitres et le séparateur sont sautés), `1` / `2` / `3` (ou `Ctrl+Tab` /
`Ctrl+Shift+Tab`) passent d'une des trois vues à l'autre, `Esc` ferme. `Enter`
n'exécute jamais une action : chaque bouton a sa propre lettre, indiquée dans la
ligne d'aide sous les boutons. Pour la ligne sélectionnée :

- **Annuler jusqu'ici** (`a`) : annule cette action et toutes les plus récentes,
  de la plus récente à la plus ancienne, après une seule confirmation qui les
  liste toutes. Une étape que le démon juge risquée redemande avant de s'exécuter.
- **Restaurer seulement celle-ci** (`s`) : ramène ce que cette seule action a
  supprimé et garde tout ce qui a été fait après. Une branche ou un tag supprimé
  revient sur le commit qu'il avait (si son nom est pris entre-temps, on vous en
  demande un autre, avec un nom libre déjà rempli) ; un stash abandonné est remis
  au sommet de la liste des stashes ; un fichier abandonné ou supprimé est
  réécrit à sa place s'il n'a pas changé depuis, sinon à côté, sous le nom
  `nom.restored` (rien n'est jamais écrasé) ; un distant, un amont, une ligne
  d'ignore ou une ligne d'attributs retirés sont ajoutés de nouveau. Un reset,
  un amend, un rebase, un merge ou un commit reçoit une branche
  `restore/<branche>-<sha>` à l'ancien sommet : votre branche courante et vos
  fichiers ne bougent jamais. La confirmation montre exactement ce qui va être
  fait, et la restauration est une nouvelle action : `Ctrl+Z` la reprend.
- **Annuler toute la séquence** (`x`) : proposé tant que les entrées les plus récentes sont celles
  d'une même exécution de séquence (voir [Exécutions de séquences](#exécutions-de-séquences)).
- **Rétablir jusqu'ici** (`t`) sur une ligne annulée.
- **Restaurer main là où Commit l'a laissée** (`r`, ou `Alt+Shift+Z` /
  `Édition : Restaurer la branche là où l'action bloquée l'a laissée` depuis
  l'extérieur du panneau) : quand un autre programme a déplacé la branche et
  bloqué Annuler, le reflog sait encore où l'action l'avait laissée.

Un clic droit sur une ligne (ou `.`) donne les mêmes actions sous forme de menu.

Une ligne faite par une étape d'une [séquence](sequences.md) porte un badge, `⟳ Backup`,
et sa ligne de détail dit de quelle étape il s'agit (« Faite par la séquence « Backup »,
étape 3/4. »).

![L'Historique d'annulation après la suppression d'une branche et deux commits : la ligne de la branche supprimée est sélectionnée et peut être restaurée seule](../images/undo-history.png)

## Le Reflog

Le menu de la ref d'une branche locale a **Reflog...** ; `Édition : Reflog...`
montre celui de HEAD, et l'Historique d'annulation a un onglet pour lui. Il
liste les positions qu'a eues la branche, de la plus récente à la plus ancienne :
SHA court, la raison enregistrée, il y a combien de temps, qui - y compris les
déplacements faits dans un shell. Le message et les fichiers modifiés de la
position sélectionnée s'affichent à côté de la liste.

Le reflog est lu 200 positions à la fois. Un reflog plus long continue tout
seul : déplacer le curseur (ou défiler) près de la fin de la liste charge la page
suivante, et la dernière ligne indique **... Charger la suite** tant qu'il en
reste. **Charger la suite** (`c`, ou `Reflog : Charger la suite` dans la palette)
la charge tout de suite et affiche « positions chargées : N » dans la ligne d'état. Les dépôts
au format reftable de git sont lus comme les autres.

![Le Reflog de main : chaque position qu'elle a eue, la plus récente en haut, avec la sélectionnée décrite à droite](../images/reflog.png)

**Afficher dans le Log** (`l`), **Créer une branche ici** (`b`), **Créer un tag ici** (`t`),
**Appliquer à l'arbre de travail** (`a`), **Enregistrer en stash** (`e`) (les trois
derniers fonctionnent comme dans Travail perdu ci-dessous) et **Restaurer
`<branche>` à cette position** (`r`). La restauration est annulable. Quand la branche est extraite
et que vous avez des modifications locales qu'elle ne peut pas déplacer sans
écraser, une liste déroulante demande ce qu'il faut en faire : garder mes
modifications, soft, mixed ou hard. Hard abandonne les modifications de vos
fichiers mais les sauvegarde d'abord, donc `Ctrl+Z` les ramène encore (c'est dit
en rouge).

## Travail perdu

`Édition : Retrouver les commits perdus...` (aussi dans le menu des commits du
Log) trouve les commits qu'aucune branche, aucun tag ni aucun stash n'atteint :
ceux qu'un reflog nomme encore (les commits d'une branche supprimée), ceux que
seule une ancre d'annulation retient, et les orphelins. La liste apparaît
aussitôt et l'analyse plus lente des commits orphelins la complète pendant
qu'un indicateur tourne ; elle ne bloque jamais l'interface.

![Travail perdu : les deux commits d'une branche supprimée (retrouvés par le reflog) et des commits orphelins](../images/lost-work.png)

**Créer une branche
ici** (`b`) récupère un commit sous un nom de votre choix, **Créer un tag ici**
(`t`) le marque d'un tag (nom par défaut `restored-<id court>`), **Cherry-pick**
(`c`) l'applique à la branche courante. Un commit perdu n'est pas dans le Log :
cette vue n'a donc pas de « Afficher dans le Log » (la vue Reflog le garde pour
les commits encore atteignables).

**Récupérer les changements** remet ce qu'a changé le commit sans le commiter :
**Appliquer à l'arbre de travail** (`a`) rejoue les changements du commit sur les
fichiers, en tout ou rien (l'index et HEAD ne bougent pas ; un commit de merge est
refusé). Quand ils ne peuvent pas s'appliquer proprement, rien n'est touché et la
boîte propose **Enregistrer en stash** (`e`), qui garde les mêmes changements dans
un nouveau stash ; cette action est d'ailleurs toujours disponible seule. Les deux
sont journalisées : `Ctrl+Z` remet les fichiers en place, ou supprime de nouveau le
stash. Les mêmes actions existent en `gitpoulpi apply-commit <rev>` et
`gitpoulpi stash from-commit <rev> [-m M]`, et la vue Reflog les a aussi.

### Volets, hunks et fenêtre

Reflog et Travail perdu montrent la liste des commits à gauche et, à droite,
l'en-tête du commit sélectionné, **chaque fichier qu'il a modifié** et les hunks du
fichier sélectionné (les hunks seuls, à la largeur du volet). La liste des fichiers
est celle du volet de droite de la fenêtre principale : `Chemin` ou `Arbre` (`v`,
ou cliquez sur la bascule), un dossier se plie avec `Enter`, `Left` ou `Right`, un
clic ou les flèches choisissent un fichier et ses hunks apparaissent en dessous (le
démon est interrogé hors du fil de l'interface ; les derniers fichiers sont gardés).
La liste des commits, celle des fichiers et les hunks ont chacun une barre de
défilement que vous pouvez presser et glisser.

- `Tab` / `Shift+Tab` parcourent les volets (liste, fichiers, hunks) ; les flèches,
  `PageUp`, `PageDown`, `Home` et `End` agissent sur le volet qui a le clavier (sa
  bordure est allumée). `Enter` sur un fichier passe à ses hunks.
- **Séparateurs** : glissez celui entre la liste et le volet de droite, ou celui
  entre les fichiers et les hunks ; double-cliquez un séparateur pour le
  réinitialiser. Au clavier : `Alt+Left` / `Alt+Right`, `Alt+Up` / `Alt+Down`,
  `Alt+0` réinitialise les deux. La palette a `Reflog / Travail perdu : Réinitialiser
  la taille des volets`, `Volet suivant` et `Liste des fichiers en chemins ou en arbre`.
- `Ctrl+N` affiche ses badges sur les trois onglets de vue et les trois volets,
  comme dans la fenêtre principale ; une touche de badge y saute.
- **Déplacer et redimensionner la boîte** : glissez la barre de titre pour la
  déplacer, glissez les bordures gauche, droite et basse ou un coin du bas (la
  poignée `◢`) pour la redimensionner (jamais en dessous de 60 x 16, toujours à
  l'écran). La croix de fermeture garde son clic et relâcher hors de la boîte ne
  la ferme jamais. Le pendant clavier est `Ctrl+N` puis `m` : les flèches
  déplacent, `Shift`+flèches redimensionnent, `0` réinitialise, `Enter` ou `Esc`
  quittent (la bordure l'indique tant que le mode est actif). La palette a
  `Dialogue : Déplacer/redimensionner` et `Dialogue : Réinitialiser la position et
  la taille`. Que la place soit gardée ou non dépend de `ui.dialog_geometry`
  (Paramètres > Apparence > Disposition) : `session` (par défaut), `saved`
  (restaurée au prochain lancement) ou `never`.

## Depuis la palette de commandes

Chaque action de ligne des trois vues est aussi une ligne de la palette de
commandes : le panneau se pilote sans souris et sans lettre à retenir. Appuyez
sur `Ctrl+P` pendant que le panneau est ouvert : la palette s'ouvre par-dessus,
et ses lignes **Historique d'annulation :**, **Reflog :** et **Travail perdu :**
agissent sur la ligne sélectionnée de la vue affichée.

| Vue | Lignes de la palette |
|---|---|
| Historique d'annulation | `Historique d'annulation : Annuler jusqu'ici...`, `Rétablir jusqu'ici...`, `Annuler toute la séquence`, `Restaurer seulement celle-ci...`, `Restaurer la branche là où elle était...`, `Actualiser`, `Fermer` |
| Reflog | `Reflog : Afficher dans le Log`, `Créer une branche ici...`, `Créer un tag ici...`, `Appliquer les changements à l'arbre de travail`, `Enregistrer en stash`, `Restaurer à cette position...`, `Charger la suite`, `Actualiser` |
| Travail perdu | `Travail perdu : Créer une branche ici...`, `Créer un tag ici...`, `Cherry-pick`, `Appliquer les changements à l'arbre de travail`, `Enregistrer en stash`, `Relancer la recherche` |

Les lignes sont grisées (avec la raison) tant que leur vue n'est pas ouverte ;
les trois ouvertures, `Édition : Historique d'annulation...`, `Édition : Reflog...`
et `Édition : Retrouver les commits perdus...`, font aussi basculer un panneau
déjà ouvert vers leur vue. Une ligne que la sélection ne peut pas exécuter dit
pourquoi dans la ligne d'état au lieu de ne rien faire : « cette action est déjà
annulée : utilisez Rétablir jusqu'ici », la raison pour laquelle une entrée ne
peut pas être restaurée seule, « seuls une branche ou HEAD peuvent être ramenés
en arrière », etc. Les lignes en `...` ouvrent la même confirmation ou la même
saisie de nom que le bouton.

## Exécutions de séquences

Une [séquence](sequences.md) (Backup : stage de tout, message de commit de l'IA, commit, push)
fait **une entrée de journal ordinaire par étape qui change quelque chose** : ici le stage
et le commit, pas le message de l'IA et pas le push (un push n'est jamais
annulé, voir plus bas). Le journal est par dépôt et strictement de la plus récente à la plus ancienne, donc
une exécution ne peut pas être une seule entrée : chacune garde ses propres vérifications, et chacune est marquée de
l'exécution qui l'a faite (`sequence:Backup#<run> 3/4`). Les anciens journaux n'ont pas de marque
et se chargent comme avant.

**Annuler toute la séquence** annule les entrées d'une exécution, de la plus récente à la plus ancienne, comme un
Annuler ordinaire chacune. Elle est proposée quand les entrées les plus récentes du journal sont
celles d'une même exécution, consécutives : une entrée d'une autre exécution, une modification que vous avez faite à la main ou
une modification faite hors de GitPoulpi entre les deux met fin au groupe. Où la trouver :

- l'Historique d'annulation : le bouton `Annuler toute la séquence` (`x`), le menu du clic droit sur la ligne,
  et la ligne de palette `Historique d'annulation : Annuler toute la séquence` ;
- la palette de commandes : `Édition : Annuler toute la séquence` (le nom de la séquence figure sur
  la ligne tant qu'elle s'applique) ;
- la ligne de commande et MCP : `gitpoulpi undo --sequence` (plus bas).

Chaque entrée est vérifiée seule, exactement comme un Annuler simple : un commit déjà
sur son amont demande « annuler quand même ? » avant de partir (répondez et le reste
continue), et une branche qui a bougé ou un merge en cours refuse. Le premier
refus (ou un « non » à une question) **arrête** et dit quelle étape et pourquoi : *l'annulation de
« Backup » s'est arrêtée à l'étape 3/4 (Commit « wip » (a81b9e4)) : ... Actions restantes : 2*.
Ce qui a été annulé reste annulé, les entrées plus anciennes restent dans le journal, et un
Annuler simple (ou le bouton de nouveau) continue à partir de là.

Seul le journal de ce dépôt est annulé. Une exécution qui a aussi agi dans un autre
dépôt (une étape worktree a déplacé les étapes suivantes dans l'onglet du nouveau worktree)
a laissé des entrées dans le journal propre à ce dépôt ; ouvrez-le et annulez là. **Rétablir**
reste entrée par entrée : après avoir annulé toute une exécution, `Ctrl+Y` ramène l'étape la plus
récente, puis la suivante, chacune avec sa propre vérification.


## Ce qui peut être annulé

| Zone | Opérations |
|---|---|
| Commits | commit, amend, reword, le compositeur de commits IA (tous ses commits en une seule étape) |
| Branches et tags | créer, supprimer, renommer, définir ou retirer l'amont, créer une branche de suivi, créer / supprimer / annoter un tag, fast-forward, extraire une pull request comme branche |
| Se déplacer | checkout d'une branche ou d'un commit (y compris un HEAD détaché) |
| Historique | merge (fast-forward ou commit de merge), rebase (simple et interactif, une fois terminé), cherry-pick, revert, reset (soft, mixed et hard), la partie locale d'un pull, **pull de toutes les branches** (toutes les branches déplacées, en une seule étape ; les fichiers de la branche extraite reviennent avec) |
| Modifications locales | stage et unstage (fichier, dossier, hunk, lignes, tout), abandon (fichier, dossier, hunk, lignes, un checkout de fichier), appliquer un patch ou les changements d'un commit (`apply-commit`), **supprimer un fichier du disque**, **ignorer** (fichier, extension ou dossier : la ligne du `.gitignore` choisi), **traiter comme binaire ou texte** (la ligne de `.gitattributes`) |
| Stashes | push, apply, pop, drop, enregistrer en stash les changements d'un commit (`stash from-commit`) |
| Distants et configuration | ajouter, retirer, renommer, changer l'URL ou l'URL de push d'un distant, suivre et ne plus suivre avec Git LFS |
| Sur un serveur | **supprimer une branche ou un tag sur un distant** (depuis le menu de la ref, la palette, `branch delete-remote`, `tag delete-remote`, `tag delete --both`, ou une fin de workflow qui supprime la branche distante) : voir ci-dessous |

Tant qu'un merge, un cherry-pick ou un revert est arrêté sur des conflits,
**Enregistrer et marquer résolu**, le stage et l'unstage sont aussi annulables
(chacun est repris avant celui qui est dessous), et **Annuler le merge** (ou le
cherry-pick, le revert) est annulable : l'annuler ramène toute l'opération
arrêtée, avec ses conflits (les trois versions de chaque fichier non résolu),
les fichiers déjà résolus et stagés, les marqueurs de conflit dans l'arbre de
travail et le message de merge ; rétablir l'abandonne à nouveau. Le stage et les
résolutions cessent d'être annulables une fois l'opération terminée (commitée
ou abandonnée). *Unstage* et *Unstage de tout* ne touchent jamais un fichier
encore en conflit : il n'a aucune modification stagée à reprendre.

Tout ce qu'un abandon ou un reset hard jette est d'abord conservé, de sorte que
l'annuler restitue les octets exacts, fichiers non suivis, bits exécutables et
liens symboliques compris. Annuler un stash supprimé redonne le même stash, avec
le même id de commit (il se place au sommet de la liste des stashes, comme
`stash@{0}`, pas à son ancienne position). Annuler le retrait d'un distant
redonne ses branches de suivi, son alias `origin/HEAD` et les paramètres d'amont
des branches qui le suivaient.

### Supprimer une branche ou un tag sur un distant

Avant la suppression, GitPoulpi demande au serveur ce que contient la branche ou le tag
à cet instant et l'enregistre : le distant, le nom et le commit exact (pour un tag
annoté, l'objet tag lui-même, donc son annotation est conservée). La suppression n'est
faite que si le serveur contient encore cette valeur.

**Annuler la repousse** sous le même nom, au même commit, sans jamais forcer. Comme
c'est un changement sur un serveur que d'autres voient, Annuler **demande toujours
d'abord** et le dit (« Annuler recrée la branche topic sur origin à 3f2a1c0... ») ; les
étapes de l'entrée le disent aussi (`gitpoulpi undo --dry-run`). Il utilise les mêmes
identifiants qu'un push. La branche de suivi distant revient avec elle.
**Rétablir** la supprime de nouveau, avec la même sécurité.

Annuler refuse, sans rien changer, quand :

- la branche ou le tag a été **recréé sur le distant** depuis, ou déplacé : il est à un
  autre commit que celui enregistré, et pousser écraserait le travail de quelqu'un
  d'autre (le refus nomme les deux commits) ;
- le commit (ou l'objet tag) **n'est plus dans votre dépôt** (il a été supprimé par le
  ramasse-miettes) : l'annulation est impossible.

« Supprimer en local et sur le distant » est **une seule entrée** : Annuler restaure
ensemble la branche ou le tag local et distant, Rétablir les supprime de nouveau tous
les deux. Supprimer seulement la branche distante laisse la locale tranquille ;
l'annuler fait de même. Une fin de workflow qui supprime aussi la branche distante est
annulable en bloc.

## Ce qui ne peut pas être annulé, et pourquoi

- **La plupart de ce qui est sur un serveur** : push, push d'un tag, push LFS,
  verrouiller et déverrouiller, ouvrir une pull request. Cet état est partagé avec
  d'autres personnes ; annuler un push serait un force-push ou une suppression chez
  elles, et GitPoulpi ne le fera pas à votre place. (Supprimer une branche ou un tag
  distant est l'exception : l'annulation ne remet que ce que le serveur contenait, et
  refuse si quelque chose y a changé.) Annuler n'est d'ailleurs jamais bloqué par une
  telle opération.
- **Fetch** : il ne déplace que des refs de suivi distant, qui reflètent le
  serveur. Le prochain fetch le referait.
- **Abandonner un rebase** : l'annulation n'est pas proposée (un rebase déplace
  des branches et garde son propre état ; l'abandon est lui-même le chemin du
  retour). Abandonner un **merge, cherry-pick ou revert** *est* enregistré, voir
  plus haut.
- **Créer un worktree**, **retirer un worktree** (ce qui a été sauvegardé avant
  reste : le commit, le stash ; la branche supprimée avec lui est journalisée,
  donc Annuler rend la branche, pas le dossier), **créer un fichier de patch**, cloner, init, actions
  d'hébergeur (pull requests, tickets) et appels IA : ils ne font pas partie de
  l'historique du dépôt.
- **Créer un nouveau fichier vide** depuis le menu du fichier, la suppression
  automatique des fichiers `*.orig` après un merge et l'enregistrement d'un
  fichier dans l'éditeur intégré ne sont pas enregistrés. Changer la **commande
  SSH** (`core.sshCommand`) non plus, ni changer l'**extraction partielle** (rien n'est
  perdu ; *Désactiver* ramène tous les fichiers).
- **Écrire dans `.git/info/exclude` ou `.git/info/attributes`** (le choix « ce clone uniquement » des listes d'ignorance et de traitement binaire/texte) : ces fichiers sont hors de l'arbre de travail, ils ne sont donc pas enregistrés.
- **Les modifications de fichiers de l'arbre de travail faites en dehors de
  GitPoulpi** (un éditeur, un script, un agent IA qui écrit du code) ne sont
  jamais capturées : aucun instantané n'existe, donc un `git reset --hard` ou un
  `git checkout -- fichier` fait dans un shell perd pour de bon les modifications
  non commitées. Ce qui *est* capturé du travail d'un shell, c'est ce qui vit
  dans les refs : voir la section suivante.

## Changements faits en dehors de GitPoulpi

Quand vous travaillez avec un agent IA (ou un shell à côté de GitPoulpi), la plus
grande partie du travail git se passe hors de la fenêtre : commits, nouvelles
branches et worktrees, branches supprimées, resets, rebases, stashes. GitPoulpi
les voit aussi. Le démon garde les dernières refs qu'il a vues (branches
locales, tags, HEAD, liste des stashes, liste des worktrees) et, chaque fois que
le dépôt a changé sans que ce soit le fait de GitPoulpi, enregistre **une entrée
par changement cohérent** dans l'Historique d'annulation, marquée **extérieure**,
avec la raison que git a notée (`commit: fix login`, `rebase (finish)`,
`reset: moving to HEAD~1`, `checkout: moving from a to b`) et le nom que git a
enregistré pour l'auteur. GitPoulpi ne sait pas distinguer un shell d'un agent IA
et ne devine jamais ; l'entrée dit seulement qu'elle a été faite à l'extérieur.

- **Une rafale est une seule entrée.** Vingt commits faits entre deux
  observations forment une entrée, de la première valeur à la dernière ; un
  rebase est une entrée, enregistrée à sa fin (rien n'est enregistré tant
  qu'un rebase ou un merge est arrêté).
- **Les refs supprimées sont épinglées.** Une branche, un tag ou un stash qui a
  disparu est gardé en vie sous `refs/gitpoulpi/undo/` à son ancien commit avant
  que quoi que ce soit puisse l'élaguer : **Restaurer seulement celle-ci** (ou
  `Ctrl+Z`) ramène la branche que l'agent a supprimée, même après le reflog et un
  ramasse-miettes.
- **Pendant que GitPoulpi était fermé.** L'instantané est enregistré à côté du
  journal (`.git/gitpoulpi/refs-snapshot.json`) : ce qui s'est passé quand aucun
  GitPoulpi ne tournait est retrouvé au démarrage suivant et marqué « pendant
  que GitPoulpi était fermé ».
- **`Ctrl+Z` les traite comme n'importe quelle entrée** : les refs reviennent en
  arrière, une branche supprimée est recréée, un stash abandonné est de nouveau
  enregistré, un checkout extérieur rebascule sur l'ancienne branche. Une branche
  extraite recule comme avec `git reset --keep` : les fichiers suivent, et quand
  une de vos modifications locales gêne, l'annulation **refuse et ne change
  rien**. Rétablir la réapplique. **Restaurer seulement celle-ci** marche aussi :
  une branche supprimée revient et les actions suivantes restent ; une branche
  déplacée revient sous la forme `restore/<branche>-<sha>`.
- **La ligne d'état le dit une fois** : « changement extérieur enregistré -
  Ctrl+Z pour annuler » quand une nouvelle entrée extérieure arrive et que la
  fenêtre est au premier plan.
- Comme le changement extérieur est maintenant l'entrée la plus récente, l'ancien
  refus « bloqué par un changement extérieur » disparaît presque : vous annulez
  d'abord le changement extérieur.

Ce qui n'est pas enregistré : les déplacements de fetch seul des branches de
suivi distant (elles reflètent le serveur), les refs cachées de GitPoulpi, un
worktree ajouté ou retiré sans aucun changement de branche (l'entrée de la
branche qu'il porte dit « worktree ajouté <chemin> » ; annuler ne déplace que des
refs, cela ne retire pas le worktree), et tout ce qui est dans les fichiers.
`general.journal_outside = false` (Paramètres > Général, « Enregistrer les
changements faits hors de GitPoulpi ») désactive tout cela : rien d'extérieur
n'est enregistré et un changement extérieur bloque l'entrée la plus récente,
comme avant.

## Quand l'annulation refuse

Avant d'agir, GitPoulpi vérifie que tout ce que l'opération a touché est encore
exactement tel que l'opération l'a laissé. Sinon, il **ne change rien** et dit
ce qui a bougé :

- `main` est à 3f2a1c0, pas à a81b9e4 où Commit l'avait laissé (un commit fait
  dans un shell, par exemple) ;
- un fichier, l'index, un stash ou une section de configuration a été modifié
  depuis ;
- un merge, un rebase ou un cherry-pick est en cours ;
- l'opération ne s'est jamais terminée parce que GitPoulpi a été arrêté pendant
  qu'elle tournait (l'entrée est marquée *interrompue*, n'est jamais annulée
  automatiquement, et `gitpoulpi undo --list` montre ce qu'elle s'apprêtait à
  changer pour que vous le corrigiez à la main).

L'annulation est strictement dernier entré, premier sorti : une entrée la plus
récente bloquée bloque aussi les plus anciennes, car annuler une opération
ancienne sous une plus récente encore en vigueur reviendrait à deviner. Si vous
remettez les choses en place vous-même, l'entrée redevient valide. Les
modifications qui ne touchent rien de ce qu'une entrée a touché (un commit fait
dans un shell sur une autre branche, un fetch) ne bloquent rien.

### Une entrée bloquée que vous ne pouvez pas lever : Oublier

Quand l'entrée la plus récente est bloquée pour de bon (le dépôt a évolué en
dehors de GitPoulpi, ou l'entrée ne s'est jamais terminée), rien en dessous ne
peut non plus être annulé. L'infobulle d'Annuler et le refus le disent alors et
nomment la sortie : **Édition : Oublier l'annulation bloquée** dans la palette (elle demande
d'abord confirmation), ou `gitpoulpi undo-forget` en ligne de commande. Oublier
ne retire que cette entrée du journal ; le dépôt n'est jamais touché, et
l'entrée ne peut plus être annulée ni rétablie. L'entrée du dessous devient
alors la plus récente.

Deux cas se règlent d'eux-mêmes :

- un merge ou un rebase en conflit que vous terminez avec git simple : l'entrée
  est fermée la prochaine fois que GitPoulpi lit l'état du dépôt ;
- les fichiers d'un pop ou apply de stash en conflit que vous résolvez avec
  **Enregistrer et marquer résolu** : Annuler remet le conflit tant que le
  fichier résolu est encore stagé et non commité.

## Confirmations

Annuler ne demande confirmation que lorsque c'est risqué :

- le commit (ou merge, rebase, cherry-pick, revert) que vous reprenez est déjà
  sur l'amont de la branche : l'annuler réécrit un historique que vous avez
  publié, et vous devriez forcer le push du résultat ;
- vous avez des modifications non commitées et l'annulation déplace l'arbre de
  travail : vos modifications sont conservées, mais l'annulation s'arrête sans
  rien changer si l'un d'eux est dans un fichier qu'elle doit réécrire.

- l'entrée a supprimé une branche ou un tag sur un distant : l'annuler **la repousse
  sur le serveur** (voir plus haut), donc il demande toujours, tout comme le
  rétablissement qui la supprime de nouveau là-bas.

Annuler toute une séquence demande une seule fois pour la liste (dans le panneau), puis chaque entrée
risquée demande encore comme ci-dessus.

Rétablir, et toute annulation d'un simple déplacement de ref, ne demande
jamais : l'autre sens est à une touche.

## Jusqu'où on remonte

`general.undo_depth` (Paramètres > Général) est le nombre d'opérations conservées.
La valeur par défaut est **100** ; `0` signifie illimité. La baisser oublie
aussitôt les plus anciennes entrées. Lancer une nouvelle opération après une
annulation écarte ce que vous auriez pu rétablir, comme dans n'importe quel
éditeur.

L'historique est conservé par worktree (un worktree lié a le sien), dans le
dossier git (`.git/gitpoulpi/journal.json`), si bien qu'il survit à la
fermeture du programme et ne fait jamais partie de `git status` ni d'un commit.

## Le désactiver

`general.undo_enabled = false` (Paramètres > Général, « Annulation activée »)
désactive toute la fonction (et avec elle l'enregistrement des changements extérieurs). Rien n'est enregistré ni conservé, donc le stage,
l'abandon et le commit sont un peu plus rapides. Les boutons Annuler et
Rétablir disparaissent et `Ctrl+Z` affiche « L'annulation est désactivée ».

**Le désactiver efface l'historique** : le fichier journal et les refs cachées
décrites plus bas sont supprimés, la première fois qu'une requête atteint le
démon pendant qu'il est désactivé. Le réactiver repart d'un historique vide.

## Ce qui est stocké

Pour garantir qu'une annulation ne peut jamais perdre de données, chaque commit,
objet tag, stash ou instantané dont une entrée pourrait avoir besoin est gardé
en vie par une ref cachée sous `refs/gitpoulpi/undo/` aussi longtemps que
l'entrée est dans l'historique. Elles sont exclues des listes propres de
GitPoulpi, mais un `git log --all` ou `git for-each-ref` dans un shell les
montre, et `git push --mirror` les pousserait (utilisez plutôt `git push --all`).
Elles sont supprimées quand leur entrée sort de l'historique ou quand
l'annulation est désactivée. Rien d'autre n'est jamais supprimé.

## En ligne de commande et par MCP

```
gitpoulpi undo [--dry-run] [--expect <id>] [--yes]
gitpoulpi redo [--dry-run] [--expect <id>] [--yes]
gitpoulpi undo --list [--all]
gitpoulpi undo --sequence [--dry-run] [--yes]
gitpoulpi undo-forget [--dry-run] [--redo] [--expect <id>] [--yes]
```

- `--dry-run` décrit les étapes et ne change rien.
- `--list` affiche le journal, la plus récente en dernier (`--all` ajoute les
  entrées qui ne sont plus rétablissables mais encore conservées), avec l'id et
  l'état de chaque entrée.
- `--expect <id>` fait refuser l'appel comme *périmé* sauf si cette entrée est
  toujours la plus récente : un script (ou une seconde fenêtre) n'annule jamais
  autre chose que ce qu'il a regardé.
- `--sequence` annule l'exécution de séquence au sommet du journal (voir
  [Exécutions de séquences](#exécutions-de-séquences)) : les entrées consécutives les plus récentes qu'un seul
  `sequence run` a faites, de la plus récente à la plus ancienne, un `undo` chacune avec ses propres vérifications.
  `--dry-run` les liste ; sans `--yes`, la première entrée qui en demande un
  (code de sortie **3**) l'arrête, en nommant l'étape, avec ce qui a été annulé jusque-là dans
  `undone` et ce qui reste dans `left`. Un refus sort avec le code **2** et `stopped.refused` ;
  quand le sommet du journal n'est pas celui d'une même exécution, la réponse est
  `{"no_sequence": {...}}`, code 2. Les entrées portent leur exécution dans `undo --list`
  (`summary.origin` : `name`, `run_id`, `step`, `of`). Pas avec `--expect`.
- `undo-forget` retire du journal l'entrée la plus récente bloquée (`--redo` :
  l'entrée de rétablissement la plus récente). Il refuse quand cette entrée
  n'est pas bloquée, et sort avec le code **3** sans `--yes`. C'est une mutation
  pour MCP, mais non destructive : le dépôt n'est pas touché.
- Code de sortie **0** fait, **2** refusé (le JSON nomme la raison, par exemple
  `RemoteRefMoved` quand une branche distante supprimée a été recréée), **3** risqué
  et demande `--yes` (annuler la suppression d'une branche ou d'un tag distant, ce qui
  pousse vers le serveur).

Revenir en arrière sans tout annuler après coup :

```
gitpoulpi undo-restore <id> [--name N] [--dry-run]      (aussi : undo restore <id>)
gitpoulpi reflog [<ref>] [--skip N] [--limit N]
gitpoulpi reflog-restore <ref> <oid> [--mode keep|soft|mixed|hard] [--dry-run]   (aussi : reflog restore ...)
gitpoulpi lost-commits [--skip N] [--limit N] [--no-deep] [--budget-ms MS]
```

- `undo --list` montre, pour chaque entrée, ce que **restaurer seulement
  celle-ci** pourrait ramener (`restore.items`) ou pourquoi rien
  (`restore.not_restorable`), et qui l'a faite : `"source": "gitpoulpi"`, ou
  `"source": "outside"` avec `outside.message` (la raison notée par git) et
  `outside.committer` pour un changement fait par un shell ou un agent IA.
  `undo`, `redo` et `undo-restore` prennent les entrées extérieures comme les
  autres.
- `undo-restore <id>` ramène ce que cette seule entrée a supprimé (une branche
  ou un tag supprimé, un stash abandonné, un fichier abandonné ou supprimé, un
  distant retiré avec ses branches de suivi et ses amonts, une ligne d'ignore) et
  garde toutes les opérations suivantes. Il n'écrase jamais : un fichier qui a
  changé depuis reçoit la version conservée à côté, sous le nom
  `<nom>.restored` ; un nom de branche pris est un refus (code 2) qui en
  suggère un libre, donnez-le avec `--name` ; après un reset, un amend ou un
  rebase, il crée une nouvelle branche à l'ancien sommet et ne déplace jamais
  votre branche. La restauration est une entrée de journal, donc
  `gitpoulpi undo` la reprend.
- `reflog` liste où se trouvaient `HEAD` ou une branche, avec la raison donnée
  par git, y compris ce qu'un shell a fait. `reflog-restore` déplace une branche
  (ou `HEAD`) à l'une de ces positions ; une branche extraite avec des
  modifications non commitées demande `--mode`. Elle est aussi journalisée.
- `lost-commits` liste les commits qu'aucune branche, aucun tag, aucun distant
  ni aucun stash n'atteint. Pour en récupérer le travail sans le commiter :
  `apply-commit <rev>` (dans l'arbre de travail, tout ou rien) ou
  `stash from-commit <rev>` (dans un stash) ; les deux sont journalisées.
- Quand un déplacement fait dans un shell bloque l'entrée la plus récente, le
  refus porte la position du reflog qui contient ce que l'entrée attend
  (`reason.RefMoved.restore`), à passer à `reflog-restore`.

Deux commandes existent pour que les actions ci-dessus soient annulables depuis
toutes les interfaces : `gitpoulpi rm-file <paths>` (supprimer des fichiers de
l'arbre de travail) et `gitpoulpi ignore <pattern> [--file <path/.gitignore>]`
(ajouter une ligne à un `.gitignore` ; rien ne se passe quand la ligne y est
déjà ; l'annulation remet le fichier tel qu'il était, ou le supprime si la
commande l'a créé, et refuse si le fichier a été modifié depuis).

Le serveur MCP a un outil par commande, donc `undo`, `redo`, `rm-file` et
`ignore` sont aussi des outils. `undo`, `redo` et `rm-file` sont classés comme
destructifs (contrôlés comme `reset`, voir [MCP et agents](mcp-and-agents.md)) ;
`ignore` est une mutation ordinaire. Le journal est un fichier du dépôt, donc
`gitpoulpi undo` dans un shell reprend ce que la fenêtre a fait, et la fenêtre
peut annuler ce qu'un script a fait.
