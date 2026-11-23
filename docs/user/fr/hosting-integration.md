[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · **Français** · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · [Polski](../pl/hosting-integration.md) · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Intégrations d'hébergeurs et de suivi de tickets

GitPoulpi se connecte à ces produits :

- **Hébergement** (pull requests, tickets, clonage depuis votre compte, clés SSH) :
  GitHub, GitHub Enterprise, GitLab, GitLab auto-hébergé, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea et Forgejo
  (Codeberg compris). **Gerrit Code Review** est de l'hébergement pour la revue seulement : ses changes sont les
  pull requests, il n'a pas de tickets.
- **Suivi de tickets** (tickets seulement) : Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud et Server) et Redmine.
- **Notes Markdown locales** (tickets seulement, sans jeton, sans réseau) : un dossier de notes Markdown, par exemple un dossier d'un vault Obsidian, voir [Tickets locaux et Obsidian](local-issues.md).

Chaque intégration utilise un **jeton personnel**. Il n'y a ni compte GitPoulpi ni
connexion OAuth. Les intégrations sont facultatives et **activées dépôt par dépôt** :
rien n'est récupéré tant que vous n'avez pas lié un dépôt. Votre jeton est stocké
dans votre assistant d'identifiants, jamais dans `config.toml`. Les opérations
`git` simples (fetch, pull, push) n'ont pas besoin d'intégration : elles utilisent
les identifiants que votre `git` a déjà.

**Où aller ensuite.** Cette page est la mise en place : jetons, connexion, liaison d'un dépôt, clés SSH. Une fois un compte connecté :

- les sections PULL REQUESTS et ISSUES du panneau latéral, le tableau des tickets, la création d'une pull request ou d'un ticket, la pastille de la barre d'outils et les références dans les messages de commit : **[Pull requests et tickets](pull-requests-and-issues.md)** ;
- la vue dans l'application d'un ticket ou d'une demande, avec ses commentaires, l'écriture et le nom de publication : **[Conversations](conversations.md)** ;
- lire et relire une demande, avec des fils de commentaires et une revue en attente : **[Relire une pull ou merge request](review.md)** ;
- les pull requests et les tickets de tout un espace de travail dans une seule liste : **[Le Radar](radar.md)**.

## 1. Créer un jeton chez l'hébergeur

Le formulaire de connexion a un lien *Générer un jeton sur <hôte>*. Il ouvre la
page des jetons de l'hébergeur avec les bonnes portées déjà sélectionnées. Pour le
faire à la main :

| Hébergeur | Où | Portées |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (dépôts, pull requests, tickets), `read:user` (qui vous êtes), `admin:public_key` (seulement pour ajouter ou retirer des clés SSH) |
| GitLab, GitLab auto-hébergé | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, sur `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Créez le jeton **pour cette organisation**, pas pour « All accessible organizations » : les jetons globaux cessent de fonctionner le 2026-12-01. |
| Azure DevOps Server | La même page sur votre serveur (`https://<server>/<collection>/_usersSettings/tokens`) | Les deux mêmes portées |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. Pour les clés SSH, aussi `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Photo de profil > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, sur `<hôte>/user/settings/applications` | Lecture de `user` ; lecture et écriture de `repository`, `issue` et `user` (le dernier pour les clés SSH). Gitea n'affiche aucune portée après coup, GitPoulpi ne peut donc pas avertir d'une portée manquante : un appel refusé le dit. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. La clé agit en votre nom. |
| YouTrack (Cloud et Server) | Votre profil > Account Security > New token, sur `<hôte>/users/me?tab=account-security` | La portée `YouTrack`. Un jeton permanent agit en votre nom. |
| Gerrit | Settings > HTTP Credentials > Generate new token (serveurs récents ; les plus anciens : Generate new password), sur `<hôte>/settings/#HTTPCredentials` | Aucune. Le jeton est le **mot de passe HTTP** et agit en votre nom ; le nom d'utilisateur se met à côté. |
| Redmine | My account > API access key > Show, sur `<hôte>/my/account`. L'administrateur doit avoir activé **Administration > Settings > API > Enable REST web service**. | Aucune. La clé agit en votre nom. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (le type classique, sans portées) | Aucune. Le jeton agit en votre nom. |
| Jira Data Center | Profile > Personal Access Tokens | Aucune. Le jeton agit en votre nom. |
| Trello | D'abord une **clé d'API** depuis `trello.com/apps/admin` (créez un Power-Up, puis générez une clé d'API). Ensuite un **jeton** depuis `trello.com/1/authorize?...&key=<votre clé>`. | `read`, `write` |

Donnez au jeton un nom comme « GitPoulpi » et une date d'expiration. Copiez le
jeton : l'hébergeur ne l'affiche qu'une fois.

## 2. Le connecter

1. Appuyez sur `Ctrl+,` et choisissez **Intégrations**.
2. Choisissez le produit à gauche. Les produits sont en deux groupes :
   **Hébergement** et **Suivi de tickets**.
3. Appuyez sur **Se connecter**. Pour ajouter un second compte du même produit,
   utilisez **+ Ajouter un profil**.

   ![La page Intégrations : les produits à gauche et un bouton Se connecter pour GitHub](../images/settings-integrations.png)

4. Remplissez le formulaire et collez le jeton. Le jeton s'affiche en clair pendant
   que vous le tapez, assurez-vous donc que personne ne regarde. Le formulaire
   liste les portées dont il a besoin.

   ![Le formulaire de connexion : un champ Personal Access Token, les portées repo, read:user et admin:public_key, un lien pour générer un jeton sur github.com, et les boutons Se connecter et Annuler](../images/settings-connect.png)

5. Appuyez sur **Se connecter**. GitPoulpi vérifie le jeton auprès de l'hébergeur en
   arrière-plan. La page affiche alors une carte avec votre nom et votre identifiant,
   *Connecté* en vert, et la section des clés SSH. Un jeton qui n'a pas l'une des
   portées voulues se connecte quand même, et les portées manquantes sont listées en
   rouge.

Sur la carte : **Modifier** rouvre le formulaire. **Déconnecter** supprime le jeton
et garde le profil. **Retirer** supprime les deux, après confirmation.

Clavier sur la page Intégrations : `Tab` / `Shift+Tab` passent de la liste des
produits à la bande des profils et au formulaire. `a` ajoute un profil, `e` le
modifie, `d` ou `Delete` le retire, `o` ouvre la page des jetons, `Esc` revient
d'un cran.

### Ce que demande chaque formulaire

Le lien *Générer un jeton* ne fonctionne qu'après avoir renseigné l'hôte ou
l'organisation.

- **GitHub, GitLab :** seulement le jeton. **GitHub Enterprise** et **GitLab
  auto-hébergé** demandent aussi le **Domaine de l'hôte** (`gitlab.example.com` ;
  https sauf si vous saisissez `http://`). Un nom sans point (un service Docker ou
  une machine du réseau local) demande un port ou `http://` : `gitea:3000`,
  `http://nas`. Un serveur auto-hébergé peut être en
  http simple, par exemple un conteneur local sur `http://localhost:8080`. Un jeton envoyé en http
  simple circule en clair : n'utilisez http que sur un réseau local ou de confiance.
