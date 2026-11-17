[English](../README.md) · [Čeština](../cs/README.md) · [Deutsch](../de/README.md) · [Español](../es/README.md) · **Français** · [Italiano](../it/README.md) · [日本語](../ja/README.md) · [한국어](../ko/README.md) · [Nederlands](../nl/README.md) · [Polski](../pl/README.md) · [Português (Brasil)](../pt-BR/README.md) · [Português (Portugal)](../pt-PT/README.md) · [Русский](../ru/README.md) · [Svenska](../sv/README.md) · [Türkçe](../tr/README.md) · [Українська](../uk/README.md) · [简体中文](../zh-CN/README.md)

# Documentation utilisateur de GitPoulpi

<p align="center"><img src="../../../assets/logo/gitpoulpi-512.png" alt="La pieuvre GitPoulpi" width="128"></p>

GitPoulpi est un client Git pour le terminal avec un graphe de commits
cliquable, une palette de commandes, l'intégration des hébergeurs et des
outils de suivi de tickets, une aide IA facultative, ainsi qu'une CLI et un
serveur MCP qui font tout ce que fait l'interface. Ces pages sont dans l'ordre
de lecture : commencez par le haut, et passez aux pages de référence quand
vous en avez besoin.

![La fenêtre principale de GitPoulpi](../images/hero.png)

## Pour commencer

1. **[Installation](installation.md)** : ce qu'il vous faut, archives de
   release, gestionnaires de paquets, compilation depuis les sources. Pour un
   binaire plus petit, voir **[Builds personnalisés](custom-builds.md)**.
   Comment GitPoulpi se tient à jour (la pastille de version, le panneau de
   mise à jour, le retour en arrière) : **[Mises à jour](updates.md)**. Pour
   l'exécuter dans un conteneur : **[Exécuter dans Docker](docker.md)**. Pour
   les administrateurs, flux internes, miroirs et politique de mise à jour :
   **[Distribution en entreprise](enterprise-distribution.md)**.
2. **[Premiers pas](getting-started.md)** : vos cinq premières minutes, du
   lancement à votre premier commit.
