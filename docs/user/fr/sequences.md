Une **séquence** est une liste nommée de commandes exécutées l'une après l'autre : par
exemple *Stager tous les changements*, *IA : Générer le message de commit*, *Commit*, *Push*
sous une seule touche **Backup**. Chaque étape est une commande de la
[palette de commandes](keybindings.md) ; une séquence a sa propre ligne dans la palette
(`Séquence : <nom>`), peut être liée à une touche, et se modifie dans
**Paramètres > Séquences** ou dans `config.toml`.

## La page Séquences

**Paramètres > Séquences** est une [page à liste](settings.md#listes) : les séquences à
gauche, les lignes de la séquence sélectionnée à droite. Chaque modification est appliquée et
écrite aussitôt, et la palette et la keymap suivent sans redémarrage.

```
 Séquences
+- Séquences -------------+  Modification de la séquence 1/2
| Cette installation      |  Nom          Backup
| > Backup        Alt+B   |  Identifiant  backup
|   Worktree et agent     |  Description  Stage de tout, ...
| Ce dépôt                |  Portée       Cette installation
|   Release               |  Raccourci    [ Alt+B ] [ Effacer ]
+-------------------------+  Étapes       Stage all > IA : ... > Commit > Push
| [+ Ajouter v] [Dup] [Suppr]|           [ Modifier les étapes (4) ] [ Lancer maintenant ]
| [Haut] [Bas]            |
+-------------------------+
```

* **+ Ajouter** propose **Séquence vide** et les trois modèles, **À partir d'un modèle >
  Sauvegarde**, **Composer et push** et **Worktree et agent**. Un modèle est un
  point de départ : rien n'est actif tant que vous ne l'ajoutez pas, et ce que vous ajoutez est
  une copie que vous pouvez modifier. Les modèles sont nommés dans la langue de l'interface au
  moment de leur ajout.
* **Dupliquer**, **Supprimer** (demande d'abord), **Haut** / **Bas** ou un glissement réordonnent
  la liste (`Insert`, `Ctrl+D`, `Delete`, `Alt+Up`, `Alt+Down`). L'ordre de
  la liste est l'ordre dans `config.toml`.
* Une ligne qui a un problème est marquée en rouge et dit pourquoi (aucune étape, une étape que cette
  version ne connaît pas, une pause `Jamais` à laquelle manque l'option dont elle a besoin).
  Une séquence avec des problèmes **peut être enregistrée**, les étapes futures inconnues sont conservées
  telles quelles, mais sa ligne de palette reste grisée tant qu'elle n'est pas corrigée.

### Les lignes d'une séquence

| Ligne | Ce qu'elle fait |
|---|---|
| **Nom** | Ce que montre la palette : `Séquence : <nom>`. |
| **Identifiant** | Le nom court utilisé dans `config.toml`, en ligne de commande et comme clé du raccourci (`a-z`, `0-9`, `_`, `-`). Le renommer déplace le raccourci avec lui, dans la même écriture. |
| **Description** | Facultative, pour vous. |
| **Portée** | Voir ci-dessous. |
| **Raccourci** | Un bouton qui montre la touche. `Enter` ou un clic ouvre la même capture de touche que **Clavier et souris** : les conflits et les avertissements du terminal s'affichent pendant que vous appuyez, et une touche qu'une autre action utilise est refusée de la même façon. **Effacer** la retire. La touche est enregistrée sous `[keybindings.sequence] <id>`. |
| **Étapes** | Les étapes dans l'ordre. **Modifier les étapes** les ouvre. |
| **Lancer maintenant** | Lance la séquence dans la fenêtre courante. |

### Portée

Chaque séquence a une portée, enregistrée avec elle :

| Portée | Où elle est enregistrée | Qui la voit |
|---|---|---|
| **Cette installation** (par défaut) | `[[sequences]]` dans `config.toml`, avec cette installation pour propriétaire | Seulement cette installation de GitPoulpi. |
| **Toutes les installations de cet ordinateur** | `[[sequences]]` dans `config.toml`, sans propriétaire | Toutes les installations qui partagent le fichier. |
| **Ce dépôt uniquement** | `.gitpoulpi/sequences.toml` dans le dépôt courant | Quiconque a le dépôt. Elle ne compte qu'une fois que vous l'avez approuvée (voir ci-dessous). |

Changer la portée déplace la définition : choisir **Ce dépôt uniquement**
l'écrit dans le fichier du dépôt et la retire de `config.toml`, et
inversement. Une portée de dépôt demande qu'un dépôt soit ouvert.

### Séquences de dépôt et approbation

Le fichier `.gitpoulpi/sequences.toml` peut être commité, il peut donc venir d'autres
personnes, et une séquence exécute des commandes sur votre machine. Les séquences d'un dépôt ne
sont donc **pas proposées tant que vous ne les approuvez pas**, comme les commandes d'un fichier
de workflow commité.

Quand un dépôt a un tel fichier que personne n'a encore approuvé, un avis apparaît au-dessus
du Log : *Ce dépôt définit 2 séquences* avec **Examiner**, **Approuver** et
**Ignorer**. Il ne bloque jamais rien.

- **Examiner** liste chaque séquence avec chaque étape, sa pause et ses options,
  en lecture seule. **Approuver** y approuve ce qui a été listé ; **Fermer** (ou `Esc`)
  n'approuve rien. Si la liste est plus longue que la fenêtre, Approuver n'y est pas proposé
  (agrandissez la fenêtre) : vous n'approuvez jamais ce que vous n'avez pas pu lire.
- **Approuver** (le bouton, ou *Séquence : Approuver les séquences de ce dépôt* dans la palette)
  approuve le fichier exactement tel qu'il est maintenant.
- **Ignorer** masque l'avis jusqu'à ce que le fichier change.
- Au clavier : la palette a *Séquence : Examiner / Approuver / Ignorer les séquences de ce
  dépôt*.

Ce qui est enregistré est une empreinte des ids, des commandes, des pauses et des
options, conservée dans l'état local du dépôt (jamais dans le dépôt). Si
l'un d'eux change, par exemple après un pull, l'avis revient sous la forme *Les
séquences de ce dépôt ont changé* et elles quittent la palette et les touches
jusqu'à ce que vous les approuviez de nouveau. Renommer une séquence ou modifier sa description
ne change rien. Une séquence que vous écrivez vous-même dans les Paramètres reste approuvée quand
le fichier l'était déjà (ou était vide) avant votre modification.

Une fois approuvées, les séquences de dépôt apparaissent sous la forme `Séquence : <nom>` dans la
palette comme les vôtres et peuvent recevoir un raccourci dans les Paramètres. Une séquence de même
id qu'une des vôtres est un conflit : la vôtre l'emporte et l'autre est signalée
comme doublon.

En ligne de commande : `gitpoulpi sequence list` montre les séquences du fichier
avec `trusted` et un état `repo_trust` (`untrusted`, `changed`, `trusted`) ;
`gitpoulpi sequence trust` (et `untrust`, `--repo PATH` pour un autre
dépôt) est à lancer par une personne après avoir lu `sequence show`.
`sequence run` refuse une séquence de dépôt non approuvée. Les agents IA ne peuvent pas
approuver : il n'existe aucun outil MCP pour cela.

## Étapes

**Modifier les étapes** remplace la liste par les étapes de la séquence (le titre
dit **Étapes de <nom>**) et les lignes par celles de l'étape sélectionnée. **Retour**
(la croix en haut des lignes) ou `Esc` revient aux séquences.

* **+ Ajouter** ouvre **Ajouter une étape** : chaque commande qu'une séquence peut lancer, sous son nom
  de palette (celles de l'IA avec l'étincelle), filtrée à mesure que vous tapez avec la
  correspondance de la palette. Une étape que cette version ne peut pas lancer est grisée avec la
  raison. `Enter` ou un clic l'ajoute à la fin ; `Esc` ou la croix ferme
  la fenêtre.
* **Dupliquer**, **Supprimer**, **Haut** / **Bas** et le glissement fonctionnent comme pour les
  séquences.
* Une commande destructrice (supprimer une branche, abandonner des modifications...) n'est jamais
  proposée : une séquence ne peut pas en lancer.

### Pause

Chaque étape a une **Pause** :

| Pause | Signification |
|---|---|
| **Par défaut** | L'étape ne s'arrête que quand elle a besoin d'une réponse, sur sa boîte de dialogue habituelle (un nom de branche, un message de commit encore vide...). |
| **Toujours** | S'arrêter avant cette étape et demander de continuer. |
| **Jamais** | Ne jamais s'arrêter. Les options de l'étape doivent lui donner tout ce dont elle a besoin : une étape qui demande un nom a besoin que son option **Nom** soit renseignée. |

Une étape qui demande toujours avant de s'exécuter (un force push) ne propose pas
**Jamais** : la ligne dit pourquoi. **Jamais** sur l'étape de message IA est autorisé
comme choix explicite.

### Options

Les lignes sous **Pause** sont les options de l'étape : un texte **Nom** ou **Message**,
une **Branche**, une case à cocher comme **Ouvrir dans un nouvel onglet**, ou une liste déroulante
remplie d'après votre configuration (un **Distant**, un **Profil IA**, une entrée
**Ouvrir dans**). Les textes acceptent les variables `${branch}`, `${date}`, `${time}` et
`${n}` (un numéro qui rend le nom unique). Effacer un texte retire l'option.

## Créer et modifier avec l'IA

Avec l'[IA activée](ai.md#brouillon-de-séquence) (Paramètres > IA), deux entrées apparaissent :
**Ajouter > Créer avec l'IA...** (aussi la ligne de palette *Séquence : Créer avec l'IA...*,
qui ouvre les Paramètres sur cette page avec la boîte de dialogue) et **Modifier avec l'IA...** à côté
de **Lancer maintenant** pour la séquence sélectionnée. Aucune n'existe tant que l'IA est désactivée.

```
+- Créer une séquence avec l'IA ---------------------------- x -+
| Décrivez la séquence voulue                                   |
| +-----------------------------------------------------------+ |
| | stage de tout, message de commit écrit par l'IA,          | |
| | commit, puis push                                         | |
| +-----------------------------------------------------------+ |
| Entrée ajoute une ligne. Ctrl+Entrée envoie. Échap ferme.     |
| [ ] Envoyer le contexte (noms des distants, profils d'IA, ... |
|    Désactivé : seule votre description est envoyée.           |
| L'IA ne fait que proposer. Rien n'est enregistré avant ...    |
| [ Envoyer ] [ Annuler ]                                       |
+---------------------------------------------------------------+
```

Vous écrivez ce que vous voulez avec vos mots ; **Envoyer** (ou `Ctrl+Enter`, `Ctrl+S`)
interroge le modèle, via le même profil, le même indicateur de tâches et le même `Esc` que toute
autre action IA (et la même question « Ce qui est envoyé »). La réponse est une
**proposition** que vous lisez avant que quoi que ce soit n'arrive :

* le nom, l'id, la description et les étapes avec leur pause et leurs options ;
* pour **Modifier avec l'IA**, un diff par rapport à la séquence actuelle : `+` une étape ajoutée,
  `-` une étape retirée, `~` une étape dont la pause ou les options ont changé, `~ Nom : « a »
  -> « b »` pour un champ ;
* **Écarté** : une étape proposée par le modèle qui n'est pas autorisée (inconnue,
  destructrice, une séquence dans une séquence, une pause impossible), avec la
  raison ; les étapes valides sont gardées ;
