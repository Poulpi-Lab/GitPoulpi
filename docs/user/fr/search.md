[English](../search.md) · [Čeština](../cs/search.md) · [Deutsch](../de/search.md) · [Español](../es/search.md) · **Français** · [Italiano](../it/search.md) · [日本語](../ja/search.md) · [한국어](../ko/search.md) · [Nederlands](../nl/search.md) · [Polski](../pl/search.md) · [Português (Brasil)](../pt-BR/search.md) · [Português (Portugal)](../pt-PT/search.md) · [Русский](../ru/search.md) · [Svenska](../sv/search.md) · [Türkçe](../tr/search.md) · [Українська](../uk/search.md) · [简体中文](../zh-CN/search.md)

# Recherche

`Ctrl+Alt+F` (ou `Ctrl+P`, puis `Recherche : Espace de travail...`) ouvre un
chercheur approximatif sur **tous les dépôts dans lesquels vous travaillez** : les
dépôts de l'[espace de travail](workspaces.md)
actif s'il y en a un, sinon les onglets ouverts. Le bouton **Recherche globale** (la loupe et son libellé dans
la barre de titre, juste à gauche de l'engrenage) l'ouvre aussi. Il trouve :

- des **dépôts** (par nom, puis par chemin),
- des **branches** (locales et de suivi distant) et des **tags**,
- des **worktrees** (par dossier, puis par chemin et branche),
- des **stashes** (par message),
- des **commits** (par sujet ; le corps d'un commit n'est cherché que lorsque son
  sujet ne correspond pas et que vous avez tapé trois caractères ou plus),
- des **pull ou merge requests** et des **tickets** (par titre, `#numéro`, auteur
  et branche).

![La fenêtre de recherche sur deux dépôts ouverts : des branches, un stash et un commit correspondant à « main », chacun avec son dépôt à droite](../images/workspace-search.png)

Les résultats sont groupés par type, le meilleur en premier dans chaque groupe.
Chaque ligne a l'icône du type, les caractères correspondants en surbrillance, un
détail atténué (l'upstream d'une branche, l'auteur et la branche d'une demande, la
position d'un stash) et le nom du dépôt à droite, si bien que deux dépôts ayant une
branche `main` ne sont jamais confondus. Un indicateur d'activité apparaît dans le
titre quand un dépôt indexe encore ses commits (la première recherche dans un gros
dépôt peut prendre un moment) ; les autres types sont là tout de suite et les
commits arrivent dès qu'ils sont prêts.

## Y aller

| Touche | Effet |
|---|---|
| `Up` / `Down`, `PageUp` / `PageDown`, `Home` / `End` | Parcourir les résultats (les titres de groupe sont sautés). |
| `Enter` | Va au résultat : l'onglet du dépôt s'ouvre (ou est activé), puis le résultat est affiché. |
| `Esc` | Ferme la recherche. |
| `Alt+1` .. `Alt+8` | Active ou désactive un type (les pastilles au-dessus de la liste, dans leur ordre). `Alt+0` les active tous. |

Ce que veut dire « y aller » :

| Résultat | Enter |
|---|---|
| Branche ou tag | La sélectionne dans le volet Refs et son commit dans le Log. |
| Commit | Le sélectionne dans le Log, en chargeant d'abord plus de Log quand il est plus ancien que ce qui est chargé. |
| Stash | Sélectionne sa ligne dans la section STASHES. |
| Worktree | L'ouvre comme onglet. |
| Pull/merge request, ticket | Sélectionne sa ligne dans la barre latérale (dans l'onglet de ce dépôt). |
| Dépôt | Bascule sur son onglet (en l'ouvrant s'il ne l'est pas). |

`Ctrl+N Backspace` (ou `Retour` dans la palette) ramène là où vous étiez, comme
tout changement de vue fait pour vous par un agent. Rien n'est extrait, mergé ni
modifié en allant quelque part.

À la souris : un clic sur une ligne la sélectionne, un double-clic y va, un clic
sur une pastille active ou désactive ce type, la molette fait défiler, et un clic
à l'extérieur ferme la fenêtre.

## Restreindre avec un préfixe

Commencez la requête par un préfixe pour ne chercher qu'un type, quoi qu'indiquent
les pastilles :

| Préfixe | Type |
|---|---|
| `r:` | dépôts |
| `b:` | branches |
| `t:` | tags |
| `w:` | worktrees |
| `s:` | stashes |
| `c:` | commits |
| `pr:` (ou `mr:`, `p:`) | pull ou merge requests |
| `i:` | tickets |
| `#` | pull/merge requests et tickets (`#12` trouve le numéro 12) |

`fix: login` est une requête ordinaire, pas un préfixe : seule une lettre isolée
(ou `pr:` / `mr:`) suivie de deux-points, ou un `#` initial, compte.

## Comment ça correspond

Les mêmes trois niveaux que la palette de commandes : votre texte tel quel, puis tel
quel n'importe où, puis ses caractères dans l'ordre mais séparés. Une
correspondance plus tôt et un texte plus court gagnent. Majuscules et minuscules
sont équivalentes.

## Pull requests et tickets

Ils viennent de ce que GitPoulpi a déjà chargé pour chaque dépôt (les listes
derrière les sections PULL REQUESTS et ISSUES de la barre latérale), gardé dans un
petit fichier de cache dans le dossier `.gitpoulpi/` du dépôt. La recherche
**n'appelle jamais GitHub ni GitLab** : un dépôt dont les demandes n'ont jamais été
chargées dans une fenêtre n'en a aucune à trouver, et ce que vous voyez est aussi
frais que la dernière actualisation de la barre latérale. Sans compte lié, il n'y a
rien à trouver, et rien à configurer.

## En ligne de commande

`gitpoulpi search <requête> [--workspace ID] [--kind K]... [--limit N]` fait la même
chose et affiche du JSON (voir la
[Référence CLI](workspaces.md)) ; le serveur MCP
a un outil `search`, si bien qu'un agent trouve ce que vous trouvez.
`search.commit_limit` dans la
[Référence de configuration](config-reference.md#search-la-recherche-dans-lespace-de-travail)
borne le nombre de commits indexés par dépôt (`0` désactive la recherche de commits).

## Changer la touche

`Recherche : Espace de travail...` est une commande de la palette et la touche est
l'action `global.workspace_search` : réaffectez-la dans Paramètres > Clavier et
souris, ou dans la table `[keybindings.global]` (`workspace_search = "Ctrl+Alt+F"`).