3. **[Visite de l'interface](interface-tour.md)** : le nom de chaque zone de la
   fenêtre, avec des images.

## Trouver une fonctionnalité

| Je veux... | Page | Comment l'atteindre dans l'application |
|---|---|---|
| Travailler sur deux branches à la fois | [Worktrees](worktrees.md) | La section **WORKTREES** du volet Refs (`+` en crée un ; masquée tant qu'il n'y en a pas : palette **Barre latérale : Afficher la section Worktrees**) ; palette **Refs : Créer un worktree...** |
| Travailler avec des dépôts imbriqués | [Sous-modules](submodules.md) | La section **SUBMODULES** du volet Refs ; palette **Git : Mettre à jour tous les sous-modules**, **Refs : Mettre à jour le sous-module (sélection)** |
| Résoudre un conflit de merge | [Résoudre les conflits de merge](merge-conflicts.md) | `Enter` sur une ligne `!` du volet Status, le bouton **Résoudre les conflits...** du bandeau, ou `Shift+E` dans la vue Diff ; palette **Conflits : Ouvrir l'éditeur de merge** |
| Relire une pull ou merge request | [Relire une pull ou merge request](review.md) | `Enter` sur une ligne de la section **PULL REQUESTS**, `e` sur une ligne du Radar, `Alt+R` dans la recherche de l'espace de travail ; palette **Hébergeur : Relire une pull request...** |
| Parcourir, créer et gérer les pull requests et les tickets | [Pull requests et tickets](pull-requests-and-issues.md) | Les sections **PULL REQUESTS** et **TICKETS** du volet Refs (`n` en crée un) ; palette **Pull requests : Créer une pull request...**, **Hébergeur : Nouveau ticket...**. Connectez d'abord un compte : [Hébergeurs et suivi de tickets](hosting-integration.md) |
| Lire et répondre à la discussion d'un ticket ou d'une demande | [Conversations](conversations.md) | `Enter` sur une ligne de ticket, une carte du tableau ou une ligne du Radar ; palette **Hébergeur : Ouvrir la conversation d'un ticket...** |
| Voir mes tickets sous forme de tableau | [Le tableau des tickets](pull-requests-and-issues.md#le-tableau-des-tickets) | Le bouton **Tableau** de l'en-tête TICKETS ; palette **Tickets : Ouvrir le tableau** |
| Garder des tickets en notes Markdown, sans jeton | [Tickets locaux et Obsidian](local-issues.md) | La section TICKETS, fournisseur `Markdown local - <nom>` ; palette **Tickets locaux : Ouvrir dans Obsidian...** |
| Réécrire auteurs, dates, messages ou fichiers sur de nombreux commits | [Réécriture de l'historique](history-rewrite.md) | `Ctrl+G Shift+H` ; palette **Réécriture : ouvrir l'atelier** |
| Annuler (ou rétablir) une opération | [Annuler et rétablir](undo-redo.md) | `Ctrl+Z` / `Ctrl+Y` ; `Alt+Z` ouvre l'historique des annulations ; palette **Édition : Annuler** |
| Suivre de gros fichiers avec Git LFS | [Git LFS](lfs.md) | Palette **LFS : Suivre un motif...**, **LFS : Initialiser pour ce dépôt** |
| Trouver une branche, un commit, une pull request ou un ticket dans plusieurs dépôts | [Recherche](search.md) | `Ctrl+Alt+F`, ou le bouton Recherche globale de la barre de titre ; `f` cherche dans le graphe de commits du dépôt courant |
| Voir en une liste tout ce qui m'attend | [Le Radar](radar.md) | `Alt+R` ; palette **Radar : Ouvrir** |
| Grouper des dépôts et les fetcher ou les puller ensemble | [Espaces de travail](workspaces.md) | `Alt+W` change d'espace, `Alt+J` ouvre l'onglet Espaces de travail ; palette **Espace de travail : Gérer...** |
| Obtenir de l'aide de l'IA (messages de commit, explications, descriptions) | [Assistance IA](ai.md) | `Ctrl+,` > **IA** > **Activer l'IA**, puis `Ctrl+G i` rédige un message de commit ; palette **IA : ...** |
| Trouver une commande ou une touche | [Raccourcis clavier](keybindings.md) | `?` dans l'application liste les touches, `Ctrl+P` trouve n'importe quelle commande |

## Usage quotidien

4. **[Travailler avec le Log](log-and-history.md)** : lire le graphe, la
   recherche, branches, tags, stashes, glisser des refs, rebase,
   cherry-pick, reword et amend, bandeaux, pull et push, auto-stash.
5. **[Stage et commit](staging-and-committing.md)** : les modifications locales,
   diffs, stager des hunks et des lignes, écrire des commits, le compositeur de
   commits, blame, historique de fichier et l'éditeur intégré.
6. **[Images](image-view.md)** : les images dans la vue Diff et la vue Fichier,
   avant et après, zoom, Modifier avec.
7. **[Résoudre les conflits de merge](merge-conflicts.md)** : l'éditeur de
   merge (A et B côte à côte au-dessus du Résultat), choisir des lignes et des
   blocs, cartes pour les conflits binaires, LFS, supprimés, liens symboliques
   et sous-modules, et comment un agent propose une résolution.
8. **[Worktrees](worktrees.md)** : une seconde copie extraite sur une autre
   branche : la section WORKTREES, créer, ouvrir, merger et retirer des
   worktrees liés.
9. **[Sous-modules](submodules.md)** : la section SUBMODULES, leur mise à jour,
   et les garder alignés avec les checkouts et les pulls.
10. **[Annuler et rétablir](undo-redo.md)** : `Ctrl+Z` / `Ctrl+Y`, ce qui peut
    ou non être annulé, quand l'annulation refuse ou demande, jusqu'où elle
    remonte, la désactiver.
11. **[Git LFS](lfs.md)** : gros fichiers, suivi, verrous, et ce qui se passe
    sans `git-lfs` installé.
12. **[Thèmes](themes.md)** : les thèmes intégrés, n'importe quel thème avec
    n'importe quelle palette de voies, aperçu en direct, vos propres fichiers
    de thème, `gitpoulpi theme`.
13. **[Espaces de travail](workspaces.md)** : des groupes nommés de dépôts avec
    leurs propres onglets et un profil de commit par défaut facultatif, la
    pastille de la barre d'onglets, l'onglet Espaces de travail, Fetch tout, Pull tout, Cloner les dépôts absents.
14. **[Recherche](search.md)** : un seul chercheur approximatif (`Ctrl+Alt+F`)
    sur les branches, tags, worktrees, stashes, commits, pull/merge requests et
    tickets de tous les dépôts de l'espace de travail ou des onglets ouverts.
15. **[Le Radar](radar.md)** : les pull et merge requests, tickets et
    travail en cours d'un espace de travail (ou de vos onglets ouverts) dans une
    seule liste : filtres, épingles, reports, et ce que peut faire chaque ligne.
16. **[Pull requests et tickets](pull-requests-and-issues.md)** : les sections
    PULL REQUESTS et TICKETS du panneau latéral, le tableau des tickets, créer
    une pull request ou un ticket, la pastille de la barre d'outils, les
    références dans les messages de commit.
17. **[Conversations](conversations.md)** : lire la discussion d'un ticket ou
    d'une demande et y répondre dans la fenêtre, publier en tant que, ce que
    chaque hébergeur ne peut pas faire.
18. **[Relire une pull ou merge request](review.md)** : la vue de revue avec des
    fils de discussion, une revue en attente et Approuver / Demander des
    modifications.
19. **[Ouvrir dans](open-in.md)** : la liste de la barre de titre qui ouvre le
    dossier de l'onglet actif dans un terminal, un multiplexeur, un éditeur ou
    un gestionnaire de fichiers.
20. **[Suivre le dossier de l'onglet actif](following-the-tab-folder.md)** :
    « Quitter et cd ici », et le wrapper `gitpoulpi shell-init` (`gp`) pour
    bash, zsh, fish et PowerShell.
    Quel terminal affiche le mieux GitPoulpi : **[Choisir un terminal](terminals.md)**.
    Ce qui passe dans tmux, zellij, screen, herdr et les panneaux de terminal :
    **[Multiplexeurs](multiplexers.md)**.

Aussi : **[Plusieurs distants pour une branche](multiple-remotes.md)** : cibles de push et sources de
pull, le dialogue des distants, Push vers tous les distants ou vers des distants choisis, résoudre un
distant qui a bougé, Continuer après un conflit, la CLI et un alias git seul.

**[Workflows de branches](branching-workflows.md)**, les modèles prédéfinis Git Flow, GitHub
Flow, GitLab Flow et Trunk-based ou vos propres bases, modèles de nom de branche
et types de branche (`gitpoulpi flow ...`).

**[Réécriture de l'historique](history-rewrite.md)** : l'atelier (`Ctrl+G Shift+H`)
qui réécrit les auteurs, les dates, les messages et le contenu des fichiers sur de
nombreux commits, avec aperçu, scripts Lua et une seule annulation.

## Clavier et souris

21. **[Raccourcis clavier](keybindings.md)** : chaque touche, par zone, et
    chaque commande de la palette. Dans l'application, `?` affiche la même
    liste et `Ctrl+P` trouve n'importe quelle commande.
22. **[Souris et gestes](mouse-and-gestures.md)** : cliquer, glisser,
    sélectionner et copier, barres de défilement et retour au survol.
    Tout retrouver dans vos dépôts : **[Recherche](search.md)** (`Ctrl+Alt+F`) :
    refs, worktrees, stashes, commits, pull/merge requests et tickets de tout
    l'espace de travail.

## Paramètres et intégrations

23. **[Paramètres](settings.md)** : chaque page et chaque option de l'écran des
    Paramètres, la zone de recherche, les profils, et comment réaffecter les
    raccourcis.
24. **[Assistance IA](ai.md)** : l'IA propre à GitPoulpi (désactivée par
    défaut) : un agent local comme `claude -p`, Anthropic, OpenAI, OpenRouter,
    Ollama ou LM Studio ; instructions ; messages de commit, explications,
    descriptions de pull request et compositeur de commits ; confidentialité et
    dépannage.
25. **[Hébergeurs et suivi de tickets](hosting-integration.md)** : GitHub, GitLab,
    Gitea et Forgejo, Azure DevOps, Bitbucket, Gerrit et les outils de suivi de tickets :
    jetons et leurs portées, connexion, lier un dépôt, clés SSH, limites connues. Voir aussi
    **[Tickets locaux et Obsidian](local-issues.md)** : un dossier de notes
    Markdown comme source de tickets, sans jeton, avec une Base Obsidian.

## Automatisation

26. **[Référence CLI](cli-reference.md)** : chaque commande `gitpoulpi`, avec
    `--json` pour les scripts.
27. **[MCP et agents](mcp-and-agents.md)** : laisser un agent externe (Claude
    Code, Cursor, Codex, ...) piloter GitPoulpi : enregistrer le serveur,
    `.mcp.json`, les interrupteurs de permission et leurs valeurs par défaut,
    chaque outil, et comment un agent vous montre des choses dans votre
    fenêtre. La skill d'agent toute prête est
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Référence et aide

28. **[Référence de configuration](config-reference.md)** : chaque clé de
    `config.toml`, sa valeur par défaut et son effet, les paramètres par dépôt,
    les variables d'environnement.
29. **[Dépannage et FAQ](troubleshooting.md)** : icônes, rendu du graphe,
    touches qui n'arrivent pas, démons bloqués, vitesse, pointeurs LFS.

## Captures d'écran

Chaque image de ces pages est une vraie capture d'un build de release, sur un
petit dépôt de démonstration avec des auteurs inventés. Les images sont dans
[`images/`](../images/). `scripts/docs-screenshots.sh` construit les dépôts de
démonstration et les reprend. Elles sont prises dans le terminal `kitty`, qui
dessine le graphe des commits et les avatars des auteurs sous forme d'images.
Les terminaux sans protocole graphique affichent à la place un graphe en
caractères.

## Pour les contributeurs

[CLAUDE.md](../../../CLAUDE.md), la [vue d'ensemble de l'architecture](../../architecture/overview.md)
et la [feuille de route](../../roadmap.md) décrivent la conception et ce qui
est prévu ; [CHANGELOG.md](../../../CHANGELOG.md) liste ce qui a changé dans
chaque release.

---

GitKraken est une marque de son propriétaire respectif. GitPoulpi est un projet indépendant, ni affilié à ce propriétaire, ni approuvé ou parrainé par lui ; le nom n'est cité que pour décrire la compatibilité et l'inspiration.
