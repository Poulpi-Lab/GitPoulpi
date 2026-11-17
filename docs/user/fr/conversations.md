[English](../conversations.md) · [Čeština](../cs/conversations.md) · [Deutsch](../de/conversations.md) · [Español](../es/conversations.md) · **Français** · [Italiano](../it/conversations.md) · [日本語](../ja/conversations.md) · [한국어](../ko/conversations.md) · [Nederlands](../nl/conversations.md) · [Polski](../pl/conversations.md) · [Português (Brasil)](../pt-BR/conversations.md) · [Português (Portugal)](../pt-PT/conversations.md) · [Русский](../ru/conversations.md) · [Svenska](../sv/conversations.md) · [Türkçe](../tr/conversations.md) · [Українська](../uk/conversations.md) · [简体中文](../zh-CN/conversations.md)

# Conversations

La vue de conversation d'un ticket, d'un work item, d'une carte ou d'une pull / merge request. Elle demande un compte et un dépôt lié : voir [Intégrations d'hébergeurs et de suivi de tickets](hosting-integration.md) ; les listes qui l'ouvrent sont dans [Pull requests et tickets](pull-requests-and-issues.md) ; relire du code est dans [Relire une pull ou merge request](review.md).

Une **conversation** est tout ce qui se dit sur un ticket, un work item, une carte, une modification ou une pull / merge request : la description, les commentaires, les revues, les fils sur le code et les événements (étiquettes, assignations, changements d'état, pushes). GitPoulpi la lit chez l'hébergeur, l'affiche mise en forme et vous laisse répondre sans quitter la fenêtre.

### L'ouvrir

| Depuis | Comment |
|---|---|
| Une ligne de TICKETS | `Enter` ou un double-clic ; **Ouvrir la conversation** du menu de la ligne ; la palette *Hébergeur : Ouvrir la conversation d'un ticket...* (une liste des tickets à l'écran). **Ouvrir dans le navigateur** du menu de la ligne garde le navigateur. |
| Une ligne de PULL REQUESTS sur Azure DevOps ou Bitbucket | `Enter` ou **Ouvrir la conversation**. Sur GitHub, GitLab, Gitea, Forgejo et Gerrit, la ligne ouvre la [revue](review.md), dont la page Aperçu est cette même conversation (la chronologie, les touches et la zone de commentaire en dessous). |
| Une carte du [tableau des tickets](pull-requests-and-issues.md#le-tableau-des-tickets) | `Enter` ou un double-clic. |
| Le [Radar](radar.md) | `i` sur une ligne de ticket ou de demande, **Ouvrir la conversation** du menu de la ligne, ou *Radar : Ouvrir la conversation du ticket*. |
| Une référence dans une autre conversation | Cliquez dessus (ou `l`, puis choisissez-la) : `#7`, `!12`, `ABC-12` d'une source liée s'ouvrent dans l'application et s'empilent sur une pile Retour. |
| Un agent | `gitpoulpi ui show issue <n> --view conversation` (aussi `ui show pr`), voir [la référence du CLI](cli-reference.md). |

**Dépôts non clonés.** Une ligne du Radar d'un dépôt dont vous n'avez pas de clone ouvre aussi sa conversation : la vue ne demande que le profil d'intégration et le projet, si bien que le Radar l'héberge dans sa propre fenêtre, et `Esc` rend le Radar. Ce qui demande un clone est écarté : une référence de commit s'ouvre chez l'hébergeur au lieu du Log, et *Laisser l'IA lire le dépôt* est grisé avec sa raison (voir [le chapitre IA](ai.md#aide-à-la-rédaction-sur-un-hébergeur)). Une demande d'un dépôt cloné a sa revue à la place (`e`), et **Ouvrir la conversation** du Radar dit qu'il n'y a pas de conversation à lire ici pour elle ; un ticket d'un dépôt cloné l'ouvre dans l'onglet de ce dépôt.

Un ticket d'un hébergeur qui n'en a aucune à lire (Bitbucket et Gerrit n'ont pas de tickets) ouvre le navigateur comme avant, et une version compilée sans la fonctionnalité `conversation` fait de même partout ([versions personnalisées](custom-builds.md)).

### La vue

Une vue plein écran : l'en-tête (état, qui l'a ouvert et quand, les étiquettes, le nombre de commentaires, **Afficher les événements**, **Retour**, **Actualiser**, **Ouvrir dans le navigateur**), la ligne **Publication en tant que**, la chronologie, et dessous la zone de commentaire.

![La conversation du ticket #7 : la description avec un lien vers #9, deux commentaires, un événement d'étiquette affiché par Afficher les événements, et la zone Nouveau commentaire vide avec Écrire, Aperçu et Côte à côte](../images/issue-conversation.png)

![La page Aperçu de la merge request !58 : la description, un fil de code non résolu et un résolu avec leurs liens de fichier et de ligne, les commentaires, Répondre et Citer sur le fil sélectionné, et la zone de commentaire](../images/pr-conversation.png)

- La chronologie se lit du plus ancien au plus récent et commence sur la fenêtre la plus récente (300 entrées au plus). **Charger les plus anciens** en haut (`m`, palette *Conversation : Charger les entrées plus anciennes*) lit la fenêtre précédente, avec le nombre restant quand l'hébergeur le dit.
- Un commentaire montre son auteur, sa date, une pastille `modifié`, une pastille `bot` pour les bots, `interne` pour les notes privées, et les boutons **Répondre**, **Citer**, **Modifier**, **Supprimer** et **Liens** que permet l'entrée sélectionnée. Un fil sur du code montre `chemin:ligne`, `résolu` ou `non résolu`, `obsolète`, et ses commentaires. Une revue montre son verdict.
- Les événements tiennent chacun sur une ligne atténuée (*carol a ajouté l'étiquette lfs*). **Afficher les événements** (`v`) les masque ; le choix est gardé par dépôt. Les hébergeurs sans historique d'événements (YouTrack, work items Azure DevOps, notes locales) n'ont pas cette case.
- Le texte est rendu en Markdown : titres, listes imbriquées et de tâches, citations, alertes à la manière de GitHub, code coloré, tableaux, liens, images en `[image: alt]`. Le texte arrivé dans le balisage propre à l'hébergeur est d'abord converti (voir [Conversion du balisage](#conversion-du-balisage)). Un bloc de code `mermaid` reçoit un bouton **Ouvrir le diagramme** qui montre la source du diagramme dans une visionneuse défilante (`Enter` la copie) ; la palette a *Conversation : Ouvrir le diagramme de l'entrée*. Il faut une version avec à la fois `conversation` et `mermaid`.
- Dans l'Aperçu de la revue, le `chemin:ligne` d'un fil sur du code est un lien vers le code (`G`, *Conversation : Ouvrir le code du fil de discussion* ; voir [Revue](review.md)) ; dans une conversation ouverte seule (un ticket, une ligne du Radar), il n'y a pas de page de code où aller.
- La vue relit toutes les 60 secondes et sur `F5` (**Actualiser**).

**Touches.** Chacune a un bouton, une ligne de menu ou une ligne de palette (*Conversation : ...*), et `Ctrl+N` met des badges sur les boutons.

| Touche | Action |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Entrée précédente / suivante |
| `PageUp`, `PageDown`, `Home`, `End`, la molette, la barre de défilement | Défiler |
| `Tab`, `Shift+Tab` | Chronologie, zone de commentaire, puis boutons de l'en-tête |
| `r` | Répondre : dans le fil de l'entrée sélectionnée, ou en citant son texte là où l'hébergeur n'a pas de fils |
| `Q` | Citer l'entrée sélectionnée dans un nouveau commentaire |
| `c` | Nouveau commentaire |
| `e`, `d` | Modifier, supprimer un de vos commentaires (une question précède la suppression) |
| `l`, `Enter` | Les liens de l'entrée : `Enter` suit le seul lien, ou ouvre le menu des liens |
| `y`, `o` | Copier l'adresse de l'entrée, l'ouvrir dans le navigateur |
| `n` | Entrée non lue suivante |
| `v`, `m`, `F5` | Afficher ou masquer les événements, Charger les plus anciens, Actualiser |
| `Alt+Left`, `Backspace` | Retour dans la pile Retour |
| `q`, `Esc` | Fermer (demande d'abord quand la zone contient du texte) |

### Éditeur Markdown

**Nouveau commentaire** (`c`, ou `Tab` jusqu'à la zone), **Répondre** (`r`), **Citer** (`Q`) et **Modifier** (`e`) utilisent le même éditeur Markdown : **Écrire**, **Aperçu** et **Côte à côte** (`Alt+P` alterne Écrire et Aperçu, `Alt+L` active Côte à côte et demande une fenêtre de 100 colonnes). `Ctrl+Enter` (aussi `Ctrl+S` et `Alt+Enter`) ou **Envoyer** publie, **Annuler** abandonne la zone, et `Alt+E` ouvre **Écrire avec l'IA** là où l'IA est activée ([le chapitre IA](ai.md#aide-à-la-rédaction-sur-un-hébergeur)). L'aperçu est mis en page par le même moteur de rendu que la chronologie, avec les mêmes références. Quand l'hébergeur n'accepte pas le Markdown, le pied nomme le balisage dans lequel le texte est envoyé (*Envoyé en wiki Jira*, *Envoyé en Textile*).

![Une réponse à un ticket en cours d'écriture dans la vue Côte à côte : le brouillon Markdown à gauche, son aperçu rendu à droite, et Envoyer](../images/issue-reply-composer.png)

- Le commentaire est ajouté à la chronologie aussitôt et prend sa place quand l'hébergeur répond. Si l'hébergeur refuse, l'entrée disparaît et la zone se rouvre avec votre texte et la raison.
- Un texte non envoyé est gardé par élément (`remote-conversations.toml` dans le dossier de configuration) et le retrouve à la prochaine ouverture de cette zone ; l'envoyer ou l'abandonner le supprime. Fermer une zone qui contient du texte demande d'abord.
- **Répondre** sur un fil (un fil de revue, une discussion GitLab, un commentaire Linear, un fil Bitbucket ou Azure DevOps) publie dans ce fil. Là où l'hébergeur n'a qu'une liste plate, **Répondre** commence un nouveau commentaire citant l'entrée sélectionnée (`> **auteur**`), comme **Citer**.
- **Note interne** (case à cocher, `Alt+I`) : sur GitLab et Redmine, le commentaire n'est visible que des membres du projet. Elle n'apparaît que sur les hébergeurs qui ont de telles notes, et seulement sur un nouveau commentaire.
- Publier est une action sortante : rien n'est journalisé et **rien ne peut être annulé depuis GitPoulpi**. Sans connexion, la zone reste, **Envoyer** est grisé avec la raison, et l'en-tête dit que la copie enregistrée est affichée.

**Qui peut modifier ou supprimer.** Seuls vos propres commentaires portent **Modifier** et **Supprimer**. Le mot de l'hébergeur lui-même passe d'abord quand il en donne un (`viewerCanUpdate` et `viewerCanDelete` de GitHub, `can_edit` ailleurs), puis votre login est comparé à l'auteur du commentaire, sans tenir compte de la casse ; les commentaires d'un bot d'App doivent aussi venir d'un compte bot. Tant que le login d'un bot de GitHub App n'est pas connu, seuls comptent les commentaires que cette installation a publiés (le dépôt garde leurs identifiants) : rien n'est deviné d'après un nom. Dans un fil, **Modifier** et **Supprimer** agissent sur le dernier commentaire du fil qui est le vôtre. Un hébergeur qui ne peut ni modifier ni supprimer (voir [le tableau](#ce-que-chaque-hébergeur-ne-peut-pas-faire)) montre les boutons grisés avec la raison.

### Publier en tant que

La ligne sous l'en-tête dit en tant que qui vos messages sont publiés, et la même ligne est dans chaque zone :

| Ligne | Signification |
|---|---|
| *Publication en tant que ann (jeton personnel)* | Le jeton du profil : votre propre login (`GET /user`). |
| *Publication en tant que ci-bot (commande de jeton du dépôt)* | La [commande de jeton](hosting-integration.md#8-agir-comme-bot-github-app) du dépôt sert pour l'API d'hébergement. |
| *Publication en tant que my-app[bot] (bot GitHub App)* | Un jeton d'installation. Le nom vient de **Publier en tant que**, sinon de l'hébergeur. |
| *Publication en tant que bot GitHub App (son nom n'est pas encore connu : réglez Publier en tant que dans les paramètres)* | Aucun nom configuré, aucun appris pour l'instant. |
| *Publication en tant que ... (bot de projet)* | Un jeton d'accès de projet ou de groupe GitLab. |
| *Écriture en tant que kai (note locale)* | Un dossier de notes Markdown : le `local.me` du profil, sinon l'utilisateur connecté. |
| *Le compte de publication n'est pas encore connu. Réglez Publier en tant que dans les paramètres.* | Rien n'a pu être appris. |

**Publier en tant que** est un nom : pour une GitHub App, c'est `my-app[bot]`. Réglez-le pour tous les dépôts dans **Paramètres > Intégrations > Publier en tant que** (`general.hosting_post_as`), ou pour un seul dépôt dans **Paramètres > Propre au dépôt : SSH et identifiants > Hébergement**, qui l'emporte. Laissé vide, GitPoulpi interroge GitHub (GraphQL `viewer { login }`, qu'un jeton d'installation peut ou non satisfaire) et, à défaut, apprend le nom de l'auteur du premier commentaire qu'il publie et le garde avec le dépôt. Un jeton personnel n'en a jamais besoin. Voir [Agir comme bot GitHub App](hosting-integration.md#8-agir-comme-bot-github-app).

### Références croisées

Les références d'une conversation sont soulignées et cliquables. Les sources TICKETS et PULL REQUESTS liées du dépôt sont comparées d'abord : `ABC-12` d'un projet Jira lié s'ouvre donc dans l'application même dans un commentaire GitHub ; ce que la grammaire propre à l'hébergeur ajoute ouvre sa page après la confirmation du lien. Un identifiant de commit s'ouvre dans le Log quand le dépôt est cloné et que l'identifiant a au moins 7 caractères, chez l'hébergeur sinon. Rien dans un code en ligne, un bloc de code, un lien Markdown, une adresse e-mail ou une URL n'est une référence, et `C#1`, `UTF-8`, `SHA-256` et `ISO-8601` n'en sont pas.

| Hébergeur | Reconnu |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, identifiants de commit |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (un projet voisin), `@user`, `~label` et `%milestone` en pastilles, identifiants de commit |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, identifiants de commit |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, mentions `{uuid}`, identifiants de commit |
| Gerrit | valeurs `Change-Id` (`I` et 40 chiffres hexadécimaux), identifiants de commit |
| Redmine | `#7`, révisions `r123`, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, seulement pour un projet ou une équipe d'une source liée |
| Trello | adresses de cartes (`trello.com/c/...`) |
| Notes locales | `[[note]]`, `[[note\|alias]]` et liens `.md` relatifs vers une autre note du dossier |

`gitpoulpi hosting markdown --body T --id I` affiche ce que l'application fait d'un texte, références comprises.

### Conversion du balisage

Les hébergeurs qui ne parlent pas Markdown sont convertis dans les deux sens : vous lisez et écrivez du Markdown partout. Ce qu'un convertisseur ne comprend pas reste du texte lisible.

- **Jira** (Cloud et Data Center) stocke les commentaires en wiki par l'API v2. Ce que vous lisez est converti en Markdown ; ce que vous écrivez est converti en wiki, et le pied dit *Envoyé en wiki Jira*.
- **Redmine** met le texte en forme en Textile ou en Markdown, ce que l'API ne dit pas : la ligne **Format de texte du serveur** du profil (Paramètres > Intégrations, `markup`, Textile par défaut) l'indique à GitPoulpi. Le Textile est converti en Markdown pour la lecture et reconverti en Textile pour l'écriture.
- **Azure DevOps Server** renvoie les commentaires de work items en HTML et les accepte en HTML : ils sont affichés comme le Markdown converti à partir de lui et écrits en HTML. Azure DevOps Services et les fils de pull requests acceptent le Markdown.
- **Gerrit** est en texte brut : il n'est pas rendu en Markdown et ce que vous écrivez est envoyé tel que saisi.
- Chaque importeur retire d'abord les séquences de contrôle et d'échappement : un commentaire ne peut donc pas déplacer le terminal.

### La copie enregistrée, Charger les plus anciens, non lu

- Chaque conversation que vous ouvrez est enregistrée (`<dossier d'état>/conversations/<intégration>/<projet>/<type>-<n>.json`, réservée à l'utilisateur, sans jeton, 2 Mo par fichier au plus, les 50 fichiers les plus récents gardés). Elle s'affiche aussitôt à l'ouverture de l'élément et est actualisée derrière. Quand l'hébergeur est injoignable, la copie enregistrée reste à l'écran sous *Hors ligne : copie enregistrée il y a 3 min (raison)* et Envoyer est grisé.
- **Non lu :** ce que vous avez vu en dernier est retenu par conversation (les 200 dernières sont gardées). Les entrées plus récentes d'une autre personne reçoivent une barre d'accent et une pastille `nouveau`, et `n` saute à la suivante. Une conversation ouverte pour la première fois n'a rien de nouveau.

### Ce que chaque hébergeur ne peut pas faire

Chaque `non` ci-dessous est une vraie limite : le contrôle est masqué ou grisé avec la raison, et le CLI répond par une erreur qui nomme l'hébergeur avant que rien ne soit envoyé.

| Hébergeur | Éléments | Répondre dans un fil | Modifier et supprimer les siens | Résoudre un fil | Événements | Notes internes | Écrit en |
|---|---|---|---|---|---|---|---|
| GitHub | tickets, pull requests | pull requests | les deux | pull requests | oui | non | Markdown GitHub |
| GitLab | tickets, merge requests | oui (discussions) | les deux | merge requests | oui | oui | Markdown GitLab |
| Gitea | tickets, pull requests | pull requests | les deux | pull requests | oui | non | Markdown |
| Forgejo | tickets, pull requests | pull requests | les deux | non (l'hébergeur n'a pas de point d'accès) | oui | non | Markdown |
| Azure DevOps | work items, pull requests | pull requests | les deux | non (statut affiché, non modifié) | pull requests seulement | non | Markdown (HTML pour les work items de Server) |
| Bitbucket (Cloud, Data Center) | pull requests seulement | oui | les deux | non (affiché, non modifié) | oui | non | Markdown |
| Gerrit | modifications seulement | oui | non (ni l'un ni l'autre n'est possible) | oui | oui | non | texte brut |
| Jira | tickets | non (liste plate) | les deux | non | oui (historique des changements) | non (demande un rôle de projet) | wiki Jira |
| Linear | tickets | oui (réponses à un commentaire) | les deux | non | oui | non | Markdown |
| YouTrack | tickets | non | les deux | non | non | non | Markdown |
| Redmine | tickets | non | modifier seulement (Redmine 5 et suivants), jamais supprimer | non | oui (journaux) | oui (notes privées) | Markdown ou Textile |
| Trello | cartes | non | les deux | non | oui (déplacements) | non | Markdown |
| Notes locales | notes | non | celles de l'auteur local | non | non | non | Markdown |

Aussi : aucun hébergeur ne permet de démarrer un nouveau fil sur une ligne depuis la conversation (faites-le dans la revue), et les réactions ne sont affichées que sur GitHub. Un texte plus long que ce que l'hébergeur accepte est refusé (GitHub 65 536 caractères, GitLab 1 000 000, Azure DevOps 150 000, Jira 32 767, Trello 16 384) : le CLI vérifie d'abord, dans l'application le message de l'hébergeur est affiché et votre texte est gardé. Une note locale garde ses commentaires dans une dernière section `## Comments` du fichier de la note lui-même, une entrée `### auteur - heure` chacune, si bien qu'Obsidian et un shell les voient aussi.

En ligne de commande, les mêmes conversations sont `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit` et `comment-delete`, et `hosting review conversation` (voir [la référence du CLI](cli-reference.md)) ; pour les agents ce sont des outils MCP du même nom (voir [MCP et agents](mcp-and-agents.md)).
