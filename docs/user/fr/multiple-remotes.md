[English](../multiple-remotes.md) · [Čeština](../cs/multiple-remotes.md) · [Deutsch](../de/multiple-remotes.md) · [Español](../es/multiple-remotes.md) · **Français** · [Italiano](../it/multiple-remotes.md) · [日本語](../ja/multiple-remotes.md) · [한국어](../ko/multiple-remotes.md) · [Nederlands](../nl/multiple-remotes.md) · [Polski](../pl/multiple-remotes.md) · [Português (Brasil)](../pt-BR/multiple-remotes.md) · [Português (Portugal)](../pt-PT/multiple-remotes.md) · [Русский](../ru/multiple-remotes.md) · [Svenska](../sv/multiple-remotes.md) · [Türkçe](../tr/multiple-remotes.md) · [Українська](../uk/multiple-remotes.md) · [简体中文](../zh-CN/multiple-remotes.md)

# Plusieurs distants pour une branche

Beaucoup de dépôts vivent sur plus d'un hébergeur : un principal (`origin`), un miroir sur un autre hébergeur, une sauvegarde. GitPoulpi permet à une branche de pousser vers tous d'un clic et, si vous le voulez, de tirer depuis plusieurs, sans changer ce que fait git tout seul. Rien ici ne demande de compte, et tout est stocké dans le `.git/config` du dépôt, donc git et les autres outils continuent de fonctionner.

## Cibles de push et sources de pull

- L'**amont** est la notion propre à git (`branch.<name>.remote` et `merge`). C'est le distant *principal* de la branche. GitPoulpi ne change jamais son fonctionnement.
- Une **cible de push** est un distant supplémentaire vers lequel la branche pousse aussi. Le bouton Push envoie la branche vers l'amont et vers chaque cible de push.
- Une **source de pull** est un distant supplémentaire dont un pull prend aussi les commits. Elle est désactivée tant que vous ne l'activez pas (la case **Pull aussi**) : fusionner deux distants que vous n'avez pas choisis n'est pas un comportement par défaut que quelqu'un veuille.

Une cible peut utiliser un autre nom de branche sur ce distant (`backup` reçoit `feat-x`).

## Le dialogue des distants

Faites un clic droit sur une branche et choisissez **Distants de <branche>...** (aussi dans la liste déroulante Push, dans la palette sous `Git : Distants de la branche...`, et en cliquant sur le badge `⇉3` de la ligne de la branche dans la section Refs). Il liste chaque distant du dépôt :

| Colonne | Signification |
|---|---|
| Distant, Branche | Le distant et le nom de la branche qui s'y trouve. **Renommer sur le distant...** change ce nom pour la ligne sélectionnée. |
| Push | Cochez pour faire du distant une cible de push de cette branche. |
| Pull aussi | Cochez pour qu'un pull intègre aussi ce distant. L'amont est toujours tiré. |
| État | Où en est la branche par rapport à lui : à jour, en avance, en retard, nouvelle, sans lien. |

Une ligne marquée « distant disparu » désigne un distant que la branche mentionne encore mais que le dépôt n'a plus ; **Retirer** l'oublie. Le même dialogue règle l'amont (la liste **Amont**) et **Retirer l'amont** l'efface. **Appliquer** écrit le tout comme un seul changement annulable.

Un dépôt peut aussi donner les mêmes cibles supplémentaires à chaque nouvelle branche : **Paramètres > Git et encodage** (la page propre au dépôt), « Cibles de push supplémentaires pour les nouvelles branches » (une case par distant). Elles sont copiées dans une branche quand elle reçoit son premier amont ; les branches existantes ne sont pas touchées.

## Pousser