* les **remarques** du modèle, et un **raccourci suggéré** qui est seulement affiché, avec
  l'indication de si la touche est déjà prise. Il n'est jamais attribué : utilisez ensuite le bouton
  **Raccourci**.

```
+- Modification proposée ----------------------------------- x -+
| + ajouté   - retiré   ~ modifié                               |
| ~ Nom : « Daily » -> « Daily 2 »                              |
| Étapes (3)                                                    |
| + 1. Git : Fetch de tous les distants                         |
|   2. Git : Stager tous les changements                        |
| - Git : Commit...                                             |
| ~ 3. Git : Push : valeurs par défaut -> Pause : Toujours      |
| Écarté                                                        |
| étape 4 `reset_hard` : les étapes destructrices sont interdites |
| [ Accepter ] [ Modifier avant d'enregistrer ] [ Rejeter ] [ Retour ] |
+---------------------------------------------------------------+
```

* **Accepter** l'écrit en une seule sauvegarde (une nouvelle séquence, ou le remplacement de
  celle qui est modifiée, qui garde son id, sa portée et sa touche) et la sélectionne dans la liste.
* **Modifier avant d'enregistrer** fait de même et ouvre ses étapes pour que vous puissiez ajuster
  les lignes (cette page enregistre chaque modification aussitôt, la séquence est donc déjà
  enregistrée).