- **Azure DevOps :** l'**Organisation**, le nom dans `dev.azure.com/<organization>`.
  Une adresse `https://<org>.visualstudio.com` est aussi acceptée. Deux
  organisations demandent deux profils. Un dépôt utilise le profil de l'organisation
  que nomme son distant. **Azure DevOps Server** demande l'**URL de la collection**,
  par exemple `tfs.corp/tfs/DefaultCollection`. GitPoulpi n'utilise pas l'assistant
  d'identifiants du système pour Azure DevOps : Git Credential Manager y stocke un
  jeton de connexion, pas un jeton d'accès personnel.
- **Bitbucket :** votre **E-mail du compte Atlassian** et un jeton d'API. Si
  l'e-mail est vide, le jeton est envoyé comme jeton porteur : utilisez cela pour un
  jeton d'accès d'espace de travail ou de dépôt. GitPoulpi trouve vos espaces de
  travail à partir du jeton. **Bitbucket Data Center :** l'**URL du serveur** et un
  jeton d'accès HTTP.
- **Gitea, Forgejo :** le **Domaine de l'hôte** et le jeton. **Forgejo** démarre sur
  `https://codeberg.org` : remplacez-le pour un serveur auto-hébergé. Gitea n'a pas de
  valeur publique par défaut. Une adresse `http://localhost:3000` marche pour un serveur local.
  Les deux types partagent un seul client (Forgejo est un fork de Gitea avec le même
  `/api/v1`) ; le jeton est envoyé comme `Authorization: token <token>`. L'assistant
  d'identifiants du système n'est pas utilisé pour eux.
- **Jira Cloud :** l'**URL du site** (`acme.atlassian.net`), votre **E-mail du compte
  Atlassian** et un jeton d'API classique. Les jetons Jira à portées ne sont pas
  pris en charge. **Jira Data Center :** l'**URL du serveur** et un jeton d'accès
  personnel.
- **Trello :** la **clé d'API** et le **jeton**. La clé d'API est publique, elle est
  donc gardée dans `config.toml` (sous `account`). Le jeton est secret et va dans
  l'assistant d'identifiants. Le formulaire a deux liens : *Obtenir une clé d'API sur Trello* et,
  après avoir tapé la clé, *Générer un jeton*. La clé et le jeton sont envoyés dans
  un en-tête, jamais dans une URL.
- **Linear :** seulement la **clé d'API** : Linear n'a qu'un hôte public, il n'y a
  donc pas d'adresse à saisir. La clé est envoyée telle quelle dans l'en-tête `Authorization`
  (Linear n'accepte pas `Bearer` pour les clés personnelles). Pour que les références `ENG-12`
  des messages de commit ouvrent la bonne page, enregistrez l'URL du profil avec votre
  espace de travail (`https://linear.app/acme`, depuis la CLI : `--url`) ; les lignes de tickets
  elles-mêmes portent toujours l'adresse exacte.
- **YouTrack :** l'**URL du site** (`acme.youtrack.cloud` pour Cloud, votre propre
  serveur sinon, avec son chemin de contexte s'il en a un, par exemple
  `host/youtrack`) et un jeton permanent, envoyé comme `Authorization: Bearer`. Le
  même client sert Cloud et Server (`<url>/api`).

