[English](../staging-and-committing.md) · [Čeština](../cs/staging-and-committing.md) · [Deutsch](../de/staging-and-committing.md) · [Español](../es/staging-and-committing.md) · **Français** · [Italiano](../it/staging-and-committing.md) · [日本語](../ja/staging-and-committing.md) · [한국어](../ko/staging-and-committing.md) · [Nederlands](../nl/staging-and-committing.md) · [Polski](../pl/staging-and-committing.md) · [Português (Brasil)](../pt-BR/staging-and-committing.md) · [Português (Portugal)](../pt-PT/staging-and-committing.md) · [Русский](../ru/staging-and-committing.md) · [Svenska](../sv/staging-and-committing.md) · [Türkçe](../tr/staging-and-committing.md) · [Українська](../uk/staging-and-committing.md) · [简体中文](../zh-CN/staging-and-committing.md)

# Stage et commit

Comment voir ce que vous avez changé, choisir exactement ce qui entre dans un
commit, écrire le commit, et regarder les fichiers plus en profondeur (blame,
historique, l'éditeur intégré). Des termes comme *ligne WIP* et *hunk* sont
expliqués dans la [visite de l'interface](interface-tour.md).

## Les modifications locales

Le travail non commité est la **première ligne du Log** (cercle en pointillés,
`// WIP`, avec des compteurs `✎ n` modifiés et `+ n` ajoutés). Sélectionnez-la et
le volet de droite montre les modifications locales :

![Les modifications locales : fichiers unstaged et staged à droite, avec le formulaire de commit dessous](../images/hero.png)

- **Fichiers unstaged** : fichiers modifiés, ajoutés (non suivis) et supprimés qui
  ne sont pas encore dans le prochain commit.
- **Fichiers staged** : ce que contiendra le prochain commit.
- **Fichiers en conflit** (un `!` rouge, tant qu'un merge, un rebase, un cherry-pick ou un revert est arrêté) : `Enter` ouvre l'éditeur de merge, voir [Résoudre les conflits de merge](merge-conflicts.md).
- **Les fichiers renommés ou déplacés** sont une seule ligne, `nouveau <- ancien`, dans les deux listes (un fichier
  supprimé sur le disque et un nouveau dont au moins 50 % du contenu est identique comptent comme un déplacement
  avant même que vous les stagiez). Stage, Unstage et Abandon agissent sur les deux noms : stager stage la
  suppression et le nouveau fichier, si bien que git voit un seul renommage, et c'est UNE seule étape d'annulation (`Undo Stage rename ancien -> nouveau`). Ouvrir la ligne montre le diff du contenu
  par rapport à l'ancien nom, avec l'en-tête `ancien -> nouveau (N % de similarité)` ; un déplacement sans
  modification dit **Aucun changement de contenu**. Il en va de même pour les fichiers d'un commit, et
  `gitpoulpi status` / `gitpoulpi diff` rapportent `renamed_from`.
- **Chemins** ou **Arbre** (`m`) liste les fichiers à plat ou en dossiers. **Tout
  replier** (`z`) replie les dossiers. **Tous les fichiers** (`a`) montre tout
  l'arbre au lieu des seuls fichiers modifiés. Faites un clic droit sur l'espace vide de la liste pour
  **Tout déplier** / **Tout replier**, sur une ligne de dossier pour **Déplier tous les sous-dossiers** /
  **Replier tous les sous-dossiers** ; touches `=`, `-`, `*`, `_` (voir
  [Souris et gestes](mouse-and-gestures.md)).
- Le titre dit combien de fichiers ont changé sur quelle branche.

### Stage, unstage, abandon

| Pour | Faites |
|---|---|
| Stager un fichier | Survolez sa ligne et cliquez sur **Stage fichier**, ou sélectionnez-le et appuyez sur `Ctrl+Space`, ou `Ctrl+G s` |
| Stager un dossier | **Stage dossier** sur la ligne du dossier |
| Tout stager | **Stage de tout** dans le titre de la section, ou `Ctrl+G a` |
| Unstager | Les mêmes boutons et touches dans la section Staged (`Ctrl+G u`) |
| Abandonner les modifications d'un fichier | Clic droit sur le fichier > **Abandonner les modifications...** (demande d'abord ; `Ctrl+Z` ramène les modifications) |

### Fichiers sensibles

Stager un fichier, un dossier ou tout vérifie d'abord ce que cela ajouterait. Un fichier
dont le nom correspond à `[privacy] sensitive_patterns` (`.env` et `.env.*` sauf ceux en
`.example`, `.sample` et `.template`, `*.pem`, `*.key`, les clés privées SSH,
`*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`, `*.cer`, `*.der`, `*.kdbx`), ou
dont le texte renferme un secret que l'analyseur reconnaît, n'est pas stagé. La question
**Fichiers sensibles** liste chaque fichier avec sa raison (un motif, ou le type de secret
avec un aperçu masqué, jamais la valeur) et propose **1. Stage quand même** ou **2. Annuler,
rien n'est staged**. Aucune réponse n'est mise en évidence tant que vous n'appuyez pas sur
une flèche ou un chiffre, donc un Enter par mégarde ne répond à rien ; `Esc`, la croix de
fermeture et un clic à l'extérieur annulent. C'est la même question au clavier, avec un
bouton, un menu, la palette ou un glisser-déposer, car tous envoient la même requête.
Annuler ne stage rien, pas même les fichiers ordinaires du même stage.
Un secret trouvé dans un fichier nomme sa ligne (`config/app.env:12`). Pour regarder avant
de décider, cliquez une trouvaille (ou parcourez-les avec `Tab` / `Shift+Tab`, `Ctrl+J` /
`Ctrl+K`) et appuyez sur le bouton **Afficher dans le fichier**, `S`, ou sur la ligne de palette **Fichiers
sensibles : Afficher dans le fichier** : le fichier s'ouvre dans la vue Fichier, défilé jusqu'à cette ligne et la
surlignant, et la question revient intacte quand vous fermez la vue Fichier (`Esc`). Regarder ne stage rien.

Les fichiers texte jusqu'à 1 Mio chacun et 32 Mio au total sont lus ; les fichiers plus
gros et les fichiers binaires ne sont jugés que sur leur nom. `[privacy] staging_guard = false`
désactive la vérification. Stager un seul hunk ou quelques lignes n'est pas vérifié : vous
travaillez déjà sur ce fichier.

Faites un clic droit sur un fichier pour son menu : stage ou unstage, abandon,
copier le chemin, historique et blame du fichier, ouvrir dans votre éditeur
externe, outil de diff ou gestionnaire de fichiers (réglés dans
[Paramètres > Diff et éditeur](settings.md#diff-et-éditeur)), et plus.

## Ignorer des fichiers, et les traiter comme binaires ou comme texte

Clic droit sur un fichier > **Ignorer** pour l'ignorer lui ou son extension, ou sur une ligne de dossier > **Ignorer tous les fichiers du dossier**. GitPoulpi demande d'abord dans quel fichier écrire la règle : le `.gitignore` de la racine du dépôt (par défaut), le `.gitignore` de chaque dossier jusqu'à celui du fichier, ou `.git/info/exclude` (ce clone uniquement, jamais versionné). Un fichier qui n'existe pas encore est marqué *nouveau* et créé à la validation. La ligne est écrite relativement au fichier choisi : `src/a/b.txt` devient `/a/b.txt` dans `src/.gitignore`. Déplacez-vous avec les flèches, tapez pour filtrer la liste, `Enter` écrit, `Esc` annule. `Ctrl+Z` annule une écriture dans un `.gitignore` ; `.git/info/exclude` est hors de l'arbre de travail et n'est pas annulable.

![La liste qui demande dans quel fichier va une règle d'ignorance : le .gitignore du dépôt, celui du dossier du fichier ou .git/info/exclude, chacun avec la ligne qui serait écrite](../images/ignore-picker.png)

Clic droit sur un fichier > **Traiter comme binaire** (ou **Traiter comme texte** quand git le traite déjà comme binaire) écrit une règle `.gitattributes` : `binary`, ou `text diff`. Choisissez le fichier seul ou tous les fichiers de son extension, puis le `.gitattributes` où écrire, avec la même liste que ci-dessus (et `.git/info/attributes` pour ce clone uniquement). Une règle déjà écrite pour le même motif est remplacée, jamais contredite. Le diff du fichier se met à jour aussitôt, et la ligne d'état dit ce qui a été écrit et où (et prévient si une autre règle l'emporte encore). Une ligne de dossier a **Traiter les fichiers du dossier comme** > **Binaire** ou **Texte**. Les mêmes règles sont dans la palette de commandes (**Modifications locales : Traiter le fichier sélectionné comme binaire...**) et en ligne de commande (`gitpoulpi treat binary|text <chemins>`). La vue Fichier suit aussi la règle : un fichier traité comme texte montre ses lignes même s'il contient un octet NUL.

## Menus, sélection multiple et conversion dans les listes de fichiers

Les listes de fichiers du panneau de droite (Unstaged, Staged, les fichiers modifiés d'un commit et **Voir tous les fichiers**) ont les mêmes menus et les mêmes gestes. Faites un clic droit sur un fichier, une ligne de dossier ou la partie vide d'une liste, ou appuyez sur `.` (`Shift+F10`, la touche Menu) sur la ligne qui a le focus, ou lancez **Fichiers : ouvrir le menu de la ligne ou de la sélection** depuis la palette.

- **Un fichier d'un commit** (et un fichier de **Voir tous les fichiers**) : **Ouvrir le diff**, **Ouvrir le fichier**, **Historique du fichier**, **Blame du fichier**, ouvrir dans votre éditeur, dans le programme par défaut ou dans **Ouvrir avec...**, **Afficher dans le dossier**, **Modifier le fichier**, **Copier le chemin du fichier**, **Copier le chemin complet du fichier**, **Ignorer**, **Traiter comme binaire** ou **Traiter comme texte**, **Convertir**, **Restaurer le fichier depuis ce commit** (demande d'abord ; comme `git checkout <commit> -- <fichier>`, cela stage aussi le fichier, et `Ctrl+Z` ramène le fichier précédent) et **Supprimer le fichier...**. Une ligne qui concerne la copie de travail est grisée, avec sa raison, quand le fichier n'est pas dans l'arbre de travail. **Diff avec le répertoire de travail** ouvre le diff de la version du commit contre le fichier sur disque. **Créer un patch depuis les modifications du fichier...** écrit un patch de la modification de ce fichier dans le commit (une sélection et un dossier l'ont aussi ; dans **View all files** sur les changements en cours, il crée le patch des modifications du fichier).
- **Une ligne de dossier** : **Ouvrir le dossier**, **Copier le chemin du fichier**, **Ignorer tous les fichiers du dossier**, **Traiter les fichiers du dossier comme**, **Convertir**, **Créer un fichier dans ce dossier**.
- **La partie vide d'une liste** : **Créer un fichier** et **Ouvrir le dossier du dépôt**. La liste Unstaged ajoute **Stage de tout** et **Abandonner toutes les modifications...**, la liste Staged **Unstage de tout**.

![Le menu contextuel d'un fichier d'un commit : ouvrir le diff, l'historique et le blame du fichier, ouvrir dans un éditeur ou le programme par défaut, copier les chemins, ignorer, convertir, restaurer depuis le commit et supprimer](../images/commit-file-menu.png)

### Sélectionner plusieurs fichiers

Les gestes sont ceux du Log. **Ctrl+clic** ajoute une ligne à la sélection ou la retire. **Shift+clic** (**Alt+clic** dans les terminaux qui gardent Shift+clic) prend la plage depuis l'ancre. Un clic simple recommence et ouvre le diff comme avant. `Esc` efface la sélection. Au clavier, `Space` ou `Insert` bascule la ligne courante dans toutes les listes (dans Unstaged et Staged, `Ctrl+Space` stage la sélection, ou la ligne courante) et `Shift+Up` / `Shift+Down` étendent la plage. Une ligne de dossier sélectionne tous les fichiers qu'elle contient, repliée ou non. Les lignes sélectionnées sont surlignées et la ligne d'état les compte. `Ctrl+A` sélectionne toutes les lignes de la liste active.

Faites un clic droit sur une ligne sélectionnée (ou appuyez sur `.`) pour le menu de toute la sélection, titré **N fichiers** : stage ou unstage, abandon, ignorer, traiter comme binaire ou texte, convertir, copier les chemins, restaurer depuis le commit, supprimer. Stage, unstage et restauration sont une action chacun ; l'abandon et la suppression demandent d'abord et disent combien de fichiers. Un clic droit sur une ligne hors de la sélection recommence la sélection avec cette ligne.

![Quatre fichiers sélectionnés dans la liste des fichiers non indexés, avec le menu intitulé 4 fichiers : indexer, annuler, ignorer, traiter comme binaire ou texte, convertir, copier les chemins et supprimer](../images/file-pane-multiselect.png)

### Convertir l'encodage, les fins de ligne et l'indentation

**Convertir** est dans le menu d'un fichier, d'un dossier ou d'une sélection. `gitpoulpi convert` fait la même chose en ligne de commande.

- **Encodage** : UTF-8, UTF-8 BOM, UTF-16 LE et BE, Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK, Big5, EUC-KR.
- **Fins de ligne** : LF, CRLF ou CR.
- **Indentation** : espaces ou tabulations en début de ligne, avec la largeur d'une tabulation.

Les fichiers de l'arbre de travail sont réécrits en arrière-plan avec les mêmes fonctions que la ligne d'état de l'éditeur. Un fichier qui n'est pas de l'UTF-8 valide et n'a pas de marque d'ordre des octets est lu comme du Windows-1252, comme dans l'éditeur. Les fichiers binaires, absents ou que l'encodage cible ne peut pas représenter sont ignorés et signalés (*12 fichiers convertis, 3 ignoré(s) (3 binaire(s))*), jamais abîmés ; un fichier qui ne peut pas être écrit déclenche une notification d'échec. Tout le lot est une seule entrée du journal d'annulation : un `Ctrl+Z` remet tous les fichiers. Un fichier ouvert dans l'éditeur bloque la conversion tant qu'il n'est pas fermé.

## Lire un diff

Cliquez sur un nom de fichier (ou appuyez sur `d` dessus) et son diff remplace le
Log au centre. `Esc` ou `q` ramène le Log.

![Un diff avec deux hunks, chacun avec des boutons Abandonner le hunk et Stage du hunk](../images/diff.png)

La barre au-dessus change la vue :

- **En ligne** montre le diff unifié ; **Côte à côte** met l'avant et l'après
  l'un à côté de l'autre (`m` fait défiler les dispositions).
- **Hunks** (`Shift+H`) ne montre que les hunks modifiés avec une ligne de
  contexte. Désactivez-le pour voir tout le fichier.
- **Esp.** (`s`) masque les modifications qui ne diffèrent que par les espaces.
- **Retour** (`w`) replie les longues lignes au lieu de les tronquer.
- **Carte** (`Shift+M`) montre une minimap de tout le fichier à côté de la barre de
  défilement. Cliquez dessus pour sauter. Les autres occurrences d'un mot
  sélectionné sont teintées, et marquées sur la barre de défilement.
- `n` et `p` (ou les flèches de la barre) sautent d'un hunk à l'autre.
- **Auto** (le dernier bouton, `Shift+C`) est l'encodage de lecture du fichier. Cliquez dessus (ou
  appuyez sur `Shift+C`, ou lancez *Diff : Choisir l'encodage...* depuis la palette) et choisissez-en
  un dans la liste : **Détection automatique** d'abord, puis UTF-8, UTF-8 BOM, UTF-16 LE et BE,
  Windows-1252, ISO-8859-1, ISO-8859-15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS, EUC-JP, GBK,
  Big5 et EUC-KR (tapez pour filtrer la liste). Les deux côtés du fichier sont relus, en octets
  exacts, dans cet encodage : un fichier Latin-1 affiche ses accents et un fichier UTF-16 (que git
  tient pour binaire) affiche ses lignes. Le choix est retenu pour le chemin de ce fichier jusqu'à
  la fermeture, dans cette vue et dans la [revue](review.md). Tant qu'un encodage est choisi, les
  hunks sont ceux de cette vue, donc stager et abandonner sont désactivés (choisissez
  **Détection automatique** pour stager de nouveau). Une lecture lente affiche *lecture en ...* dans
  le titre après un instant.

![Le même diff en disposition côte à côte, avant et après](../images/diff-split.png)

Vous pouvez sélectionner du texte à la souris (glisser, ou double-clic sur un
mot) et le copier avec `y` ou `Ctrl+C` ; seul le texte du fichier est copié,
jamais les numéros de ligne ni les marqueurs `+` et `-`.

### Stager un hunk ou une seule ligne

Sur un diff de l'arbre de travail, chaque en-tête de hunk a **Stage du hunk** et
**Abandonner le hunk** (côté Staged, **Unstage du hunk**).

- `Space` stage le hunk sous le curseur.
- Pour stager quelques lignes, appuyez sur `v` pour démarrer une sélection,
  déplacez-vous avec `j`/`k`, et appuyez sur `Enter`. Seules les lignes
  *modifiées* comptent.
- `x` abandonne le hunk et `Shift+X` les lignes sélectionnées. Les deux demandent
  d'abord, et `Ctrl+Z` ramène les modifications.
- Clic droit (ou `.`) sur un hunk pour son menu : stage, unstage, abandon,
  historique du fichier, blame, copie.

Le diff d'un commit est en lecture seule. Une image modifiée s'affiche comme une
image ; voir [Images](image-view.md).

## Écrire le commit

Le formulaire de commit est en bas du volet de droite, et son **Résumé** est aussi
le champ de la ligne des modifications locales du Log : taper dans l'un apparaît dans
l'autre.

![Le formulaire de commit rempli avec un message écrit par l'IA : résumé avec compteur de caractères, description, et le bouton de commit vert](../images/ai-commit-message.png)

- **Résumé** (le compteur indique combien de caractères sur 72 restent) et
  **Description**. `Enter` dans le résumé passe à la description, `Enter` dans
  celle-ci ajoute une ligne.
- **Commiter N fichiers** (le bouton vert) ou `Ctrl+Enter` commite les modifications
  stagées. Si rien n'est stagé, le bouton le dit et ne fait rien.
- **Amender le commit précédent** (`Shift+A`) fond les modifications stagées et le
  nouveau message dans le dernier commit. Un commit déjà poussé demandera alors un
  force push (avec lease).
- **Ignorer les hooks** (`Shift+H`, ou **Commit : Basculer Ignorer les hooks** dans la palette) commite
  sans exécuter les hooks `pre-commit` et `commit-msg` du dépôt. Le choix vaut pour un seul commit ;
  **Paramètres > Paramètres du dépôt > Commit** permet de la cocher au départ. Elle a sa propre ligne juste sous **Amender le commit précédent**, avec son libellé complet ; seul un volet très étroit rétrécit le libellé en **Sans hooks**, puis en la seule case (son infobulle la nomme). Elle ne disparaît jamais. Sur un formulaire très court, elle partage la ligne d'Amender, en rétrécissant de la même façon.
- `c` dans le volet Status place le curseur dans le formulaire. `Esc` le quitte.
- `Ctrl+G c` et `Ctrl+G Enter` (**Git : Commiter...** et **Git : Commiter le stage
  maintenant** dans la palette) forment une seule commande, le bouton de commit
  depuis n'importe où. Avec un résumé dans le formulaire et quelque chose de stagé,
  elle commite aussitôt, en respectant **Amender le commit précédent** et
  **Ignorer les hooks**, sans déplacer le focus. Sinon elle ne commite rien : elle
  place le curseur dans le résumé du formulaire (en affichant le volet s'il était
  masqué ou replié) et la ligne d'état indique quel raccourci commite (`Ctrl+Enter`
  par défaut). Sans rien de stagé, elle le dit. Rien ne se saisit dans la ligne
  d'état. Tant qu'une requête IA est en cours, elle ne commite rien et le signale
  (`Esc` annule la requête). `Ctrl+Z` annule le commit comme n'importe quel autre.
- Par dépôt, les [Paramètres](settings.md#propre-au-dépôt-commit) peuvent **pousser après chaque commit**, faire
  démarrer le formulaire avec un **modèle de commit** (et retirer ses commentaires `#` du message) et rendre un merge
  **squash** : le formulaire s'ouvre alors avec le message de squash de git.
- Avec une intégration d'hébergeur liée, `Alt+R` dans le formulaire insère une
  référence de ticket ; voir [Hébergeurs et suivi de tickets](hosting-integration.md).

Avec l'[IA](ai.md) activée, le bouton **`✦`** à droite du résumé écrit un message
à partir des modifications stagées (`Ctrl+G i`), et **Composer des commits avec l'IA**
répartit votre travail en plusieurs commits, ci-dessous. Le texte va dans le
formulaire pour que vous le modifiiez ; rien n'est commité tant que vous
n'appuyez pas sur le bouton de commit.

**Composer des commits avec l'IA** est à l'extrémité droite de la ligne Modifier et rétrécit avec le volet de droite
au lieu de disparaître : le libellé complet, puis **Composer avec l'IA**, puis le seul **`✦`**. Il garde son clic, son
infobulle et sa touche (`Ctrl+G Shift+C`) à toutes les largeurs.

Les boutons **`✦`** (générer le message) et **`#`** (insérer une référence de ticket) à l'extrémité droite de la ligne du
résumé rétrécissent de la même façon : dans un volet étroit ils perdent leurs marges, puis se replient en un seul bouton
**`⋯`** dont le menu les liste tous les deux, si bien qu'aucun ne disparaît.

## Le compositeur de commits : répartir vos modifications en commits

*(Demande que l'IA soit activée : voir [Assistance IA](ai.md).)* Appuyez sur
**Composer les commits avec l'IA** (ou `Ctrl+G C`, ou clic droit sur la ligne des
modifications locales). Le modèle propose plusieurs commits, chacun avec ses fichiers ;
chaque fichier modifié est dans exactement un commit, et un plan invalide est
renvoyé une fois au modèle. La proposition s'ouvre comme une revue où rien n'a
encore été commité :

![Le compositeur de commits : quatre commits proposés, chacun déployable jusqu'à ses fichiers, avec des contrôles monter, descendre, modifier et abandonner et les boutons Réinitialiser, Annuler et Créer 4 commits](../images/composer.png)

| Pour | Faites |
|---|---|
| Déplier ou replier un commit | `Left` / `Right`, ou cliquez sur le chevron ; `*` bascule tout |
| Monter ou descendre un commit | Les `▲` `▼` de sa ligne, ou `Alt+Up` / `Alt+Down` (`K` / `J`) |
| Déplacer un fichier vers un autre commit | Glissez-le sur le commit (depuis la liste, ou depuis la liste des modifications locales à droite) ; ou `m` sur le fichier et choisissez le commit dans la liste déroulante ; ou `Alt+Up` / `Alt+Down` sur le fichier |
| Mettre un fichier dans un commit à part | `n` sur le fichier, ou le bouton `+ own commit` |
| Reformuler un commit | Le crayon, `e`, `Enter` ou un double-clic : modifiez le message, `Enter` le garde, `Esc` l'abandonne |
| Laisser un commit de côté | `drop` (`d` ou `Delete`) ; ses fichiers restent non commités. `restore` le ramène |
| Recommencer | **Réinitialiser** (`r`) revient au plan du modèle |
| Les créer | **Créer N commits** (`Ctrl+S`, `Ctrl+Enter` ou `a`) |
| Quitter | **Annuler** ou `Esc` : rien n'est fait |

Créer les commits unstage tout, puis pour chaque commit stage ses fichiers et
commite, si bien que votre arbre de travail n'est jamais touché. Si une étape
échoue, il s'arrête et dit combien de commits ont été faits. Ce sont des
fichiers entiers qui sont stagés : un fichier qui n'était stagé qu'en partie est
commité en entier. Choisissez quel profil et quelles instructions le compositeur
utilise dans *Paramètres > IA* ; voir [Assistance IA](ai.md#le-compositeur-de-commits).

## Vue Fichier, Blame et Historique du fichier

La barre de vue a **Vue fichier** (le contenu du fichier), **Vue diff**, **Blame** et le
sélecteur `Fichier │ Tout` de la [colonne des commits](interface-tour.md#la-colonne-des-commits)
(sa liste `Fichier` est l'Historique du fichier) ; les mêmes sont dans le menu clic droit d'un fichier. Ils montrent
le fichier tel qu'au commit d'où vous venez, ou tel que non commité (`@ WIP`)
quand vous l'avez ouvert depuis les modifications locales.

- **Blame** et l'**Historique du fichier** sont des vues à part entière : voir [Blame](log-and-history.md#blame) et [Historique du fichier](log-and-history.md#historique-du-fichier) dans *Travailler avec le Log*. `Enter` sur un hunk du blame ou une ligne de l'historique va à ce commit dans le Log.
- `y` ou `Ctrl+C` copie le texte sélectionné (dans Blame, seulement le code).
- `Shift+G` saute à la dernière ligne. `q` ferme la vue.

## L'éditeur intégré

**Éditer** (`e`) ouvre la *copie du dossier de travail* du fichier dans un petit
éditeur au-dessus du volet central : coloration syntaxique, numéros de ligne,
annuler et rétablir, recherche et une ligne d'état (position, fin de ligne,
indentation, langage). Il modifie toujours la copie de l'arbre de travail, même quand vous
l'avez ouvert depuis le diff d'un ancien commit.

![L'éditeur montrant un fichier Rust avec coloration syntaxique et numéros de ligne ; la barre d'outils a Enregistrer, Annuler, Rétablir et Chercher](../images/editor.png)

| Touche | Action |
|---|---|
| `Ctrl+S` | Enregistrer (le fichier est écrit dans un fichier temporaire puis renommé, il n'est donc jamais à moitié écrit ; fins de ligne, BOM et saut de ligne final sont préservés) |
| `Ctrl+Z`, `Ctrl+Y` | Annuler, rétablir |
| `Ctrl+F`, `F3` | Chercher, chercher le suivant |
| `Tab`, `Shift+Tab` | Indenter, désindenter (selon le style du fichier) |
| `Ctrl+A`, `Ctrl+C`, `Ctrl+X`, `Ctrl+V` | Tout sélectionner, copier, couper, coller |
| `Esc` | Fermer. Avec des modifications non enregistrées, il demande **Enregistrer** (`s`), **Abandonner** (`d`) ou **Annuler** (`c`) |

Les fichiers binaires sont refusés. La
taille de tabulation, les fins de ligne, la coloration syntaxique, les numéros de
ligne et le retour à la ligne sont dans *Paramètres > Diff et éditeur* ; voir
[Paramètres](settings.md#diff-et-éditeur).

**Suivre la sélection** : Tant que l'éditeur est ouvert, sélectionner un autre fichier dans le volet de droite l'ouvre aussi dans l'éditeur, après avoir proposé d'enregistrer les modifications non sauvegardées. Un fichier qui ne peut pas être édité s'ouvre dans sa vue habituelle. Choisir un fichier d'un commit ou d'une comparaison quitte l'éditeur pour le diff de ce fichier, en demandant d'abord **Enregistrer**, **Abandonner** ou **Annuler** s'il y a des modifications non enregistrées ; la colonne des commits, que l'éditeur n'a pas, revient avec le diff. Annuler garde le fichier actuel. La fermeture de l'éditeur ramène sur la vue du fichier sélectionné. Activé par défaut.

La ligne d'état, à droite, indique l'**encodage**, les **fins de ligne** et l'**indentation**
du fichier, puis la position du curseur et la syntaxe. Les trois premiers sont des
boutons : chacun ouvre une liste déroulante (flèches ou molette, taper pour filtrer,
`Enter` pour choisir, `Esc` pour fermer). La palette (`Ctrl+P`) ouvre les mêmes menus : *Éditeur : Changer l'encodage*, *Éditeur : Rouvrir avec l'encodage*, *Éditeur : Convertir vers l'encodage*, *Éditeur : Convertir les fins de ligne*, *Éditeur : Changer l'indentation*.

![Le menu d'encodage de l'éditeur ouvert sous la pastille UTF-8 de la barre : rouvrir le fichier avec un autre encodage, ou le convertir](../images/editor-encoding-menu.png)

![Le menu des fins de ligne de l'éditeur ouvert sous la pastille LF : convertir le fichier en LF, CRLF ou CR](../images/editor-eol-menu.png)

- **Encodage.** Un fichier s'ouvre dans l'encodage qu'il annonce (un BOM : UTF-8 ou
  UTF-16), sinon en UTF-8, sinon en Windows-1252, et la barre dit lequel. **Rouvrir
  avec l'encodage** relit les octets du fichier avec un autre encodage (il demande
  d'abord si vous avez des modifications non enregistrées) : utilisez-le quand la
  détection s'est trompée. **Convertir vers l'encodage** garde le texte et change
  l'encodage dans lequel le fichier est enregistré (UTF-8, UTF-8 BOM, UTF-16 LE et BE,
  Windows-1252, ISO-8859-1 et -15, Windows-1250, Windows-1251, KOI8-R, Shift_JIS,
  EUC-JP, GBK, Big5, EUC-KR). Rien n'est écrit avant l'enregistrement, et Annuler
  revient en arrière. Si le texte contient des caractères que la cible ne peut pas
  représenter, la ligne d'état indique combien et où est le premier, et rien n'est
  converti ; l'enregistrement refuse de la même façon, il n'écrit jamais `?`.
- **Fins de ligne.** Affiche `LF`, `CRLF`, `CR`, ou `Mixte` quand le fichier en mêle
  plusieurs. Le menu convertit tous les sauts de ligne du fichier en LF, CRLF ou CR
  en une seule étape d'annulation. Les nouvelles lignes que vous tapez suivent la fin
  de ligne du fichier. Votre choix l'emporte sur le paramètre *Fins de ligne*.
- **Indentation.** Affiche `Espaces : 4` ou `Tabulations : 4`. Le menu choisit ce qu'insèrent
  `Tab` et l'indentation automatique (espaces ou tabulations), la taille (1 à 8 : le
  pas d'indentation et la largeur d'une tabulation dans ce fichier), et convertit les
  blancs de début de chaque ligne en espaces ou en tabulations, en une seule étape
  d'annulation.

## Stash, patches et Git LFS

- **Stash** et **Pop** sont dans la barre d'outils ; voir
  [Travailler avec le Log](log-and-history.md#branches-tags-et-le-panneau-refs).
- **Créer un patch** et **Appliquer un patch** sont dans les menus du commit et des
  refs ; la CLI a `gitpoulpi patch`.
- Les gros fichiers stockés avec Git LFS apparaissent comme de petits fichiers
  pointeurs jusqu'à ce que vous récupériez leur contenu ; voir [Git LFS](lfs.md).

## En ligne de commande

Tout ce qui est sur cette page est scriptable :

```
gitpoulpi --json status
gitpoulpi stage --file src/lib.rs
gitpoulpi stage --all --allow-sensitive   # after the refusal below
gitpoulpi --json diff
gitpoulpi commit -m "feat: add the helper"
gitpoulpi discard --file src/lib.rs --hunk 1
```

`gitpoulpi stage` refuse un fichier sensible de la même façon : rien n'est stagé, le code
de sortie est 2 et `--json` affiche `{error: "sensitive_files", files: [{path, reason}]}`
(une règle et un aperçu masqué, jamais une valeur). `--allow-sensitive` est la nouvelle
tentative. Un agent MCP reçoit la même réponse et ne peut pas passer `allow_sensitive`
sauf si `[mcp] allow_sensitive_staging = true` : on lui dit de vous demander.

Voir la [référence CLI](cli-reference.md).
