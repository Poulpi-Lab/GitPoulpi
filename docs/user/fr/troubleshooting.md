[English](../troubleshooting.md) · [Čeština](../cs/troubleshooting.md) · [Deutsch](../de/troubleshooting.md) · [Español](../es/troubleshooting.md) · **Français** · [Italiano](../it/troubleshooting.md) · [日本語](../ja/troubleshooting.md) · [한국어](../ko/troubleshooting.md) · [Nederlands](../nl/troubleshooting.md) · [Polski](../pl/troubleshooting.md) · [Português (Brasil)](../pt-BR/troubleshooting.md) · [Português (Portugal)](../pt-PT/troubleshooting.md) · [Русский](../ru/troubleshooting.md) · [Svenska](../sv/troubleshooting.md) · [Türkçe](../tr/troubleshooting.md) · [Українська](../uk/troubleshooting.md) · [简体中文](../zh-CN/troubleshooting.md)

# Dépannage et FAQ

Commencez par les réponses rapides ci-dessous ; les sections plus longues couvrent Git
LFS et le rendu du graphe. Si rien n'aide, ouvrez un ticket sur le dépôt GitHub avec
votre OS, votre terminal, `gitpoulpi --version`, et ce que vous avez fait.

## Quand quelque chose échoue

Une opération qui échoue n'affiche ni fenêtre ni carte encadrée : la **ligne d'état** le dit, en rouge, avec le titre et la raison (*erreur : Fetch : échec. Le dépôt distant est injoignable.*), et le message reste jusqu'à ce qu'un autre le remplace. Le bouton Activité, à son extrémité droite, compte les erreurs que vous n'avez pas encore consultées. Cliquez sur le message rouge, ou lancez `Journal d'activité : Ouvrir la dernière erreur` dans la palette, pour ouvrir le Journal d'activité sur cette entrée. Tout échec que vous avez déclenché fonctionne ainsi, pas seulement les opérations git :

- **Les opérations git** : un fetch, un push, un pull, un checkout, un merge, un rebase, un commit, un stash...
- **Un clonage** (et la création d'un nouveau dépôt) : « Échec du clonage ».
- **Un appel à GitHub ou GitLab** : les listes de dépôts des sélecteurs et de la boîte de dialogue Cloner, la création d'une pull request ou d'un ticket, la vérification d'un jeton ou d'une clé SSH sur la page Intégrations : « Échec de la requête à l'hébergeur ».
- **Un programme externe** : l'éditeur, l'outil de diff ou de merge, l'explorateur de fichiers, le navigateur, le terminal ou le lanceur, un éditeur d'images, les boutons *Tester* des Paramètres : « Échec de l'outil externe ».
- **Une requête IA** (un message de commit, une explication, une suggestion pour un conflit...) : « Échec de la requête IA ».
- **Un fichier qui n'a pas pu être écrit** : l'Enregistrement de l'éditeur intégré, l'éditeur de merge, un paramètre qui n'a pas pu être enregistré : « Échec de l'enregistrement ».

Le message rouge est sur la ligne d'état quelle que soit la vue au premier plan : un onglet de dépôt, Accueil, le Radar, les Espaces de travail, les Paramètres (sur la dernière ligne de l'écran), sous le Journal d'activité ou une boîte de dialogue. Là où la ligne d'état est recouverte (Accueil, Radar, Espaces de travail, Paramètres), le message s'écrit sur sa dernière ligne jusqu'à ce que vous ayez consulté l'erreur. Il vous suit d'un onglet à l'autre. Une opération encore en cours quand vous changez d'onglet n'est pas suivie : ouvrez le Journal d'activité pour voir comment elle s'est terminée.

Les listes des sections PULL REQUESTS et TICKETS qui ne peuvent pas être actualisées gardent à la place leur propre ligne dans la section (elles s'actualisent d'elles-mêmes, un message toutes les quelques secondes ne ferait que du bruit). Une boîte de dialogue encore ouverte (Cloner, Créer une pull request...) garde aussi son propre message d'une ligne, pour que vous puissiez corriger le champ et réessayer.

L'entrée du Journal d'activité contient tout ce que montrait l'ancienne fenêtre :

- **Titre et raison.** « Fetch : échec », puis une phrase qui dit pourquoi en mots simples, puis quoi faire ensuite. Reconnus : le dépôt distant a des commits que vous n'avez pas (*pull d'abord*), identifiants refusés (*vérifiez Paramètres > Intégrations ou votre clé SSH*), un arrêt sur des conflits, des modifications locales qui seraient écrasées (*commitez, stashez ou abandonnez*), le dépôt distant injoignable, et un autre processus git qui tient un verrou sur le dépôt. Pour tout le reste, l'entrée montre la première ligne de git lui-même. Un clonage ou un appel à l'hébergeur qui échoue de la même façon (identifiants refusés, hébergeur injoignable) est formulé de la même manière ; les autres échecs reçoivent une phrase sur ce qui a échoué (« Le dépôt n'a pas pu être cloné. », « Le programme n'a pas pu être lancé. »).
- **Les détails**, affichés quand l'entrée est ouverte (`Enter`, ou le clic qui l'a ouverte) : l'opération, la commande git exécutée, son code de sortie et les dernières lignes de ce qu'elle a affiché (pour la bibliothèque git intégrée, sa classe d'erreur et son code). Les identifiants, jetons et en-têtes `Authorization` sont masqués avant tout affichage, journalisation ou copie.
- **Copier** (le bouton, ou `c`) place sur le presse-papiers un bloc de texte fait pour être collé dans une discussion avec quelqu'un, ou une IA, qui peut l'expliquer : la version de GitPoulpi, le système, le terminal, le backend, l'opération en échec avec ses détails et la version de git.
- **Un bouton pour la prochaine étape**, quand il y en a une, à côté de Copier (ou `r`) : *Pull d'abord* (un push que le dépôt distant rejetterait), *Stash puis réessayer* (des modifications locales gênent un checkout, un merge, un rebase ou un pull), *Réessayer* (le dépôt distant était injoignable), *Ouvrir les Intégrations* (identifiants). Il exécute l'étape et ferme la fenêtre. Pour l'échec d'un autre dépôt que l'onglet au premier plan, seul *Ouvrir les Intégrations* est proposé : une nouvelle tentative s'exécuterait dans le mauvais dépôt.
- Sans la souris, la palette de commandes propose `Journal d'activité : Ouvrir la dernière erreur`, `Journal d'activité : Copier la dernière erreur` et `Journal d'activité : Lancer l'action suggérée` (le bouton de l'entrée, pour l'erreur que montre la ligne d'état).

Un push que le dépôt distant a refusé parce qu'il a avancé garde sa propre boîte de dialogue (pull d'abord, force push with lease, annuler) ; les faits sont quand même dans le Journal d'activité.

### Le Journal d'activité

Le bouton **Activité** à l'extrémité droite de la ligne d'état (un nombre dessus compte les erreurs que vous n'avez pas encore consultées), `Ctrl+N` puis `Shift+A`, et `Affichage : Journal d'activité` dans la palette ouvrent la fenêtre. `Ctrl+N` affiche un badge sur chaque onglet, zone, champ, case à cocher et bouton de la fenêtre (et sur la première entrée de la liste) ; appuyer sur le caractère fait ce que ferait un clic dessus. Deux onglets, **Opérations Git** et **Application**, que l'on change avec `Tab`, un clic, `g` (Opérations Git) ou `p` (Application), et une zone **Périmètre** dessous qui s'applique aux deux (voir plus bas).

