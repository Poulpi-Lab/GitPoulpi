[English](../workspaces.md) · [Čeština](../cs/workspaces.md) · [Deutsch](../de/workspaces.md) · [Español](../es/workspaces.md) · **Français** · [Italiano](../it/workspaces.md) · [日本語](../ja/workspaces.md) · [한국어](../ko/workspaces.md) · [Nederlands](../nl/workspaces.md) · [Polski](../pl/workspaces.md) · [Português (Brasil)](../pt-BR/workspaces.md) · [Português (Portugal)](../pt-PT/workspaces.md) · [Русский](../ru/workspaces.md) · [Svenska](../sv/workspaces.md) · [Türkçe](../tr/workspaces.md) · [Українська](../uk/workspaces.md) · [简体中文](../zh-CN/workspaces.md)

# Espaces de travail

Un **espace de travail** est un groupe nommé de dépôts : ceux sur lesquels vous
travaillez ensemble (un front, son API, la doc). Il retient lesquels sont
ouverts en onglets, et peut porter un **profil de commit par défaut**, pour que
les commits de ces dépôts utilisent la bonne identité sans que vous changiez à
la main. Un dépôt peut être dans plusieurs espaces de travail, et n'en avoir
aucun est tout à fait normal : « Aucun espace de travail » est la liste d'onglets
ordinaire d'avant.

Les espaces de travail sont stockés dans `workspaces.toml`, à côté de
`config.toml`, partagés par toutes les sessions, la fenêtre, la CLI et MCP
(`gitpoulpi workspace ...`, voir la [Référence CLI](cli-reference.md)). Un espace de travail ajouté, renommé ou supprimé depuis un shell ou par `gitpoulpi workspace ...` pendant que la fenêtre est
ouverte apparaît tout seul en une seconde environ (l'onglet Espaces de travail, la pastille de la barre d'onglets et sa liste), sans
aucune touche. Fermer ou
supprimer un espace de travail ne touche jamais un dépôt sur le disque. Les
modifications d'espaces de travail sont des paramètres, comme les favoris et les
profils : elles ne font pas partie d'[Annuler et rétablir](undo-redo.md), et
Supprimer demande d'abord.

## La pastille d'espace de travail

Dès que vous avez un espace de travail, une pastille apparaît dans la barre
d'onglets juste après le `⌂` : un point coloré et le nom de l'espace de travail
actif, ou « Aucun espace de travail » (atténué). Sur une barre étroite elle se
réduit au point, et sur une barre très étroite elle disparaît.