- **Bouton Push** sur une branche qui a des cibles supplémentaires : pousse vers toutes. Quand chaque distant prend simplement les nouveaux commits, l'opération part aussitôt et la vue de progression montre chaque distant passer à une coche. Si quelque chose demande une décision (un distant a des commits que vous n'avez pas), le dialogue du plan s'ouvre à la place.
- **Push ▾** (la flèche à côté du bouton ; aussi le bouton lui-même quand **Bouton Push** est sur **ask** dans les Paramètres) :
  - `Push vers tous (origin, gitlab, backup)`
  - `Push vers <distant> seulement`, un par distant de la branche
  - `Push vers des distants choisis...` : toujours présent, pour toute branche, même avec un seul distant
  - `Distants de <branche>...`
- Le menu du clic droit de la branche et la palette (`Git : Push vers toutes les cibles`, `Git : Push vers un seul distant...`, `Git : Push vers des distants choisis...`) proposent les mêmes actions sous les mêmes noms.

**Push vers des distants choisis...** ouvre la liste de tous les distants du dépôt, cochés d'après l'amont et les cibles de la branche. Elle ne vaut que pour ce push ; cochez **Mémoriser pour cette branche** pour enregistrer le choix comme cibles de push de la branche.

### Le plan

Avant un push groupé, GitPoulpi regarde chaque distant (une requête rapide, et un téléchargement de ce distant seulement quand il diffère) pour qu'un problème sur le troisième distant apparaisse avant le début du premier push. Le dialogue liste chaque distant, où en est la branche et ce qui sera fait :

| Situation | Action par défaut | Aussi proposé |
|---|---|---|
| Le distant n'a rien de la branche, ou il lui manque des commits | Push | Ignorer |
| Le distant est en avance (vous n'avez rien de nouveau pour lui) | Avance rapide depuis lui | Ignorer |
| Le distant diffère parce que vous avez rebasé ou amendé ce que vous aviez poussé | Push forcé (lease) | Merge, Rebase, Ignorer |
| Le distant a des commits de quelqu'un d'autre | Merge depuis lui (Rebase quand **Bouton Pull** est sur rebase) | Rebase, Push forcé (lease), Ignorer |
| L'historique ne partage rien avec le vôtre | Ignorer | Push forcé (lease) |
| L'hébergeur a refusé la branche (protégée) | Ignorer | aucun : forcer serait refusé aussi |

L'action de chaque ligne est une liste que vous pouvez changer (Gauche/Droite la parcourt, Entrée l'ouvre, `s` ignore la ligne). Un merge indique s'il entrerait en conflit et dans quels fichiers. Un **push forcé** est toujours confirmé, nomme le distant et le nombre de commits qu'il remplace, utilise toujours un lease (jamais un force nu) et ne peut pas être annulé : le résultat dit quel commit a été remplacé et la commande qui le remet. **Ignorer les problèmes** met chaque ligne en difficulté sur Ignorer. **Lancer** (Ctrl+Entrée) démarre.

Les merges, avances rapides et rebases passent d'abord, dans l'ordre de la liste, pour que votre branche atteigne son état final une seule fois ; ensuite les pushes partent un distant à la fois, l'amont en premier. Il n'y a pas de « tout ou rien » atomique entre les distants : chacun finit avec son propre résultat, et l'échec de l'un (réseau, hook) n'arrête pas les autres. **Réessayer les échecs** (ou `Git : Relancer les envois échoués`) les retente. Fermer la vue de progression n'arrête jamais l'opération ; `Git : Afficher le Push groupé` la ramène.

### Conflits et Continuer

Si un merge ou un rebase de l'opération s'arrête sur des conflits, le dialogue se ferme et la bannière de merge habituelle avec l'éditeur de conflits prennent le relais. Une bande sur le Log affiche `Push groupé en pause (gitlab) : résolvez et concluez, puis Continuer.` avec les boutons **Continuer** et **Abandonner le groupe**.

- Résolvez les conflits et concluez le merge ou le rebase (dans GitPoulpi ou avec git).
- **Continuer** reste grisé tant que ce n'est pas fait, et il n'est **jamais pressé à votre place** : rien ne reprend tout seul. Appuyez dessus (ou lancez `Git : Continuer le Push groupé`) et l'opération pousse de nouveau vers chaque distant depuis votre nouveau sommet. Après un rebase, un distant qui garde les anciens commits revient au dialogue du plan (il lui faut maintenant un push forcé) au lieu d'être forcé en silence.
- **Abandonner le groupe** (confirmé ; `Git : Abandonner le Push groupé`) jette le merge ou le rebase que l'opération a démarré et oublie l'opération. Les pushes déjà faits restent.

