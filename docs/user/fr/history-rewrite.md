[English](../history-rewrite.md) · [Čeština](../cs/history-rewrite.md) · [Deutsch](../de/history-rewrite.md) · [Español](../es/history-rewrite.md) · **Français** · [Italiano](../it/history-rewrite.md) · [日本語](../ja/history-rewrite.md) · [한국어](../ko/history-rewrite.md) · [Nederlands](../nl/history-rewrite.md) · [Polski](../pl/history-rewrite.md) · [Português (Brasil)](../pt-BR/history-rewrite.md) · [Português (Portugal)](../pt-PT/history-rewrite.md) · [Русский](../ru/history-rewrite.md) · [Svenska](../sv/history-rewrite.md) · [Türkçe](../tr/history-rewrite.md) · [Українська](../uk/history-rewrite.md) · [简体中文](../zh-CN/history-rewrite.md)

# Réécriture de l'historique

Réécrire l'historique modifie des commits qui existent déjà : qui les a écrits et
quand, leurs messages, les fichiers qu'ils contiennent, leurs signatures.
GitPoulpi le fait avec une **recette** (un périmètre et une liste ordonnée de
règles) et trois étapes dont aucune ne saute la deuxième :

1. **Aperçu** calcule le nouvel historique sans rien toucher : totaux, effet de
   chaque règle, branches et tags qui bougeraient, une ligne par commit modifié.
2. **Appliquer** y déplace vos branches et tags locaux. C'est une entrée de
   l'[historique d'annulation](undo-redo.md) : **Annuler** remet chaque ref
   déplacée (et les fichiers extraits) à sa place.