![La liste déroulante des espaces de travail au-dessus de la barre d'onglets : Aucun espace de travail, Orbit (coché), Side projects, Archive et Gérer les espaces de travail, avec la pastille Orbit et son profil « Work - de l'espace Orbit » dans la barre d'outils](../images/workspace-switch.png)

- **Cliquez dessus**, appuyez sur `Alt+W`, ou lancez
  `Espace de travail : Changer...` depuis la palette pour ouvrir une liste
  déroulante : vos espaces de travail, « Aucun espace de travail » et « Gérer les
  espaces de travail... » (qui ouvre l'onglet Espaces de travail).
- **Changer d'espace de travail change les onglets.** Les onglets que vous aviez
  ouverts sont gardés pour l'espace de travail que vous quittez, et ceux de la
  cible reviennent (la première fois, ses dépôts qui existent). Revenez et tout
  est là où vous l'aviez laissé. La fenêtre redémarre comme pour un changement
  d'onglet ; le démon reste.
- **L'espace de travail est mémorisé** : le prochain lancement y revient.
- **Fermer le dernier onglet** d'un espace de travail le garde actif et affiche
  l'onglet Accueil.
- **Le `+` (nouvel onglet) quand un espace de travail est actif** ajoute le dépôt
  choisi à la liste d'onglets sans rien demander. Pour le mettre dans l'espace de
  travail lui-même, utilisez « Ajouter à l'espace de travail » dans le menu du
  clic droit de l'onglet (ou `Espace de travail : Ajouter l'onglet à l'espace de
  travail`) ; « Retirer de l'espace de travail » l'en enlève et laisse l'onglet
  ouvert.

Sans espace de travail, il n'y a pas de pastille : créez-en un d'abord depuis
l'onglet Espaces de travail ou la palette.

## L'onglet Espaces de travail

Ouvrez-le depuis le menu du `+`, avec `Alt+J` ou avec `Espace de travail : Gérer...` dans la palette ; son `x` (ou `Ctrl+W`) le ferme, et il reste ouvert d'un lancement à l'autre jusqu'à ce que vous le fermiez.
Chaque espace de travail est une ligne avec son point coloré, son nom, le nombre
de dépôts qu'il liste (et combien sont absents), son profil par défaut, et
« actif » quand la fenêtre est dedans. `Right`, `Space` ou un clic le déplie
jusqu'à ses dépôts.

![L'onglet Espaces de travail : Orbit déplié jusqu'à ses deux dépôts, Side projects, et Archive marqué « 1 absent(s) »](../images/workspaces.png)

| Par espace de travail | Touche | |
|---|---|---|
| Ouvrir | `Enter` | Le rend actif et ouvre ses onglets. |
| Modifier... | `e` | Le formulaire ci-dessous. |
| Supprimer... | `x` | Demande d'abord. Les dépôts ne sont pas touchés. |
| Fetch tout | `Shift+F` | Un dépôt après l'autre, progression dans la ligne d'état, échecs listés à la fin. |
| Pull tout | `Shift+P` | Pareil, en avance rapide seulement : il ne s'arrête jamais sur un merge. |
| Cloner les absents | `m` | Clone chaque dépôt absent dont un distant a été enregistré. |
| Nouvel espace de travail... | `n` | |
| Depuis les onglets... | `a` | Un nouvel espace de travail avec vos onglets ouverts cochés. |

Un dépôt **absent** (son dossier a disparu) reste listé, marqué, avec
**Localiser...** (`l` ou `Enter` : indiquez où il se trouve maintenant),
**Cloner** (quand son URL `origin` a été enregistrée à son ajout) et **Retirer**
(`x`). Chaque ligne a aussi un menu `⋮` (clic droit, `.` ou `Shift+F10`) avec les
mêmes actions, et chaque action est dans la palette sous `Espace de travail :`.

## Le formulaire Nouveau / Modifier

![Le formulaire Modifier l'espace de travail : nom, couleur, description, profil par défaut et la liste à cocher des dépôts](../images/workspace-edit.png)

- **Nom** : obligatoire, unique.
- **Couleur** : une liste déroulante de couleurs du thème ; le point de la
  pastille et des lignes.
- **Description** : texte libre.
- **Profil** : une liste déroulante de vos `[[profiles]]`, ou « Aucun ». Il est
  facultatif.
- **✦ Instructions IA** : seulement avec l'IA activée, voir [ci-dessous](#instructions-ia-dun-espace-de-travail).
- **Dépôts** : une liste à cocher de tous les dépôts que GitPoulpi connaît
  (onglets ouverts, favoris, récents, ceux de l'espace de travail, absents
  compris). `Space` coche une ligne ; **Ajouter un dossier...** (`Ctrl+B`) choisit
  un dépôt sur le disque. L'URL d'origine de chaque nouveau dépôt est lue depuis
  le dépôt et enregistrée (sans aucun identifiant), ce qu'utilise ensuite Cloner.

## Instructions IA d'un espace de travail

Avec l'IA activée (*Paramètres > IA*), le formulaire a une ligne **✦ Instructions IA** avec un bouton **Modifier...** (`Enter` sur
la ligne). Il ouvre un petit éditeur de ce qui est dit à l'IA dans **chaque dépôt de l'espace de travail** : choisissez *Toutes les actions IA* ou une
seule action (message de commit, explication, message de stash, pull request, compositeur de commits, résolution de conflits,
revue de code) dans la liste **Pour**, choisissez **Combiner** - *Ajouter aux globales* (envoyées après les instructions globales) ou
*Remplacer les globales* (envoyées à leur place) - et écrivez le texte. **Appliquer** garde l'ensemble, **Enregistrer** sur le formulaire
l'écrit dans `workspaces.toml` avec l'espace de travail ; **Annuler** abandonne ce que vous avez saisi. Avec l'IA désactivée, la ligne est
masquée et les instructions de l'espace de travail sont laissées telles quelles.

Les mots de l'espace de travail se placent entre les instructions globales et celles du dépôt (voir
[Instructions IA](ai.md#instructions) pour l'ordre et pour ce que fait *remplacer*). Un dépôt qui appartient à
plusieurs espaces de travail utilise l'espace de travail actif de la fenêtre quand il le contient, sinon le premier qui le contient. Depuis un
script : `gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode append|replace] (--text ... |
--clear)` ; `workspace show` les affiche sous `ai_instructions`.

```toml
[[workspace]]
id = "acme"
name = "Acme"
repos = ["/home/u/src/acme-api"]

[workspace.ai_instructions]       # every AI action in this workspace
text = "Use the Acme glossary."
mode = "add"                       # or "replace"

[workspace.ai_overrides.code_review]   # one action
text = "Flag missing tests."
mode = "add"
```

## Le profil par défaut

Les commits sont signés par le premier de ces profils qui s'applique :

1. le profil que vous avez choisi **à la main** pour ce dépôt (le champ de profil
   de la barre d'outils),
2. le profil de **l'espace de travail actif**, quand le dépôt en est membre,
3. `general.active_profile`.

Le champ de profil de la barre d'outils dit d'où il vient : « Work - de l'espace
Acme ». Le champ s'affiche aussi quand vous n'avez qu'un seul profil, tant que le
profil par défaut d'un espace de travail est en vigueur. Un espace de travail
dont le profil n'existe plus cesse simplement de s'appliquer. En CLI,
`commit --workspace <id>` (ou `GITPOULPI_WORKSPACE`) résout de la même façon.

## Touches

| Touche | Action |
|---|---|
| `Alt+W` | Ouvrir la liste déroulante des espaces de travail |
| `Alt+J` | Ouvrir l'onglet Espaces de travail |
| `n`, `a` | Nouvel espace de travail, nouveau depuis les onglets ouverts |
| `e` | Modifier l'espace de travail |
| `Shift+F`, `Shift+P`, `m`, `l` | Fetch tout, Pull tout, Cloner les absents, Localiser |

Toutes peuvent être réaffectées dans Paramètres > Clavier et souris ; la liste
complète est dans [Raccourcis clavier](keybindings.md).
