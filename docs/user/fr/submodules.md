[English](../submodules.md) · [Čeština](../cs/submodules.md) · [Deutsch](../de/submodules.md) · [Español](../es/submodules.md) · **Français** · [Italiano](../it/submodules.md) · [日本語](../ja/submodules.md) · [한국어](../ko/submodules.md) · [Nederlands](../nl/submodules.md) · [Polski](../pl/submodules.md) · [Português (Brasil)](../pt-BR/submodules.md) · [Português (Portugal)](../pt-PT/submodules.md) · [Русский](../ru/submodules.md) · [Svenska](../sv/submodules.md) · [Türkçe](../tr/submodules.md) · [Українська](../uk/submodules.md) · [简体中文](../zh-CN/submodules.md)

# Sous-modules

Un sous-module est un autre dépôt imbriqué dans le vôtre à un commit fixe. GitPoulpi les liste dans le panneau latéral Refs, les met à jour depuis l'interface, la CLI ou la palette de commandes, et peut les garder alignés avec le commit que vous extrayez.

## La section SOUS-MODULES

Quand un dépôt a un `.gitmodules`, le panneau latéral Refs reçoit une section **SOUS-MODULES** (sous les autres ; elle est absente quand il n'y en a pas). Une ligne par sous-module : une marque d'état, son chemin et un commit court.

| Marque | État | Signification |
|---|---|---|
| `✓` | à jour | extrait au commit que le dépôt enregistre |
| `↑` | modifié | extrait à un autre commit (il a avancé, ou vous avez changé de branche sans le mettre à jour) |
| `○` | non cloné | déclaré dans `.gitmodules` et enregistré, mais son dossier est vide : mettez-le à jour pour le cloner |
| `✗` | en conflit | le dépôt a des conflits de merge sur le sous-module |

| Action | Souris | Clavier |
|---|---|---|
| Ouvrir le sous-module dans son propre onglet | double-clic sur la ligne | le sélectionner, `Enter` (palette : `Refs : Ouvrir le sous-module dans un onglet`) |
| Menu : **Mettre à jour (cloner au besoin)**, **Initialiser**, **Ouvrir dans un nouvel onglet** | clic droit sur la ligne | le sélectionner, la touche de menu contextuel (par défaut `.`) |
| Mettre à jour le sous-module sélectionné (le clone d'abord si nécessaire) | menu | `Ctrl+P` : `Refs : Mettre à jour le sous-module (sélection)` |
| L'enregistrer dans `.git/config` sans le cloner | menu | `Ctrl+P` : `Refs : Initialiser le sous-module (sélection)` |
| Mettre à jour tous les sous-modules | | `Ctrl+P` : `Git : Mettre à jour tous les sous-modules` |
| Aller à la section | clic sur son en-tête | `Ctrl+P` : `Navigation : Aller à Refs > Sous-modules` |

Une mise à jour peut faire un fetch depuis le remote du sous-module ; elle suit donc **Paramètres > SSH et identifiants > Identifiants HTTPS** comme tout autre transfert. Un sous-module non cloné n'a rien à ouvrir dans un onglet ; la ligne d'état le dit et renvoie vers Mettre à jour.

## Garder les sous-modules à jour

**Paramètres > Synchronisation > Garder les sous-modules à jour** (`general.keep_submodules_up_to_date`, désactivé par défaut). Quand il est activé, un **checkout** ou un **pull** réussi est suivi de `git submodule update --init --recursive`. Cela ne s'exécute que si un sous-module n'est plus aligné avec ce que le dépôt enregistre désormais, de sorte qu'un dépôt sans sous-modules n'est pas ralenti. Si cette mise à jour échoue, le checkout ou le pull a bien eu lieu et l'erreur le dit.

## En ligne de commande

```
gitpoulpi submodule list                       # path, state, url (--json: the full list)
gitpoulpi submodule init [<path>...]           # register, do not clone
gitpoulpi submodule update [<path>...] [--init] [--recursive]
```

Sans chemin, tous les sous-modules sont concernés. Ces trois commandes existent aussi comme outils MCP (`submodule_list` est une lecture ; `init` et `update` sont des mutations et suivent `allow_mutations` de `[mcp]`).

## Notes

- Seul le premier niveau est listé ; les sous-modules d'un sous-module apparaissent quand vous l'ouvrez dans un onglet (ou utilisez `--recursive` pour les mettre à jour).
- Les versions récentes de git refusent de cloner un sous-module à *chemin local* via le transport `file` sauf si `protocol.file.allow` l'autorise. Cela ne concerne que les sous-modules dont l'URL est un dossier local ; réglez-le dans votre configuration git globale si vous en utilisez.
