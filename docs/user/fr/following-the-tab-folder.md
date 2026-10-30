[English](../following-the-tab-folder.md) · [Čeština](../cs/following-the-tab-folder.md) · [Deutsch](../de/following-the-tab-folder.md) · [Español](../es/following-the-tab-folder.md) · **Français** · [Italiano](../it/following-the-tab-folder.md) · [日本語](../ja/following-the-tab-folder.md) · [한국어](../ko/following-the-tab-folder.md) · [Nederlands](../nl/following-the-tab-folder.md) · [Polski](../pl/following-the-tab-folder.md) · [Português (Brasil)](../pt-BR/following-the-tab-folder.md) · [Português (Portugal)](../pt-PT/following-the-tab-folder.md) · [Русский](../ru/following-the-tab-folder.md) · [Svenska](../sv/following-the-tab-folder.md) · [Türkçe](../tr/following-the-tab-folder.md) · [Українська](../uk/following-the-tab-folder.md) · [简体中文](../zh-CN/following-the-tab-folder.md)

# Suivre le dossier de l'onglet actif

GitPoulpi peut avoir plusieurs dépôts ouverts dans des onglets. Le shell depuis
lequel vous l'avez lancé reste dans son dossier : un programme ne peut pas
changer le dossier de son shell parent. GitPoulpi propose deux moyens d'obtenir
un shell dans le dossier de l'onglet actif :

- **Quitter & cd ici** : GitPoulpi se ferme, et votre shell passe dans le
  dossier de l'onglet actif. Cela demande une petite fonction shell, affichée
  par `gitpoulpi shell-init`.
- **Ouvrir dans** : ouvre le dossier de l'onglet actif dans un terminal, un
  volet de multiplexeur, un éditeur ou le gestionnaire de fichiers. Aucune
  configuration du shell n'est nécessaire. Voir [Ouvrir dans](open-in.md).

```
┌🐙 GitPoulpi vX.Y.Z -- /work/other-repo - Open in ▾ ── ⌕ Recherche globale ⚙ × ┐
│ ⌂  main-repo x  other-repo x                   Quit & cd here           │
```

- **Ouvrir dans** se trouve dans la barre de titre, après le chemin du dépôt.
  Il est masqué quand la fenêtre est trop étroite.
- **Quitter & cd ici** se trouve à l'extrémité droite de la barre d'onglets. Il
  n'apparaît que si les deux conditions sont réunies :
  - GitPoulpi a été lancé via la fonction shell (ci-dessous).
  - L'onglet actif n'est pas le dossier où vous avez lancé GitPoulpi.
- Les commandes sont toujours dans la palette (`Ctrl+P`) : `Onglet : Quitter et faire cd ici`,
  `Onglet : Ouvrir dans...` (`Alt+O`) et `Onglet : Ouvrir dans l'entrée par défaut` (`Ctrl+Alt+O`).
  Sans la fonction shell, `Onglet : Quitter et faire cd ici` vous explique comment la mettre en
  place.

Une sortie normale (`q`) ne change jamais le dossier de votre shell. Seul
**Quitter & cd ici** le fait, sauf si vous choisissez `auto` (ci-dessous).

## Mettre en place la fonction shell

`gitpoulpi shell-init <shell>` affiche une fonction nommée `gp`. Utilisez
`--name` pour choisir un autre nom. La fonction lance
`gitpoulpi --cwd-file <fichier temporaire> ...`. Quand vous quittez avec
**Quitter & cd ici**, GitPoulpi écrit le dossier dans ce fichier, et la fonction
se place dans ce dossier. GitPoulpi n'installe ni ne modifie rien : vous
ajoutez une ligne au fichier de démarrage de votre shell.

| Shell | À ajouter dans | Ligne |
|---|---|---|
| bash | `~/.bashrc` | `eval "$(gitpoulpi shell-init bash)"` |
| zsh | `~/.zshrc` | `eval "$(gitpoulpi shell-init zsh)"` |
| fish | `~/.config/fish/config.fish` | `gitpoulpi shell-init fish \| source` |
| PowerShell | votre `$PROFILE` (`notepad $PROFILE`) | `gitpoulpi shell-init pwsh \| Out-String \| Invoke-Expression` |

Lancez ensuite GitPoulpi avec `gp` au lieu de `gitpoulpi`. Les arguments sont
transmis (`gp -s work`), et le code de sortie est celui de `gitpoulpi`.

**Un `gp` existant n'est jamais remplacé sans avertissement.** Si votre shell a
déjà une commande, un alias ou une fonction nommée `gp`, le script affiche un
avertissement et ne définit rien. Choisissez un autre nom, par exemple
`gitpoulpi shell-init bash --name gpl`. Dans PowerShell, la fonction remplace
l'alias intégré `gp` (`Get-ItemProperty`) et un alias qui pointe vers
`gitpoulpi`. Tout autre `gp` est conservé, avec un avertissement.

Toutes les commandes `gitpoulpi` acceptent `--cwd-file`, mais seule la TUI
écrit le fichier.

## Paramètre : Suivre le dossier de l'onglet actif

Paramètres > Ouvrir dans > **Suivre le dossier de l'onglet actif** (`ui.tab_dir_follow`) :

| Valeur | Comportement |
|---|---|
| `buttons` (défaut) | **Quitter & cd ici** et **Ouvrir dans** comme décrit plus haut. |
| `auto` | Pas de bouton **Quitter & cd ici**. **Ouvrir dans** reste. Chaque sortie place votre shell dans le dossier de l'onglet actif (quand vous avez lancé GitPoulpi avec la fonction shell). L'entrée **Ouvrir dans** par défaut s'exécute à chaque ouverture d'un nouvel onglet. |

En mode `auto`, l'entrée par défaut ne s'exécute que pour les onglets que vous
ouvrez vous-même : depuis l'onglet Accueil, un worktree, un clone ou une
initialisation. Elle ne s'exécute pas pour les onglets restaurés au démarrage,
pour le premier onglet, ni quand vous passez à un onglet déjà ouvert.

Les anciens paramètres `ui.terminal_launch` et `ui.terminal_launch_command` ont
été déplacés dans la liste **Ouvrir dans**. Voir
[Ouvrir dans](open-in.md#anciennes-clés).
