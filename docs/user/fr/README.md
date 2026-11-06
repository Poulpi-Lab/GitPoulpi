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

## Usage quotidien

4. **[Travailler avec le Log](log-and-history.md)** : lire le graphe, la
   recherche, branches, tags, stashes, worktrees, glisser des refs, rebase,
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
8. **[Annuler et rétablir](undo-redo.md)** : `Ctrl+Z` / `Ctrl+Y`, ce qui peut
   ou non être annulé, quand l'annulation refuse ou demande, jusqu'où elle
   remonte, la désactiver.
9. **[Git LFS](lfs.md)** : gros fichiers, suivi, verrous, et ce qui se passe
   sans `git-lfs` installé.
10. **[Sous-modules](submodules.md)** : la section SUBMODULES, leur mise à jour,
    et les garder alignés avec les checkouts et les pulls.
11. **[Thèmes](themes.md)** : les thèmes intégrés, n'importe quel thème avec
    n'importe quelle palette de voies, aperçu en direct, vos propres fichiers
    de thème, `gitpoulpi theme`.
12. **[Espaces de travail](workspaces.md)** : des groupes nommés de dépôts avec
    leurs propres onglets et un profil de commit par défaut facultatif, la
    pastille de la barre d'onglets, l'onglet Espaces de travail, Fetch tout, Pull tout, Cloner les dépôts absents.
13. **[Recherche](search.md)** : un seul chercheur approximatif (`Ctrl+Alt+F`)
    sur les branches, tags, worktrees, stashes, commits, pull/merge requests et
    tickets de tous les dépôts de l'espace de travail ou des onglets ouverts.
14. **[Le Radar](radar.md)** : les pull et merge requests, tickets et
    travail en cours d'un espace de travail (ou de vos onglets ouverts) dans une
    seule liste : filtres, épingles, reports, et ce que peut faire chaque ligne.
    Lire et relire l'une de ces demandes dans l'application, avec des fils de
    discussion, une revue en attente et Approuver / Demander des modifications :
    **[Relire une pull request](review.md)**.
15. **[Ouvrir dans](open-in.md)** : la liste de la barre de titre qui ouvre le
    dossier de l'onglet actif dans un terminal, un multiplexeur, un éditeur ou
    un gestionnaire de fichiers.
16. **[Suivre le dossier de l'onglet actif](following-the-tab-folder.md)** :
    « Quitter et cd ici », et le wrapper `gitpoulpi shell-init` (`gp`) pour
    bash, zsh, fish et PowerShell.
    Quel terminal affiche le mieux GitPoulpi : **[Choisir un terminal](terminals.md)**.

Aussi : **[Workflows de branches](branching-workflows.md)**, les modèles prédéfinis Git Flow, GitHub
Flow, GitLab Flow et Trunk-based ou vos propres bases, modèles de nom de branche
et types de branche (`gitpoulpi flow ...`).

**[Réécriture de l'historique](history-rewrite.md)** : l'atelier (`Ctrl+G Shift+H`)
qui réécrit les auteurs, les dates, les messages et le contenu des fichiers sur de
nombreux commits, avec aperçu, scripts Lua et une seule annulation.

## Clavier et souris

17. **[Raccourcis clavier](keybindings.md)** : chaque touche, par zone, et
    chaque commande de la palette. Dans l'application, `?` affiche la même
    liste et `Ctrl+P` trouve n'importe quelle commande.
18. **[Souris et gestes](mouse-and-gestures.md)** : cliquer, glisser,
    sélectionner et copier, barres de défilement et retour au survol.
    Tout retrouver dans vos dépôts : **[Recherche](search.md)** (`Ctrl+Alt+F`) :
    refs, worktrees, stashes, commits, pull/merge requests et tickets de tout
    l'espace de travail.

## Paramètres et intégrations

19. **[Paramètres](settings.md)** : chaque page et chaque option de l'écran des
    Paramètres, la zone de recherche, les profils, et comment réaffecter les
    raccourcis.
20. **[Assistance IA](ai.md)** : l'IA propre à GitPoulpi (désactivée par
    défaut) : un agent local comme `claude -p`, Anthropic, OpenAI, OpenRouter,
    Ollama ou LM Studio ; instructions ; messages de commit, explications,
    descriptions de pull request et compositeur de commits ; confidentialité et
    dépannage.
21. **[Hébergeurs et suivi de tickets](hosting-integration.md)** : GitHub, GitLab,
    Gitea et Forgejo, Azure DevOps, Bitbucket, Gerrit et les outils de suivi de tickets :
    jetons et leurs portées, connexion, lier un dépôt, pull requests et
    tickets, références
    dans les messages de commit, clés SSH, limites connues. Voir aussi
    **[Tickets locaux et Obsidian](local-issues.md)** : un dossier de notes
    Markdown comme source de tickets, sans jeton, avec une Base Obsidian.

## Automatisation

22. **[Référence CLI](cli-reference.md)** : chaque commande `gitpoulpi`, avec
    `--json` pour les scripts.
23. **[MCP et agents](mcp-and-agents.md)** : laisser un agent externe (Claude
    Code, Cursor, Codex, ...) piloter GitPoulpi : enregistrer le serveur,
    `.mcp.json`, les interrupteurs de permission et leurs valeurs par défaut,
    chaque outil, et comment un agent vous montre des choses dans votre
    fenêtre. La skill d'agent toute prête est
    [`skills/gitpoulpi/SKILL.md`](../../../skills/gitpoulpi/SKILL.md).

## Référence et aide

24. **[Référence de configuration](config-reference.md)** : chaque clé de
    `config.toml`, sa valeur par défaut et son effet, les paramètres par dépôt,
    les variables d'environnement.
25. **[Dépannage et FAQ](troubleshooting.md)** : icônes, rendu du graphe,
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
