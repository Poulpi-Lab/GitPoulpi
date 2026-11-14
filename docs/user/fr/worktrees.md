[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · **Français** · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Un worktree est une seconde copie extraite du dépôt : un autre dossier, sur une
autre branche, avec le même historique. La section WORKTREES du volet Refs les
liste ; le principal est marqué `(main)`.

## Afficher la section

La section est masquée dans un dépôt qui n'a pas encore de worktree lié. Pour
l'afficher quand même (elle indique alors « Aucun worktree : + pour en créer
un »), utilisez la commande de la palette **Barre latérale : Afficher la section
Worktrees**, cochez **WORKTREES** dans la liste d'affichage de la barre latérale
(clic droit sur un en-tête de section), ou lancez l'une des commandes de création
ci-dessous. GitPoulpi s'en souvient pour ce dépôt (voir **Sections du volet
gauche** dans les [Paramètres](settings.md)) ; un autre dépôt sans worktree
démarre toujours avec la section masquée.

## Créer un worktree

Le **+** de l'en-tête WORKTREES (clavier : `+` dans la section ; palette : **Refs :
Créer un worktree...**) ouvre un menu :

- **Depuis la branche courante** : la branche est extraite dans le dossier
  principal, donc la boîte de dialogue propose une nouvelle branche dessus
  (`<branche>-wt`).
- **Depuis le commit sélectionné** : grisé tant qu'aucun commit n'est sélectionné
  dans le Log. Le worktree démarre détaché à cet endroit, sauf si vous nommez une
  nouvelle branche.
- **Choisir une branche...** : la boîte de dialogue s'ouvre sur sa liste **Réf**.

La palette propose les mêmes trois (**Refs : Créer un worktree depuis la branche
courante**, **Git : Créer un worktree sur le commit sélectionné...**, **Refs :
Créer un worktree en choisissant une branche**). Les anciennes entrées
fonctionnent toujours : **Créer un worktree depuis...** dans le menu d'une
branche, d'un tag ou d'un commit.

La boîte de dialogue propose tout et vous laisse tout changer :

| Champ | Signification |
|---|---|
| **Nom** | Le nom dont le dossier est tiré. Proposé à partir de la branche ou de l'identifiant court. |
| **Dossier** | Un voisin du dépôt, `<dépôt>-<nom>`. Il suit le Nom tant que vous ne tapez rien dans Dossier ; **Parcourir...** choisit l'emplacement. |
| **Réf** | La branche ou le commit d'où part le worktree (une liste ; tapez pour chercher). La changer propose un nouveau Nom. |
| **Nouvelle branche** | Vide, elle extrait la branche de la Réf à cet endroit (ou détache sur un commit). Obligatoire quand la branche est déjà extraite. |
| **Ouvrir le nouveau worktree dans un onglet** | Activé par défaut. |

Un dossier non vide, un nom invalide ou une branche qui existe déjà sont signalés
sur la ligne propre de la boîte de dialogue, et **Créer** reste grisé.

## Le menu d'une ligne

Clic droit sur un worktree (ou sélectionnez-le et appuyez sur la touche menu,
`Shift+F10` ou `.`) :

- **Ouvrir dans un nouvel onglet** : comme un double-clic ou `Enter`.
- **Ouvrir dans** : les entrées d'[Ouvrir dans](open-in.md) dont la case **Peut
  ouvrir un worktree** est cochée (Paramètres > Ouvrir dans ; chaque entrée est
  cochée par défaut). Elles s'exécutent avec le dossier du worktree comme
  `${cwd}`.
- **Merge de <branche> dans...** : choisissez la branche dans laquelle merger (la
  branche du worktree principal par défaut). Le merge ordinaire s'exécute dans le
  worktree où cette branche est extraite, avec vos options de merge ; un conflit
  s'y résout par le flux habituel des conflits. Une branche extraite nulle part
  est refusée, avec la raison. Non proposé pour un worktree détaché.
- **Retirer le worktree...** : jamais le principal.

La palette offre les mêmes actions sur la ligne sélectionnée (**Refs : Ouvrir le
worktree dans...**, **Refs : Merge de la branche du worktree dans...**, **Refs :
Retirer le worktree...**, **Refs : Ouvrir le worktree dans un onglet**).

## Retirer un worktree

Un worktree propre reçoit une simple confirmation. Si sa branche est déjà mergée
(dans son amont, la branche du worktree principal ou `main` / `master`), la case
**Supprimer aussi la branch <b> (mergée dans <into>)** est cochée par défaut ; une
branche non mergée indique qu'elle reste. **Merge dans... puis retirer** merge
d'abord et ne retire qu'une fois le merge passé sans conflit.

Avec des modifications non commitées (suivies ou non), la boîte de dialogue les
liste (nombre, défilement) et propose :

- **Commit puis retirer** : un champ de message dans la boîte de dialogue ; toutes
  les modifications sont stagées (`.gitignore` respecté) et commitées sur la
  branche du worktree, puis le worktree est retiré. Un worktree détaché nomme une
  branche pour le commit dans la même boîte de dialogue.
- **Stash puis retirer** : toutes les modifications, fichiers non suivis compris,
  sont mises en stash avec un message modifiable (par défaut « WIP sur
  <dossier> »), puis le worktree est retiré. La liste des stashes est partagée par
  tous les worktrees : le stash est ensuite dans les STASHES du dépôt principal,
  prêt à être dépilé.
- **Ouvrir pour commiter** : ouvre le worktree dans un onglet avec le message de
  commit sélectionné ; rien n'est retiré.
- **Retirer quand même** : abandonne les modifications après une seconde
  confirmation explicite.

## Annuler

Retirer un worktree n'est pas annulable : le dossier a disparu. Ce qui a été
sauvegardé avant reste : le commit est sur sa branche, le stash dans la liste. La
suppression de branche qu'ajoute **Supprimer aussi la branch** est la suppression
ordinaire, journalisée : **Annuler** rend la branche (pas le dossier). Créer un
worktree ne fait pas non plus partie de l'historique.

## Depuis un script ou un agent

`gitpoulpi worktree changes <path>` liste les modifications non commitées d'un
worktree. `gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` le retire ; les
outils MCP sont `worktree_changes` et `worktree_remove`. Voir la
[référence de la CLI](cli-reference.md).
