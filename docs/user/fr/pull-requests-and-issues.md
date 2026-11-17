[English](../pull-requests-and-issues.md) · [Čeština](../cs/pull-requests-and-issues.md) · [Deutsch](../de/pull-requests-and-issues.md) · [Español](../es/pull-requests-and-issues.md) · **Français** · [Italiano](../it/pull-requests-and-issues.md) · [日本語](../ja/pull-requests-and-issues.md) · [한국어](../ko/pull-requests-and-issues.md) · [Nederlands](../nl/pull-requests-and-issues.md) · [Polski](../pl/pull-requests-and-issues.md) · [Português (Brasil)](../pt-BR/pull-requests-and-issues.md) · [Português (Portugal)](../pt-PT/pull-requests-and-issues.md) · [Русский](../ru/pull-requests-and-issues.md) · [Svenska](../sv/pull-requests-and-issues.md) · [Türkçe](../tr/pull-requests-and-issues.md) · [Українська](../uk/pull-requests-and-issues.md) · [简体中文](../zh-CN/pull-requests-and-issues.md)

# Pull requests et tickets

Lire, créer et gérer des pull / merge requests et des tickets depuis le panneau latéral, le tableau et la barre d'outils. Connectez d'abord un compte et liez un dépôt : voir [Intégrations d'hébergeurs et de suivi de tickets](hosting-integration.md). Tout ce qui se dit sur un ticket ou une demande (commentaires, revues, fils) est dans [Conversations](conversations.md) ; lire et relire une demande est dans [Relire une pull ou merge request](review.md).

## Pull requests et tickets dans le panneau latéral

### PULL REQUESTS

La section a un en-tête avec le compteur et un `+` (créer une pull request), les
listes du fournisseur et du projet, une zone de recherche et une liste d'état
(*Ouvertes*, *Mergées / fermées*, *Toutes*). Viennent ensuite quatre groupes, chacun
avec son compteur :

- **Mes pull requests** : vous en êtes l'auteur.
- **Assignées à moi**.
- **En attente de ma relecture** : vous êtes relecteur demandé.
- **Toutes les pull requests** : replié au départ.

La zone de recherche filtre chaque groupe (titre, numéro, auteur, branche). Chaque
changement de la liste d'état interroge à nouveau l'hébergeur.

Une ligne commence par un symbole d'état : `●` ouverte (vert), `○` brouillon, `◆`
mergée (violet), `✕` fermée (rouge). La demande de la branche extraite est en gras.
Les lignes se lisent `#12  titre` (GitHub, Bitbucket) ou `!53  titre` (GitLab,
Azure DevOps).

### TICKETS

Le titre de la section nomme le fournisseur : `GITHUB ISSUES`, `JIRA ISSUES`, et
ainsi de suite. Il se lit `... WORK ITEMS` pour Azure DevOps et `... CARDS` pour
Trello. Sous les listes déroulantes : une zone de recherche, la liste d'état
(*Ouverts*, *Qui me sont assignés*, *Fermés*, *Tous*) et les tickets. Une ligne Jira se
lit `ABC-12  titre`, avec le type et le statut estompés quand le panneau latéral est
assez large.

![La section ISSUES du panneau latéral avec le menu clic droit du ticket #7 ouvert, Ouvrir la conversation en premier](../images/issues-sidebar.png)

### Actions sur une ligne

- `Enter` ou un double-clic ouvre la ligne : une pull request dans la [revue](review.md) (GitHub, GitLab, Gitea, Forgejo, Gerrit), un ticket, et une demande sur Azure DevOps ou Bitbucket, comme [conversation](conversations.md) dans la fenêtre, et tout le reste dans votre navigateur. **Ouvrir dans le navigateur** du menu de la ligne ouvre toujours le navigateur.
- Clic droit, ou `.`, ouvre le menu de la ligne :
  - **Ouvrir la conversation** (tickets, et demandes sans vue de revue), **Ouvrir dans le navigateur**, **Copier l'URL**, **Copier le numéro**.
  - Pour une pull request : **Modifier sur <fournisseur>**, **Basculer sur la branche
    source**, **Voir les commits de la branche source**. Pour une demande venant d'un
    fork, le checkout se lit **Basculer sur la pull request (fork)**, et **Voir** est
    grisé.
  - Pour un ticket : **Créer une branche depuis le ticket**. Cela ouvre la saisie de
    nouvelle branche avec un nom comme `12-fix-the-thing` (ou
    `ABC-123-fix-the-thing` pour Jira).
  - Sur GitHub, GitLab, Gitea et Forgejo, une ligne de pull request a aussi **Relire...**
    (aussi **Enter** sur la ligne) : la revue intégrée, voir
    [Relire une pull ou merge request](review.md).
  - Sur GitHub, GitLab, Gitea et Forgejo, les deux types de ligne ont aussi **Modifier les assignés...**,
    **Modifier les étiquettes...** et **Définir le jalon...** (et **Modifier les relecteurs...** sur
    une pull request) : voir
    [Assignés, relecteurs, étiquettes et jalon](#assignés-relecteurs-étiquettes-et-jalon).
- **Modifier sur <fournisseur>** a trois autres accès : le crayon `✎` à l'extrémité
  droite d'une ligne de pull request survolée ou sélectionnée, `e` sur la ligne
  sélectionnée, et la commande de palette *Intégrations : Modifier la pull request liée sur l'hébergeur*
  (pour la demande de la branche extraite).
- Si la branche source n'est pas encore dans le dépôt, GitPoulpi le dit. Faites
  d'abord un fetch (`Ctrl+G f`).

**Checkout d'une pull request.** GitPoulpi récupère la demande dans une branche
locale `pr/<n>` (`mr/<n>` sur GitLab) qui la suit, et bascule dessus. Sur GitHub, GitLab,
Gitea et Forgejo cela fonctionne aussi pour les forks (GitPoulpi récupère la ref head spéciale
de l'hébergeur). Azure DevOps et Bitbucket récupèrent la branche source de la
demande. Une demande venant d'un fork y est refusée : ajoutez le fork comme distant
et extrayez sa branche vous-même. Annuler revient à la branche précédente. CLI :
`gitpoulpi hosting checkout-pr <n>`.

**Relire une pull request en ligne de commande (et pour un agent).** Sur GitHub
(Enterprise compris), GitLab (auto-hébergé compris), Gitea et Forgejo, une demande peut être
lue et relue sans quitter GitPoulpi : `hosting review show <n>` (fichiers,
état du merge, vérifications), `hosting review threads <n>` (fils de discussion de la revue et
discussion), `hosting review comment`, `hosting review reply`, `hosting review resolve`
et `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. Le diff vient de votre dépôt, pas de
l'hébergeur : `hosting review fetch <n>` récupère la tête de la demande dans
`refs/gitpoulpi/pr/<n>` et sa branche cible dans `refs/gitpoulpi/pr/<n>-base`
(aucune des deux n'est une branche, rien n'est extrait). Les numéros de ligne sont ceux de ce
diff : `new` est la ligne en tête, `old` la ligne à la base du merge. GitLab
n'a pas d'état de revue « demander des modifications » : il est refusé, sans rien envoyer.
Gitea et Forgejo acceptent un commentaire sur une seule ligne (pas de plages) ; un fil est les
commentaires d'une ligne d'un fichier. Gitea peut résoudre un fil, Forgejo 11 non
(l'hébergeur n'a pas de point d'accès pour cela : l'appel le dit). Une réponse sur un serveur
sans point d'accès de réponse est publiée comme une revue d'un commentaire sur la même ligne.
Gitea refuse d'approuver ou de rejeter votre propre demande.
Sans clone : ajoutez `--project owner/name` (GitLab : `group/project`) à `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` et `suggest` pour relire un projet qui n'est pas le dépôt courant,
`hosting review diff <n> --project P` lit le diff (les hunks de chaque fichier) depuis l'API de l'hébergeur, et
`hosting file <path> --project P [--ref R]` lit un fichier à une révision (GitHub, GitLab, Gitea, Forgejo et Gerrit ; sur Gerrit le diff d'un
patch set est lu fichier par fichier et le fichier vient de la branche ou, pour un identifiant de commit, du commit). Un fichier que l'hébergeur
ne montre pas dans le diff (trop gros) est marqué `omitted`.
Les outils MCP sont les mêmes commandes (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Assignés, relecteurs, étiquettes et jalon

Sur GitHub (Enterprise compris), GitLab (auto-hébergé compris), Gitea et
Forgejo, vous pouvez changer à qui une pull request ou un ticket est assigné, qui la relit, ses étiquettes
et son jalon sans quitter GitPoulpi. Les autres fournisseurs n'en montrent rien.

- **Menu de la ligne** : **Modifier les assignés...**, **Modifier les relecteurs...** (pull requests
  seulement), **Modifier les étiquettes...**, **Définir le jalon...**.
- **Palette** : *Hébergeur : Modifier les assignés...*, *Hébergeur : Modifier les relecteurs...*,
  *Hébergeur : Modifier les étiquettes...*, *Hébergeur : Définir le jalon...* listent chacune d'abord les
  demandes et tickets de la barre latérale à choisir. Dans le
  [Radar](radar.md), les quatre agissent sur la ligne sélectionnée.
- Le sélecteur s'ouvre aussitôt et charge depuis l'hébergeur les personnes ou étiquettes du projet (et ce
  que l'élément a maintenant). Une ligne d'attente s'affiche après 150 ms environ.
  Tapez pour filtrer (correspondance approximative), `Up` / `Down` déplacent, `Space` coche ou
  décoche, `Enter` applique, `Esc` annule, `Tab` atteint les boutons. Un clic
  sur une ligne la coche, et la molette fait défiler. Les personnes montrent un badge d'initiales,
  les étiquettes leur couleur.
- Le jalon est une liste déroulante (avec **Aucun jalon** pour l'effacer).
- Après **Appliquer**, l'hébergeur est mis à jour, et les listes et le Radar montrent le
  changement après leur prochaine actualisation, qui démarre tout de suite.

Les formulaires de création (une pull request, un ticket) ont les mêmes choix sous forme de rangée de
boutons : `Assignés : aucun`, `Relecteurs : aucun`, `Étiquettes : aucun`,
`Jalon : aucun`. Sur GitHub, GitLab, Gitea et Forgejo, le bouton Étiquettes remplace le champ d'étiquettes
saisies du formulaire de ticket. Si un hébergeur accepte la nouvelle pull request mais
refuse certaines de ses étiquettes ou personnes, la ligne d'état le dit.

CLI : `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>`, et `--assignee`, `--reviewer`, `--label`,
`--milestone` sur `hosting pr` et `hosting issue create` ; voir la
[référence CLI](cli-reference.md).

### Touches dans les sections

| Touche | Action |
|---|---|
| `Enter` / double-clic | ouvrir la conversation dans GitPoulpi là où l'hébergeur en a une (une pull request sur GitHub, GitLab, Gitea ou Gerrit ouvre la revue), sinon le navigateur |
| `o` | ouvrir la ligne dans le navigateur |
| `.` / clic droit | menu de la ligne |
| `e` | modifier la pull request sélectionnée sur l'hébergeur |
| `n` | nouvelle pull request (PULL REQUESTS) ou nouveau ticket (TICKETS) |
| `v` | ouvrir la liste d'état |
| `r` | actualiser |
| `Enter` / `Space` sur un en-tête | replier ou déplier la section |

### Reconnexion et hors ligne

Quand le jeton est absent, rejeté ou n'a pas une portée, la première ligne de la
section se lit `Reconnect (token rejected)`. Cliquez dessus, ou appuyez sur
`Enter`, pour ouvrir le profil dans Paramètres > Intégrations.

Quand l'hébergeur est injoignable, les dernières listes restent à l'écran et
l'en-tête montre leur âge (`· 5m old`). La dernière réponse est aussi mise en cache
sur disque (`.gitpoulpi/hosting-cache.json`), de sorte que les listes s'affichent
avant que le réseau ne réponde.

## Le tableau des tickets

Le bouton **Tableau** de l'en-tête TICKETS (palette : **Tickets : Ouvrir le tableau**) montre les tickets de la source liée comme un tableau
Kanban par-dessus la fenêtre. Les colonnes dépendent de la source :

| Source | Colonnes | Déplacer une carte |
|---|---|---|
| Notes Markdown locales | les statuts (`statuses` du profil, puis tout autre en usage) | réécrit le `status` de la note |
| GitHub, GitLab, Gitea, Forgejo | **Ouverts**, puis les étiquettes que vous avez choisies, puis **Fermés** | échange l'étiquette de statut ; vers **Fermés**, ferme le ticket, vers une colonne de nouveau, le rouvre |
| Trello | les listes ouvertes du tableau, de gauche à droite | déplace la carte vers la liste |
| Redmine | les statuts des tickets | change le statut du ticket (les règles de workflow de Redmine s'appliquent) |
| Jira (Cloud et Data Center) | les statuts du projet, à faire en premier, terminé en dernier | exécute la transition de workflow qui mène à la colonne (`GET` puis `POST .../transitions`) ; sans telle transition la carte revient et la ligne d'état dit `no transition from X to Y` |
| Azure DevOps (Services et Server) | les états des types d'éléments de travail du projet, par catégorie d'état | définit `System.State` ; un élément de travail dont le type n'a pas l'état de la colonne revient, avec la raison (`a Bug has no state Done`) |

Sur un projet GitHub ou GitLab, **Colonnes** (ou `c`, ou **Tableau : Définir les étiquettes des colonnes...**) demande les étiquettes qui forment les
colonnes, dans l'ordre, par exemple `status::todo, status::doing, status::review`. Il propose les étiquettes du projet qui
ressemblent à des statuts. Le choix est enregistré pour le dépôt. Sans étiquettes, le tableau est Ouverts / Fermés. Déplacer une carte
ne touche qu'aux étiquettes de statut que vous avez listées : ses autres étiquettes restent. Un tableau d'hébergeur liste les tickets ouverts et les 60
tickets fermés le plus récemment.

Une transition Jira qui demande des champs obligatoires (une résolution, par exemple) est refusée par Jira et la carte
revient avec le message de Jira : faites celle-là sur le site.

Le même tableau est disponible pour les scripts et les agents : `gitpoulpi hosting board` affiche les colonnes avec leurs cartes
et `gitpoulpi hosting board move <carte> <colonne>` en déplace une (voir [la référence CLI](cli-reference.md)).

Une carte montre le numéro, le titre, les assignés en initiales, les étiquettes et le jalon. Glissez-la sur une autre colonne,
ou `Shift+Left` / `Shift+Right`, ou **Tableau : Déplacer la carte vers...** (`m`). Elle bouge aussitôt ; après 150 ms environ, un indicateur d'attente
dit que l'hébergeur travaille encore, et si l'hébergeur refuse, la carte revient et la ligne d'état dit pourquoi. `Enter` ouvre
la [conversation](conversations.md) du ticket dans GitPoulpi, et le tableau revient quand vous la fermez ; `e` modifie assignés, étiquettes et jalon ; `n` commence un nouveau ticket dans
la colonne sélectionnée ; `a`, `l`, `M` et `/` filtrent par assigné, étiquette, jalon et texte, `x` efface les filtres ; `r`
relit l'hébergeur. Le tableau se relit quand les listes de la barre latérale s'actualisent. Le Radar n'a pas de tableau.

## Créer une pull request

Ouvrez le formulaire avec le `+` de l'en-tête PULL REQUESTS, `n` dans la section, ou
la commande de palette *Pull requests : Créer une pull request...*. Un menu de
branche a aussi **Démarrer une pull request depuis <branche>...**.

Le formulaire a :

- Un onglet par profil connecté qui a des pull requests (`Left` / `Right` sur la
  bande d'onglets).
- Des listes **Dépôt source / Branche** et **Dépôt cible / Branche**. Les longues
  listes sont filtrables. Vous pouvez saisir une branche ou un dépôt que les listes
  n'affichent pas.
- **Titre**, rempli avec le sujet du commit au sommet.
- **Description**, sur plusieurs lignes (`Enter` ajoute une ligne, `Tab` quitte le
  champ).
- Des boutons **Assignés**, **Relecteurs**, **Étiquettes** et **Jalon** (GitHub,
  GitLab, Gitea et Forgejo), qui ouvrent le sélecteur.
- **Créer en brouillon**.
- Avec l'[IA](ai.md) activée : **Générer titre et description**.

Le formulaire s'ouvre sur le profil, le dépôt et les branches de la branche extraite
(`feature -> main`). Un dépôt source différent crée une demande depuis un fork.

**Créer une pull request** (ou `Ctrl+S`) crée la demande chez l'hébergeur. Le
résultat s'ouvre dans votre navigateur et la ligne d'état la nomme.

## Créer un ticket

Quand la section TICKETS a un fournisseur **et** un projet, son en-tête a un `+`
(*+ Nouveau ticket*). Aussi : `n` dans la section, ou la commande de palette
*Hébergeur : Nouveau ticket...*.

Le formulaire a **Titre** (obligatoire), **Description** (plusieurs lignes) et
**Étiquettes** (séparées par des virgules, par exemple `bug, ui` ; sur GitHub, GitLab, Gitea et Forgejo, les
boutons Étiquettes, Assignés et Jalon ouvrent le sélecteur à la place). Jira et Azure
DevOps ajoutent une liste **Type**. Trello ajoute une liste **Liste**. Linear et YouTrack
ajoutent une liste **Statut** (l'état du workflow, ou la valeur `State`, dans lequel le ticket
démarre). Redmine ajoute une liste **Type** avec les trackers du projet (Bug, Feature,
Support...) ; il n'a pas d'étiquettes, donc il n'y a pas de bouton Étiquettes pour lui et une étiquette
donnée en ligne de commande est refusée. Le formulaire remplit ces listes depuis l'hébergeur à son ouverture.

**Créer le ticket** (ou `Ctrl+S`) le crée chez l'hébergeur. `Esc` ou **Annuler**
ferme le formulaire. Si l'hébergeur refuse (jeton rejeté, portée manquante, projet
inconnu, étiquette inconnue), le formulaire montre la raison et garde ce que vous
avez saisi. En cas de succès, la ligne d'état nomme le nouveau ticket et la section
TICKETS est actualisée.

CLI : `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## Créer sur la page web de l'hébergeur

Le formulaire Nouveau ticket et le formulaire Créer une pull request ont un bouton
**Créer sur <fournisseur>**, en haut à droite. La touche est `Alt+W`. Il ouvre le
formulaire web de l'hébergeur dans votre navigateur, rempli avec ce que vous avez
saisi jusque-là. La boîte reste ouverte. Il ne demande aucun jeton et ne fait aucun
appel d'API, il fonctionne donc aussi après une erreur d'API.

- GitHub n'a pas de paramètre d'URL pour *brouillon*, la case brouillon n'est donc
  pas reportée. GitLab reçoit un préfixe de titre `Draft: `.
- GitLab ne prend les étiquettes que par id, elles sont donc ajoutées sous la forme
  d'une action rapide `/label ~"x"` à la fin de la description.
- Une très longue description est coupée pour que l'URL reste sous 6000 caractères,
  avec une note `[...truncated: paste the rest]`.
- Bitbucket Data Center n'a pas d'adresse de formulaire web, il n'a donc pas de
  bouton **Créer sur <fournisseur>**.

CLI : `gitpoulpi hosting issue create ... --web` et `gitpoulpi hosting pr --web`
affichent la même URL.

## La pastille de la barre d'outils

À l'extrémité de la barre d'outils (à droite par défaut), une pastille montre la demande de la
branche extraite : `GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) ou
`BB #12` (Bitbucket). Sa couleur indique l'état : vert ouverte, gris brouillon,
violet mergée, rouge fermée.

- Survolez-la pour le titre, l'état et `source → cible`.
- Cliquez dessus, ou appuyez sur `Ctrl+G g`, pour ouvrir la demande dans le
  navigateur.
- Clic droit dessus pour **Ouvrir dans le navigateur**, **Copier l'URL**,
  **Actualiser** et **Changer de profil**.

Une branche poussée sans demande montre une pastille `+ PR` estompée. Cliquez
dessus pour démarrer une pull request. Il n'y a pas de pastille quand aucun profil
ne correspond, quand la branche n'est pas poussée, ou quand l'hébergeur était
injoignable. Sur une fenêtre étroite, la pastille est masquée pour laisser de la
place aux boutons de la barre d'outils.

## Quand les listes se rafraîchissent

Les listes se rafraîchissent :

- à l'ouverture d'un dépôt ;
- quand la branche extraite change ;
- après un push ou un fetch ;
- toutes les *Actualisation pull requests et tickets (minutes)*
  (`general.hosting_refresh_minutes`, défaut 5, `0` désactive le minuteur ;
  Paramètres > Intégrations) ;
- quand vous fermez les Paramètres ;
- quand vous appuyez sur `r`, lancez *Intégrations : Actualiser les pull requests et les tickets*, ou
  changez un filtre.

Les requêtes ne bloquent jamais la fenêtre.

Les hébergeurs limitent le nombre d'appels. GitPoulpi renvoie l'`ETag` de la dernière
réponse, si bien qu'une liste inchangée coûte peu (rien sur GitHub). Quand un
hébergeur signale qu'il reste peu d'appels, la ligne d'état le dit. Quand il n'en
reste plus, GitPoulpi cesse d'interroger cet hébergeur jusqu'à la remise à zéro de
la limite, et les listes restent telles quelles.

## Terminologie

GitHub, Azure DevOps et Bitbucket disent « pull request ». GitLab dit « merge
request ». *Terme pour les demandes de revue* (`ui.pr_terminology`, dans Paramètres >
Intégrations) choisit le mot pour chaque libellé, menu, commande de palette et
message :

- **Auto (selon l'hébergeur)** : suit le fournisseur lié du dépôt (« Pull request »
  tant que rien n'est lié).
- **Pull request** ou **Merge request** : toujours ce mot.

Le changement s'applique aussitôt. La CLI et MCP disent toujours « pull/merge
request ». Cette page dit « pull request ».

## Références dans les messages de commit et les noms de branche

Quand les tickets ou les pull requests du dépôt sont liés, les références dans le
message de commit des Détails du commit sont soulignées. Un clic ouvre la page
liée. Clavier : la commande de palette *Intégrations : Ouvrir une référence de commit (#ticket / !demande)...*
(un menu quand le message a plusieurs références). Un dépôt sans rien de lié n'a
pas de liens.

Ce qui compte comme référence dépend du fournisseur de la section TICKETS :

| TICKETS liés à | Dans un message | Dans un nom de branche |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` et `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, seulement quand `ABC` est la clé du projet lié. Du texte comme `UTF-8`, `SHA-256` ou `ISO-8601` n'est jamais un lien. | `ABC-123` n'importe où dans le nom (`feature/ABC-123-x`), en majuscules |
| Linear | `ENG-123`, seulement quand `ENG` est la clé de l'équipe liée (ou d'une équipe de la liste d'équipes du profil). `UTF-8` n'est jamais un lien. | Le `branchName` propre à Linear quand vous créez la branche depuis un ticket ; `ENG-123` n'importe où dans le nom est relu |
| YouTrack | `ABC-123`, seulement quand `ABC` est le nom court du projet lié (ou d'un projet de la liste de projets du profil) | `ABC-123` n'importe où dans le nom |
| Redmine | `#123`, seulement quand Redmine est la source TICKETS liée (une source de pull requests garde son propre signe) | `123-fix-login` |
| Trello | une adresse `https://trello.com/c/<shortLink>` ; `#12` seulement quand une carte en cache porte le numéro 12 | `12-fix-login` |

| PULL REQUESTS liées à | Dans un message |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Quand les deux listes correspondent à `#N`, le ticket l'emporte. CLI :
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**Dans les conversations.** Le texte d'un ticket ou d'une demande est lu avec la grammaire de son hébergeur, plus large que le tableau ci-dessus : `owner/repo#7`, `!7`, `GH-7`, `@login`, les identifiants de commit et les autres formes listées dans [Références croisées](conversations.md#références-croisées). Les sources liées du dépôt sont comparées d'abord : une clé Jira dans un commentaire GitHub ouvre donc encore son ticket dans l'application. `gitpoulpi hosting markdown --body TEXT --id I` montre ce que l'application trouve dans un texte.

### Insérer une référence de ticket

Dans le compositeur de commits, le bouton `#▾` à l'extrémité droite du Résumé ouvre
une liste filtrable des tickets liés. Le ticket nommé par la branche courante vient
en premier, marqué *(de la branche)*. `Enter` ou un clic insère sa référence
(`ABC-123`, `#12`, `AB#12`) au niveau du curseur. Clavier : `Alt+R` dans le
compositeur. Palette : *Commit : Insérer une référence de ticket...*. Sans liste de
tickets liée, le bouton est grisé et dit *liez d'abord une liste de tickets dans la barre latérale*.
GitPoulpi n'ajoute jamais une référence tout seul.