- **Application** : les lignes de journal du programme lui-même, celles de la fenêtre et du démon en arrière-plan, la plus récente d'abord. La zone **Niveau** (une liste déroulante : Erreur, Avertissement, Info, Débogage, Trace ; `l` l'ouvre) dit combien en montrer ; demander plus de détail que ce qui est enregistré relève aussi le niveau d'enregistrement (et l'enregistre dans `log.level`). Dessous, **Afficher** a une case par niveau (Erreur, Avertissement, Info, Débogage, Trace, chacune avec le nombre de lignes qu'elle contient) pour masquer ou afficher les lignes de ce niveau, par exemple pour ne voir que les erreurs, et **Tout** / **Rien** cochent ou décochent toutes les cases. Les touches `1` à `5` basculent les niveaux dans cet ordre, `a` est Tout, `n` est Rien ; la palette propose `Journal d'activité : Afficher ou masquer le niveau ...`, `Journal d'activité : Afficher tous les niveaux` et `Journal d'activité : N'afficher aucun niveau`. Les niveaux plus détaillés que le choix de la zone Niveau sont grisés (rien n'est enregistré pour eux ; survolez-en un pour savoir pourquoi). Les cases cochées sont mémorisées jusqu'à la fermeture de l'application, et seul l'onglet Application les a. **Filtre** (`/`) garde les lignes qui contiennent votre texte. **Tout copier (filtré)** copie ce qui est affiché (les cases et le filtre s'appliquent tous deux) ; **Ouvrir le dossier des journaux** ouvre le dossier dans votre gestionnaire de fichiers. **Effacer le journal** (`x`, ou la commande de palette `Journal d'activité : Effacer le journal de l'application`) demande d'abord confirmation, puis vide les fichiers du journal de tous les jours conservés et la liste à l'écran ; c'est irréversible et absent de l'historique d'annulation. Une longue ligne n'est jamais coupée : voir « Longues lignes » ci-dessous.

  ![Le Journal d'activité, onglet Application : la zone Niveau, les cases Afficher par niveau avec Tout et Rien, le filtre et les lignes du journal](../images/activity-log.png)

- **Opérations Git** (l'onglet sur lequel la fenêtre s'ouvre) : les opérations, la plus récente d'abord, chacune avec l'heure à laquelle elle a eu lieu et sa durée (`Push - 123 ms`) : ouverture du dépôt, chargement du graphe des commits, fetch, pull, push, commit, checkout, merge, rebase, stash, clone. Les échecs, les messages d'état, un clone qui a échoué et le Fetch all d'un espace de travail y sont listés aussi ; `Enter` ou un clic en ouvre un (sa raison, l'étape suivante, la commande, le code de sortie et la sortie), `c` ou son bouton Copier le copie, `r` ou son bouton lance l'étape suggérée. `Tab` (ou `p`) passe à l'onglet Application ; `Left`/`Right` font défiler latéralement les longues lignes.

  ![Le Journal d'activité, onglet Opérations Git : chaque opération avec son heure et sa durée, un fetch en échec en rouge](../images/activity-log-repository.png)

- **Quelle action a produit une erreur.** Chaque échec dit ce qui l'a produit, dans l'entrée, dans le journal de l'application et dans le rapport copié. L'opération est nommée comme le ferait une personne (`Remove remote`, `Fetch all remotes`, `Hosting request: create pull request`, `Token command`) avec ses principaux arguments (`remote=cm6`, une branche, un chemin ; jamais le corps d'un message, et les URL et jetons sont masqués comme partout ailleurs), puis le dépôt, qui l'a demandée (`TUI`, `CLI` ou `MCP`) et l'identifiant d'opération, le numéro que le démon a donné à la requête (et son numéro de tâche pour un fetch, un push ou un pull). Dans l'onglet Opérations Git, l'en-tête de l'entrée se lit `Remove remote failed (remote=cm6) - ... [CLI #12]` et ses détails commencent par `Opération :`, `Dépôt :`, `Demandée par :` et `Identifiant de l'opération :` ; une opération sans titre propre est titrée par ce qu'elle était (`Remove remote : échec`). Dans l'onglet Application, la ligne d'échec commence par les mêmes mots et une étiquette : `Remove remote failed (remote=cm6) [op #12, CLI, repo name]: backend error: no such remote: cm6`. Une telle ligne ouvre son opération : cliquez dessus, ou sélectionnez-la et appuyez sur `Enter` (la commande de palette `Journal d'activité : Afficher l'opération de cette ligne`). `]` et `[` sélectionnent la ligne d'échec suivante et la précédente (`Journal d'activité : Opération en échec suivante` / `Journal d'activité : Opération en échec précédente`) ; avec une ligne sélectionnée, les flèches déplacent la sélection au lieu de faire défiler. L'entrée qui s'ouvre est celle de la fenêtre quand c'est elle qui a lancé l'opération ; pour un échec de la CLI ou d'un agent sur la même session, la fenêtre fabrique une entrée à partir de la ligne. Le rapport d'erreur copié contient les lignes `operation:`, `arguments:`, `repository:`, `requested by:` et `operation id:`, et peut donc être collé tel quel. Depuis un script, l'erreur `--json` d'une commande en échec ne change pas ; la réponse du démon à une requête en échec porte les mêmes faits dans un champ `context` ajouté.

- **Longues lignes** (les deux onglets) : un en-tête ou une ligne de détail plus large que la fenêtre n'est jamais coupé. Une barre de défilement horizontale apparaît sous la liste ; faites défiler latéralement avec `Left` / `Right` (réassignables en `activity_log.scroll_left` / `activity_log.scroll_right`), `Shift` + la molette ou une molette horizontale, `Journal d'activité : Défiler à gauche` / `Journal d'activité : Défiler à droite` dans la palette, ou la barre (glissez le curseur, ou appuyez sur la piste pour amener le curseur sous le pointeur). `Home` revient au début, et changer d'onglet place l'autre onglet au bord gauche. Un texte renvoyé à la ligne (un résumé, une action suggérée) reste renvoyé à la ligne.

- **Périmètre** (une liste déroulante sous les onglets, `s` l'ouvre, ou `Journal d'activité : Choisir le périmètre` dans la palette) : ce dont parlent les deux onglets. Les lignes sont **Ce dépôt (nom)** (le dépôt de l'onglet depuis lequel vous avez ouvert le journal ; absent depuis Accueil, qui n'en a pas), **Tous les dépôts**, chaque **espace de travail** et chaque dépôt que la fenêtre connaît. Depuis un onglet de dépôt, le journal s'ouvre sur *Ce dépôt*, depuis Accueil (le bouton Activité, `Ctrl+N` puis `Shift+A`) sur *Tous les dépôts* ; `Ctrl+N` puis `Shift+G` et `Affichage : Journal d'activité global` dans la palette s'ouvrent toujours sur *Tous les dépôts*, tout comme un clic sur l'erreur rouge d'un échec dont le dépôt n'est pas l'onglet au premier plan (un clone qui a échoué, par exemple). Votre dernier choix est mémorisé pour la session, séparément pour les deux façons de l'ouvrir. Sur l'onglet Application, une ligne appartient à un dépôt quand le programme l'a écrite à son sujet (un push en échec, la durée d'une opération) ; sous *Tous les dépôts*, chacune de ces lignes dit de quel dépôt elle est (`[nom]`). Une ligne qui ne concerne aucun dépôt (démarrage, paramètres, le démon lui-même) et une entrée qui n'en a aucun (un enregistrement qui a échoué) sont **globales à l'application** : elles s'affichent sous tous les périmètres, marquées `[app-wide]` quand le périmètre est plus étroit que *Tous les dépôts*, si bien qu'un filtre ne masque jamais un échec. La palette propose aussi `Journal d'activité : Opérations Git` et `Journal d'activité : Journal de l'application` pour les onglets.

- **Copier le diagnostic** (un bouton sur les deux onglets, `d`, toujours sur tout le journal quel que soit le périmètre ; et `Affichage : Copier le diagnostic` dans la palette) copie la version, le système, le terminal et le backend, le dernier échec avec ses détails et les 200 dernières lignes du journal au niveau choisi : le bloc à donner à une IA quand quelque chose ne va pas.

Les durées sont mesurées par la fenêtre (de la requête à la réponse du démon, aller-retour compris) ; ce que fait la CLI ou MCP n'est donc pas chronométré. `log.timings` (activé par défaut) désactive les durées, `log.extended` (désactivé par défaut) liste aussi le travail en arrière-plan, l'actualisation périodique et l'auto-fetch ; les deux se trouvent dans Paramètres > Général près des autres paramètres du journal, et chaque opération est de toute façon écrite dans le journal de l'application (`info` pour ce que vous avez demandé, `debug` pour l'arrière-plan).

Les fichiers journaux s'appellent `gitpoulpi-YYYY-MM-DD.log`, sont conservés une semaine par défaut et dix ans au plus (`log.retention_days`, de 1 à 3650), élagués au démarrage de GitPoulpi, et renouvelés à `log.max_file_mb` (5 Mio). Où ils se trouvent : [référence de configuration](config-reference.md#log-le-journal-de-lapplication). Depuis un script, l'erreur `--json` d'une commande échouée porte les mêmes faits sous `detail` ; voir la [référence CLI](cli-reference.md).

Un clic sur le message rouge ouvre le journal sur *Tous les dépôts*, avec cette entrée ouverte. La liste des opérations Git est celle de la session ; l'onglet Application garde le journal du programme d'un lancement à l'autre.

**Tâches en arrière-plan.** Un fetch, push, pull, clone ou transfert LFS, le Fetch all / Pull all / Clone missing d'un espace de travail, une requête IA et une pull request ou un ticket en cours d'envoi sont des tâches : le bord droit de la ligne d'état montre celle qui tourne avec un `[x]` qui l'annule, et un clic sur son libellé (ou `Ctrl+N` puis `Shift+J`, la commande `Tâches : afficher les tâches en arrière-plan`) ouvre le panneau avec un bouton Annuler par tâche (aussi `c` ou `Suppr`). Un clone tourne en arrière-plan ; une requête IA et un parcours d'espace de travail survivent à un changement d'onglet et sont appliqués quand vous revenez à leur dépôt. Une tâche au-delà du point où l'arrêter n'est plus sûr est grisée avec la raison ; une tâche annulée laisse le dépôt comme il était.

## Réponses rapides

**La barre d'outils et le panneau latéral montrent des carrés ou des points
d'interrogation à la place des icônes.** Utilisez une Nerd Font ou toute police avec
couverture des emoji dans votre terminal. Le graphe et le texte fonctionnent quand même
sans elles.

**Le graphe de commits est fait de caractères, pas d'images.** C'est le repli
automatique : les images demandent un terminal qui parle le protocole graphique kitty
(kitty, Ghostty, herdr avec les graphiques kitty activés) ou le protocole iTerm2
(WezTerm, Rio, mintty), ou sixel (Windows Terminal, foot, Tabby, Konsole, et tmux quand le terminal derrière lui accepte sixel). La console Windows, zellij et screen reçoivent des
caractères. Voir la section sur le graphe plus bas pour forcer l'un ou l'autre, [Choisir un terminal](terminals.md) et [Multiplexeurs](multiplexers.md).

**La souris ne fait rien, ou sélectionner du texte à la souris ne marche pas.**
GitPoulpi capture la souris pour que les clics l'atteignent. Pour sélectionner du texte
avec le terminal lui-même, maintenez `Shift` pendant que vous glissez (dans la plupart
des terminaux). Vous pouvez aussi sélectionner dans le diff (glisser, puis `y`). Pour
désactiver la capture de la souris, utilisez *Paramètres > Clavier et souris > Souris
activée* (redémarrage nécessaire). Le clavier atteint chaque commande.

**Un raccourci ne marche pas.** Votre terminal, tmux ou un script AutoHotkey peut
prendre la touche avant que GitPoulpi ne la voie (`Ctrl+,` arrive souvent comme un
simple `,` dans les terminaux Linux et macOS ; `Ctrl+Shift+N` ressemble à `Ctrl+N`).
Utilisez les alternatives (`Ctrl+N` puis `,` ouvre les Paramètres ; `Ctrl+P` atteint chaque
commande), ou réaffectez l'enchaînement : voir [Paramètres](settings.md#clavier-et-souris).

**`Ctrl+Tab` ne change pas d'onglet.** kitty, WezTerm, Ghostty et Windows
Terminal gardent `Ctrl+Tab` et `Ctrl+Shift+Tab` pour leurs propres onglets : la
touche n'arrive donc jamais à GitPoulpi. `Ctrl+PageDown` / `Ctrl+PageUp`
marchent toujours. Pour donner ce couple à GitPoulpi, libérez-le dans la
configuration du terminal, puis redémarrez le terminal (ou rechargez sa
configuration) :

- kitty, dans `kitty.conf` (un `map` sans action supprime l'affectation par
  défaut ; avec `--when-focus-on` et un critère de fenêtre comme `title:...`,
  kitty peut la supprimer seulement tant qu'une fenêtre correspondante a le
  focus, voir la documentation des mappings de kitty) :

  ```
  map ctrl+tab
  map ctrl+shift+tab
  ```

- WezTerm, dans `wezterm.lua` :

  ```lua
  config.keys = {
    { key = 'Tab', mods = 'CTRL', action = wezterm.action.DisableDefaultAssignment },
    { key = 'Tab', mods = 'CTRL|SHIFT', action = wezterm.action.DisableDefaultAssignment },
  }
  ```

- Ghostty, dans son fichier de configuration :

  ```
  keybind = ctrl+tab=unbind
  keybind = ctrl+shift+tab=unbind
  ```

- Windows Terminal, dans `settings.json` (`"id": null` supprime l'affectation
  par défaut) :

  ```json
  "keybindings": [
    { "id": null, "keys": "ctrl+tab" },
    { "id": null, "keys": "ctrl+shift+tab" }
  ]
  ```

  (Les anciens fichiers de paramètres listent `nextTab` / `prevTab` sous
  `actions` avec ces touches : supprimez-y ces touches.)

Le terminal perd alors son propre changement d'onglet par `Ctrl+Tab`. Un
terminal qui ne peut envoyer cette touche que par le protocole clavier kitty
a besoin qu'il soit activé (c'est le cas par défaut, voir
[Raccourcis clavier](keybindings.md#touches-que-le-terminal-ne-peut-pas-envoyer-et-le-protocole-clavier-kitty)).

**Je ne me souviens pas d'un raccourci.** Appuyez sur `Ctrl+P` et tapez ce que vous
voulez ; chaque résultat montre son raccourci. `?` liste tout.

**Une option est grisée dans les Paramètres.** Elle dépend d'une autre option. La ligne dit
laquelle (par exemple, *Force de la teinte* demande que *Fond des voies du graphe* soit
sur *Teinte de ligne*).

**« Auto-stash en attente » au-dessus du Log.** GitPoulpi a stashé vos modifications pour
lancer une opération et n'a pas pu les remettre. Utilisez **Pop maintenant**, **Voir le
stash**, **Repartir de zéro** (annule l'opération et remet vos modifications) ou **Abandonner**
du bandeau ; voir [Auto-stash](log-and-history.md#auto-stash).

**Un agent dit `no_viewer` ou `deferred`.** `no_viewer` : aucune fenêtre GitPoulpi n'est
ouverte pour ce dépôt (ou *Laisser les agents IA piloter la vue* est désactivé).
`deferred` : la fenêtre montre une pastille ; `Ctrl+N Enter` l'accepte. Voir
[MCP et agents](mcp-and-agents.md#montrer-des-choses-à-lutilisateur).

**J'ai ouvert GitPoulpi et je vois une fenêtre vide ou `log error: backend error`.**
Vérifiez que vous l'avez lancé dans un dépôt Git (sinon vous arrivez sur l'onglet
Accueil : utilisez **Parcourir**, **Cloner** ou **Init**). Lancez ensuite `gitpoulpi
daemon status` ; `gitpoulpi daemon restart` redémarre le processus d'arrière-plan de ce
dépôt. Appuyez sur `r` pour actualiser.

**Deux fenêtres sur le même dépôt se comportent bizarrement.** Elles partagent
volontairement un démon par dépôt, si bien qu'une modification dans l'une apparaît dans
l'autre. Si un démon semble bloqué, `gitpoulpi daemon stop --all` arrête le démon de
chaque session ; la fenêtre suivante en démarre un neuf.

**Un commit, un pull ou un push demande un mot de passe, ou échoue à s'authentifier.**
GitPoulpi utilise l'assistant d'identifiants et le ssh-agent de votre `git` pour le
réseau. Assurez-vous d'abord que `git fetch` fonctionne dans un terminal simple. Les
jetons GitHub/GitLab pour les pull requests sont distincts et facultatifs : voir
[Hébergeurs et suivi de tickets](hosting-integration.md).

**Un fetch, un pull ou un push échoue avec « The configured credential helper failed », « The SSH host key of the remote is not known yet » ou une connexion refusée, et son entrée du Journal d'activité a un bouton « SSH et identifiants... ».**
L'échec vient de la façon dont ce dépôt se connecte. Le bouton ouvre
Paramètres > SSH et identifiants, où vous choisissez une autre source pour ce
dépôt : les assistants propres à git en ignorant ceux du dépôt, un compte
d'intégration, un fichier de clé (avec un fichier known_hosts), ou un assistant ou
une commande de jeton de votre cru ; **Tester la connexion** vérifie ensuite chaque
distant. Pour un assistant qui n'a pas pu s'exécuter, `git config --show-origin
--get-all credential.helper` montre où il est défini.

**« GitPoulpi did not send the account token to ... ».** Le distant du dépôt
pointe vers un serveur qui n'est pas l'adresse à laquelle le compte choisi est
lié (souvent une réécriture `url.<x>.insteadOf` dans la configuration du dépôt
lui-même), donc le jeton a été retenu. Corrigez le distant ou choisissez une autre
source sur la page.

**« This repository's token command failed », « ... changed since you confirmed it », « This repository's sign-in cannot be used ».**
Les choix de la page ne peuvent pas être appliqués : la commande de jeton n'a
affiché aucun jeton (ses dernières lignes d'erreur sont citées, secrets retirés),
un script qu'elle exécute a changé depuis que vous l'avez confirmée (confirmez de
nouveau la commande sur la page), ou le choix désigne un fichier de clé ou un
compte qui n'existe plus. Chaque lecture a le bouton **SSH et identifiants...**.

**OpenSSH dit « bad permissions » pour un fichier de clé.** D'autres comptes
peuvent lire la clé. Dans Paramètres > SSH et identifiants, la ligne **Fichier de
clé privée** montre un avertissement avec **Limiter à mon compte**.

**« staging... » reste dans la ligne d'état.** Une actualisation en arrière-plan tourne
encore ; il disparaît quand la liste des fichiers se met à jour. `r` force une
actualisation.

**Un très gros dépôt est lent.** Baissez *Commits initiaux dans le graphe* sur la page
Paramètres > Journal et graphe (le Log en charge plus quand vous défilez si *Chargement
progressif* est activé) ; masquez la colonne *Modifs* du Log et désactivez les avatars ;
désactivez *Annulation activée* si vous n'avez pas besoin d'annuler (aucune comptabilité
du tout sur le stage et les commits). `GITPOULPI_TIMING=1` écrit les durées par requête
dans `gitpoulpi-timing.log` dans le dossier temporaire, utile à joindre à un rapport de
performance.

**Les boutons IA manquent ou sont gris.** L'IA est désactivée par défaut ; voir
[Assistance IA](ai.md#dépannage).

**GitPoulpi s'est fermé tout seul.** Chaque sortie est enregistrée : cherchez
`gitpoulpi-tui-exit.log` (et `gitpoulpi-tui-panic.txt` après un plantage) dans le dossier
de configuration (`gitpoulpi config path` affiche le fichier de paramètres, qui s'y trouve),
et joignez-le à un ticket.

**Où sont mes paramètres ?** `gitpoulpi config path`. L'état par dépôt est dans
`.gitpoulpi/repo-state.toml` à l'intérieur du dépôt.

**GitPoulpi envoie-t-il quelque chose quelque part ?** Seulement ce que vous demandez :
des opérations git vers vos distants, des requêtes d'hébergeur après avoir connecté un
compte, des requêtes IA après avoir appuyé sur un bouton IA, et, si vous activez les
avatars, un hash des e-mails d'auteurs vers Gravatar. Il n'y a aucune télémétrie. Voir
[Assistance IA](ai.md#confidentialité).

## Toute écriture échoue : « Unable to create index.lock »

Une commande git morte (un plantage, un terminal tué) peut laisser `.git/index.lock`, et
alors chaque modification du dépôt échoue. Quand le fichier a plus de 10 secondes, GitPoulpi
affiche une bande au-dessus du Log : **Supprimer index.lock** (palette : *Dépôt : Supprimer l'index.lock périmé* ;
CLI : `gitpoulpi index-lock remove`). Elle demande d'abord et ne supprime que ce
fichier. GitPoulpi ne peut pas savoir de façon fiable si un autre programme git tourne
encore, assurez-vous donc qu'aucun ne tourne (un IDE, un autre terminal) avant de
confirmer. Dans un worktree lié, le verrou est celui du worktree, dans
`.git/worktrees/<name>`.

## « git am » s'est arrêté et Abandonner le rebase ne fait rien

Un `am` (application de patches depuis une boîte mail) n'est pas un rebase. Un `git am`
arrêté a sa propre bande, **Applying patches (git am)**, avec Résoudre les conflits,
Continuer, Passer et Annuler (palette : *git am : Continuer / Ignorer ce patch / Abandonner* ;
CLI : `conflicts am-continue|am-skip|am-abort`). Annuler demande d'abord, abandonne les
patches appliqués jusque-là et remet la branche où elle était. Ils ne sont pas
annulables, comme les boutons du rebase.

## Annuler est grisé et dit « Impossible d'annuler ... » (entrée bloquée)

L'entrée la plus récente du journal est bloquée : le dépôt a évolué en dehors de GitPoulpi,
ou l'entrée a été interrompue. Survolez Annuler pour la raison ; si elle est bloquée pour
de bon, l'infobulle renvoie à **Édition : Oublier l'annulation bloquée** (palette) ou à
`gitpoulpi undo-forget`. Cela demande d'abord et ne retire que l'entrée du journal ; le
dépôt n'est pas touché. Voir [Annuler et rétablir](undo-redo.md).

## Les gros fichiers s'affichent en trois lignes de texte (« les fichiers LFS apparaissent comme des pointeurs »)

Un fichier qui devrait être une image, un modèle ou un gros binaire contient `version https://git-lfs.github.com/spec/v1` et une ligne `oid sha256:...` : c'est un **pointeur** Git LFS, et le vrai contenu n'a pas été téléchargé (ou les filtres LFS ne sont pas installés).

- Regardez le volet Status : un dépôt qui utilise LFS alors que `git-lfs` manque affiche un bandeau ambre. Installez le programme (https://git-lfs.com ; Windows : fourni avec Git for Windows ou `winget install GitHub.GitLFS` ; Debian/Ubuntu : `apt install git-lfs` ; macOS : `brew install git-lfs`) et rouvrez ou appuyez sur `r`.
- `gitpoulpi lfs status` dit si le programme est installé, si les filtres et le hook sont en place et quels motifs sont suivis ; `gitpoulpi lfs files` liste les fichiers pointeurs et lesquels sont `local`.
- Avec `git-lfs` installé, lancez **`LFS : Pull des objets`** dans la palette (ou `gitpoulpi lfs pull`) : cela télécharge les objets et remplace les pointeurs. Cela installe d'abord les filtres LFS de ce dépôt s'il n'y en a pas.
- Si le téléchargement échoue, le message nomme la raison (identifiants du distant, un serveur qui n'a plus l'objet). GitPoulpi n'a besoin d'aucun compte propre : LFS utilise les identifiants habituels de votre distant (assistant d'identifiants ou ssh-agent).
- Le clonage d'un dépôt LFS n'échoue jamais à cause de LFS ; si l'étape de téléchargement n'a pas pu s'exécuter, le clone est là avec des pointeurs, et `LFS : Pull des objets` le termine plus tard. `gitpoulpi clone --no-lfs` saute le téléchargement exprès.
- Vous pouvez désactiver tout LFS dans Paramètres > Git et encodage > Git LFS (ou `[git] lfs_enabled = false`).

## « git-lfs not found » / « Git LFS is not installed »

Le message `Git LFS is not installed. This repository stores large files with Git LFS; ...` (ou une boîte de dialogue avec un bouton *Ouvrir la page d'installation*, ou le propre `git-lfs: command not found` / `'lfs' is not a git command` de git) signifie que le dépôt est configuré pour LFS - son `.gitattributes`, ses filtres `.git/config` ou son hook `pre-push` appellent `git-lfs` - et que le programme n'est pas dans le `PATH` que voit GitPoulpi. Installez-le (lien ci-dessus). Sous Windows, le programme est fourni avec Git for Windows ; si `git lfs version` fonctionne dans un terminal mais que GitPoulpi dit toujours qu'il manque, lancez GitPoulpi depuis un terminal qui voit le même `PATH`. Pour cacher LFS à GitPoulpi tout en laissant le comportement propre de git tranquille, désactivez `git.lfs_enabled`.

## Le graphe est dessiné avec des caractères, pas comme une image (herdr, tmux, autres terminaux)

GitPoulpi ne dessine la colonne Graph et les avatars des auteurs en pixels que là où il est sûr que le terminal les affichera : images en ligne iTerm2 dans WezTerm nu, Rio, mintty et iTerm2 antérieur à la 3.7.7 ; les images sixel dans Windows Terminal, foot, Tabby, Konsole et mlterm (et dans tmux quand le terminal derrière lui accepte sixel) ; le protocole graphique Kitty dans kitty, Ghostty, iTerm2 3.7.7 ou plus récent et **herdr quand la configuration propre de herdr règle `kitty_graphics = true`** (`[terminal]`, ou `[experimental]` sur un herdr plus ancien) et que herdr tourne dans WezTerm, kitty ou Ghostty. Partout ailleurs il dessine des caractères, ce qui est toujours correct.

- Dans herdr avec les graphiques Kitty activés mais une configuration que GitPoulpi ne trouve pas (ou en s'appuyant sur le défaut de herdr), réglez `ui.image_protocol = "kitty"` (Paramètres > Journal et graphe > Protocole d'image) et redémarrez.
- Dans un terminal qui montre des carrés ou du bruit là où le graphe devrait être, réglez `ui.image_protocol = "none"` ou `ui.graph_renderer = "text"`.
- `GITPOULPI_TERMINAL_IMAGE=off` désactive les images quelle que soit la configuration.

**herdr remote (`herdr --remote <hôte>`, ou `ssh` puis `herdr`) : le graphe en texte est le défaut.** Via SSH, GitPoulpi tourne sur l'hôte distant et ne peut lire que l'environnement du distant et le `config.toml` de herdr. Qu'une image Kitty apparaisse ou non est décidé par le client herdr *local* (son propre paramètre `kitty_graphics` et son propre terminal), que GitPoulpi ne peut pas voir : quand le client ne dessine pas les images, les cellules restent vides. GitPoulpi dessine donc le graphe en texte chaque fois qu'il détecte herdr via SSH (`SSH_CONNECTION` / `SSH_CLIENT` / `SSH_TTY`). Si votre herdr local a `kitty_graphics = true` et un terminal qui dessine les images Kitty (kitty, Ghostty), activez-le avec `ui.graph_renderer = "image"` (ou `ui.image_protocol = "kitty"`) sur le distant. Les images sont envoyées en ligne (`t=d`), ce qui marche sur n'importe quel flux d'octets. Si la colonne est vide, réglez `ui.graph_renderer = "text"`.

**Un rectangle plus clair derrière les voies.** L'image est transparente (PNG RGBA à alpha droit) partout où rien n'est dessiné, si bien que le fond du terminal et la bande de sélection transparaissent. Les terminaux au protocole Kitty, **WezTerm** et **Rio** composent cet alpha ; GitPoulpi réserve alors des cellules vides ordinaires sous l'image et l'envoie après elles. Tout autre terminal au protocole iTerm2 (iTerm2 plus ancien, mintty) n'est pas connu pour mélanger l'alpha des PNG, l'image y porte donc un fond opaque de la couleur du thème, qui peut apparaître comme un rectangle sur un terminal dont le fond diffère ; `ui.graph_renderer = "text"` le supprime. Avec `ui.graph_lane_background = "off"`, rien n'est peint derrière les voies sur les chemins transparents.

**Windows : la colonne du graphe est vide, sur les mauvaises lignes, ou l'écran se déchire (WezTerm 20240203, Rio, mintty).** La pseudo-console derrière ces terminaux perd ou déplace les séquences d'image ([Choisir un terminal](terminals.md#windows)). Réglez `ui.graph_renderer = "text"`, ou utilisez Windows Terminal.

Dans tmux, zellij, screen et herdr, voir [Multiplexeurs](multiplexers.md) pour ce qui passe et la configuration à ajouter.

Voir `ui.graph_renderer` et `ui.image_protocol` dans la [référence de configuration](config-reference.md).