La bande survit à un redémarrage de GitPoulpi, et la CLI voit la même opération enregistrée.

## Tirer

- Le bouton **Pull** tire l'amont exactement comme avant, sauf si la branche a des sources de pull : il tire alors l'amont et chaque source, dans l'ordre, chacune dans le mode réglé par **Bouton Pull** dans les Paramètres (merge, fast-forward seul ou rebase). Une source qui ne peut pas avancer en fast-forward est fusionnée (ou rebasée) comme l'amont ; avec fast-forward seul, une source qui demande plus pose d'abord la question.
- **Pull ▾** ajoute `Pull depuis toutes les sources (origin + gitlab)` quand il y a des sources, et `Pull depuis <distant> seulement` pour chaque distant de la branche : un pull ponctuel de la branche de ce distant, quelles que soient les cases.
- La palette a `Git : Pull depuis toutes les sources` et `Git : Pull depuis un seul distant...`.
- Un pull qui s'arrête sur des conflits laisse la même bande, intitulée pull groupé.

## Paramètres

Les quatre sont dans **Paramètres > Git et encodage**.

| Réglage | Par défaut | Signification |
|---|---|---|
| **Bouton Push** | direct | direct pousse aussitôt ; ask ouvre d'abord le menu Push. |
| **Bouton Pull** | merge | Le mode du bouton Pull (merge, fast-forward seul ou rebase), et des sources de pull. |
| **Vérifier chaque distant avant un push** | activé | Regarder chaque distant avant un push groupé (la CLI le suit ; `--no-preflight` le désactive pour un appel). |
| **Cibles de push supplémentaires pour les nouvelles branches** (par dépôt) | aucune | Les distants vers lesquels une nouvelle branche pousse aussi. |

## En ligne de commande

```
gitpoulpi branch upstream add gitlab --pull      # a push target that is also a pull source
gitpoulpi branch upstream list                    # what the branch is set up for
gitpoulpi push --all-targets                      # upstream plus every target
gitpoulpi push --to gitlab --to backup:feat-x     # exactly these, this once
gitpoulpi pull --all-sources                      # upstream plus every pull source
gitpoulpi remote-group plan                       # what a grouped push would do, nothing changed
gitpoulpi remote-group continue                   # after resolving and concluding a stop
```

Quand un distant demande une décision, la commande répond par le plan (code de sortie 3) et ne change rien ; répondez avec `--resolve gitlab=merge`. Voir [la référence de la CLI](cli-reference.md) pour chaque option. Le serveur MCP a les mêmes outils.

## Avec git seul

Les cibles sont de simples clés de configuration, donc un script ou un alias peut les utiliser sans GitPoulpi :

```
git config --get-all branch.feature.gitpoulpi-push-to     # one value per line: remote or remote:branch

# push the current branch to its upstream and to every target
git config alias.pushall '!b=$(git symbolic-ref --short HEAD); git push; for t in $(git config --get-all branch.$b.gitpoulpi-push-to); do r=${t%%:*}; d=${t#*:}; [ "$d" = "$t" ] && d=$b; git push "$r" "$b:$d"; done'
```

Une valeur est `<remote>` (même nom de branche) ou `<remote>:<branch>`, ce que démêle la ligne `d=${t#*:}`. L'alias simple ne regarde pas les distants d'abord, ne propose pas de merge et ne peut pas continuer après un conflit : c'est le rôle du dialogue.

## Bon à savoir

- Deux distants qui pointent vers la même adresse sont repérés (« gitlab et mirror partagent la même URL (push) ») ; le second est ignoré par défaut.
- Un hébergeur qui protège une branche refuse un push forcé ; la ligne le dit et ne propose qu'Ignorer.