3. **Push** (un push forcé avec lease) et **Purge** sont des étapes à part,
   volontaires, parce qu'elles sont irréversibles : voir
   [Pousser le résultat](#pousser-le-résultat) et
   [Supprimer l'ancien historique](#supprimer-lancien-historique-purge).

L'atelier, la palette, les commandes `gitpoulpi rewrite` et les outils MCP
exécutent le même moteur : un aperçu fait dans l'un est ce qu'applique un autre.
[En ligne de commande](#en-ligne-de-commande) liste les options ;
[Créer des règles avec l'IA](#créer-des-règles-avec-lia) remplit l'atelier à partir d'une description ;
[Propositions des agents IA](#propositions-des-agents-ia) dit ce qu'un agent peut
et ne peut pas faire ici.

**Les règles, par ce qu'elles changent**

| Groupe | Règles | Section |
|---|---|---|
| Auteurs | Remplacer un auteur, Anonymiser les auteurs, Appliquer le .mailmap | [Auteurs et .mailmap](#auteurs-et-le-mailmap) |
| Dates | Décaler les dates, Changer de fuseau horaire, Rester dans des plages horaires, Date du committer = date de l'auteur | [Dates](#dates) |
| Messages | Chercher et remplacer, Supprimer des lignes, Ajouter ou retirer un trailer, Corriger l'encodage des messages | [Messages](#messages) |
| Contenu et chemins des fichiers | Chercher et remplacer dans les fichiers, Retirer des fichiers, Retirer les gros fichiers, Déplacer ou renommer des chemins, Convertir les fins de ligne, Retirer le bit exécutable, Convertir l'encodage des fichiers | [Fichiers et chemins](#fichiers-et-chemins) |
| LFS | Migrer des fichiers vers LFS, Ramener des fichiers depuis LFS | [Déplacer des fichiers vers ou hors de Git LFS](#déplacer-des-fichiers-vers-ou-hors-de-git-lfs) |
| Signatures | Retirer les signatures (et re-signature à l'application) | [Signatures](#signatures) |
| Script | Lua, shell, Python | [La règle Script](#la-règle-script) |

**Avant de commencer**

- Un aperçu ne change rien. Appliquer ne change que vos refs locales et ne pousse
  jamais.
- Les commits réécrits reçoivent de nouveaux identifiants. Tous ceux qui ont
  l'ancien historique doivent être prévenus : [Pousser le résultat](#pousser-le-résultat)
  rédige la note.
- Un secret qui a été poussé est compromis. **Renouvelez-le d'abord**, puis
  réécrivez, poussez en force et purgez.
- Les commits qui restent tels quels gardent leurs identifiants et leurs
  signatures.

## L'atelier

`Ctrl+G Shift+H` (ou **Réécriture : ouvrir l'atelier** dans la palette) ouvre
l'atelier de réécriture par-dessus le Log. Il n'a pas de bouton dans la barre
d'outils : les actions avancées se trouvent dans la palette (chaque ligne
`Réécriture : ...` ouvre l'atelier avec la règle correspondante déjà ajoutée),
dans le sous-menu **Réécrire l'historique** du menu d'un commit (*De ce commit à
HEAD...*, *Commits sélectionnés...* quand plusieurs sont sélectionnés, *Ouvrir
l'atelier...*) et derrière ce raccourci.

`Ctrl+N` affiche un badge sur chaque contrôle de l'atelier (périmètre, règles, champs, onglets, boutons) ; appuyer sur un badge fait ce que ferait un clic sur ce contrôle, et `Esc` retire les badges (voir [Badges](keybindings.md#badges)).

![L'atelier de réécriture de l'historique : la barre de portée, la liste de deux règles Remplacer un auteur avec les champs de la règle sélectionnée, et le résumé de l'aperçu qui dit combien de commits changent](../images/history-rewrite.png)

- **Barre du périmètre :** les refs qui seront déplacées (branche courante, toutes
  les branches locales, branches et tags au choix, commits sélectionnés), la
  case *Tags* et, après *De ce commit à HEAD...*, une case pour ne commencer
  qu'après le parent de ce commit.
- **Règles :** une liste numérotée, appliquée dans l'ordre. La case active ou
  désactive une règle, les flèches la déplacent, la croix la supprime
  (*Rétablir la règle supprimée* ou `Ctrl+Z` la ramène). *Ajouter une règle*
  classe les types par groupe. Une règle qui nécessite un programme absent reste
  grisée, avec la raison.
- **Paramètres d'une règle :** un clic sur un champ texte le modifie ; un clic
  sur un champ à plusieurs valeurs ouvre sa liste ; les cases se cochent et se
  décochent.
- **Aperçu** (`F5`) calcule le nouvel historique sans rien modifier et affiche
  une barre de progression. Les onglets : **Résumé** (totaux, effet de chaque
  règle, avertissements), **Commits** (une ligne par commit modifié, chargée par
  pages depuis le démon ; le filtre liste les commits modifiés, supprimés ou
  poussés), **Branches et tags** (ce qui bouge, et les branches protégées déjà
  poussées, chacune avec une case), **Impact** (ce que signifie réécrire un
  historique poussé : un push forcé avec lease, les collaborateurs, les
  signatures), **Secrets**, **Identités** et **Taille** (voir plus bas).
- **Fichiers d'un commit :** `Enter` (ou un double-clic) sur une ligne de
  l'onglet Commits ouvre la liste des fichiers que les règles de contenu y ont
  modifiés et, à côté, la différence du fichier sélectionné. La croix, `Esc` ou
  un clic à l'extérieur la ferme. Les lignes retirées par une règle de secrets
  sont masquées.
- **Appliquer** (`Ctrl+S` ou `Ctrl+Enter`) devient disponible dès que l'aperçu
  correspond aux règles. Les questions sans réponse par défaut sont posées à
  chaque fois : que faire des signatures des commits signés, et comment extraire
  un dossier. Le panneau de résultat propose **Annuler**, qui replace chaque ref
  déplacée à sa position d'origine.
- **Enregistrer / Charger** conservent une recette dans le dépôt (partagée) ou
  pour vous seul ; les valeurs secrètes ne sont jamais enregistrées.

Chaque bouton a un raccourci : la liste figure dans Paramètres > Clavier et
souris (contexte *Réécriture*) et dans le guide `keybindings.md`.

### Auteurs et le .mailmap

- **Remplacer un auteur** repère un auteur ou un committer par son nom et/ou son
  e-mail (un texte littéral ou une expression régulière) et lui donne un nouveau
  nom et/ou un nouvel e-mail. *Appliquer à* choisit l'auteur, le committer ou les
  deux ; *Inclure aussi les auteurs des tags* fait suivre les tags annotés qui
  sont recréés. Le committer d'origine est conservé, sauf si l'exécution dit
  *moi, maintenant*.
- **Anonymiser les auteurs** remplace chaque identité par un nom numéroté
  (`Contributor 1`, `contributor-1@anonymous.invalid` ; les deux modèles sont à
  vous, avec `{n}` pour le numéro). Une même personne reçoit toujours le même
  numéro au cours d'une exécution ; les identités que vous listez comme *à garder*
  sont laissées telles quelles.
- **L'onglet Identités** (aussi **Réécriture : fusionner des identités
  (.mailmap)...** dans la palette) sert à corriger une personne qui a commité
  sous plusieurs noms et adresses **sans rien réécrire**. Il liste chaque nom et
  adresse du périmètre choisi, avec le nombre de commits écrits et
  commités, et ceux que le `.mailmap` du dépôt associe déjà (`associée à Nom
  <email>`). Cochez les identités qui sont la même personne (`Space`, ou un
  clic ; *Tout cocher* / *Tout décocher*), puis **Fusionner vers...** (`m`) : une
  boîte demande laquelle est conservée (celle qui a le plus de commits est en
  premier) et **ajoute au `.mailmap` une ligne par autre identité** (`Nom <email>
  Ancien Nom <ancien@email>` ; une ligne déjà présente n'est pas répétée, le
  fichier est créé s'il manque). Rien n'est commité : commitez vous-même le
  `.mailmap` pour le partager. Le Log, le blame et l'historique d'un fichier
  affichent aussitôt le nom canonique (réglage `log.use_mailmap`, activé par
  défaut ; les commits enregistrés ne changent jamais, il n'y a donc rien à
  pousser en force). La ligne d'état dit ce qui a été écrit et propose
  **Annuler** (`u`), qui remet l'ancien `.mailmap` (c'est une entrée de
  l'historique d'annulation comme les autres). `r` relit la liste.
- **Un `.mailmap` plutôt qu'une réécriture.** Quand une règle d'auteur modifie
  des commits qui sont déjà sur un distant, le Résumé dit *Un .mailmap corrige
  l'affichage sans réécrire. Utiliser .mailmap à la place ?* avec un bouton
  **Utiliser .mailmap** (`m` sur le Résumé). Il ouvre l'onglet Identités avec les
  identités visées par la règle cochées, et l'identité que la règle imposerait
  proposée comme celle à conserver, même si aucun commit ne la porte encore.
- **La règle `.mailmap`** (*Réécriture : appliquer .mailmap à l'historique...*, ou
  `rewrite preview --mailmap`) réécrit bel et bien : les auteurs et les
  committers (à votre choix) deviennent ce que le `.mailmap` de l'arbre de
  travail leur donne, le résultat de `git log --use-mailmap`. Utilisez-la quand
  l'historique lui-même doit porter les noms canoniques, par exemple avant de le
  publier ; c'est une réécriture comme les autres (aperçu, Appliquer, push forcé).
- **Règles d'identité et trailers.** Une règle d'identité peut aussi changer les
  personnes nommées dans les trailers : cochez **Aussi les trailers**
  (`--map-trailers`). Tout trailer dont la valeur est exactement `Nom <email>` et
  correspond à *L'e-mail est* / *Le nom est* de la règle reçoit le nouveau nom et
  le nouvel e-mail, quoi que la règle fasse à l'auteur et au committer. C'est
  désactivé par défaut : un `Signed-off-by` atteste qui a signé, le modifier est
  donc une décision à part.

### Dates

Les quatre règles de dates agissent sur la date de l'auteur, celle du committer
ou les deux (*Dates* dans le volet de la règle).

- **Décaler les dates** décale chaque date d'une durée (`+2h`, `-1d`, `90m`,
  `1d 2h`). Le fuseau horaire dans lequel chaque commit a été écrit est conservé.
- **Changer de fuseau horaire** écrit les dates avec un autre décalage UTC.
  *Conserver* dit ce qui reste fixe : l'**instant** (le même moment, l'horloge
  affiche une autre heure) ou l'**heure affichée** (la même heure locale, l'instant
  change).
- **Rester dans des plages horaires** déplace chaque commit qui tombe hors des
  heures autorisées (par exemple `Mon-Fri 18:00-23:30; Sat,Sun 10:00-20:00`) vers
  l'heure autorisée la plus proche, ou seulement plus tôt, ou seulement plus tard
  (*Déplacer vers*). *Jamais avant son parent* conserve l'ordre de l'historique ;
  la *Graine* rend reproductible le petit écart aléatoire : la même graine sur le
  même historique donne les mêmes dates.
- **Date du committer = date de l'auteur** donne au committer (nom, e-mail et
  date) ceux de l'auteur, ce qu'un `git commit --amend` ou un rebase a défait.

Les dates sont des règles comme les autres : aperçu, vérification dans l'onglet
**Commits** (chaque ligne montre l'ancienne et la nouvelle date), puis
application. L'identifiant d'un commit dépend de ses dates : une règle de dates
change donc tous les commits situés après le premier qu'elle touche.

### Messages

- **Chercher et remplacer dans les messages** et **Supprimer des lignes de
  message** prennent un texte littéral ou une expression régulière (*Expression
  régulière*, *Respecter la casse*) ; un remplacement vide supprime. Un message qui
  n'est pas de l'UTF-8 valide n'est jamais touché par ces règles : lancez d'abord
  **Corriger l'encodage des messages** si vous voulez le modifier.
- **Trailers.** *Ajouter ou retirer un trailer* (*Réécriture : ajouter ou retirer
  des trailers...* dans la palette, ou `--trailer-add`, `--trailer-remove`,
  `--trailer-map`) modifie les lignes `Clé: valeur` à la fin d'un message, celles
  que connaît `git interpret-trailers` (`Signed-off-by`, `Co-authored-by`,
  `Reviewed-by`...). La liste **Action** choisit *Ajouter* (la clé **Trailer**, sa
  **Valeur**, et **S'il existe déjà** : *Ajouter (sauf s'il est identique)*,
  *Remplacer les existants* ou *Laisser le commit tel quel*), *Retirer* (tous les
  trailers de cette clé, ou seulement ceux dont la valeur contient **Valeur
  contenant**) ou *Modifier la valeur* (**Chercher** dans la valeur devient
  **Remplacer par**, par exemple un ancien e-mail par un nouveau dans
  `Signed-off-by`). Les clés se comparent sans tenir compte de la casse. Seules les
  lignes de trailers changent, le reste du message reste exactement tel quel, et un
  message qui n'a pas encore de trailers reçoit une ligne vide et un bloc à lui.
  L'onglet Commits montre chaque changement sous la forme `+Signed-off-by: ...` /
  `-Co-authored-by: ...`. Le dernier paragraphe d'un message est son bloc de
  trailers (jamais le titre) : une ligne comme `Note: voir le ticket` dans le corps
  n'est donc pas un trailer, sauf si elle est seule dans le dernier paragraphe avec
  d'autres trailers.
- **Identifiants de commit dans les messages.** **Actualiser les ids de commit**
  (`--update-message-hashes`, désactivé par défaut ; en bas du volet des règles qui
  modifient les messages) fait suivre la réécriture aux identifiants écrits dans
  les messages : un identifiant abrégé de 7 caractères hexadécimaux ou plus
  (`This reverts commit 1a2b3c4`, `voir 1a2b3c4d5e`) qui désigne exactement un
  commit de l'historique réécrit est remplacé par le nouvel identifiant de ce
  commit, coupé à la même longueur. Un identifiant qui correspond à plusieurs
  commits, à aucun, ou à un commit que la réécriture crée plus tard est laissé tel
  quel ; le Résumé dit combien ont été remplacés et combien laissés.
- **Corriger l'encodage des messages** (*Réécriture : corriger l'encodage des
  messages...*, ou `--fix-message-encoding [--fix-name-encoding]`) réencode les
  messages en UTF-8, dans deux cas :
  - Un commit avec un en-tête `encoding` (`ISO-8859-1`, `Windows-1252`,
    `Shift_JIS`, tout ce que git et les tables d'encodage connaissent) est décodé
    et écrit en UTF-8, et l'en-tête disparaît : le texte se lit pareil partout,
    sans conversion. Les noms de l'auteur et du committer de ce commit étaient dans
    le même encodage et sont décodés avec lui, si bien qu'un commit n'est jamais
    converti à moitié.
  - Le **mojibake**, de l'UTF-8 qui a été lu comme du Latin-1 ou du Windows-1252
    (`RÃ©Ã©criture` pour `Réécriture`), n'est réparé **que si l'aller-retour est
    exact** : le texte doit se réencoder en octets qui sont de l'UTF-8 valide, et
    abîmer le résultat de la même façon doit redonner l'original. Un message qui
    contient simplement `Ã©` à côté de vraies lettres, ou des caractères hors des
    anciens alphabets, ne passe pas et reste tel quel.

  *Corriger aussi les noms* (`--fix-name-encoding`) répare de la même façon les
  noms d'auteur et de committer abîmés. Ce qui ne peut pas être décidé (un nom
  d'encodage inconnu, des octets qui ne sont pas valides dans l'encodage déclaré,
  un message qui n'est pas de l'UTF-8 et ne déclare rien, un texte qui a l'air
  abîmé sans qu'on puisse le prouver) est **laissé tel quel et compté** : le Résumé
  dit *n messages laissés tels quels : leur encodage ne peut pas être décidé*, et
  `counts.encoding_left` donne le nombre. L'onglet Commits montre chaque
  correction avec l'ancien et le nouveau message et une ligne *encodage*
  (`ISO-8859-1` devient `UTF-8`).

### Fichiers et chemins

- **Règles sur le contenu des fichiers** (*Chercher et remplacer dans les
  fichiers*, *Retirer des fichiers de l'historique*, *Retirer les gros
  fichiers*) : le volet vérifie chaque valeur pendant la saisie et affiche la
  raison sous le champ (un motif qui ne compile pas, un motif de chemin
  incorrect, une taille inférieure à un octet). Les tailles acceptent des unités
  (`500 KiB`, `5 MiB`, `1 GiB`). Un motif de chemin sans barre oblique
  correspond à n'importe quelle profondeur. Les *Valeurs secrètes* se saisissent
  dans un champ masqué, ne servent que pour l'exécution en cours et ne sont
  jamais enregistrées ; l'aperçu masque les lignes qui les contenaient. *Commits
  vides* indique si un commit que les règles ont laissé vide est supprimé (par
  défaut) ou conservé. Les fichiers binaires et ceux qui dépassent la limite de
  taille sont laissés tels quels et comptés dans le Résumé.
- **Déplacer des chemins.** *Déplacer ou renommer des chemins* (*Réécriture :
  déplacer ou renommer des chemins...* dans la palette) a trois variantes, choisies
  dans la liste **Déplacement** :
  - *Renommer un chemin* (**De**, **Vers** ; `--move-path FROM=TO`) : les mêmes
    fichiers et dossiers apparaissent au nouveau chemin dans chaque commit qui
    avait l'ancien ; rien d'autre ne change, chaque fichier garde son identifiant.
    Un commit où le nouveau chemin existe déjà n'est jamais écrasé : l'aperçu est
    refusé et liste les commits et les chemins en conflit (choisissez un nom libre,
    ou déplacez d'abord l'existant). Chemins exacts seulement, sans jokers.
  - *Tout déplacer dans un dossier* (**Dossier** ; `--into-subdir DIR`) : chaque
    chemin de chaque commit reçoit le préfixe. Utile pour fusionner des dépôts sans
    collision de noms.
  - *Extraire un dossier comme racine* (**Dossier** ; `--extract DIR`) : le dossier
    devient la racine, les commits qui ne le touchent pas sont supprimés (un commit
    qui le crée n'a pas de parent), les merges qui ne réunissent plus rien sont
    simplifiés, et les branches et tags qui n'ont rien du dossier sont supprimés.
    On vous demande **à chaque fois** comment cela se termine :
    - *Dans un nouveau clone, ouvert dans un nouvel onglet* (`--extract-mode
      clone`) : une copie de ce dépôt est faite dans un dossier que vous choisissez
      (le nouveau dépôt reçoit un dossier portant le nom du dossier extrait), y est
      réécrite, puis nettoyée pour ne contenir que le nouvel historique, chaque
      branche et chaque tag local qui a le dossier. Ce dépôt n'est pas touché. Le
      nouveau dépôt n'a pas de remote (l'original n'est pas son amont).
    - *Sur place : l'historique de ce dépôt est réécrit* (`--extract-mode
      in-place`) : la réécriture habituelle, aperçu, Appliquer, annulation. Les
      branches sans le dossier sont supprimées (l'annulation les ramène) ; la
      branche extraite doit l'avoir.
    - *Comme sous-module de ce dépôt* (`--extract-mode submodule`) : le nouveau
      dépôt est créé comme dans le cas du clone, puis **un nouveau commit** sur la
      branche courante remplace le dossier par un sous-module qui pointe vers lui
      (`.gitmodules` et le lien), le dossier restant un sous-module extrait.
      L'historique de ce dépôt n'est **pas** réécrit : ses anciens commits gardent
      les fichiers du dossier, rien de ce qui pointe vers eux (tags, autres clones)
      ne casse. L'URL de `.gitmodules` est le chemin local du nouveau dépôt :
      changez-la (`git config -f .gitmodules submodule.<dossier>.url`) une fois qu'il
      est sur un hébergeur. Des fichiers modifiés ou non suivis dans le dossier le
      refusent. Annuler remet le dossier ordinaire.

    Le Résumé dit que l'aperçu montre l'historique du nouveau dépôt et dans quel
    dossier il ira. Les règles qui viennent après une extraction (un renommage, un
    retrait) s'appliquent à l'arbre extrait ; pour un clone ou un sous-module,
    l'extraction doit être la première règle de chemin. Les règles de chemin
    s'exécutent après les règles de contenu : un glob de suppression désigne donc les
    chemins tels qu'ils étaient.

#### Fins de ligne, bit exécutable et encodage des fichiers

Trois règles du groupe **Contenu** du menu Ajouter (ou les lignes de palette
*Réécriture : convertir les fins de ligne...*, *Retirer le bit exécutable...* et
*Réécriture : convertir l'encodage des fichiers...*). Toutes trois prennent
**Fichiers** (des globs ; un motif sans barre oblique correspond à n'importe quelle
profondeur ; vide veut dire tous les fichiers) et **Sauf les fichiers**. Elles ne
changent que ce que les globs désignent, dans chaque commit du périmètre. Les
fichiers binaires (un octet NUL dans les 8 premiers Kio) ne sont jamais touchés :
le Résumé les compte comme *laissés tels quels*.

- **Convertir les fins de ligne** (`--eol lf|crlf|cr --eol-paths GLOB...
  [--eol-exclude GLOB...]`). *Convertir en* : LF (Linux, macOS), CRLF (Windows) ou
  CR (Mac classique). Chaque fin de ligne d'un fichier texte concerné devient celle
  choisie : un fichier aux fins de ligne mélangées ressort uniforme ; un fichier
  déjà correct garde sa version intacte. *Suivre .gitattributes* (désactivé par
  défaut, `--eol-follow-attributes`) lit le `.gitattributes` racine de chaque
  commit : les fichiers marqués `-text` ou `binary` sont ignorés et ceux marqués
  `eol=lf` ou `eol=crlf` gardent cette fin de ligne plutôt que celle que vous avez
  choisie.
- **Retirer le bit exécutable** (`--strip-exec GLOB... [--strip-exec-exclude
  GLOB...]`). Les fichiers concernés passent du mode 100755 au mode 100644 ; leur
  contenu n'est pas touché. Au moins un motif de fichier est exigé : retirer tous
  les exécutables casserait vos scripts.
- **Convertir l'encodage des fichiers** (`--reencode TARGET [--from SOURCE]
  [--encoding-fallback ENC] --encoding-paths GLOB... [--encoding-exclude
  GLOB...]`). *Convertir en* : UTF-8 (par défaut), UTF-8 avec BOM, UTF-16 LE ou BE
  (écrits avec une marque d'ordre des octets), Windows-1252 ou ISO-8859-1. *Les
  fichiers sont en* : par défaut, l'encodage de chaque fichier est détecté (une
  marque d'ordre des octets, sinon de l'UTF-8 valide, sinon le *Repli*,
  Windows-1252 si vous ne le changez pas) ; nommez l'encodage pour l'imposer, mais
  un mauvais choix donne un texte faux (de l'UTF-8 lu comme du Latin-1 double
  chaque accent). Un fichier que l'encodage source ne sait pas lire, ou qui contient
  un caractère que la cible ne peut pas stocker (un signe euro en ISO-8859-1), reste
  exactement tel quel et est compté dans le Résumé.

Les règles s'exécutent dans l'ordre de la liste, sur le même fichier : convertir
un fichier en UTF-16 puis demander des fins de ligne le laisse tel quel (ce n'est
plus du texte simple). Appliquer est une seule entrée de la liste d'annulation,
comme toute réécriture.

### Chercher des secrets

- **Chercher des secrets :** `gitpoulpi rewrite scan-secrets` (et l'outil MCP
  `rewrite_scan_secrets`) lit une fois l'historique et liste les préfixes de
  jetons (GitHub, GitLab, Slack, AWS, Stripe, npm, Google, OpenAI, Anthropic),
  les clés privées, les JWT, les mots de passe dans des URL et les lignes
  `password = valeur` qui semblent aléatoires, chacun avec son fichier, son
  premier et son dernier commit et un aperçu masqué comme `ghp_****abcd`. La
  recherche n'est pas exhaustive, et la valeur n'est jamais affichée, journalisée
  ni enregistrée. `--remove-finding ID` transforme un résultat en règle qui le
  retire de chaque commit. Un secret qui a été poussé est compromis :
  renouvelez-le d'abord, puis réécrivez, poussez en force et purgez.
- **L'onglet Secrets** fait la même chose depuis l'atelier, pour le périmètre
  choisi : **Chercher des secrets** (`s`) lance la recherche comme une
  tâche (la barre de progression affiche *Analyse*, **Annuler** ou `Esc` l'arrête).
  Chaque résultat est une ligne avec son type, l'aperçu masqué, le fichier et le
  nombre d'autres fichiers qui le contiennent ; sous la liste : le premier et le
  dernier commit où il a été vu, et combien de commits et de versions de fichier
  le contiennent. L'onglet indique combien de fichiers binaires ou trop gros n'ont
  pas été lus, et que la recherche n'est pas exhaustive. Tous les résultats
  démarrent cochés ; **Retirer de l'historique** (`r`) transforme les résultats
  cochés en une seule règle qui les désigne par identifiant (jamais par valeur),
  que vous prévisualisez et appliquez comme n'importe quelle autre. Les résultats
  deviennent périmés après une réécriture ou un changement de périmètre : l'onglet
  le dit, relancez la recherche. L'atelier ne détient jamais la valeur d'un
  secret, il ne peut donc pas l'afficher.

### Trouver ce qui pèse

- **Trouver ce qui pèse :** `gitpoulpi rewrite analyze [--top N]` (et l'outil MCP
  `rewrite_analyze`) est l'analyse de taille, comme `filter-repo --analyze` : les
  plus gros fichiers avec leur chemin, leur taille, leur premier et leur dernier
  commit et s'ils sont encore dans HEAD, les dossiers les plus lourds, les
  fichiers supprimés de HEAD que l'historique stocke encore, et les totaux. Elle
  ne lit que les tailles et les chemins, jamais le contenu des fichiers, et ne
  modifie rien.
- **L'onglet Taille** fait la même chose depuis l'atelier (`F7`, ou *Analyser les
  tailles de l'historique* dans la palette, l'ouvre et lance l'analyse) :
  **Analyser les tailles** (`s`) est une tâche (*Analyse*, **Annuler** ou `Esc`
  l'arrête). **Afficher** choisit la liste (`v`) : *Fichiers* (les plus grosses
  versions de fichier distinctes, chacune marquée *dans HEAD*, *ancienne version*
  d'un fichier qui existe toujours, ou *supprimé*), *Dossiers* (tout ce qui a
  jamais été stocké en dessous de chacun) ou *Supprimés* (les chemins sortis de
  HEAD que l'historique garde). **Tri** (`o`) se fait par taille, par chemin ou en
  mettant les supprimés d'abord. Cochez des lignes (`Space`, `Alt+A` / `Alt+U`
  pour tout / rien) : l'onglet dit ce qu'elles pèsent, honnêtement « jusqu'à
  environ », car l'espace n'est rendu qu'après la réécriture, une purge des
  anciens objets et un `gc`. **Retirer de l'historique** (`r`) transforme les
  chemins cochés en règle de chemins (caractères spéciaux protégés, un chemin déjà
  dans une règle n'est pas ajouté deux fois) ; **Retirer tout fichier de plus de
  ...** (`b`, liste Fichiers) écrit une règle de taille juste en dessous du plus
  petit fichier coché. Prévisualisez et appliquez-les comme n'importe quelle autre
  règle. Les listes montrent les 200 plus grosses lignes ; la CLI dit combien il
  y en a de plus et `--top` les liste. Les lignes deviennent périmées après une
  réécriture ou un changement de périmètre : relancez l'analyse.

### Signatures

- **Signatures :** un commit réécrit ne peut pas garder sa signature (elle
  signait les anciens octets) : quand des commits signés sont réécrits, on vous
  demande donc, à chaque fois : *Retirer les signatures* (le nombre est affiché
  d'abord), *Re-signer seulement les commits qui étaient signés* ou *Re-signer
  chaque commit réécrit*. Il n'y a pas de réponse par défaut. Les commits qui
  restent tels quels gardent la leur. La re-signature utilise votre propre
  configuration, comme un commit signé : `gpg.format` (`openpgp`, `ssh` ou
  `x509`), `user.signingkey` et `gpg.program` / `gpg.ssh.program` /
  `gpg.x509.program`. La quatrième ligne de la question, **Vérifier la signature**
  (`4`), signe une fois une charge factice et la vérifie : elle dit combien de
  temps dure une signature et estime l'exécution pour le nombre de commits
  concernés, ou pourquoi cela ne marche pas (clé absente, agent verrouillé,
  programme non installé, avec la façon de l'installer). L'aperçu fait le même
  test avant de lire l'historique et refuse avec cette raison : une configuration
  défaillante ne laisse donc jamais une réécriture à moitié signée. Les commits
  sont signés **pendant l'aperçu** (leurs identifiants contiennent la signature) :
  un aperçu qui re-signe prend donc le temps de la signature, le Résumé dit
  *Commits signés à nouveau : n, en 12,5 s*, et une demande de phrase secrète peut
  s'ouvrir à ce moment : déverrouillez d'abord votre agent (Vérifier la signature
  fait exactement cela). **Annuler** s'arrête entre deux signatures et ne laisse
  rien derrière lui. Seules les signatures de commits sont recréées ; un tag signé
  perd toujours sa signature.
- **La règle *Retirer les signatures*** (menu Ajouter, groupe *Signatures* ;
  *Réécriture : retirer les signatures...* dans la palette ; `--remove-signatures`)
  retire la signature de chaque commit du périmètre, même quand rien d'autre ne
  change, et ne pose aucune question sur la re-signature. En CLI : `rewrite
  check-signing` et `--signatures strip|resign-signed|resign-all`.

### Annuler une réécriture

**Annuler** (le bouton du panneau de résultat, *Annuler* de la barre d'outils avec
`Ctrl+Z` dans le Log, ou `gitpoulpi undo`) remet chaque branche, tag et note
déplacés, et les fichiers extraits avec eux, à partir des anciens commits que la
réécriture garde épinglés. Il demande d'abord confirmation quand une branche
réécrite est déjà sur son amont (annuler en local n'annule pas un push) et quand
vous avez des modifications non commitées : elles sont conservées, et un fichier
que vous avez modifié et que l'ancien historique écraserait arrête l'annulation (les
fichiers bougent comme avec `reset --keep`). Annuler cesse d'être possible pour une
réécriture dès que vous [purgez](#supprimer-lancien-historique-purge) son ancien
historique ; la liste de l'historique la marque alors *purgée*.

## Déplacer des fichiers vers ou hors de Git LFS

Deux règles du groupe *LFS* du menu Ajouter réécrivent l'historique pour que les
gros fichiers vivent dans [Git LFS](lfs.md) dès l'origine (ou en sortent). Elles
nécessitent le programme `git-lfs` : sans lui, les règles sont grisées avec le
conseil d'installation et un aperçu est refusé. C'est notre propre moteur, pas
`git lfs migrate` : vous avez donc l'aperçu, le périmètre, l'annulation et la
protection du push de toute autre règle.

- **Migrer des fichiers vers LFS** (*Réécriture : migrer des fichiers vers
  LFS...*, ou `--lfs-import PATTERN` avec `--lfs-min-size BYTES`). Chaque fichier
  qui correspond aux motifs (`*.psd`, `assets/**` ; un motif sans barre oblique
  correspond à n'importe quelle profondeur) et pèse au moins *Seulement les fichiers
  d'au moins* (vide : toute correspondance) devient un pointeur LFS dans chaque
  commit qui le contient. Son contenu va dans le stockage LFS local
  (`.git/lfs/objects`, une fois par contenu distinct) à l'application, et le
  `.gitattributes` racine reçoit `<motif> filter=lfs diff=lfs merge=lfs -text` dans
  chaque commit réécrit qui contient un pointeur (créé s'il n'y en a pas, les autres
  lignes intactes, jamais en double). Un fichier qui est déjà un pointeur est laissé
  tel quel. Le Résumé dit *3 fichiers (12 MiB) passent en LFS. Objets LFS à
  écrire : 3.* et l'onglet Commits liste *passe big.psd en LFS* par commit.
- **Ramener des fichiers depuis LFS** (*Réécriture : ramener des fichiers depuis
  LFS...*, ou `--lfs-export PATTERN`). Les pointeurs qui correspondent redeviennent
  les vrais fichiers, et les lignes de `.gitattributes` de ces motifs sont retirées.
  Les objets doivent être dans le stockage LFS local : quand certains n'y sont pas,
  l'aperçu est refusé avec la liste et un bouton **Télécharger les objets LFS** qui
  les récupère depuis `origin` ; refaites ensuite l'aperçu. Un import suivi d'un
  export des mêmes motifs redonne exactement les mêmes commits.

Après Appliquer, les fichiers extraits sont le vrai contenu (pas des pointeurs) et
**Annuler** remet l'ancien historique. Le panneau de résultat a **Envoyer les
objets LFS** (`l`) : la même chose que `LFS : Push All Objects`, pour pouvoir
pousser le nouvel historique sans qu'un push ultérieur reste bloqué sur des objets
manquants (un simple push des branches réécrites les envoie aussi, par le hook
pre-push qu'installe Appliquer). La note pour les collaborateurs dit qu'ils ont
besoin de git-lfs et de `git lfs pull`.

Bon à savoir : Appliquer installe les filtres LFS du dépôt (`git lfs install
--local`) quand ils manquent, parce que git en a besoin pour comparer les vrais
fichiers aux pointeurs. Seul le `.gitattributes` racine est écrit, jamais un
fichier imbriqué. Un fichier *sous* la taille minimale qui correspond à un motif
reste un fichier ordinaire, mais git voudra le transformer en pointeur à son
prochain ajout : `git status` l'affiche donc comme modifié jusqu'à ce que vous le
commitiez. L'import lit les gros fichiers en flux, l'export ne garde qu'un fichier
en mémoire à la fois.

## Pousser le résultat

Appliquer une réécriture ne pousse jamais rien. Les branches et les tags réécrits
qui étaient sur un distant demandent un push forcé, et ce push ne force jamais à
l'aveugle : chaque ref est envoyée avec un lease sur l'identifiant que le distant
avait au moment de l'aperçu (et non sur ce que dit maintenant votre branche de
suivi), si bien que le push d'un collègue entre-temps est refusé pour cette ref au
lieu d'être écrasé, et les refs partent dans un seul push tout ou rien
(`--atomic`) quand le serveur le permet. Depuis la CLI :

```
gitpoulpi rewrite push --dry-run        # interroge le distant, n'envoie rien
gitpoulpi rewrite push --yes            # l'envoie
gitpoulpi rewrite note                  # ce que les autres doivent lancer
gitpoulpi rewrite maps --out ./maps     # commit-map et ref-map
```

Les branches protégées (réglage `rewrite.protected_branches`, par défaut `main`,
`master`, `develop`, `release/*`) sont laissées de côté une fois poussées, à la
réécriture puis de nouveau au push, sauf si vous les nommez
(`--include-protected main`) ou les incluez toutes. La protection de branche de
l'hébergeur décide toujours si le push forcé est accepté ; son refus s'affiche
ref par ref.

Dans l'atelier, le panneau de résultat après un Appliquer fait tout cela :

- **Envoyer les références réécrites au distant** liste les refs que la
  réécriture a déplacées et qu'un distant possède, chacune avec une case
  (`Space`). Les branches protégées démarrent décochées, avec *protégée* à côté :
  cochez-les une par une, ou utilisez *Tout cocher* / *Tout décocher*
  (`Alt+A` / `Alt+U`). **Push les branches réécrites** (`p`) envoie les refs
  cochées ; la réponse est une ligne par ref (*poussé*, *déjà à jour*, *refusé : le
  distant a bougé depuis l'aperçu*, *refusé : protégé par l'hébergeur*, *ignoré :
  branche protégée*, *non poussé*). Une ligne refusée a **Fetch et examiner**
  (`f`) : l'atelier se ferme et les distants sont récupérés, pour que le Log
  montre ce que les autres ont poussé. Rien n'est écrasé : le lease protège leur
  travail, et vous refaites un aperçu avant de réessayer.
- **Copier la note** (`c`) place sur le presse-papiers la note pour les
  collaborateurs, dans la langue de l'interface autour des commandes git, qui ne
  sont pas traduites.
- **Ouvrir la carte des commits** (`m`) montre où se trouvent la carte des commits
  et la carte des refs ; un clic sur un chemin (ou `1` / `2`) le copie.

Un push ne s'annule pas : si vous faites ensuite un `undo` de la réécriture,
seul votre dépôt change, et l'annulation le dit avant de demander confirmation.
Envoyez la note à tous ceux qui ont l'ancien historique : elle indique l'ancien
et le nouveau sommet de chaque branche et de chaque tag, et les commandes à lancer
(`git fetch`, puis `git reset --hard origin/<branche>` ou `git rebase --onto`).
Les deux cartes sont gardées 24 heures avec l'exécution ; copiez-les ailleurs si
le nettoyage de dépôt de GitLab ou un script en a besoin.

## Quand l'hébergeur protège une branche

Votre hébergeur peut protéger une branche de son côté (protection de branche et
rulesets de GitHub, branches protégées de GitLab, stratégies de branche d'Azure
DevOps, restrictions de branche de Bitbucket) et refuser le push forcé dont une
réécriture a besoin. Quand le dépôt est **lié** à son hébergeur (la section des
pull requests de la barre latérale, voir
[Intégration des hébergeurs](hosting-integration.md)), l'atelier demande à
l'hébergeur quelles branches il protège, avec votre propre jeton :

- Ces branches sont **laissées de côté par défaut** dans la réécriture, exactement
  comme les branches protégées de `rewrite.protected_branches` et du workflow :
  l'onglet Refs les liste avec une case chacune et les boutons *Tout cocher* /
  *Tout décocher*, et dit pourquoi : *protégée sur GitHub*.
- Si le jeton ne peut pas lire la protection (il n'a pas la portée requise, ou vous
  n'êtes pas administrateur), l'atelier dit *Protection de l'hébergeur inconnue* et
  ne considère rien comme protégé sur la seule parole de l'hébergeur. La
  réécriture n'est jamais bloquée pour autant. Un dépôt qui n'est pas lié n'est
  jamais interrogé.

Quand une branche protégée gêne (l'aperçu a été refusé parce que toutes les
branches sont protégées, ou l'hébergeur a rejeté le push : *refusé : protégé par
l'hébergeur*), le bouton **Comment contourner** (`Alt+H`) explique les trois
façons de passer, avec la page de réglages exacte de votre fournisseur et de votre
dépôt (sélectionnez un lien et copiez-le avec `c`, ou ouvrez-le avec `o`).
GitPoulpi n'en fait jamais aucune à votre place :

1. **Lever temporairement la protection** dans les réglages de l'hébergeur, pousser,
   puis la remettre.
2. **Pousser la réécriture sous un nouveau nom de branche**, puis faire de cette
   branche la branche par défaut (ou renommer), et supprimer l'ancienne quand plus
   personne ne l'utilise.
3. **En dernier recours, supprimer et recréer le dépôt distant** et tout y pousser.
   Cela fait perdre les pull / merge requests et leurs revues, les tickets, le lien
   des forks avec l'ancien historique, les étoiles et les observateurs, les
   réglages, secrets et variables de CI, ainsi que les webhooks et les clés de
   déploiement.

En ligne de commande, le même texte revient sous `protected_by_host` dans le JSON
de `rewrite preview` / `rewrite run` (quand l'hébergeur protège une branche du
périmètre), d'un `rewrite push` refusé et de `rewrite note` : `provider`,
`branches`, `ways` (chacune avec ses `steps` et `links`), `recreate_loses` et
toute l'explication dans `text`.

## Supprimer l'ancien historique (purge)

Après une réécriture, les anciens commits sont toujours dans votre dépôt :
l'annulation de la réécriture les garde (c'est ce qui la rend possible), tout
comme les reflogs. Si vous avez réécrit pour vous débarrasser d'un secret, ce
secret est toujours dans le dossier `.git`. La purge supprime l'ancien historique
pour de bon :

```
gitpoulpi rewrite purge                 # demande : liste ce qui est supprimé et perdu (code de sortie 3)
gitpoulpi rewrite purge --yes           # la fait
```

Elle retire les épingles d'annulation de la réécriture, fait expirer les entrées
de reflog qui citent les anciens commits, lance `git gc --prune=now`, puis vérifie
que chaque ancien commit et chaque ancienne version de fichier a bien disparu. La
confirmation dit d'abord ce que vous perdez : **l'annulation n'est plus
possible** pour la réécriture (et pour toute opération antérieure dont
l'annulation avait aussi besoin des anciens commits) ; elles restent dans la liste
de l'historique, marquées comme purgées. Une fois lancée, une purge ne peut ni
être interrompue ni être annulée. Elle ne touche à aucun distant.

Tout ce qui retient encore l'ancien historique bloque la purge, et rien n'est alors
modifié. La liste nomme chaque élément et ce qu'il faut faire : une branche ou un
tag qui ne faisait pas partie de la réécriture, un stash sur l'ancien historique
(supprimez-le, ou gardez-le et acceptez que l'ancien historique reste), un autre
arbre de travail dont le HEAD s'y trouve, et les branches de suivi distant qui
pointent encore vers l'ancien historique (poussez les branches réécrites, puis
faites un fetch). La liste de vos stashes et les reflogs des autres branches ne
sont jamais touchés.

Un secret qui a été poussé est compromis : **renouvelez-le d'abord**. Après la
purge, la réponse liste ce dont l'hébergeur a encore besoin, distant par distant :
GitHub garde les anciens commits accessibles dans ses vues en cache et les refs
des pull requests (contactez le support GitHub, ou supprimez et recréez le
dépôt), GitLab propose un nettoyage de dépôt qui prend la `commit-map`
(`gitpoulpi rewrite maps --out DIR`) et la maintenance, Azure DevOps et Bitbucket
demandent une requête au support ou un nouveau dépôt, et toute personne qui a un
clone ou un fork a encore l'ancien historique.

### L'écran de purge

Dans l'atelier, le panneau de résultat d'une réécriture a un bouton **Purger
l'ancien historique...** (touche `e`), et la palette a **Réécriture : purger
l'ancien historique...** (elle agit sur la réécriture la plus récente qui n'est
pas encore purgée, même après la fermeture de l'atelier). L'écran liste ce que la
purge supprime et ce que vous perdez (les opérations dont l'annulation disparaît
avec elle ; celles marquées *perd aussi son annulation* ne sont pas des réécritures
mais avaient besoin des anciens commits), puis ce qui retient encore l'ancien
historique, une ligne chacun :

- une branche de suivi distant : **Fetch** les met à jour depuis les distants ;
- un stash : **Supprimer** demande d'abord confirmation, car un stash est du travail
  jamais commité ;
- une branche, un tag, un arbre de travail ou une autre ref : **Copier le nom**,
  puis réglez-le là où il se trouve (réécrivez ou supprimez la branche, extrayez
  une branche réécrite dans l'arbre de travail).

La vérification repasse après chaque correction, et **Revérifier** (`r`) la lance
à la demande. **Purger** (`p`) n'est disponible que lorsqu'il ne reste plus rien.
Il pose une dernière question, numérotée et sans réponse par défaut (`1` purge
pour de bon, `2` laisse tout comme avant), puis s'exécute ; l'écran montre l'étape
en cours et rappelle que rien ne peut l'arrêter. Le résultat dit que l'ancien
historique a disparu de ce clone, ou combien d'anciens objets y sont encore.
En dessous, les étapes à faire sur chaque distant (un lien peut être copié). La
croix, `Esc` et un clic à l'extérieur ferment l'écran, sauf pendant la purge.

Via MCP, l'outil de purge n'existe que si vous réglez `allow_history_purge =
true` dans la table `[mcp]` de `config.toml`.

## La règle Script

*Ajouter une règle > Script* (ou **Réécriture : exécuter un script...** dans la
palette) ajoute une règle de script Lua et ouvre son éditeur par-dessus l'atelier ;
`Enter` sur la ligne *Script* de la règle le rouvre. L'éditeur part d'un petit
modèle.

- **Langage** (une ligne du volet de la règle) : *Lua (intégré)*, *Script shell*,
  *Python* ou *Lua (installé)*. Tout sauf le Lua intégré est un
  [script externe](#scripts-externes-shell-python-lua) : non isolé, et un langage
  désactivé dans les paramètres ou dont l'interpréteur manque est grisé dans la
  liste, avec la raison à côté (le choisir dit quoi faire). Changer le langage
  d'un script que vous n'avez pas encore écrit remplace le modèle.
- **Script** est un éditeur de texte avec numéros de ligne. Ce que vous tapez est
  la règle à l'instant même ; il n'y a rien à confirmer.
- En dessous, la ligne de **vérification** dit `OK : rewrite_commit trouvée`, ou
  liste les problèmes avec leur ligne (`script:3: ...` ; le numéro est rouge dans
  la marge). Elle s'exécute un instant après l'arrêt de la frappe, jamais pendant
  qu'une touche est maintenue.
- **Paramètres** a une ligne par ligne `--! param` de l'en-tête du script, avec son
  libellé à côté du champ : du texte, un nombre, ou une case pour vrai/faux. Une
  valeur effacée revient au défaut de l'en-tête ; un paramètre texte sans défaut
  doit être renseigné avant l'Aperçu.
- **Fichier** (un chemin dans le dépôt, `.gitpoulpi/rewrite/script.lua` pour
  commencer) avec **Charger un fichier...** (remplace le texte) et **Enregistrer
  dans un fichier...**.
- **Tester sur la sélection** exécute le script sur les commits sélectionnés dans
  le Log, ou sur les 20 plus récents si rien n'est sélectionné, et n'écrit rien.
  Le tableau liste chaque commit (identifiant court), chaque champ qu'il
  modifierait sous la forme `ancien -> nouveau`, et les lignes `log(...)` du
  script. **Annuler** cesse d'attendre. Si le script échoue, le commit et la
  raison sont affichés.
- La croix, un clic en dehors de l'éditeur et `Esc` le ferment ; `Tab` et
  `Shift+Tab` passent entre l'éditeur, les paramètres, le fichier, les boutons et
  les résultats ; `F5` lance l'essai.

L'aperçu refuse un script qui ne compile pas : le volet Aperçu dit *un script a
échoué* avec la ligne (ou, pour un échec sur un commit, son identifiant court et
la raison). Dans une version sans scripts, ce type reste dans le menu Ajouter,
grisé.

## Écrire un script avec l'IA

Quand l'IA est activée ([Assistance IA](ai.md)), l'éditeur de script a un bouton
`✦ Écrire avec l'IA...`, et la palette a `✦ Réécriture : écrire un script avec
l'IA...`, qui ouvre l'atelier sur une nouvelle règle Script avec la boîte de
dialogue ouverte. Rien n'est envoyé avant que vous appuyiez sur **Envoyer** : la
boîte affiche, pendant que vous tapez, le texte exact qui partira.

- **Que doit faire le script ?** Dites-le avec vos mots : *décaler au lendemain
  matin chaque commit fait la nuit*, *remplacer mon ancienne adresse par la
  nouvelle, seulement avant 2024*.
- **Envoyer un échantillon de N commits** (désactivé) : cochée, l'auteur, la date
  et le sujet des commits que « Tester sur la sélection » utiliserait (votre
  sélection dans le Log, sinon les 20 plus récents) sont ajoutés pour que le
  modèle voie à quoi ressemblent vos données. Jamais le contenu des fichiers ni
  le corps des commits. Case décochée, aucune donnée de commit ne quitte la
  machine.
- **Ce qui est envoyé** liste les consignes (le contrat fixe du script, puis vos
  instructions IA) et votre demande, ainsi que le profil qui les reçoit. `Tab`
  y accède, les flèches le font défiler.
- **Envoyer** (`Tab` jusqu'au bouton, puis `Enter`). Pendant que le modèle
  travaille, la boîte attend, et `Esc` annule la requête. En cas d'échec, la boîte
  reste ouverte avec la raison pour que vous puissiez réessayer.

La réponse est vérifiée par le même contrôle du démon qu'un script tapé à la main ;
une erreur de compilation est renvoyée une fois au modèle. Le script **remplace
alors le texte de l'éditeur** (la boîte le dit quand l'éditeur contient autre
chose que le modèle de départ) et le titre de l'éditeur affiche *✦ écrit par l'IA :
relisez-le avant l'aperçu* jusqu'à ce que vous modifiiez le texte. Il n'est jamais
testé, prévisualisé ni appliqué à votre place : relisez-le, appuyez sur **Tester
sur la sélection**, puis lancez l'aperçu comme pour tout script. Le script
s'exécute dans le même bac à sable que celui que vous auriez écrit (ni fichiers,
ni horloge, ni réseau).

En ligne de commande : `gitpoulpi ai rewrite-script "<consigne>" [--sample N]`
affiche le script et le résultat de `rewrite script check` sur lui, en JSON, et
n'exécute rien (code de sortie 2 s'il ne compile toujours pas). Le profil *Script
de réécriture d'historique* de Paramètres > IA est utilisé.

## Créer des règles avec l'IA

Quand l'IA est activée ([Assistance IA](ai.md)), l'atelier a un bouton `✦ Créer des règles avec
l'IA...` à côté de Enregistrer et Charger, la touche `Alt+I` (*Créer des règles de réécriture avec
l'IA* dans Paramètres > Clavier, contexte *Rewrite*), et la palette a `✦ Réécriture : créer des
règles avec l'IA...`. Avec l'IA désactivée, ou dans un build sans IA, le bouton et la ligne de la
palette n'existent pas. Vous décrivez avec vos mots comment l'historique doit être réécrit
(*remplacer mon ancien e-mail a@b.org par c@d.org*, *retirer secrets.txt de tout l'historique*,
*décaler mes dates de deux heures*) et le modèle propose les règles et les réglages, qui
**remplissent l'atelier**. Rien n'est envoyé avant d'appuyer sur **Envoyer** : la boîte de dialogue
montre, pendant que vous tapez, le texte exact qui partira.

- **Comment l'historique doit-il être réécrit ?** Un champ multiligne : `Enter` ajoute une ligne,
  `Ctrl+Enter` ou `Ctrl+S` envoie, `Esc` et la croix annulent.
- **Envoyer le périmètre, les noms des branches et des tags et les auteurs de l'historique**
  (désactivé) : cochée, la case ajoute le périmètre de la barre de périmètre, les noms des branches
  et des tags et les auteurs distincts (nom et e-mail) des derniers commits affichés par le Log (20
  au plus), pour que le modèle puisse nommer votre e-mail exactement (en ligne de commande,
  `ai rewrite-rules --context` lit plutôt les 200 commits les plus récents). Jamais le contenu des
  fichiers ni les messages. Case décochée, seuls votre description et les instructions fixes
  quittent la machine.
- **Remplacer les règles actuelles** (seulement si l'atelier contient déjà des règles ;
  désactivé) : par défaut, les nouvelles règles sont ajoutées après les vôtres ; cochée, elles les
  remplacent (le périmètre et les réglages que la réponse indique sont appliqués dans les deux
  cas).
- **Ce qui est envoyé** liste les instructions (le vocabulaire des règles, puis vos instructions
  IA), votre demande et le profil qui les reçoit. `Tab` y va, les flèches font défiler.

Le modèle peut employer tous les types de règle de cette page sauf **Script** (du code : *Écrire un
script avec l'IA* montre le script pour relecture) et la forme « valeurs secrètes » de *Rechercher
et remplacer dans les fichiers* (les valeurs secrètes ne sont jamais confiées à un modèle :
utilisez l'onglet Secrets). Il peut régler le périmètre sur la branche courante, toutes les
branches et tags locaux, ou des branches et tags nommés qui existent, ainsi que les options
*Tags*, *committer*, *commits vides* et *ids abrégés dans les messages*. Ce que l'atelier ne sait
pas exprimer (une limite de date comme *depuis mars*, un intervalle de commits) n'est pas deviné :
le modèle le dit dans une remarque. Le sort des signatures n'est jamais décidé par le modèle : la
question vous est posée à l'application.

**L'IA ne fait que préparer des règles et des réglages.** Elle ne prévisualise, n'applique et
n'exécute jamais rien : la réponse est copiée dans l'atelier comme une recette que vous auriez
chargée, la prévisualisation affichée devient périmée, et Prévisualiser et Appliquer (avec leurs
questions et confirmations) restent les vôtres. Chaque règle est analysée et vérifiée seule par la
vérification qu'emploient l'éditeur et le moteur ; une règle qui échoue est écartée. Quand quelque
chose a été écarté, ou que le modèle a fait une remarque, une liste l'affiche une fois les règles
en place (*Écartées (les règles valides ont été gardées)*, chacune avec sa position, son nom et la
raison) et `Enter` ou `Esc` la ferme. Relisez les règles, modifiez-les, puis prévisualisez.

En ligne de commande : `gitpoulpi ai rewrite-rules "<description>" [--context]` affiche la
proposition en JSON (`recipe`, que lit `rewrite preview --recipe-json`, `problems`, `notes` et
`applied: false`) et n'exécute rien (code de sortie 2 quand quelque chose a été écarté). L'outil
MCP est `ai_rewrite_rules`. Le profil *Règles de réécriture d'historique* de Paramètres > IA est
utilisé.

## Scripts Lua

Une règle de script exécute votre fonction sur chaque commit. C'est du Lua 5.4
standard dans un bac à sable : ni fichiers, ni horloge, ni processus, ni
modules, ni `pcall`. Le même script sur le même historique donne toujours le même
résultat, et un appel qui dépasse 10 millions d'instructions, 64 MiB ou 5
secondes s'arrête avec une erreur qui nomme le commit.

```lua
--! name: Replace an email
--! param old_email: text "Email to replace"
--! param new_email: text "New email"
function rewrite_commit(c, ctx)
  if c.author.email == ctx.params.old_email then
    return { author = { email = ctx.params.new_email } }
  end
end
```

`c` contient `id`, `parents`, `tree`, `author` et `committer` (`name`, `email`,
`time`, `offset_minutes`), `message` et `signed`. `ctx` contient `index`,
`total`, `params`, `state` (renvoyez `state = {...}` pour transmettre des données
au commit suivant) et `window.before` / `window.after` (définissez
`--! window: N`). Ne renvoyez que ce qui change : `author`, `committer`,
`message`. Fonctions utilitaires : `regex_replace`, `regex_match`, `stable_hash`,
`rand`, `weekday`, `hour`, `log`, et pour les messages `trailers(message)` (une liste
de `{key, value}`), `set_trailer(message, key, value)` (remplace les trailers de
cette clé ; un quatrième argument `"add"` ou `"skip"` change cela),
`remove_trailer(message, key)`, et `at("2024-03-05 18:30", offset_minutes)` (une
date dans un décalage, en temps Unix).

En ligne de commande : `gitpoulpi rewrite script check my.lua` le compile,
`gitpoulpi rewrite script test my.lua --commit HEAD --param old_email=a@b`
montre ce qu'il modifierait sans rien écrire, et
`gitpoulpi rewrite preview --script my.lua --script-param old_email=a@b` établit
le plan de toute la réécriture.

## Scripts externes (shell, Python, Lua)

Le même contrat qu'un script Lua, exécuté par un programme de votre machine :
*Script shell* (bash ou sh ; PowerShell sous Windows, ou tout fichier `.ps1`),
*Python* et *Lua (installé)* (un `lua` à vous, pas celui qui est intégré ; il n'a
pas de bibliothèque JSON, le modèle lit donc la ligne avec un motif). **Ils ne sont
pas isolés : un script externe s'exécute avec vos droits** et peut lire ou
modifier tout ce que vous pouvez. Le volet de la règle et le titre de l'éditeur le
disent.

Comment il communique : GitPoulpi lance le script une fois par exécution et lui
envoie une requête JSON par ligne sur son entrée standard (les champs d'un commit,
`params`, le `state` renvoyé par la dernière réponse) ; il répond par une ligne
JSON sur la sortie standard, avec les mêmes champs que renvoie un script Lua
(`author`, `committer`, `message`, `state`, `log`), ou `{"error": "texte"}` pour
abandonner. Écrivez les diagnostics sur la sortie d'erreur : ils apparaissent
comme les lignes `log` du commit. Les lignes d'en-tête commencent par `#!` pour
shell et Python (`#! name: ...`, `#! param old: text "Old"` ; une ligne shebang
reste un shebang) et par `--!` pour Lua.

```sh
#!/bin/sh
#! name: Uppercase author names
while IFS= read -r line; do
  name=$(printf '%s' "$line" | sed -n 's/.*"author":{[^}]*"name":"\([^"]*\)".*/\1/p' | tr 'a-z' 'A-Z')
  printf '{"author":{"name":"%s"}}\n' "$name"
done
```

**Ce qu'il reçoit.** Un environnement vide, sauf `PATH`, `HOME` / `USERPROFILE`,
`SYSTEMROOT`, `TEMP` / `TMP` et `LANG` ; un dossier temporaire neuf comme dossier de
travail (qui contient sa propre copie du script) ; pas le chemin du dépôt.
**Limites de temps :** `rewrite.script_timeout_ms` (5000) pour chaque réponse et
`rewrite.script_total_timeout_s` (3600) pour toute la passe. En cas de
dépassement, de plantage, de sortie qui n'est pas du JSON ou d'annulation de la
tâche, GitPoulpi tue le script et tout ce qu'il a lancé, et la réécriture
s'arrête avec le commit et la raison ; rien n'a bougé.

**Quels langages existent.** `rewrite.external_scripts` dans `config.toml` les
liste : `["shell"]` par défaut ; ajoutez vous-même `"python"` et `"lua"` dans
`config.toml` (`[rewrite]`, `external_scripts = ["shell", "python"]` ; une liste
ne se règle pas avec `gitpoulpi config set`). Un langage absent de la liste est
grisé dans l'atelier et refusé par la CLI et MCP (`scripts_disabled`) ; un langage
listé dont l'interpréteur est introuvable est grisé avec le conseil d'installation
(`tool_missing`).

**Confiance.** Même quand un langage est activé, un script ne s'exécute qu'après
que vous avez approuvé son texte exact. Un script que vous tapez dans l'atelier est
le vôtre et s'exécute tout de suite ; celui qui vient d'une recette partagée ou
d'un fichier du dépôt ne l'est pas : l'Aperçu et *Tester sur la sélection*
s'arrêtent et demandent *Faire confiance à ce script ?* avec deux réponses
numérotées dont aucune n'est présélectionnée (`1` lui faire confiance et
continuer, `2` ne pas lui faire confiance, `Esc` le laisse de côté). Approuver
mémorise le SHA-256 du texte pour ce dépôt, si bien qu'un script modifié redemande
et qu'un dépôt cloné ne peut pas exécuter de code chez vous tout seul. En ligne de
commande, lisez le fichier, puis `gitpoulpi rewrite script trust FICHIER` (ou
`--trust` sur `preview`, `run`, `script check` et `script test`, qui approuve et
continue). Un agent via MCP ne peut rien approuver : il ne peut exécuter qu'un
script que vous avez approuvé avant.

**Vérification.** La ligne de vérification sous l'éditeur dit *Pas encore
approuvé* pour un script non approuvé (rien n'a été lancé) ; pour un script
approuvé, GitPoulpi le lance une fois avec un commit fictif (`"dry_run": true`) et
vérifie que la réponse respecte le contrat. `gitpoulpi rewrite script check FICHIER`
fait de même et indique `trust`, `hash` et `interpreter`.

## En ligne de commande

`gitpoulpi rewrite` (et les outils MCP du même nom) exécute le même moteur, pour
les scripts et les agents. Une **recette** est un fichier TOML ou JSON, celui
qu'enregistre l'atelier (`--recipe FICHIER`, ou `-` pour l'entrée standard), et
les options ajoutent des règles après celles de la recette.

```
gitpoulpi rewrite preview --recipe fix.toml --all     # plan, ne change rien
gitpoulpi rewrite apply --run ID --digest D           # exécute le plan relu
gitpoulpi rewrite run --fix-message-encoding --signatures strip --yes
gitpoulpi undo                                        # remet tout en place
```

- Les règles sous forme d'options : `--set-name` / `--set-email` avec
  `--match-email` (identité), `--mailmap`, `--shift-dates`,
  `--replace-message FIND=WITH`, `--trailer-add`, `--trailer-remove`,
  `--trailer-map`, `--fix-message-encoding` (`--fix-name-encoding`),
  `--remove-path`, `--remove-blobs-over`, `--replace-content`, `--eol`
  (`--eol-paths`), `--strip-exec`, `--reencode` (`--encoding-paths`),
  `--move-path`, `--into-subdir`, `--extract`, `--lfs-import`, `--lfs-export`,
  `--script`, `--remove-signatures`, `--remove-finding`. Le périmètre est la
  branche courante, `--all-local`, ou `--ref R` (répétable), à partir de
  `--since REV`.
- **Les questions sans réponse par défaut** sont des options : `--signatures
  strip|resign-signed|resign-all` quand des commits signés changent, et
  `--extract-mode` pour une extraction. Sans elles, la réponse est un refus ou un
  `needs_choice` (code de sortie 3), jamais une supposition.
- **Codes de sortie :** 0 terminé ; 1 une erreur (recette illisible, mauvaises
  options) ; 2 refus, avec la raison (`stale`, `signature_choice_needed`,
  `dirty_paths_touched`, `empty_scope`, ...) ; 3 une question à trancher, ou des
  refs poussées qui exigent `--yes`.
- `preview` garde le plan dans le dépôt : `apply` fonctionne donc depuis une
  commande ultérieure, après un redémarrage du démon, ou depuis une autre
  interface ; un plan qui ne correspond plus à son empreinte est `stale`.

La [référence CLI](cli-reference.md) donne toutes les options, et
[MCP et agents](mcp-and-agents.md) dit quels outils reçoit un agent (l'outil de
purge n'existe que si vous l'activez, et un agent ne peut pas approuver un script).

## Propositions des agents IA

Un agent peut **suggérer** une réécriture, jamais en exécuter une.
`gitpoulpi ui propose rewrite --recipe FICHIER|-` (l'outil MCP
`ui_propose_rewrite`, désactivé tant que `[mcp] allow_ui_propose = true` n'est pas
réglé) ouvre l'atelier dans votre fenêtre en cours avec les règles de la recette
déjà dans la liste et un bandeau violet, *✦ Proposé par <agent> : une réécriture
d'historique (3 règles) · Ctrl+Enter accepte · Esc rejette*.

- **Rien ne s'exécute.** Ni aperçu, ni application, ni push, ni purge : la
  proposition ne fait que remplir les règles. Vous les lisez, les modifiez si vous
  voulez, lancez l'**Aperçu**, puis appliquez avec votre propre touche. Fermer
  l'atelier rejette la proposition.
- **Ce que l'agent ne peut pas y mettre :** un script (il s'exécuterait à l'aperçu),
  plus de 50 règles, des valeurs secrètes (elles sont retirées de la recette avant
  l'envoi). Une telle proposition est refusée avec la raison.
- L'agent peut lire l'issue dans `ui state` (`accepted` quand vous avez appliqué
  les règles telles que proposées, `accepted_edited` quand vous les avez d'abord
  modifiées, `rejected` quand vous avez fermé). Une seule proposition à la fois ;
  elle est refusée tant qu'une autre de vos boîtes de dialogue est ouverte.

## Outils

La plus grande partie d'une réécriture d'historique n'a besoin de rien d'autre que
Git : changer des auteurs et des committers, appliquer un `.mailmap`, anonymiser,
décaler ou borner des dates, réécrire des messages et des trailers, remplacer du
contenu, retirer des chemins ou de gros blobs et déplacer des chemins se font tous
dans GitPoulpi.

**Les scripts Lua sont intégrés.** L'interpréteur est inclus dans GitPoulpi : un
script Lua n'a besoin d'aucun outil ni d'aucune installation. Les scripts shell,
Python et Lua installé ont besoin de leur interpréteur (voir le tableau
ci-dessous) et sont décrits dans [Scripts externes](#scripts-externes-shell-python-lua).

Quelques fonctions font appel à un programme externe. GitPoulpi le cherche (le
programme est trouvé dans votre `PATH`, sous Windows aussi via `PATHEXT` et le
dossier `usr/bin` propre à Git pour Windows, et doit répondre à `--version` en
moins de 2 secondes) ; le résultat est gardé une minute en cache et
« Revérifier » relance la recherche. Une règle qui nécessite un programme absent
reste dans la liste, grisée, avec la raison et la façon de l'installer, et
l'aperçu refuse de se lancer avant que quoi que ce soit ne s'exécute.

| Fonction | Outil | Remarques |
|---|---|---|
| Re-signer les commits | `gpg` (OpenPGP), `ssh-keygen` (`gpg.format = ssh`) ou `gpgsm` (`gpg.format = x509`) | La signature SSH ou X.509 exige aussi `user.signingkey`. `gpg.program`, `gpg.ssh.program` et `gpg.x509.program` sont respectés. |
| Migration vers LFS (les deux règles LFS) | `git-lfs` 3.0 ou plus récent | Même vérification que dans le volet LFS. |
| Scripts Python | `python` 3.8 ou plus récent (`py -3` sous Windows) | Désactivés par défaut : ajoutez `"python"` à `rewrite.external_scripts`. |
| Scripts Lua (installé) | `lua5.4`, `lua54`, `lua` ou `luajit` | Désactivés par défaut : ajoutez `"lua"` à `rewrite.external_scripts`. |
| Scripts shell | `bash`, `pwsh` (PowerShell) ou `sh` | Activés par défaut (`"shell"`). Un script PowerShell (un fichier `.ps1`, un shebang `pwsh`, ou tout script shell sous Windows) a besoin de `pwsh` ou `powershell` ; un script POSIX a besoin de `bash` ou `sh` (sous Windows, Git pour Windows les fournit). |

Conseils d'installation, Windows d'abord :

- Windows : Git pour Windows fournit `gpg`, `ssh-keygen`, `bash`, `sh` et Git LFS.
  Sinon : `winget install GnuPG.GnuPG`, Paramètres > Fonctionnalités
  facultatives > Client OpenSSH, `winget install GitHub.GitLFS`,
  `winget install Python.Python.3.12`, `winget install DEVCOM.Lua`,
  `winget install Microsoft.PowerShell`.
- Linux : les paquets de la distribution (`gnupg`, `openssh`, `git-lfs`,
  `python3`, `lua5.4`, `bash`, `powershell`).
- macOS : `brew install gnupg git-lfs python lua powershell` ; `ssh-keygen` et `sh`
  sont intégrés.