- **Gerrit :** l'**URL du serveur** (avec son chemin de contexte s'il en a un), votre **Nom d'utilisateur** et le mot de passe
  HTTP comme jeton. Ils voyagent en authentification HTTP basic ; GitPoulpi parle à l'API REST `/a/` du
  serveur et ignore la ligne `)]}'` par laquelle commence chaque réponse. Le nom d'utilisateur n'est pas secret et est gardé dans
  `config.toml` (sous `account`), le mot de passe va dans l'assistant d'identifiants. Un dépôt est lié à un *projet* Gerrit
  sous PULL REQUESTS (ses changes y sont listés) ; Gerrit n'a pas de tickets, et un change se crée
  en poussant (voir *Envoyer pour relecture* dans [Relire](review.md#gerrit)).
- **Redmine :** l'**URL du site** de votre Redmine (avec son sous-chemin s'il en a un,
  par exemple `host/redmine`) et votre clé d'API, envoyée comme `X-Redmine-API-Key`. Si
  Redmine répond 403 ou une page HTML, le service web REST est désactivé (demandez à
  l'administrateur, ou activez-le dans **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack et Redmine ne sont que des outils de suivi de tickets : pas de pull requests, pas
de dépôts à cloner et pas de clés SSH. Bitbucket n'a pas de tickets (Atlassian a
retiré Bitbucket Issues). Liez plutôt la section TICKETS d'un dépôt Bitbucket à un
projet Jira.

## 3. Où le jeton est stocké

Le jeton est stocké dans l'**assistant d'identifiants** de votre système (Git
Credential Manager sous Windows, libsecret ou `pass` sous Linux, Keychain sous
macOS). Il utilise une entrée privée qui ne peut pas être confondue avec les
identifiants d'un vrai hôte. Il n'est jamais écrit dans `config.toml` ni affiché.

Si aucun assistant d'identifiants n'est configuré, ou s'il refuse le jeton,
GitPoulpi garde le jeton **en mémoire pour cette session seulement** et vous le
dit. Reconnectez-vous après un redémarrage.

GitPoulpi cherche le jeton d'un profil dans cet ordre :

1. La variable d'environnement `GITPOULPI_TOKEN_<ID DU PROFIL>`. Utilisez-la pour
   donner un jeton sans le stocker, par exemple en CI.
2. Le jeton stocké pour le profil (`token_ref`).
3. Pour les profils GitHub et GitLab seulement : l'assistant d'identifiants du
   système pour l'hôte, quand aucun autre profil activé n'utilise le même hôte.

Chaque profil est une entrée `[[integrations]]` de `config.toml`. Voir la
[référence de configuration](config-reference.md#integrations-hébergeurs-et-suivi-de-tickets).

## Le contrôle de sécurité

Un jeton enregistré n'est sûr que dans la mesure où l'adresse à laquelle il est envoyé l'est. GitPoulpi retient donc, pour chaque intégration, à quoi elle était connectée, et s'arrête quand cela change :

- **L'adresse.** La première fois qu'une intégration est utilisée (ou connectée), son adresse est notée dans `trust.toml`, dans le dossier de données privé réservé à l'utilisateur (`<data>/private/`), et non dans `config.toml` : modifier la configuration ne peut jamais élargir ce qui est approuvé. Si l'adresse de la configuration n'est plus celle qui est notée, rien ne lui est envoyé et le jeton n'est même pas lu. Les écritures d'une même adresse (`HTTPS://Git.Example.com:443/`) sont la même adresse ; un hôte qui lui ressemble n'en est pas une.
- **Le compte.** Après la connexion d'un jeton, puis environ une fois par jour (et une fois par exécution pour un jeton qui vient de `GITPOULPI_TOKEN_*` ou de l'assistant d'identifiants), GitPoulpi demande à l'hébergeur à qui appartient le jeton et compare l'**id** du compte, pas l'identifiant de connexion. Un jeton qui appartient désormais à quelqu'un d'autre est arrêté.
- **Le certificat.** Les serveurs dont le certificat ne remonte pas à une autorité connue (une autorité privée, un serveur auto-signé) ne sont acceptés que pour la clé exacte que vous avez acceptée, et la poignée de main TLS prouve toujours que le serveur détient cette clé. Avec l'épinglage strict, la clé doit correspondre même quand le certificat remonte à une racine publique ; le prix est que chaque renouvellement de clé redemande, c'est pourquoi il est optionnel. Les hôtes publics (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) ne gardent aucune trace de certificat.
- **Le http simple.** Une adresse `http://` non loopback est refusée sauf si vous l'avez autorisée pour cette intégration.

Chaque requête est aussi tenue à l'adresse propre de l'intégration (et, pour un hôte public, à son hôte d'API comme `api.github.com`) : une requête ailleurs est refusée avant d'être envoyée, et une redirection vers une autre adresse n'est pas suivie.

**Les intégrations connectées avant l'existence de ce contrôle** reçoivent leur trace silencieusement, à partir de la configuration telle qu'elle était, la première fois qu'elles sont utilisées après la mise à niveau : elles gardent le niveau de confiance d'aujourd'hui et rien ne demande.

### En ligne de commande

Une commande arrêtée échoue avec le code de sortie 2 et affiche ce qui a changé (voir la [référence CLI](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` montre ce qui est approuvé ; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` et `--allow-insecure-http on|off` acceptent chacun une chose, en nommant la valeur exacte. Après avoir accepté une nouvelle adresse, saisissez à nouveau le jeton. Se déconnecter (`hosting logout`) ou retirer une intégration supprime son jeton, sa trace de confiance et ce qui en avait été mis en cache (les listes de dépôts enregistrées de la boîte Cloner, les listes de compte du Radar et les brouillons de revue distante de son hôte).

Un agent (MCP) ne peut rien accepter : on lui dit que le contrôle de sécurité a besoin de vous.

### Dans l'interface du terminal

Une requête arrêtée ouvre la boîte **Contrôle de sécurité de l'intégration** par-dessus ce que vous faisiez. Elle dit ce qui a changé (l'ancienne et la nouvelle adresse, l'ancien et le nouveau compte avec leurs id, ou l'ancien et le nouveau certificat sous forme d'un petit tableau avec l'empreinte SHA-256 en groupes de quatre, le sujet, l'émetteur et la validité) et que le jeton n'a pas été envoyé. Les réponses sont numérotées :

1. **Déconnecter** supprime le jeton enregistré, la trace de confiance et ce qui avait été mis en cache pour l'intégration.
2. **Approuver** la nouvelle adresse, le nouveau compte ou le nouveau certificat (pour une adresse non chiffrée : **Autoriser http simple**).
3. **Annuler** ne change rien.

Aucune réponse n'est mise en évidence tant que vous n'appuyez pas sur une flèche ou un chiffre, donc un Enter par mégarde n'accepte rien. `Esc` et la croix de fermeture annulent ; un clic à l'extérieur de la boîte ne fait rien. Un long texte défile avec la molette, PgUp/PgDn ou la barre de défilement. Plusieurs contrôles se mettent en file (un par intégration et par type de changement) et sont posés l'un après l'autre. Un contrôle annulé ne réapparaît pas tant qu'il n'a pas changé ; **Intégrations : Revoir le contrôle de sécurité** dans la palette de commandes le ramène.

Après avoir approuvé une nouvelle **adresse**, le formulaire de l'intégration s'ouvre avec le champ du jeton actif et l'indication « Il est recommandé de saisir à nouveau le jeton. » : le jeton enregistré n'y a jamais été envoyé. Dans le formulaire, une adresse changée volontairement est acceptée avec un jeton saisi pour elle ; sans jeton, le formulaire en demande un.

Le formulaire a aussi deux cases. **Épinglage strict du certificat** (pour une adresse https) épingle la clé du certificat que le serveur présente quand vous vous connectez. **Autoriser http simple (non chiffré)** n'est proposée que pour une adresse `http://` qui n'est pas cette machine, avec un avertissement rouge disant que le jeton circule en clair. La carte connectée affiche l'empreinte épinglée.

### Ce que le jeton peut faire, et jusqu'à quand

La connexion demande aussi à l'hébergeur ce que le jeton peut faire. GitHub (jetons classiques), GitLab et Bitbucket (quand il envoie les portées) indiquent leurs portées : la carte affiche une ligne jaune « Ce jeton a plus de droits que GitPoulpi n'en a besoin » listant les portées au-delà de ce que GitPoulpi utilise, les sensibles (comme `delete_repo`, `admin:org`, `sudo`) en rouge, avec le conseil de préférer un jeton à permissions détaillées avec une date d'expiration. Un jeton GitHub classique (`ghp_...`) reçoit ce conseil même quand ses portées sont exactes. On ne peut pas demander ses permissions à un jeton GitHub à permissions détaillées : seule son expiration est affichée. La carte affiche la date d'expiration quand l'hébergeur la donne. Un jeton sans expiration n'affiche aucun rappel, et les autres produits n'indiquent aucune portée et ne lèvent aucun avertissement. Les mêmes champs sont dans le JSON de `hosting integrations login` et `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). Ils sont lus à la connexion : la carte les affiche donc pour la session qui s'est connectée.

## 4. Lier un dépôt

La liaison est explicite et par dépôt. Quand un profil est connecté, le panneau
latéral Refs affiche une section **PULL REQUESTS** et une section **TICKETS** après
TAGS. Chaque section a deux listes déroulantes : le fournisseur et le projet.

- La liste des fournisseurs **propose** les comptes qui correspondent aux distants du
  dépôt (`origin` d'abord), sous forme de lignes comme `owner/name · origin · GitHub`.
  Un seul choix règle le fournisseur et le projet.
- Rien n'est lié automatiquement. Jira, Trello, Linear, YouTrack et Redmine ne sont jamais proposés, car aucun
  distant ne les nomme : choisissez-les vous-même.
- **Aucun (délier ce dépôt)**, en haut de la liste, délie la section. Rien ne la
  relie ensuite tout seul.
- La liaison est enregistrée dans le fichier `.gitpoulpi/repo-state.toml` du dépôt.

Les deux sections sont liées **séparément**. Par exemple, les pull requests peuvent
venir d'Azure DevOps et les tickets de Jira.

| Fournisseur | Projet PULL REQUESTS | Projet TICKETS |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | pareil |
| Azure DevOps | `Project/Repo` (l'organisation vient du profil) | `Project` (work items) |
| Gitea, Forgejo | `owner/name` | pareil |
| Bitbucket | `workspace/slug` (Data Center : `PROJ/slug`) | aucun : liez Jira |
| Jira | aucun | la clé du projet, par exemple `ABC` |
| Trello | aucun | un tableau |
| Linear | aucun | une clé d'équipe, par exemple `ENG` (les équipes sont listées) |
| YouTrack | aucun | le nom court d'un projet, par exemple `ABC` |
| Gerrit | le nom du projet Gerrit, par exemple `team/tool` | aucun |
| Redmine | aucun | un identifiant de projet, par exemple `bench` (la dernière partie de l'adresse du projet) |

La liste des projets TICKETS est filtrable. Vous pouvez aussi saisir un projet que la
liste n'affiche pas.

**Requête supplémentaire.** Une source de tickets a une ligne de filtre facultative
sous la liste des projets. Elle est combinée (ET) avec le projet :

- Jira : un fragment **JQL** (`component = UI AND labels = backend`).
- Azure DevOps : une condition **WIQL** (`[System.AssignedTo] = @Me`).
- YouTrack : une **requête YouTrack** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Elle est mise entre parenthèses, si bien qu'un `or` ne peut pas élargir le projet. Un
  `sort by:` de votre part remplace le `sort by: updated desc` par défaut.
- Redmine : aucune (les filtres d'état et « assigné à moi » de la section suffisent).
- Linear : aucune (les filtres d'état et « assigné à moi » de la section
  suffisent).
- Trello : une liste où vous cochez les **listes** à afficher. Sans coche, toutes les
  listes ouvertes s'affichent.
- GitHub et GitLab l'ignorent.

`Enter` garde une requête saisie, `Esc` l'abandonne. Palette :
*Tickets : Modifier la requête / les listes...*.

Depuis la CLI : `gitpoulpi hosting link pulls --id <profile>` (ou `issues`),
`--clear` pour délier, `--filter` pour la requête supplémentaire. Voir la
[référence CLI](cli-reference.md#commandes).

### Cloner depuis un compte connecté

Un profil connecté (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) est aussi une source dans la boîte de dialogue **Cloner** (le bouton **Cloner** de l'onglet Home, ou `c` dans cet onglet). Chaque profil est listé sous **Cloner par URL** comme une source à part (`GitHub · name`, `GitLab · name`) ; GitHub et GitLab ne sont jamais mélangés. Choisissez-en un et ses dépôts se chargent, groupés par propriétaire : votre propre compte d'abord, puis une section par organisation ou groupe (`quarry/tools` pour un sous-groupe GitLab), chacune sous un en-tête avec son compte. Un cadenas marque un dépôt privé.

![La boîte de dialogue Cloner sur un compte GitHub connecté : une section par propriétaire, un cadenas sur un dépôt privé](../images/clone-hosted.png)

Le champ de recherche filtre dans toutes les sections (nom, chemin complet et description) ; une section sans correspondance disparaît. `Enter` sur une ligne remplit le clone avec l'URL HTTPS de ce dépôt et un nom de dossier.

![La boîte de dialogue Cloner, la recherche réduisant la liste à une organisation](../images/clone-hosted-search.png)

La liste d'un profil déjà ouvert s'affiche aussitôt à partir d'une copie enregistrée pendant que l'hébergeur est interrogé de nouveau (*actualisation...* à côté du libellé) ; quand la réponse arrive, les nouveaux dépôts apparaissent et ceux qui ont disparu s'effacent, et le dépôt en surbrillance et le dépôt choisi restent à leur place. Si l'actualisation échoue, la liste enregistrée reste affichée avec une note, la raison est dans le Journal d'activité.

Si l'hébergeur refuse le jeton, la boîte de dialogue affiche *impossible de lister les dépôts* avec la raison à la place de la liste (et la copie enregistrée est supprimée) ; les autres sources continuent de fonctionner.

## 5. Clés SSH

### Clé SSH par profil

Un profil connecté a une section **Clé SSH** sous sa carte. Sans clé, choisissez :

- **Clé par défaut** : votre ssh-agent ou `~/.ssh`. Rien ne change.
- **Générer une clé** : un titre facultatif (par défaut `GitPoulpi <hôte> <date>`).
  Avec **Ajouter la clé publique à <hôte>** cochée, GitPoulpi l'envoie, si
  l'hébergeur ne l'a pas déjà. La clé est écrite sans phrase secrète dans
  `~/.ssh/gitpoulpi_<id du profil>_ed25519` et `.pub`. Aucun `ssh-keygen` n'est
  nécessaire. Un fichier existant n'est jamais écrasé.
- **Clé personnalisée** : saisissez un chemin ou **Parcourir** pour un fichier de
  clé privée, puis appuyez sur **Utiliser cette clé**.

Quand une clé est attachée, la section affiche `titre (SHA256:...)` avec :

- **Retirer** : détacher la clé, et la supprimer chez l'hébergeur si GitPoulpi l'y a
  ajoutée.
- **Supprimer** : pareil, et supprimer aussi les fichiers de clé, après une
  confirmation qui liste les deux chemins.

### Clé SSH pour un dépôt

Pour qu'un dépôt utilise une clé précise, ouvrez le menu de l'onglet (`Ctrl+T`, ou
clic droit sur l'onglet) et choisissez **SSH et identifiants...**, ou lancez la
commande de palette *Dépôt : SSH et identifiants...*, puis choisissez **Fichier de
clé** (ou **Clé du compte** pour la clé d'un profil d'intégration) sous SSH. Le choix
est conservé dans le dossier privé de GitPoulpi : rien n'est écrit dans le dépôt ni
dans la configuration de git. Voir
[Paramètres](settings.md#propre-au-dépôt--ssh-et-identifiants).

Un dépôt qui a encore son propre `core.sshCommand`, issu de l'ancienne épingle
« Clé SSH de ce dépôt », l'affiche sur cette page, en lecture seule, avec
**Déplacer dans GitPoulpi**.

CLI et MCP : `gitpoulpi repo-auth` (le choix par dépôt) et, pour l'épingle
`core.sshCommand` elle-même, `gitpoulpi ssh-command show | set --key <file> |
clear`.

La clé globale de Paramètres > SSH et identifiants (*Chemin de la clé privée*, ou
*Générer une paire de clés*) s'applique à chaque dépôt sans choix propre. Le
fichier de clé ou l'épingle d'un dépôt l'emporte.

## 6. Protection des branches, pour une réécriture d'historique

Quand vous réécrivez l'historique ([l'atelier](history-rewrite.md)), GitPoulpi
demande à l'hébergeur quelles branches il protège, afin de les laisser de côté
par défaut. Il ne le demande que pour un dépôt que vous avez **lié** (la section
des demandes de fusion ou des tickets de la barre latérale, ou un profil épinglé),
avec le jeton de ce profil, et un jeton qui ne peut pas le lire répond simplement
« inconnu » : rien n'échoue. Ce que répond chaque fournisseur, et ce dont le jeton
a besoin :

| Hébergeur | Ce qui est lu | Le jeton doit avoir |
|---|---|---|
| GitHub (et Enterprise Server) | l'indicateur `protected` des branches, et les rulesets de branche actifs (motifs `~DEFAULT_BRANCH`, `~ALL` et `refs/heads/...`) | un jeton classique avec `repo` (`public_repo` pour un dépôt public), ou un jeton à portée fine avec Metadata (lecture). Un ruleset que le jeton ne peut pas lire est signalé, les règles classiques comptent quand même |
| GitLab (gitlab.com et auto-hébergé) | `protected_branches` (noms et motifs `*`) | `read_api`, et un rôle qui peut voir les réglages du dépôt |
| Azure DevOps (Services et Server) | les stratégies de branche activées et bloquantes du dépôt (portées `Exact`, `Prefix` et `DefaultBranch`) | Code (lecture). La permission « force push » est un réglage de sécurité, elle n'est pas lue |
| Bitbucket Cloud | les restrictions de branche qui bloquent un push (`push`, `force`, `delete`) | `repository:admin`. Une restriction sur le modèle de branches n'est pas résolue |
| Bitbucket Data Center | les permissions de branche qui bloquent un push forcé (`fast-forward-only`, `pull-request-only`, `read-only`) | administrateur du dépôt |

La page de réglages de la protection (et de la branche par défaut, et de la
suppression du dépôt) est construite à partir de l'URL du distant du dépôt pour
GitHub, GitLab, Azure DevOps et Bitbucket, même quand le dépôt n'est pas lié, et
affichée avec les trois façons de passer quand l'hébergeur refuse un push.

## 7. Listes à l'échelle du compte dans le Radar

Le Radar surveille normalement les dépôts que vous avez clonés. Un profil peut aussi
lister ce qui vous concerne sur tout l'hébergeur, cloné ou non, avec trois interrupteurs dans
sa table `[integrations.radar]` (tous désactivés par défaut ; voir la
[référence de configuration](config-reference.md#integrationsradar-listes-à-léchelle-du-compte)) :

- **demandes de relecture** : les demandes ouvertes qui demandent votre relecture ;
- **demandes ouvertes** : toutes les demandes ouvertes de votre propre espace de noms et de chaque organisation ou
  groupe dont vous êtes membre, ou des espaces de noms que vous nommez. Jamais tout l'hébergeur ;
  les **Espaces de noms** du profil (un par ligne dans Paramètres > Intégrations > Radar, éventuellement
  en expressions régulières) limitent les trois listes : une ligne correspond au chemin complet d'un dépôt
  quand elle lui est égale ou en est un préfixe qui s'arrête à un `/` (`orbit` garde `orbit/products/x`, pas `orbitx/y`), et
  s'applique aux lignes renvoyées par l'hébergeur, ainsi que côté serveur là où la recherche de l'hébergeur
  le permet (les qualificateurs `user:` / `repo:` de GitHub) ;
- **tickets assignés** : les tickets ouverts qui vous sont assignés.

GitHub, GitLab (auto-hébergé compris), Gitea, Forgejo, Azure DevOps (Services
et Server), Bitbucket Data Center et Gerrit répondent. Comment chacun est interrogé :

| Hébergeur | Demandes | Tickets |
|---|---|---|
| Azure DevOps | les pull requests actives de toute l'organisation (`searchCriteria.reviewerId` pour celles qui vous attendent ; une demande sur laquelle vous avez déjà voté est omise) ; quand le serveur refuse la route au niveau de l'organisation, les 20 premiers projets sont interrogés un par un. Les `namespaces` sont des noms de projet | les work items qui vous sont assignés et ne sont ni Done, ni Closed, ni Removed |
| Bitbucket Data Center | votre tableau de bord : le rôle `REVIEWER` sans approbation pour les demandes de relecture, `AUTHOR` et `PARTICIPANT` pour les demandes ouvertes ; les `namespaces` sont des clés de projet | aucun (Data Center n'a pas de suivi de tickets ; une note le dit) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored` ; les demandes ouvertes sont `is:open projects:<préfixe>` par espace de noms (un préfixe de chemin de projet), ou vos changes et ceux que vous relisez quand aucun n'est nommé ; un change en work-in-progress est un brouillon | aucun |

Bitbucket Cloud n'a pas de lignes à l'échelle du compte : il n'offre aucun point d'accès qui liste les
demandes de tous les dépôts, et interroger chaque dépôt ferait trop d'appels. Ses dépôts clonés gardent leurs propres listes, et `gitpoulpi hosting
radar --id <bitbucket>` le dit. Quand Gerrit a plus de changes que la limite qu'il
renvoie, la liste est marquée coupée avec une note.

Les brouillons sont omis sauf si vous les gardez. Chaque liste est bornée : 300 lignes par type, avec une
note disant combien d'autres il y avait, et sur GitHub quelques appels de recherche par
tour, car son API de recherche n'en autorise que 30 par minute. Un espace de noms que le jeton
ne peut pas lire devient une note et les autres se chargent quand même ; un jeton refusé ou une
limite de débit fait échouer ce profil seulement. Les réponses sont réutilisées pendant cinq minutes
(`radar-account.json` dans le dossier de configuration).

Dans Paramètres > Intégrations, un profil d'hébergeur de code connecté a un bloc **Radar**
sous sa clé SSH : une case par liste (*Demandes en attente de ma relecture*,
*Toutes les demandes ouvertes*, *Inclure les brouillons*, *Tickets qui me sont assignés*, cette dernière
absente pour les hébergeurs sans tickets), une zone *Espaces de noms* (une entrée par ligne : un nom qui correspond à ce groupe ou
propriétaire et à tout ce qu'il contient, ou une expression régulière par ligne quand
*Expressions régulières* est cochée ; vide signifie le vôtre et tous ceux dont vous êtes
membre) et le choix *Demande d'un dépôt non cloné* : *Demander*,
*Le cloner* ou *Relire via l'API*.
Tout est enregistré dès que vous le changez. Les hébergeurs sans requête à l'échelle du compte (Bitbucket
Cloud) montrent à la place une ligne qui le dit. La palette de commandes atteint le bloc
avec *Paramètres : Options du Radar d'un profil...*, et la recherche des Paramètres trouve chaque
contrôle par son libellé.

En ligne de commande : `gitpoulpi hosting integrations radar <id> --review-requests on`
active une liste, `gitpoulpi hosting radar --json` affiche les listes (et
`--refresh` ignore le cache). Les agents IA ont les deux mêmes commandes via MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 8. Agir comme bot GitHub App

Un projet qui ne doit montrer le nom d'aucune personne peut tout faire chez l'hébergeur par une **GitHub App** : chaque commentaire, revue, fermeture ou réouverture et changement d'étiquette apparaît alors comme `<app-slug>[bot]`. Cela marche pour un dépôt à la fois et laisse vos autres dépôts sur votre propre jeton.

1. Créez une GitHub App pour l'organisation avec les permissions **Issues** et **Pull requests** en *Read & write* (et *Contents : read*, pour que les pull requests puissent être listées et relues), et installez-la sur le dépôt.
2. Écrivez un petit script qui affiche un **jeton d'installation** sur sa sortie standard (la clé privée de l'App signe un JWT de courte durée, échangé contre un jeton valable une heure). GitPoulpi ne voit jamais la clé.
3. Dans **Paramètres > Propre au dépôt : SSH et identifiants**, choisissez **Commande de jeton** sous HTTPS, saisissez le script comme **Commande** et l'hôte (`github.com`) sous **Hôtes**, et cochez **Utiliser pour l'API d'hébergement (tickets, pull requests)**. Enregistrez et confirmez la commande (une personne la confirme toujours, avec son SHA-256). En ligne de commande : `gitpoulpi repo-auth set --https token-command --hosts github.com --use-for-hosting --yes --command ./app-token.sh`.

Ce qui change pour ce dépôt :

- Ses appels de tickets et de pull requests (liste, création, commentaire, revue, merge, fermeture, étiquettes) utilisent le jeton de la commande, avec la même limite de 30 secondes, le même cache en mémoire et la même rédaction que la connexion propre à git. Quand l'hébergeur répond 401 parce que le jeton a expiré, la commande est redemandée une fois. Si la commande ne peut pas s'exécuter, l'appel est refusé : votre jeton personnel n'est jamais le repli.
- Il n'y a pas de « moi » : un jeton d'installation ne peut pas demander qui il est. L'en-tête de section nomme le profil avec **(bot App)** et les filtres construits sur « moi » (*Assignés à moi*, *Les miennes*, *En attente de ma relecture*) n'ont rien à comparer. Rien n'est affiché comme une erreur.
- **Publier en tant que.** Chaque message que publie GitPoulpi est affiché avec le nom sous lequel il est publié (*Publication en tant que my-app[bot] (bot GitHub App)* sous l'en-tête d'une [conversation](conversations.md#publier-en-tant-que) et dans sa zone de commentaire ; `posted_as` dans la réponse du CLI). Avec un jeton personnel, c'est votre login (`GET /user`). Avec un jeton d'App, c'est le nom **Publier en tant que**, par exemple `my-app[bot]` : réglez-le pour tous les dépôts dans **Paramètres > Intégrations** (`general.hosting_post_as`, aussi `gitpoulpi config set general.hosting_post_as my-app[bot]`) ou pour un dépôt dans **Paramètres > Propre au dépôt : SSH et identifiants > Hébergement**, qui l'emporte. Laissé vide, GitPoulpi interroge GitHub (GraphQL `viewer { login }`) et sinon apprend le nom de l'auteur du premier commentaire qu'il publie, et le garde avec le dépôt. Ce nom sert aussi à reconnaître vos propres commentaires : modifier et supprimer ne sont donc proposés que sur eux (les commentaires d'une App doivent aussi venir d'un compte bot) ; tant qu'il n'est pas connu, seuls les commentaires que cette installation a publiés peuvent être changés.
- Une App ne peut pas être assignée. Combinez la configuration avec l'assignation bloquée ci-dessous.
- Activer ou désactiver **Utiliser pour l'API d'hébergement** demande une personne, comme une nouvelle commande.

Seuls les profils GitHub utilisent ainsi le jeton du dépôt ; les autres hébergeurs gardent le jeton propre au profil. (Un jeton d'accès de projet ou de groupe GitLab est aussi un bot : c'est le jeton du profil, affiché comme *bot de projet*, et rien n'est à régler.)

### Bloquer l'assignation dans un dépôt

**Paramètres > Propre au dépôt : SSH et identifiants** a une ligne **Assignation** (**Autorisée** par défaut, ou **Bloquée** ; enregistrée aussitôt). Bloquée, l'assignation est impossible dans les tickets et les pull requests de ce dépôt, quel que soit celui qui la demande :

- le sélecteur d'assignés est retiré des formulaires de nouveau ticket et de nouvelle pull request (avec une note qui dit pourquoi), la palette *Hébergeur : Modifier les assignés...* est grisée avec la raison, et le menu de la ligne le dit au lieu de l'ouvrir ;
- chaque client refuse une assignation avant que rien ne soit envoyé, y compris les assignés passés à la création d'un ticket ou d'une pull request : le CLI répond `{"refused": {"reason": "assignment_blocked", ...}}` avec le code de sortie 2, et un agent par MCP reçoit le même refus ;
- les assignés qui existent déjà restent visibles (en lecture seule).

`gitpoulpi hosting assignment` affiche le réglage, `--block` l'active et `--allow` le désactive de nouveau ; seule une personne peut autoriser de nouveau (le serveur MCP ne le peut pas).

## 9. En ligne de commande

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

Voir la [référence CLI](cli-reference.md#commandes) pour chaque commande. Un jeton
n'est jamais accepté en argument, et le serveur MCP ne propose pas
`hosting login`, si bien qu'un jeton ne passe jamais par un agent. `hosting login`
affiche `stored: "os"` quand le gestionnaire d'identifiants du système garde le jeton, ou
`stored: "session"` avec `store_error` quand le gestionnaire l'a refusé et que le jeton
ne vit qu'en mémoire pour cette session.

## Dépannage

| Symptôme | Remède |
|---|---|
| *Se connecter* échoue avec une erreur d'authentification | Le jeton est faux, expiré ou révoqué. Créez-en un nouveau. |
| Une portée est listée en rouge | Le jeton ne l'a pas. La connexion fonctionne, mais cette fonction non. Ajoutez la portée chez l'hébergeur, puis **Modifier** et collez le nouveau jeton. |
| Le panneau latéral n'a pas de section PULL REQUESTS | Aucun profil activé n'est connecté. Connectez-en un, puis liez le dépôt dans la liste déroulante de la section. |
| Une ligne *Reconnect* dans une liste | Le jeton a été perdu (un redémarrage sans assistant d'identifiants) ou il a expiré. Cliquez sur la ligne pour ouvrir le profil. |
| Une instance auto-hébergée échoue | Vérifiez que le **Domaine de l'hôte** n'a pas de chemin, et que votre ordinateur fait confiance au certificat de l'instance. |

## Limites connues

- **Pull requests depuis des forks :** Azure DevOps et Bitbucket ne peuvent pas
  extraire une demande venant d'un fork (voir [Actions sur une ligne](pull-requests-and-issues.md#actions-sur-une-ligne)).
  Azure DevOps ne peut pas créer une pull request depuis un fork.
- **Trello** montre au plus les 1000 premières cartes d'un tableau. Utilisez le
  filtre de listes pour restreindre. **Jira**, **Linear** et **YouTrack** listent au plus 500
  tickets par actualisation. Utilisez un fragment JQL ou une requête YouTrack pour restreindre.
- **Linear** prend un seul assigné par ticket : si vous en choisissez plusieurs dans le sélecteur,
  le premier est envoyé. Son « jalon » est le **cycle** de l'équipe (les jalons de projet
  ne sont pas utilisés). Celui de **YouTrack** est le champ `Fix versions` ; une
  version n'a pas de numéro, donc le sélecteur numérote les non publiées selon leur
  position dans le projet. Aucun des deux n'a de pull requests.
- **Redmine** prend un seul assigné par ticket (le premier du sélecteur), n'a pas d'étiquettes,
  et liste au plus 500 tickets par actualisation. Les assignés sont les membres du projet
  (personnes et groupes) et se distinguent par leur nom affiché. Les colonnes du tableau Kanban
  sont les statuts des tickets, et déplacer une carte change le statut ; les règles de workflow
  par défaut de Redmine s'appliquent (un rôle qui n'a pas le droit de cette transition reçoit le
  refus et la carte revient). « Resolved » n'est pas un statut fermé dans les données par défaut
  de Redmine : la colonne Fermés est celle des statuts « Closed » et « Rejected ».
- **Bitbucket Data Center** n'a pas de liens navigateur pour les nouvelles pull
  requests, branches ou commits. Les pages de pull requests s'ouvrent normalement.
- **Azure DevOps** n'affiche pas d'avatars d'auteurs. Les filtres « miennes » et
  « relecture demandée » demandent un point d'accès qui peut refuser les jetons
  d'accès personnels. Dans ce cas le profil se connecte sans nom d'utilisateur et
  ces deux filtres ne sont pas disponibles. Un jeton invalide reçoit une page de
  connexion au lieu d'une erreur ; GitPoulpi le signale comme un jeton rejeté.
- **Gitea et Forgejo :** un brouillon est une demande dont le titre commence par `WIP:`
  (le serveur n'a pas d'indicateur de brouillon à la création). Une demande liste comme relecteurs
  tous ceux qui l'ont déjà relue, et retirer une demande de revue ne retire que les
  demandes en attente. Les pull requests de la branche locale d'un fork s'extraient
  par `refs/pull/<n>/head`, comme GitHub. Les mentions sont cherchées avec le filtre
  `mentioned_by` du serveur. Le serveur n'affiche pas les portées du jeton, donc une
  portée manquante ne se découvre que quand un appel est refusé.
- **Le tableau des tickets** déplace les cartes sur toutes les sources de tickets. Sur Jira, un déplacement exécute la transition de workflow
  qui mène à la colonne ; une transition qui demande des champs (une résolution, par exemple) est refusée par Jira et la carte revient
  avec son message. Sur Azure DevOps, un work item dont le type n'a pas l'état cible revient avec la raison.
- **Bitbucket :** les mots de passe d'application n'existent plus ; utilisez un
  jeton d'API. Les dépôts sont listés espace de travail par espace de travail.
- **Jira Cloud :** seuls les jetons d'API classiques fonctionnent (pas de jetons à
  portées, pas d'OAuth). Un nouveau ticket Jira reçoit une description en texte
  brut.
- **Conversations :** les 300 entrées les plus récentes se chargent d'abord (**Charger les plus anciens** pour le reste) ; un commentaire, une modification ou une suppression publiés depuis GitPoulpi est une action sortante et ne peut pas être annulé depuis lui ; ce que chaque hébergeur ne peut pas faire est dans [le tableau](conversations.md#ce-que-chaque-hébergeur-ne-peut-pas-faire). Sur Gerrit, un message ou un commentaire ne peut être ni modifié ni supprimé, et le texte de Gerrit est brut.
- Créer une pull request, un work item, un ticket ou une carte se fait chez
  l'hébergeur et **ne peut pas être annulé** depuis GitPoulpi. Extraire une demande
  peut être annulé normalement.
