[English](../local-issues.md) · [Čeština](../cs/local-issues.md) · [Deutsch](../de/local-issues.md) · [Español](../es/local-issues.md) · **Français** · [Italiano](../it/local-issues.md) · [日本語](../ja/local-issues.md) · [한국어](../ko/local-issues.md) · [Nederlands](../nl/local-issues.md) · [Polski](../pl/local-issues.md) · [Português (Brasil)](../pt-BR/local-issues.md) · [Português (Portugal)](../pt-PT/local-issues.md) · [Русский](../ru/local-issues.md) · [Svenska](../sv/local-issues.md) · [Türkçe](../tr/local-issues.md) · [Українська](../uk/local-issues.md) · [简体中文](../zh-CN/local-issues.md)

# Tickets locaux et Obsidian

Gardez vos tickets dans un dossier de notes Markdown : pas de compte, pas de jeton, pas de réseau. Le dossier peut être dans le
dépôt (`docs/issues/`, commité avec le code) ou n'importe où sur le disque, par exemple un sous-dossier d'un
coffre [Obsidian](https://obsidian.md). GitPoulpi lit et écrit les notes. Obsidian, n'importe quel éditeur ou un shell peut les modifier
en même temps, et les listes suivent.

![Les tickets Markdown locaux dans la barre latérale, avec le menu de ligne d'une note : ouvrir la note, la fermer, modifier ses étiquettes ou son jalon](../images/local-issues.png)

## Mise en place

1. **Paramètres > Intégrations > Suivi de tickets > Markdown local > Ajouter un profil.** Saisissez le dossier, ou appuyez sur
   **Parcourir...** et choisissez-le. Un dossier relatif (`docs/issues`) est lu par rapport à chaque dépôt auquel il est lié ; un
   dossier absolu (`/home/me/Vault/Projects/Issues`) est le même partout. Il n'y a pas de jeton à saisir.
   - CLI : `gitpoulpi hosting integrations add --kind local_markdown --path docs/issues`.
2. Dans un dépôt, ouvrez la section **TICKETS** de la barre latérale et choisissez le fournisseur **Markdown local - <nom>** (le dossier
   est choisi avec lui ; la liste des dépôts propose le dossier et ses sous-dossiers directs si vous en voulez un autre).
   - CLI : `gitpoulpi hosting link issues --id <profil>` (le dossier du profil est le `--project` par défaut).

Rien n'est lié tout seul : comme toute source de tickets, c'est à activer dépôt par dépôt.

## Les notes

Une note par ticket, nommée `<id>-<slug>.md` (`12-crash-on-empty-name.md`), avec un bloc de frontmatter :

```yaml
---
id: 12
title: Crash on empty name
status: todo
assignees: [kai]
labels: [bug]
milestone: v0.1
created: 2026-10-14
updated: 2026-10-14T09:30:00
---
Free Markdown. GitPoulpi never rewrites the body.
```

- Les nouveaux tickets reçoivent `max(id) + 1`.
- GitPoulpi n'écrit que les propriétés qui lui appartiennent (`title`, `status`, `assignees`, `labels`, `milestone`, `updated`).
  **Toute autre propriété, le corps, les fins de ligne et l'ordre des clés sont conservés octet pour octet.** Les écritures sont
  atomiques (un fichier temporaire, puis un renommage).
- La lecture est indulgente : une note sans `id` le prend dans son nom de fichier, une note sans `status` vaut `todo`, une note
  sans frontmatter prend son titre dans le premier `# titre`. Un fichier qui n'est pas un ticket (`readme.md`) est ignoré.
- Un statut est du texte libre. **Ouvert** veut dire « pas le statut terminé ».

## Ce que vous pouvez faire

| Dans la section TICKETS | Clavier / palette |
|---|---|
| Nouveau ticket (le `+`, ou `n`) : titre, description, statut, assignés, étiquettes, jalon | `Hébergeur : Nouveau ticket...` |
| Modifier les assignés / étiquettes / jalon depuis le menu de la ligne. Choisissez parmi ce que le dossier utilise déjà, ou **saisissez une nouvelle étiquette ou un nouvel identifiant** et prenez la ligne `Ajouter « ... »` | `Hébergeur : Modifier les étiquettes...` et ses semblables |
| **Fermer le ticket** / **Rouvrir le ticket** (met le statut terminé, ou le premier autre) | `Tickets locaux : Fermer un ticket...` / `Rouvrir un ticket...` |
| **Ouvrir la note** (votre programme par défaut pour `.md`) | `Intégrations : Ouvrir un ticket...` |
| **Ouvrir dans Obsidian** (seulement quand le dossier est dans un coffre, c'est-à-dire qu'un dossier `.obsidian/` se trouve au-dessus) | `Tickets locaux : Ouvrir dans Obsidian...` |
| Filtres : Ouverts / Fermés / Tous, « Qui me sont assignés » | la liste déroulante des filtres |

« Qui me sont assignés » correspond au paramètre `me` du profil, sinon au nom connecté, sinon au nom du profil.

Les branches et les commits marchent comme pour GitHub : « Créer une branche depuis le ticket » donne `12-crash-on-empty-name`, et `#12` dans un message
de commit ou `12-...` dans un nom de branche est reconnu.

Les changements faits hors de GitPoulpi (Obsidian, un shell, un `git pull`) apparaissent en une seconde environ : les noms de fichiers du dossier
et leurs dates de modification sont consultés dans un thread d'arrière-plan, jamais dans le thread d'affichage.

## Fichier de configuration

Dans `config.toml`, à côté du profil (chaque clé est facultative) :

```toml
[[integrations]]
id = "vault"
kind = "local_markdown"
url = "/home/me/Vault/Projects/Issues"
token_ref = "integration:vault"

[integrations.local]
statuses = ["todo", "doing", "review", "done"]   # the board's column order
done_status = "done"                            # closes an issue
reference_prefix = "LOC"                         # shown as LOC-12 instead of #12
me = "kai"                                       # who "assigned to me" means
```

Les statuts utilisés par des notes mais absents de `statuses` suivent ceux de la liste. `statuses` est l'ordre des colonnes du
[tableau](#le-tableau-kanban) (`gitpoulpi hosting issue options` l'affiche).

## Le tableau Kanban

![Le tableau d'un dossier de notes : une colonne par statut, les cartes avec leurs assignés, étiquettes et jalon](../images/issue-board.png)

Si vous travaillez seul, ou dans une équipe qui garde ses tickets à côté du code, le dossier est aussi un tableau Kanban.
**En-tête TICKETS > Tableau**, ou la ligne de palette **Tickets : Ouvrir le tableau**, l'ouvre par-dessus la fenêtre ; `Esc` le ferme.

- Les **colonnes** sont les statuts : ceux listés dans `statuses` d'abord, dans cet ordre, puis tout autre statut qu'utilise une note.
  Une colonne vide est quand même affichée. Chaque en-tête porte son nombre de cartes.
- Les **cartes** montrent le numéro du ticket, le titre (deux lignes), les assignés en initiales, les étiquettes et le jalon.
  Quand les colonnes ne tiennent pas toutes, le tableau défile latéralement (la barre dessous, la molette avec `Shift`, ou les flèches).
- **Déplacer une carte** en la glissant sur une autre colonne (la colonne s'éclaire sous le pointeur), avec
  `Shift+Left` / `Shift+Right`, ou avec **Tableau : Déplacer la carte vers...** (`m`), une liste déroulante des colonnes. La carte bouge
  immédiatement ; le `status` de la note est réécrit comme dans [la section sur les notes](#les-notes), rien d'autre dans le fichier. Si l'écriture
  échoue, la carte revient et l'erreur est sur la ligne d'état.
- **Ouvrez** une carte avec `Enter` ou un double-clic (la note s'ouvre dans votre programme par défaut pour `.md`). `e` modifie ses
  assignés, étiquettes ou jalon, `.` ou un clic droit ouvre le menu de la carte, `n` commence un nouveau ticket dans la
  colonne sélectionnée (le formulaire revient au tableau quand il se ferme).
- **Filtrez** par assigné (`a`), étiquette (`l`), jalon (`M`) ou texte (`/`) ; `x` les efface. Les filtres sont des
  listes déroulantes : tapez pour réduire, `Enter` choisit.
- **En direct** : une note modifiée depuis Obsidian ou un shell déplace sa carte en moins d'une seconde.

Chaque geste à la souris a une touche et une commande de palette (**Tableau : ...**, listées dans la page des
[raccourcis clavier](keybindings.md)).

## Bases Obsidian

**Palette : `Tickets locaux : Créer une Base Obsidian`** (CLI : `gitpoulpi hosting issue base`) écrit `issues.base` à côté des
notes, une seule fois : une vue *table* et une vue *kanban* groupée par `status`, dans l'ordre de `statuses`. Ouvrez-le dans Obsidian pour
voir les mêmes tickets comme un tableau là-bas. GitPoulpi ne modifie jamais un fichier `.base` existant (le vôtre ou un précédent) : il
refuse et le dit. Le format suit la syntaxe des Bases d'Obsidian
(<https://obsidian.md/help/bases/syntax>) ; la vue kanban demande un Obsidian récent.

## CLI et MCP

Les mêmes commandes que pour toute source de tickets y fonctionnent : `hosting issues`, `hosting issue create [--list <statut>]`,
`hosting issue edit <n> --add-label ...`, `hosting open issue <n>`, `hosting refs`. Quatre autres sont pour les notes :
`hosting issue close <n>`, `hosting issue reopen <n>`, `hosting issue move <n> --status <s>` et `hosting issue base`.
Voir la [référence CLI](cli-reference.md).

## Limites

- Un nouveau **jalon** ne peut pas être saisi dans le sélecteur (choisissez-en un que le dossier utilise déjà), mais `milestone:` dans une note peut
  être n'importe quoi et apparaît dès qu'il est enregistré.
- Seules les notes placées directement dans le dossier sont des tickets (pas les sous-dossiers).
- Il n'y a pas de suppression : retirez la note dans votre gestionnaire de fichiers ou dans Obsidian (sa corbeille est la vôtre).
