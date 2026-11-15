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

## 5. Pull requests et tickets dans le panneau latéral

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

### Actions sur une ligne

- `Enter` ou un double-clic ouvre la ligne : une pull request dans la [revue](review.md) (GitHub, GitLab, Gitea, Forgejo, Gerrit), un ticket, et une demande sur Azure DevOps ou Bitbucket, comme [conversation](#5c-conversations) dans la fenêtre, et tout le reste dans votre navigateur. **Ouvrir dans le navigateur** du menu de la ligne ouvre toujours le navigateur.
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
    [Relire une pull request](review.md).
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

## 5b. Le tableau des tickets

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
le ticket chez l'hébergeur (la note, pour les tickets locaux) ; `e` modifie assignés, étiquettes et jalon ; `n` commence un nouveau ticket dans
la colonne sélectionnée ; `a`, `l`, `M` et `/` filtrent par assigné, étiquette, jalon et texte, `x` efface les filtres ; `r`
relit l'hébergeur. Le tableau se relit quand les listes de la barre latérale s'actualisent. Le Radar n'a pas de tableau.

## 5c. Conversations

Une **conversation** est tout ce qui se dit sur un ticket, un work item, une carte, une modification ou une pull / merge request : la description, les commentaires, les revues, les fils sur le code et les événements (étiquettes, assignations, changements d'état, pushes). GitPoulpi la lit chez l'hébergeur, l'affiche mise en forme et vous laisse répondre sans quitter la fenêtre.

### L'ouvrir

| Depuis | Comment |
|---|---|
| Une ligne de TICKETS | `Enter` ou un double-clic ; **Ouvrir la conversation** du menu de la ligne ; la palette *Hébergeur : Ouvrir la conversation d'un ticket...* (une liste des tickets à l'écran). **Ouvrir dans le navigateur** du menu de la ligne garde le navigateur. |
| Une ligne de PULL REQUESTS sur Azure DevOps ou Bitbucket | `Enter` ou **Ouvrir la conversation**. Sur GitHub, GitLab, Gitea, Forgejo et Gerrit, la ligne ouvre la [revue](review.md), dont la page Vue d'ensemble est la conversation de la demande. |
| Une carte du [tableau des tickets](#5b-le-tableau-des-tickets) | `Enter` ou un double-clic. |
| Le [Radar](radar.md) | `i` sur une ligne de ticket ou de demande, **Ouvrir la conversation** du menu de la ligne, ou *Radar : Ouvrir la conversation du ticket*. |
| Une référence dans une autre conversation | Cliquez dessus (ou `l`, puis choisissez-la) : `#7`, `!12`, `ABC-12` d'une source liée s'ouvrent dans l'application et s'empilent sur une pile Retour. |
| Un agent | `gitpoulpi ui show issue <n> --view conversation` (aussi `ui show pr`), voir [la référence du CLI](cli-reference.md). |

**Dépôts non clonés.** Une ligne du Radar d'un dépôt dont vous n'avez pas de clone ouvre aussi sa conversation : la vue ne demande que le profil d'intégration et le projet, si bien que le Radar l'héberge dans sa propre fenêtre, et `Esc` rend le Radar. Ce qui demande un clone est écarté : une référence de commit s'ouvre chez l'hébergeur au lieu du Log, et *Laisser l'IA lire le dépôt* est grisé avec sa raison (voir [le chapitre IA](ai.md#aide-à-la-rédaction-sur-un-hébergeur)). Une demande d'un dépôt cloné a sa revue à la place (`e`), et **Ouvrir la conversation** du Radar dit qu'il n'y a pas de conversation à lire ici pour elle ; un ticket d'un dépôt cloné l'ouvre dans l'onglet de ce dépôt.

Un ticket d'un hébergeur qui n'en a aucune à lire (Bitbucket et Gerrit n'ont pas de tickets) ouvre le navigateur comme avant, et une version compilée sans la fonctionnalité `conversation` fait de même partout ([versions personnalisées](custom-builds.md)).

### La vue

Une vue plein écran : l'en-tête (état, qui l'a ouvert et quand, les étiquettes, le nombre de commentaires, **Afficher les événements**, **Retour**, **Actualiser**, **Ouvrir dans le navigateur**), la ligne **Publication en tant que**, la chronologie, et dessous la zone de commentaire.

- La chronologie se lit du plus ancien au plus récent et commence sur la fenêtre la plus récente (300 entrées au plus). **Charger les plus anciens** en haut (`m`, palette *Conversation : Charger les entrées plus anciennes*) lit la fenêtre précédente, avec le nombre restant quand l'hébergeur le dit.
- Un commentaire montre son auteur, sa date, une pastille `modifié`, une pastille `bot` pour les bots, `interne` pour les notes privées, et les boutons **Répondre**, **Citer**, **Modifier**, **Supprimer** et **Liens** que permet l'entrée sélectionnée. Un fil sur du code montre `chemin:ligne`, `résolu` ou `non résolu`, `obsolète`, et ses commentaires. Une revue montre son verdict.
- Les événements tiennent chacun sur une ligne atténuée (*carol a ajouté l'étiquette lfs*). **Afficher les événements** (`v`) les masque ; le choix est gardé par dépôt. Les hébergeurs sans historique d'événements (YouTrack, work items Azure DevOps, notes locales) n'ont pas cette case.
- Le texte est rendu en Markdown : titres, listes imbriquées et de tâches, citations, alertes à la manière de GitHub, code coloré, tableaux, liens, images en `[image: alt]`. Le texte arrivé dans le balisage propre à l'hébergeur est d'abord converti (voir [Conversion du balisage](#conversion-du-balisage)). Un bloc de code `mermaid` reçoit un bouton **Ouvrir le diagramme** qui montre la source du diagramme dans une visionneuse défilante (`Enter` la copie) ; la palette a *Conversation : Ouvrir le diagramme de l'entrée*. Il faut une version avec à la fois `conversation` et `mermaid`.
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
| *Publication en tant que ci-bot (commande de jeton du dépôt)* | La [commande de jeton](#13d-agir-comme-bot-github-app) du dépôt sert pour l'API d'hébergement. |
| *Publication en tant que my-app[bot] (bot GitHub App)* | Un jeton d'installation. Le nom vient de **Publier en tant que**, sinon de l'hébergeur. |
| *Publication en tant que bot GitHub App (son nom n'est pas encore connu : réglez Publier en tant que dans les paramètres)* | Aucun nom configuré, aucun appris pour l'instant. |
| *Publication en tant que ... (bot de projet)* | Un jeton d'accès de projet ou de groupe GitLab. |
| *Écriture en tant que kai (note locale)* | Un dossier de notes Markdown : le `local.me` du profil, sinon l'utilisateur connecté. |
| *Le compte de publication n'est pas encore connu. Réglez Publier en tant que dans les paramètres.* | Rien n'a pu être appris. |

**Publier en tant que** est un nom : pour une GitHub App, c'est `my-app[bot]`. Réglez-le pour tous les dépôts dans **Paramètres > Intégrations > Publier en tant que** (`general.hosting_post_as`), ou pour un seul dépôt dans **Paramètres > Propre au dépôt : SSH et identifiants > Hébergement**, qui l'emporte. Laissé vide, GitPoulpi interroge GitHub (GraphQL `viewer { login }`, qu'un jeton d'installation peut ou non satisfaire) et, à défaut, apprend le nom de l'auteur du premier commentaire qu'il publie et le garde avec le dépôt. Un jeton personnel n'en a jamais besoin. Voir [Agir comme bot GitHub App](#13d-agir-comme-bot-github-app).

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

## 6. Créer une pull request

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

## 7. Créer un ticket

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

## 8. Créer sur la page web de l'hébergeur

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

## 9. La pastille de la barre d'outils

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

## 10. Quand les listes se rafraîchissent

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

## 11. Terminologie

GitHub, Azure DevOps et Bitbucket disent « pull request ». GitLab dit « merge
request ». *Terme pour les demandes de revue* (`ui.pr_terminology`, dans Paramètres >
Intégrations) choisit le mot pour chaque libellé, menu, commande de palette et
message :

- **Auto (selon l'hébergeur)** : suit le fournisseur lié du dépôt (« Pull request »
  tant que rien n'est lié).
- **Pull request** ou **Merge request** : toujours ce mot.

Le changement s'applique aussitôt. La CLI et MCP disent toujours « pull/merge
request ». Cette page dit « pull request ».

## 12. Références dans les messages de commit et les noms de branche

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

**Dans les conversations.** Le texte d'un ticket ou d'une demande est lu avec la grammaire de son hébergeur, plus large que le tableau ci-dessus : `owner/repo#7`, `!7`, `GH-7`, `@login`, les identifiants de commit et les autres formes listées dans [Références croisées](#références-croisées). Les sources liées du dépôt sont comparées d'abord : une clé Jira dans un commentaire GitHub ouvre donc encore son ticket dans l'application. `gitpoulpi hosting markdown --body TEXT --id I` montre ce que l'application trouve dans un texte.

### Insérer une référence de ticket

Dans le compositeur de commits, le bouton `#▾` à l'extrémité droite du Résumé ouvre
une liste filtrable des tickets liés. Le ticket nommé par la branche courante vient
en premier, marqué *(de la branche)*. `Enter` ou un clic insère sa référence
(`ABC-123`, `#12`, `AB#12`) au niveau du curseur. Clavier : `Alt+R` dans le
compositeur. Palette : *Commit : Insérer une référence de ticket...*. Sans liste de
tickets liée, le bouton est grisé et dit *liez d'abord une liste de tickets dans la barre latérale*.
GitPoulpi n'ajoute jamais une référence tout seul.

## 13. Clés SSH

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

## 13b. Protection des branches, pour une réécriture d'historique

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

## 13c. Listes à l'échelle du compte dans le Radar

Le Radar surveille normalement les dépôts que vous avez clonés. Un profil peut aussi
lister ce qui vous concerne sur tout l'hébergeur, cloné ou non, avec trois interrupteurs dans
sa table `[integrations.radar]` (tous désactivés par défaut ; voir la
[référence de configuration](config-reference.md#integrationsradar-listes-à-léchelle-du-compte)) :

- **demandes de relecture** : les demandes ouvertes qui demandent votre relecture ;
- **demandes ouvertes** : toutes les demandes ouvertes de votre propre espace de noms et de chaque organisation ou
  groupe dont vous êtes membre, ou des espaces de noms que vous nommez. Jamais tout l'hébergeur ;
  les **Espaces de noms** du profil (un par ligne dans Paramètres > Intégrations > Radar, éventuellement
  en expressions régulières) limitent les trois listes : une ligne correspond au chemin complet d'un dépôt
  quand elle lui est égale ou en est un préfixe qui s'arrête à un `/` (`ptm` garde `ptm/produits/x`, pas `ptmx/y`), et
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

## 13d. Agir comme bot GitHub App

Un projet qui ne doit montrer le nom d'aucune personne peut tout faire chez l'hébergeur par une **GitHub App** : chaque commentaire, revue, fermeture ou réouverture et changement d'étiquette apparaît alors comme `<app-slug>[bot]`. Cela marche pour un dépôt à la fois et laisse vos autres dépôts sur votre propre jeton.

1. Créez une GitHub App pour l'organisation avec les permissions **Issues** et **Pull requests** en *Read & write* (et *Contents : read*, pour que les pull requests puissent être listées et relues), et installez-la sur le dépôt.
2. Écrivez un petit script qui affiche un **jeton d'installation** sur sa sortie standard (la clé privée de l'App signe un JWT de courte durée, échangé contre un jeton valable une heure). GitPoulpi ne voit jamais la clé.
3. Dans **Paramètres > Propre au dépôt : SSH et identifiants**, choisissez **Commande de jeton** sous HTTPS, saisissez le script comme **Commande** et l'hôte (`github.com`) sous **Hôtes**, et cochez **Utiliser pour l'API d'hébergement (tickets, pull requests)**. Enregistrez et confirmez la commande (une personne la confirme toujours, avec son SHA-256). En ligne de commande : `gitpoulpi repo-auth set --https token-command --hosts github.com --use-for-hosting --yes --command ./app-token.sh`.

Ce qui change pour ce dépôt :

- Ses appels de tickets et de pull requests (liste, création, commentaire, revue, merge, fermeture, étiquettes) utilisent le jeton de la commande, avec la même limite de 30 secondes, le même cache en mémoire et la même rédaction que la connexion propre à git. Quand l'hébergeur répond 401 parce que le jeton a expiré, la commande est redemandée une fois. Si la commande ne peut pas s'exécuter, l'appel est refusé : votre jeton personnel n'est jamais le repli.
- Il n'y a pas de « moi » : un jeton d'installation ne peut pas demander qui il est. L'en-tête de section nomme le profil avec **(bot App)** et les filtres construits sur « moi » (*Assignés à moi*, *Les miennes*, *En attente de ma relecture*) n'ont rien à comparer. Rien n'est affiché comme une erreur.
- **Publier en tant que.** Chaque message que publie GitPoulpi est affiché avec le nom sous lequel il est publié (*Publication en tant que my-app[bot] (bot GitHub App)* sous l'en-tête d'une [conversation](#publier-en-tant-que) et dans sa zone de commentaire ; `posted_as` dans la réponse du CLI). Avec un jeton personnel, c'est votre login (`GET /user`). Avec un jeton d'App, c'est le nom **Publier en tant que**, par exemple `my-app[bot]` : réglez-le pour tous les dépôts dans **Paramètres > Intégrations** (`general.hosting_post_as`, aussi `gitpoulpi config set general.hosting_post_as my-app[bot]`) ou pour un dépôt dans **Paramètres > Propre au dépôt : SSH et identifiants > Hébergement**, qui l'emporte. Laissé vide, GitPoulpi interroge GitHub (GraphQL `viewer { login }`) et sinon apprend le nom de l'auteur du premier commentaire qu'il publie, et le garde avec le dépôt. Ce nom sert aussi à reconnaître vos propres commentaires : modifier et supprimer ne sont donc proposés que sur eux (les commentaires d'une App doivent aussi venir d'un compte bot) ; tant qu'il n'est pas connu, seuls les commentaires que cette installation a publiés peuvent être changés.
- Une App ne peut pas être assignée. Combinez la configuration avec l'assignation bloquée ci-dessous.
- Activer ou désactiver **Utiliser pour l'API d'hébergement** demande une personne, comme une nouvelle commande.

Seuls les profils GitHub utilisent ainsi le jeton du dépôt ; les autres hébergeurs gardent le jeton propre au profil. (Un jeton d'accès de projet ou de groupe GitLab est aussi un bot : c'est le jeton du profil, affiché comme *bot de projet*, et rien n'est à régler.)

### Bloquer l'assignation dans un dépôt

**Paramètres > Propre au dépôt : SSH et identifiants** a une ligne **Assignation** (**Autorisée** par défaut, ou **Bloquée** ; enregistrée aussitôt). Bloquée, l'assignation est impossible dans les tickets et les pull requests de ce dépôt, quel que soit celui qui la demande :

- le sélecteur d'assignés est retiré des formulaires de nouveau ticket et de nouvelle pull request (avec une note qui dit pourquoi), la palette *Hébergeur : Modifier les assignés...* est grisée avec la raison, et le menu de la ligne le dit au lieu de l'ouvrir ;
- chaque client refuse une assignation avant que rien ne soit envoyé, y compris les assignés passés à la création d'un ticket ou d'une pull request : le CLI répond `{"refused": {"reason": "assignment_blocked", ...}}` avec le code de sortie 2, et un agent par MCP reçoit le même refus ;
- les assignés qui existent déjà restent visibles (en lecture seule).

`gitpoulpi hosting assignment` affiche le réglage, `--block` l'active et `--allow` le désactive de nouveau ; seule une personne peut autoriser de nouveau (le serveur MCP ne le peut pas).

## 14. En ligne de commande

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
  extraire une demande venant d'un fork (voir [Actions sur une ligne](#actions-sur-une-ligne)).
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
- **Conversations :** les 300 entrées les plus récentes se chargent d'abord (**Charger les plus anciens** pour le reste) ; un commentaire, une modification ou une suppression publiés depuis GitPoulpi est une action sortante et ne peut pas être annulé depuis lui ; ce que chaque hébergeur ne peut pas faire est dans [le tableau](#ce-que-chaque-hébergeur-ne-peut-pas-faire). Sur Gerrit, un message ou un commentaire ne peut être ni modifié ni supprimé, et le texte de Gerrit est brut.
- Créer une pull request, un work item, un ticket ou une carte se fait chez
  l'hébergeur et **ne peut pas être annulé** depuis GitPoulpi. Extraire une demande
  peut être annulé normalement.