* **Rejeter** n'écrit rien. **Retour** revient à la description pour la renvoyer.
* **Accepter** est indisponible quand il n'y a rien à accepter : aucune étape utilisable, un
  problème que la page signalerait, ou une modification qui ne change rien.

Rien n'est écrit avant **Accepter**, et une séquence n'est jamais lancée d'ici.
Fermer la boîte de dialogue (la croix, `Esc`, un clic à l'extérieur) demande « Abandonner les modifications ? »
tant qu'une description est saisie ou qu'une proposition est ouverte ; fermer les Paramètres le demande aussi.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<l'id de cette installation>"

[[sequences.steps]]
run = "stage_all"

[[sequences.steps]]
run = "ai_generate_commit_message"
pause = "always"

[[sequences.steps]]
run = "commit"

[[sequences.steps]]
run = "push"

[keybindings.sequence]
backup = "Alt+B"
```

Les clés que cette version ne connaît pas, et les étapes qu'elle ne connaît pas, sont conservées quand
la page enregistre.

## Lancer une séquence

Une séquence est une liste nommée d'étapes, chacune étant une commande de la palette (stager tout, écrire le message de commit avec l'IA, commit, push...). Lancez-la depuis la palette (`Séquence : <nom>`), avec la touche que vous lui avez donnée dans Paramètres > Clavier et souris (`[keybindings.sequence]` dans `tui.toml`), ou en ligne de commande (`gitpoulpi sequence run`).

Pendant son exécution, une bande au-dessus du Log montre `<nom> k/n`, l'étape, et ce qu'elle attend. L'exécution apparaît aussi dans le panneau des tâches sous la forme `Séquence <nom> k/n : <étape>`. Une seule séquence s'exécute à la fois ; en démarrer une deuxième dit laquelle est en cours.

**Où elle s'arrête pour demander.** Chaque étape a un réglage `pause` (`default`, `always`, `never`) :

| Étape | Par défaut | `pause = "never"` |
|---|---|---|
| Message de commit IA | le message arrive dans le champ de message ; la bande dit « Relisez le message, puis Continuer » | continue aussitôt |
| Commit | un message vide arrête toujours : le champ prend le clavier, et votre commit (ou Continuer) poursuit | indisponible quand il n'y a pas de message à commiter |
| Composer des commits (IA) | la revue s'ouvre ; Appliquer poursuit, la fermer annule | applique la proposition telle quelle |
| Créer un worktree | la boîte de dialogue s'ouvre avec le nom rempli | le crée depuis `options.name` sans la boîte |
| Push | quand `git.push_behavior = ask`, le sélecteur « Push vers... » s'ouvre | pousse vers `options.remote` |
| Force push avec bail, stage de fichiers qui ressemblent à des secrets | demande toujours | non autorisé |

`pause = "always"` sur n'importe quelle étape affiche « Suite : <étape> » avec **Continuer** et **Annuler** avant son exécution. Si le push est rejeté parce que la branche a divergé, la boîte de dialogue habituelle s'ouvre et la bande dit « Push refusé : choisissez dans la boîte de dialogue » ; faire un pull puis un push (ou un force push avec bail) poursuit l'exécution, fermer la boîte l'annule.

**Continuer** est le bouton de la bande, `Séquence : Continuer` dans la palette, ou la touche liée à `sequence.continue`. **Annuler** est la croix de fermeture de la bande, `Séquence : Annuler` (`sequence.cancel`), le panneau des tâches, ou `Esc` quand rien d'autre n'en veut (une boîte de dialogue ouverte se ferme d'abord, ce qui annule aussi). Annuler garde les étapes déjà exécutées et le dit : « <nom> annulée après l'étape k/n (<étape>) ; les étapes précédentes sont conservées ».

**Quand une étape échoue**, l'exécution s'arrête là : la ligne d'état nomme la séquence, l'étape et l'erreur, la bande devient rouge avec **Détails** (l'entrée du Journal d'activité, qui liste les options de l'étape) et une croix de fermeture, et les étapes précédentes restent faites.

**Worktrees.** Une étape de création de worktree avec `open_tab = true` déplace l'exécution dans l'onglet du nouveau worktree : les étapes suivantes (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) y agissent. Si un autre onglet est affiché, la bande dit « en attente de l'onglet <nom> » avec **Y aller**. Fermer cet onglet annule l'exécution. Une exécution ne survit pas à la fermeture de GitPoulpi.

### Un agent qui propose une exécution

Avec *Laisser les agents IA proposer des modifications que je confirme* activé (`[mcp]
allow_ui_propose`), un agent peut demander à votre fenêtre de lancer une séquence
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). Un bandeau apparaît :
*✦ Proposé par <agent> : lancer la séquence <nom>* avec **Lancer** et **Ignorer**.
Rien ne s'exécute tant que vous n'appuyez pas sur Lancer (cliquez dessus, ou `Ctrl+N` puis `Enter`) ;
Ignorer, `Esc` (quand rien d'autre n'en veut) ou *Agent : Ignorer la vue en attente* dans la palette
l'abandonne. L'exécution se déroule alors exactement comme si vous l'aviez démarrée,
avec des pauses là où la séquence le prévoit. La proposition est refusée quand la séquence
n'existe pas, est invalide, ou est une séquence de dépôt que vous n'avez pas approuvée.

## Annuler une exécution

Chaque étape qui modifie le dépôt (stage, commit, tag, branche, stash, worktree...) laisse **sa propre entrée d'annulation ordinaire**, marquée de l'exécution, si bien que `Ctrl+Z` annule toujours une étape à la fois et que l'[Historique d'annulation](undo-redo.md#lhistorique-dannulation) y montre `⟳ <nom>`. Pour reprendre toute l'exécution, il y a **Annuler toute la séquence** : le bouton de l'Historique d'annulation (`x`), la ligne de palette `Édition : Annuler toute la séquence`, ou `gitpoulpi undo --sequence`. Elle annule les entrées de l'exécution de la plus récente à la plus ancienne, chacune avec ses propres vérifications et questions (un commit déjà poussé demande d'abord), s'arrête au premier refus en nommant l'étape, et laisse les plus anciennes dans le journal. Le message IA et un push ne laissent aucune entrée : un push n'est jamais annulé. Seul le journal du dépôt courant est touché ; les étapes faites dans un worktree s'annulent depuis ce dépôt. Rétablir reste entrée par entrée. Détails : [Exécutions de séquences](undo-redo.md#exécutions-de-séquences).
