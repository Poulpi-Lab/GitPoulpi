[English](../getting-started.md) · [Čeština](../cs/getting-started.md) · [Deutsch](../de/getting-started.md) · [Español](../es/getting-started.md) · **Français** · [Italiano](../it/getting-started.md) · [日本語](../ja/getting-started.md) · [한국어](../ko/getting-started.md) · [Nederlands](../nl/getting-started.md) · [Polski](../pl/getting-started.md) · [Português (Brasil)](../pt-BR/getting-started.md) · [Português (Portugal)](../pt-PT/getting-started.md) · [Русский](../ru/getting-started.md) · [Svenska](../sv/getting-started.md) · [Türkçe](../tr/getting-started.md) · [Українська](../uk/getting-started.md) · [简体中文](../zh-CN/getting-started.md)

# Premiers pas

Cette page vous mène d'une installation toute neuve à votre premier commit.
La [visite de l'interface](interface-tour.md) nomme chaque zone de la fenêtre.
Les [Paramètres](settings.md) listent toutes les options.

## 1. Installer et lancer

Suivez [Installation](installation.md). Ouvrez ensuite un terminal dans un
dépôt Git et lancez :

```
gitpoulpi-tui
```

`gitpoulpi` sans sous-commande fait la même chose. GitPoulpi ouvre le dépôt qui
contient le dossier courant. En dehors d'un dépôt, il ouvre l'onglet
**Accueil**, où vous pouvez parcourir, cloner ou initialiser un dépôt (voir
l'[étape 7](#7-ouvrir-cloner-et-initialiser-des-dépôts)).

Les icônes demandent une Nerd Font ou une police avec emoji. Si vous voyez des
carrés, consultez le [Dépannage](troubleshooting.md).

![La fenêtre principale : le panneau latéral Refs à gauche, le graphe du Log au milieu, et les modifications locales avec le formulaire de commit à droite](../images/hero.png)

## 2. Trois touches pour apprendre tout le reste

| Touche | Ouvre |
|---|---|
| `?` | L'**aide** : toutes les touches et tous les gestes de la souris. Tapez pour la filtrer. `Ctrl+D` ouvre cette documentation dans votre navigateur. |
| `Ctrl+P` | La **palette de commandes** : tapez pour trouver une commande, `Enter` la lance. Chaque ligne affiche son raccourci. |
| `Ctrl+,` | Les **Paramètres**. L'engrenage de la barre de titre, à gauche de la ×, les ouvre aussi. |

Tout se fait au clavier comme à la souris. La palette atteint chaque commande.
Voir [Raccourcis clavier](keybindings.md) et
[Souris et gestes](mouse-and-gestures.md).

![La palette de commandes filtrée sur le mot stage, listant les commandes de stage avec leurs raccourcis](../images/palette.png)

## 3. Parcourir l'historique

Le milieu de la fenêtre est le **Log** : une ligne par commit, du plus récent
au plus ancien. Il montre les branches et les tags sous forme de pastilles
colorées, le graphe, le message, l'auteur, la date et le hash court.

- Cliquez sur un commit. Ses détails et ses fichiers modifiés apparaissent à
  droite.
- Cliquez sur un fichier modifié. Son diff s'ouvre au milieu. `Esc` le
  ferme.
- `j` / `k` ou les flèches déplacent la sélection. `f` cherche dans les
  commits.
- Faites un clic droit sur un commit pour son menu.

![Le Log avec un commit de merge sélectionné : son message, son auteur et les deux fichiers qu'il ajoute, à droite](../images/log-graph.png)

## 4. Modifier et indexer

Modifiez un fichier dans votre éditeur. La **ligne du haut du Log** montre
maintenant les modifications locales : une ligne en pointillés avec le nombre de
fichiers modifiés. Cliquez dessus. Le volet de droite affiche **Fichiers
unstaged** et **Fichiers staged**.

- Survolez un fichier et cliquez sur son bouton **Stage fichier**. Ou cliquez
  sur **Stage de tout**.
- Cliquez sur un nom de fichier pour ouvrir son diff. Vous pouvez y stager un
  **hunk** (**Stage du hunk**) ou une seule **ligne**.

Voir [Stage et commit](staging-and-committing.md).

![Le diff d'un fichier modifié, avec les boutons Stage du hunk et Abandonner le hunk au-dessus de chaque hunk](../images/diff.png)

## 5. Commiter

Saisissez un résumé, et au besoin une description, dans le formulaire de commit
en bas à droite. Cliquez ensuite sur le bouton vert **Commiter N fichiers** ou
appuyez sur `Ctrl+Enter` (ou `Ctrl+G Enter`, d'où que vous soyez). Cochez **Amender le commit précédent** pour ajouter
vos modifications au dernier commit.

Si vous avez activé l'[IA](ai.md), le bouton `✦` à côté du résumé rédige un
message à partir des modifications stagées. GitPoulpi ne commite jamais avant que
vous cliquiez sur **Commiter N fichiers**.

## 6. Partager votre travail

La barre d'outils a des boutons **Pull** et **Push**, chacun avec une liste
`▾`. Au clavier, appuyez sur `Ctrl+G` puis sur une lettre : `Ctrl+G p` fait un
pull, `Ctrl+G Shift+P` un push. Après `Ctrl+G`, la ligne d'état liste les
lettres.

GitPoulpi utilise l'authentification que votre `git` utilise déjà (assistant
d'identifiants, ssh-agent). Il n'a besoin d'aucun compte propre. Pour voir et
créer des pull requests, connectez une fois un compte d'hébergeur. Voir
[Hébergeurs et suivi de tickets](hosting-integration.md).

## 7. Ouvrir, cloner et initialiser des dépôts

Cliquez sur l'icône de maison à l'extrémité gauche de la barre d'onglets, ou
appuyez sur `Ctrl+O`. L'onglet **Accueil** liste vos dépôts ouverts, favoris et
récents, avec les boutons **Parcourir**, **Cloner** et **Init**. Chaque dépôt
s'ouvre dans son propre onglet. `Ctrl+W` ferme l'onglet courant.

![L'onglet Accueil listant les dépôts ouverts et récents avec les boutons Parcourir, Cloner et Init](../images/home.png)

**Cloner** demande une URL et un dossier, ou vous laisse choisir un dépôt dans un compte connecté (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit ; voir [Cloner depuis un compte connecté](hosting-integration.md#cloner-depuis-un-compte-connecté)). **Init** transforme un dossier en
nouveau dépôt.
Dans ces trois boîtes (et dans le sélecteur de dossier qu'elles ouvrent), `Ctrl+N` affiche un badge sur chaque champ et chaque bouton : appuyez sur le caractère pour placer le curseur dans le champ ou actionner le bouton.

| ![La boîte Cloner : où cloner, l'URL, le nom du dossier, et les boutons Annuler et Cloner](../images/clone.png) | ![La boîte Initialiser un dépôt : le dossier, le nom et la branche initiale](../images/init.png) |
|---|---|

### Extraction partielle

Un gros dépôt n'a pas besoin d'arriver en entier sur le disque. Dans la boîte Cloner, cochez **Extraction partielle (seulement certains dossiers)** et tapez les dossiers voulus, séparés par des espaces ou des virgules (`src docs/guide`). Seuls ces dossiers et les fichiers à la racine du dépôt sont extraits ; tout l'historique est quand même téléchargé, le Log, le blame et les branches fonctionnent donc comme d'habitude. Au moins un dossier est obligatoire tant que la case est cochée.

![La boîte Extraction partielle avec deux dossiers saisis, et les boutons Annuler, Désactiver et Appliquer](../images/sparse-checkout.png)

N'importe quel dépôt peut être basculé plus tard : ouvrez la palette de commandes (`Ctrl+P`) et lancez **Dépôt : Extraction partielle...**, ou utilisez **Extraction partielle...** dans le menu de l'onglet du dépôt. La boîte liste dans un seul champ les dossiers actuellement extraits : ajoutez ou retirez des noms puis appuyez sur **Appliquer** (`Enter`), ou sur **Désactiver (extraction complète)** pour ramener tous les fichiers. Tant qu'un arbre de travail est partiel, la ligne d'état affiche un petit badge `partiel`. Les mêmes commandes forment une page des *Paramètres > Extraction partielle* (une liste de chemins, un par ligne, avec les boutons **Démarrer**, **Arrêter** et **Actualiser**).

Remarques :

- C'est le mode cone de git (des dossiers, pas des motifs) ; `*`, `..` et les chemins absolus sont refusés.
- Changer l'ensemble n'est pas enregistré par Annuler : rien n'est perdu, et **Désactiver** restaure tous les fichiers. Git peut refuser d'abandonner un dossier qui contient des modifications non commitées (l'erreur s'affiche sur la ligne d'état) ; commitez-les ou stashez-les d'abord.
- La même chose depuis des scripts et des agents : `gitpoulpi clone <url> --sparse src docs` et `gitpoulpi sparse-checkout status | set | add | disable` (voir la [référence CLI](cli-reference.md)) ; le serveur MCP a les mêmes outils.

## Et ensuite

- [Visite de l'interface](interface-tour.md) : le nom de tout ce que vous voyez.
- [Travailler avec le Log](log-and-history.md) et
  [Stage et commit](staging-and-committing.md) : le travail quotidien.
- [Thèmes](themes.md) et [Paramètres](settings.md).
- [Assistance IA](ai.md) et [Hébergeurs et suivi de tickets](hosting-integration.md) :
  les deux intégrations facultatives.
- [Référence CLI](cli-reference.md) et [MCP et agents](mcp-and-agents.md) :
  les mêmes opérations depuis des scripts et des agents.
- [Dépannage](troubleshooting.md).
