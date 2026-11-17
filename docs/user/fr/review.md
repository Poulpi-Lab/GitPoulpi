[English](../review.md) · [Čeština](../cs/review.md) · [Deutsch](../de/review.md) · [Español](../es/review.md) · **Français** · [Italiano](../it/review.md) · [日本語](../ja/review.md) · [한국어](../ko/review.md) · [Nederlands](../nl/review.md) · [Polski](../pl/review.md) · [Português (Brasil)](../pt-BR/review.md) · [Português (Portugal)](../pt-PT/review.md) · [Русский](../ru/review.md) · [Svenska](../sv/review.md) · [Türkçe](../tr/review.md) · [Українська](../uk/review.md) · [简体中文](../zh-CN/review.md)

# Relire une pull ou merge request

Relisez une pull request (GitHub, Gitea, Forgejo), une merge request (GitLab) ou un change Gerrit (voir [Gerrit](#gerrit))
**sans quitter la fenêtre du dépôt**. La vue de revue montre la description, la discussion, les fichiers modifiés
avec les fils de discussion sous leurs lignes et les commits, et c'est là que vous écrivez la revue : commentaires,
approbation et demandes de modifications. Elle demande un profil pour l'un de ces hébergeurs (voir
[Hébergeurs et suivi de tickets](hosting-integration.md)) et un dépôt qui y est lié ; les autres fournisseurs ne proposent
pas de revue. Les limites de l'hébergeur sont respectées : GitLab n'a pas de verdict *Demander des modifications* et prend une ligne par commentaire.
Cette page appelle toutes ces choses une *demande*.

![La vue de revue, page Aperçu : l'en-tête de la demande avec ses boutons de métadonnées, la description, un fil de code non résolu et un résolu, les commentaires et la zone de commentaire](../images/review-overview.png)

![La page Fichiers : les fichiers modifiés avec leurs cases « vu », le diff avec un fil de discussion sous sa ligne, et un fil résolu hors des lignes affichées](../images/review-files.png)

## Ouvrir une revue

Tous les chemins mènent à la même vue :

- la liste **PULL REQUESTS** de la barre latérale : sélectionnez une ligne et appuyez sur **Enter**, ou faites un clic droit dessus et choisissez
  **Relire...** ;
- le **Radar** : le menu de la ligne, **Relire...**, ou la touche `e` (le dépôt s'ouvre, et la revue avec lui ;
  une demande d'un dépôt que vous n'avez pas cloné demande s'il faut le cloner ou la relire via l'API,
  voir [Relire sans clone](#relire-sans-clone)) ;
- la **recherche de l'espace de travail** (`Ctrl+Alt+F`) : surlignez une pull ou merge request et appuyez sur **`Alt+R`** ;
- la palette : **Hébergeur : Relire une pull request...** liste les demandes à choisir.

La vue remplit la fenêtre par-dessus le dépôt (c'est un mode de l'onglet du dépôt, pas une fenêtre à part).
**`Esc`** ou le `✕` dans son coin la quitte. À l'ouverture, la demande est lue sur l'hébergeur et son code est
récupéré avec git en arrière-plan ; un indicateur d'attente apparaît si cela dure plus qu'un instant.

## L'en-tête

Toujours visible : le numéro et le titre, l'état (Ouverte, Brouillon, Fusionnée, Fermée), les branches
(`main ← feature/moons`), l'auteur, les vérifications (`✓ vérifications réussies`, `✗ vérifications en échec`, `● vérifications en cours`),
si elle fusionne proprement ou entre en conflit, la taille, et **`3 commentaires en attente`** quand vous en avez écrit.

Les quatre boutons de métadonnées - **Assignés**, **Relecteurs**, **Étiquettes**, **Jalon** - ouvrent les mêmes sélecteurs que
la barre latérale (voir [Assignés, relecteurs, étiquettes et jalon](pull-requests-and-issues.md#assignés-relecteurs-étiquettes-et-jalon)) ;
l'en-tête affiche le résultat dès que l'hébergeur l'a pris en compte. Les boutons d'action : **Basculer** (le
checkout habituel de la branche de la demande, qui ferme aussi la vue), **Ouvrir dans le navigateur**, **Actualiser**
(**`F5`**) et **Terminer la revue**.

## Trois pages

Un contrôle segmenté dans la bordure du haut change de page : **Aperçu**, **Fichiers**, **Commits**
(**`Alt+1`**, **`Alt+2`**, **`Alt+3`**, ou un clic).

### Aperçu

La vue d'ensemble est **la conversation de la demande**, la même chronologie et la même zone de commentaire qu'un ticket
([Conversations](conversations.md)) : d'abord la description (*a ouvert ceci*), puis les commentaires,
les fils sur le code avec leur extrait de diff, les revues envoyées avec leur verdict (`approuvée`, `modifications demandées`,
`commentée`, `écartée` ; sur GitLab, les verdicts sont ses approbations) et les événements, les plus anciens d'abord, sur la
fenêtre la plus récente avec **Charger les plus anciens** en haut. La zone de commentaire est ancrée sous la chronologie (**Nouveau commentaire**) :
**Écrire**, **Aperçu** et **Côte à côte**, **Annuler** et **Envoyer**. **Un commentaire publié ici part chez l'hébergeur aussitôt**
(rien n'est en attente : les commentaires en attente sont ceux sur le code, envoyés avec la revue), apparaît tout de suite dans la liste
et prend sa place quand l'hébergeur répond ; si l'hébergeur le refuse, la zone se rouvre avec votre texte et la raison.

Les touches sont celles de la vue de conversation : **`Up`** / **`Down`** (et `Ctrl+J` / `Ctrl+K`) passent d'une entrée à l'autre,
**`r`** répond dans le fil de l'entrée sélectionnée, **`Q`** la cite dans un nouveau commentaire, **`c`** écrit un nouveau
commentaire, **`e`** / **`d`** modifient et suppriment vos propres commentaires (une question précède la suppression), **`l`** ou `Enter`
ouvre les liens d'une entrée, **`g`** son diagramme, **`y`** copie son adresse, **`o`** l'ouvre dans le navigateur, **`n`**
va à l'entrée non lue suivante, **`v`** affiche ou masque les événements, **`m`** charge les entrées plus anciennes, `Tab` passe de la
chronologie à la zone et inversement, et **Note interne** (`Alt+I`) est là sur les hébergeurs qui ont de telles notes. Les boutons de l'entrée
sélectionnée (**Répondre**, **Citer**, **Modifier**, **Supprimer**, **Liens**) font la même chose, et la palette a les lignes
*Conversation : ...* à côté des lignes *Revue : ...*. Un lien est confirmé avant de s'ouvrir, comme partout dans la
revue.

Un fil sur du code montre son `chemin:ligne` comme un lien (souligné, un vrai bouton). Un clic dessus, **`G`** sur le fil
sélectionné, ou *Conversation : Ouvrir le code du fil de discussion* dans la palette passe à la page **Fichiers**, ouvre ce
fichier et place le curseur sur la ligne commentée (du côté ancien ou nouveau, comme elle a été écrite) avec le fil ouvert
dessous (un fil replié est déplié). **Esc**, le bouton **Retour à l'aperçu** à gauche des onglets de page, ou *Revue : Retour
au commentaire dans l'aperçu* ramènent à la même entrée de l'Aperçu. Un fil dont la ligne n'est plus dans le diff (obsolète)
ouvre le fichier à la ligne la plus proche et la ligne d'état le dit ; un fil sur un fichier que la demande ne modifie plus dit
que le fichier n'est pas dans le diff. Dans la vue ticket ou Radar de la même demande, il n'y a pas de code à ouvrir : `G`
répond que la revue l'a.

Quand une touche de la conversation et une touche de la revue se rencontrent, la conversation garde la lettre sur cette page : `l` (étiquettes), `m`
(jalon), `Q` (abandon sur Gerrit), `e` / `d`, `r`. Les étiquettes, le jalon et l'abandon restent sur les boutons de l'en-tête et
dans la palette ; `a` (assignés), `p` (relecteurs), `f` (**Terminer la revue**), `b` (**Basculer**), `P`, `D`, `U` et
`Alt+1` / `Alt+2` / `Alt+3` gardent leur sens ici. Tant que la zone a le clavier, chaque touche est la sienne, sauf
`Alt+1` / `Alt+2` / `Alt+3` ; `Esc` rend le clavier à la chronologie et un deuxième `Esc` ferme la revue.

Un commentaire écrit sur une autre fenêtre ou par le CLI s'affiche ici tout seul : chaque écriture de GitPoulpi (cette fenêtre
comprise) en informe la session, et les fenêtres qui montrent la même demande la relisent.

La zone de commentaire est l'éditeur Markdown partagé avec [la vue de conversation](conversations.md#éditeur-markdown)
(aussi la zone d'une réponse à un fil, d'un commentaire sur une ligne et du résumé de **Terminer la revue**) : `Ctrl+Enter` (aussi
`Ctrl+S`, `Alt+Enter`) envoie, et là où l'IA est activée `Alt+E` ouvre **Écrire avec l'IA**
([le chapitre IA](ai.md#aide-à-la-rédaction-sur-un-hébergeur)). L'aperçu montre le texte comme l'hébergeur le rendra, références comprises
(`#7`, `!12`, `ABC-12`, identifiants de commit) ; un texte non envoyé est gardé par demande.

### Fichiers

Les fichiers modifiés à gauche (lettre d'état, `+ajouté -retiré`, un compteur `●n` de fils ouverts, un compteur `✎n` de
vos commentaires en attente) et le diff du fichier sélectionné à droite. Le diff vient de git : la tête de la demande
est récupérée dans `refs/gitpoulpi/pr/<n>` (jamais une branche, rien n'est extrait) et comparée à la
base du merge, si bien que les numéros de ligne de la marge sont exactement ceux que les commentaires utilisent.

- **`[ ]` devant un fichier** : le marquer comme *vu* (clic, ou **`v`** / `Space` sur la liste). Le titre les compte.
  Les marques « vu » sont gardées par dépôt et survivent à un redémarrage.
- **Les fils de discussion** se placent juste sous leur ligne : premier commentaire, réponses, et les boutons **Répondre** et
  **Résoudre** / **Rouvrir**. Un fil résolu se replie sur une seule ligne ; **`z`** (ou `Enter` dessus, ou un clic sur le
  `▾`) replie et déplie n'importe quel fil.
- **Les fils obsolètes**, et les fils sur des lignes que le diff ne montre pas, sont listés après les hunks sous
  *Commentaires hors des lignes affichées*, avec un badge `obsolète` quand le code a changé sous eux.
- **`c`** commente la ligne où se trouve le curseur (le petit `+` au début de la ligne fait la même chose à la
  souris). **`Shift+Up` / `Shift+Down`** (ou un glissement sur les lignes) sélectionnent d'abord plusieurs lignes et le
  commentaire couvre la plage. GitLab prend une ligne par commentaire, et la vue le dit au lieu de sélectionner.
- **`r`** répond au fil sous le curseur, **`x`** le résout ou le rouvre. Les deux sont envoyés à l'hébergeur **immédiatement**.
- Touches : `Up` / `Down` / `PgUp` / `PgDn` / `Home` / `End` déplacent ; `Tab`, `Right` et `Enter` vont de la liste des fichiers
  au diff, `Left` et `Tab` en sens inverse ; `]` et `[` sautent au fichier suivant et au précédent.

### Les options d'affichage du diff

Le volet de diff de **Fichiers** et de **Commits** est celui de la [vue Diff](staging-and-committing.md), avec la même barre sur sa première
ligne et les mêmes choix mémorisés (c'est un seul paramètre : ce que vous choisissez ici est ce que la vue Diff ouvre ensuite,
et inversement) :

- **Unifié / Côte à côte** : une colonne, ou avant et après côte à côte. En côte à côte, un fil, un commentaire en attente ou une
  suggestion se place sous la moitié à laquelle il appartient : un commentaire sur une ligne supprimée sous la moitié gauche, un commentaire sur une
  ligne ajoutée ou inchangée sous la moitié droite. Cliquez sur la moitié voulue avant d'appuyer sur **`c`** sur une ligne qui
  montre une suppression et un ajout ; au clavier, le commentaire va sur le côté nouveau.
- **Hunks** : seulement les modifications avec leur contexte, ou le fichier entier avec les modifications en place. Dans le fichier entier, un
  fil sur une ligne hors des modifications se place sous sa ligne au lieu d'être dans *Commentaires hors des lignes affichées*.
  Le texte du fichier est lu en arrière-plan ; les hunks restent à l'écran avec une ligne qui l'indique jusqu'à son arrivée (sur
  la page Commits, jusqu'à 40 fichiers d'un commit sont lus ainsi).
- **Espaces** : afficher ou ignorer les modifications qui ne diffèrent que par des espaces.
- **Retour** : retour à la ligne des lignes longues.
- **Flèches haut / bas** : le hunk précédent et le suivant.
- **Map** (**`M`**) : la minimap à côté de la barre de défilement, un seul paramètre avec celle de la vue Diff. Cliquez ou glissez dessus pour
  défiler.
- **Auto** (**`E`**, ou *Revue : Choisir l'encodage...*) : l'encodage de lecture du fichier, comme dans la vue Diff :
  la détection automatique ou l'un des encodages de la liste, choisi pour le fichier que le volet montre (sur la page Commits, le
  fichier sous le curseur). Les deux côtés du fichier sont relus dans cet encodage, en arrière-plan, et
  le choix est partagé avec la vue Diff pour le même chemin. Un fichier qui ne peut pas être lu ainsi dit pourquoi au-dessus de
  son diff.
- **`⋯`** : quand le volet est trop étroit pour tous les boutons, ceux qui ne tiennent pas sont derrière un `⋯` au bout de
  la barre ; sa liste lance chacun comme s'il était cliqué. Leurs touches et leurs lignes de palette marchent comme toujours.

La règle de la barre de défilement montre où sont les choses : les modifications (fichier entier seulement, comme dans la vue Diff), un point `●`
pour chaque fil, commentaire en attente et suggestion, et le curseur.

Les lignes longues sont coupées au bord du volet (activez **Retour** pour les voir entières) et défilent latéralement : **`Right`**
et **`Left`** sur le diff (`Left` au bord gauche revient à la liste des fichiers), la molette latérale ou `Shift`
avec la molette, la barre de défilement le long de la bordure du bas, ou *Revue : Défiler le diff vers la gauche / droite*.

### Sélectionner du texte et des lignes

L'endroit où commence un glissement décide de ce qu'il sélectionne. Un glissement qui commence **dans le code** sélectionne du texte, comme dans la vue Diff
(le double-clic sélectionne un mot) ; **`Ctrl+C`** (ou *Revue : Copier le texte sélectionné ou la ligne*) copie exactement ces
caractères, sans numéros de ligne ni marqueurs, et sans sélection de texte copie la ligne sous le curseur. Un
glissement qui commence **dans la gouttière** (les numéros de ligne, le marqueur, le `+`) sélectionne des lignes, pour les commenter
comme avant, et un double-clic dans la gouttière commence toujours un commentaire sur cette ligne.

Les touches `n`, `p`, `m`, `w`, `s` de la vue Diff sont celles de la revue, donc dans la revue les bascules sont des
majuscules : **`V`** disposition, **`H`** hunks ou fichier entier, **`S`** espaces, **`W`** retour à la ligne, **`J`** / **`K`** hunk suivant et
précédent, **`E`** l'encodage, **`M`** la minimap. Chacune a une ligne de palette (*Revue : Disposition unifiée ou côte à côte*,
...).

### Commits

Seulement les commits de la demande (pas l'historique derrière), les plus récents d'abord, et le diff de celui qui est sélectionné
(ses modifications par rapport à son parent), avec tous ses fichiers les uns sous les autres et les mêmes options d'affichage.

## Écrire une revue

Un commentaire sur le code est **en attente** : il est gardé sur votre machine (dans le fichier d'état du dépôt, donc il survit à un
redémarrage) et compté dans l'en-tête. Rien n'est envoyé avant que vous terminiez la revue. Un commentaire en attente s'affiche
sous sa ligne avec le badge `En attente - ligne 12` et deux boutons : **`e`** (Modifier) et **`d`** (Supprimer).

**Terminer la revue** (**`f`**, le bouton, ou la palette) ouvre un petit formulaire : un résumé (facultatif) et une liste
**Verdict** : *Commenter*, *Approuver*, *Demander des modifications*. Seul ce que l'hébergeur permet est listé : GitLab n'a pas
*Demander des modifications*. **Envoyer la revue** (`Ctrl+S`) envoie le résumé, le verdict et chaque commentaire en attente en
une seule fois. **Abandonner la revue...** jette les commentaires en attente après une confirmation.

Ce qui est envoyé immédiatement, sans attendre la revue : les réponses, la résolution et la réouverture, et les commentaires dans la
discussion. La zone de commentaire dit de quel genre il s'agit (*Ce commentaire est publié immédiatement* ou *Ce commentaire est ajouté à votre
revue en attente*). Si l'hébergeur refuse une réponse, la zone se rouvre avec vos mots.

Si la demande a reçu de nouveaux commits après que vous avez écrit des commentaires en attente, la vue prévient que leurs lignes ont peut-être
bougé (GitHub et GitLab placent un commentaire sur le commit que vous avez lu ; une revue d'une tête plus ancienne est refusée).

## Gerrit

Un serveur Gerrit se connecte comme les autres hébergeurs (voir [Hébergeurs et suivi de tickets](hosting-integration.md) : URL du
serveur, nom d'utilisateur, mot de passe HTTP) et se lie sous **PULL REQUESTS** à un *projet*. Ses changes sont listés comme des pull
requests (`#7`, pastille `GR`) et relus dans la même vue, avec les différences qu'apporte Gerrit :

- **Patch sets.** Un change a des patch sets numérotés (chaque envoi du commit en est un). L'en-tête a une liste déroulante
  **`Patch set N`** sur la bordure du haut, à côté des onglets de page (**`P`**, ou la ligne de palette *Revue : Choisir le patch
  set...*). Elle démarre sur le dernier ; en choisir un autre lit les fichiers et le code de ce patch set, et les
  fils écrits sur d'autres patch sets sont listés comme *obsolètes*. La revue que vous terminez est envoyée sur le patch set
  affiché.
- **Comparer avec un autre patch set.** Une seconde liste déroulante à gauche de la première, **`Comparer : base`**
  (**`D`**, ou *Revue : Comparer des patch sets...*), liste la base et chaque patch set plus ancien que celui affiché.
  Choisissez le patch set 1 pendant que le patch set 3 est affiché et la page Fichiers liste, et diffe, seulement ce que le
  patch set 3 a changé depuis le patch set 1 (la différence propre au serveur, donc un rebase sur une cible plus récente
  n'apparaît pas comme des changements) ; la page Commits montre cette différence unique. Les deux patch sets sont
  récupérés dans `refs/gitpoulpi/pr/<n>` et `refs/gitpoulpi/pr/<n>-ps<k>`. Les fils écrits sur d'autres patch sets que
  celui affiché sont *obsolètes*. Choisir à lire un patch set plus ancien que celui comparé remet la comparaison sur la
  base. Le jumeau CLI est `review show|fetch --compare-with K`.
- **Soumettre et abandonner.** Quand le serveur dit que le change peut être soumis (les votes et les vérifications
  nécessaires sont là), un bouton **Soumettre** rejoint les boutons de l'en-tête ; **Abandonner** y est tant que vous pouvez
  abandonner le change. Les deux (touches **`U`** et **`Q`**, palette *Revue : Soumettre le change...* / *Revue : Abandonner le change...*)
  demandent d'abord, dans une fenêtre qui dit que l'action a lieu sur le serveur et **ne peut pas être annulée depuis
  GitPoulpi** (un change fusionné reste fusionné ; un change abandonné peut être restauré dans l'interface web de Gerrit).
  Refuser n'envoie rien ; en cas de succès, la vue relit le change. Les jumeaux CLI sont
  `gitpoulpi hosting review submit-change <n>` et `hosting review abandon <n> [--message M]`.
- **Des votes à la place d'un verdict.** **Terminer la revue** montre une liste déroulante par label sur lequel vous pouvez voter - `Code-Review`
  (-2 à +2) et `Verified` là où le projet l'a - ne listant que les valeurs que le serveur permet **pour ce
  change et pour vous** (un développeur sans le droit ne peut pas choisir +2). Elle démarre sur votre vote actuel ; seul un vote que vous avez
  changé est envoyé. Le résumé et les commentaires en attente partent dans le même appel. Les brouillons enregistrés dans l'interface
  web de Gerrit sont publiés avec la revue.
- **Les fils de discussion** sont les chaînes de commentaires de Gerrit. Un fil est résolu quand son commentaire le plus récent dit `unresolved: false` ;
  **résoudre** publie une réponse `Done` qui le fait, **rouvrir** une réponse `Reopened`. Les réponses et la résolution vont au serveur
  immédiatement, comme sur les autres hébergeurs.
- **Envoyer pour relecture.** Gerrit crée un change quand vous poussez vers `refs/for/<branch>`. Quand la section PULL REQUESTS est
  liée à un projet Gerrit, le bouton **`+`** de la section et la ligne de palette **Hébergeur : Push pour relecture...**
  demandent la branche cible et poussent `HEAD` vers `refs/for/<branch>` (un sujet peut être donné en ligne de
  commande). Le push ne peut pas être annulé depuis GitPoulpi (abandonnez le change dans Gerrit) ; le message le dit. Son équivalent
  CLI est `gitpoulpi hosting review push`.
- Les boutons de métadonnées sont en lecture seule pour Gerrit (les relecteurs sont affichés), il n'y a pas de verdict *Demander des modifications*
  (un vote de -1 ou -2 revient au même) et la pastille des vérifications est le label `Verified`.

## ✦ Brouillon de revue IA

![Deux suggestions ✦ sous leurs lignes dans la page Fichiers, chacune avec Accepter, Modifier et Écarter, à côté d'un fil existant](../images/review-ai.png)

Avec l'IA activée (*Paramètres > IA*, voir [IA](ai.md)), l'en-tête a un bouton **`✦ Rédiger une revue...`** (aussi **`s`**,
et la ligne de palette *Revue : ✦ Rédiger une revue...*). Il ouvre les [options de la revue](#options-de-la-revue) et
**`Enter`** y interroge le modèle. C'est la seule chose qui interroge le modèle : ouvrir une revue
ne le fait jamais. Le modèle lit le titre et la description de la demande, les commentaires qui existent déjà (pour ne
pas les répéter) et le diff, et répond avec un **brouillon de revue** : le texte à envoyer, le verdict qu'il recommande (Approuver, Demander des modifications ou Commenter), un résumé général et une liste de commentaires suggérés. Un
long diff est coupé pour le modèle - par fichier, et au total - et l'Aperçu dit combien de fichiers ont été raccourcis
ou écartés. Pendant qu'il travaille, la ligne d'état montre un indicateur d'attente après un instant et **`Esc`** (ou le même bouton)
l'annule. Les instructions *Revue de code* des [quatre niveaux](ai.md#instructions) s'appliquent, si bien qu'une équipe ou un
espace de travail peut dire à quoi ressemble une bonne revue.

### Options de la revue

`s` ouvre la boîte **✦ Options de la revue**, remplie avec les valeurs par défaut du dépôt (les globales là où le
dépôt n'en a pas). `Enter` (ou le bouton **Lancer**) démarre l'exécution avec ce que montre la boîte ; `Esc`, **Annuler**
ou la croix la ferme sans rien demander.

| Contrôle | Effet |
|---|---|
| **Lancer l'agent dans le dépôt** | *Réglage du profil* (le dossier de travail du profil IA décide), *Oui* ou *Non*. Grisé, avec la raison dessous, pour un profil HTTP (il n'a pas de dossier de travail) et quand la demande est relue via l'API sans clone. |
| **Inclure CLAUDE.md**, **Inclure AGENTS.md** | Donnent au modèle ce fichier du dépôt, lu sur la branche visée par la demande. Quand l'agent travaille dans le dépôt, il les lit tout seul, et la boîte le dit. |
| **Fichier supplémentaire** | Un fichier de plus : un chemin dans le dépôt, un chemin absolu ou un lien vers un fichier d'un hébergeur. **Parcourir...** (`Ctrl+B`, ou `Entrée` sur le bouton) ouvre un sélecteur de fichier qui part de la racine du dépôt (ou du fichier déjà nommé) ; un fichier du dépôt est écrit en chemin relatif à la racine, un fichier extérieur en chemin absolu. Saisir dans le champ marche comme avant. |
| **Aucun fichier supplémentaire** | Aucun fichier supplémentaire pour cette exécution, même si le réglage global ou le dépôt en nomme un ; saisir un fichier ou en choisir un avec **Parcourir...** la décoche. Avec **Enregistrer comme valeurs par défaut de ce dépôt**, elle devient l'interrupteur *Aucun fichier supplémentaire* du dépôt. Un champ vide compte ici comme aucun, car la boîte part de ce que l'exécution hériterait. |
| **Enregistrer comme valeurs par défaut de ce dépôt** | Garde aussi les valeurs pour les prochaines revues : dans le fichier d'état du dépôt, ou dans `remote-reviews.toml` pour un dépôt qui n'est pas cloné. Seule une valeur qui diffère de la globale est gardée, donc le reste continue de suivre *Paramètres > IA*. |

Tout est atteignable avec `Tab`, `Space` coche une case et le champ de chemin est un champ de texte ordinaire. Les
mêmes quatre réglages vivent dans *Paramètres > IA* (*Revue de code*) et dans *Paramètres propres au dépôt > IA* ; voir
[Contexte de la revue](ai.md#contexte-de-la-revue). Ce qui n'a pas pu être donné au modèle (un fichier supplémentaire absent ou
illisible) et tout fichier raccourci pour tenir dans les limites sont listés sur la carte du brouillon.

Les suggestions **ne sont pas des commentaires**. Elles s'affichent à deux endroits :

- **Sous leur ligne dans Fichiers**, comme des commentaires fantômes marqués `✦` avec une gravité (`info`, `avertissement`, `problème`) et
  trois boutons : **Accepter**, **Modifier** et **Écarter**. Une suggestion sur une ligne que le diff ne montre pas est
  gardée, jamais perdue : elle est listée après les hunks sous *Suggestions de l'IA sur le fichier entier* et, une fois
  acceptée, atterrit sur la première ligne modifiée du fichier avec les mots *(à propos de la ligne N)* devant.
- **Dans l'Aperçu**, dans un bloc au-dessus de la conversation (seulement après une exécution) : le résumé, puis la liste avec les mêmes boutons et **Tout accepter** /
  **Tout écarter**.

**Accepter** (`y`) transforme une suggestion en **commentaire en attente** ordinaire (vous pouvez encore le modifier ou le supprimer) ;
**Modifier** (`w`) ouvre la zone de commentaire avec les mots du modèle d'abord et accepte ce que vous laissez ; **Écarter** (`n`)
la jette. **Tout accepter** (`A`) ne fait que remplir la liste des commentaires en attente. Sur la page Fichiers, les touches agissent sur la
suggestion sous le curseur, sur l'Aperçu sur la première de la liste (`y` et `n` redeviennent *Copier le lien* et
*Entrée non lue suivante* de la conversation une fois la liste vide). **Utiliser le résumé IA** (`i`, ou le bouton
du formulaire Terminer la revue) met le résumé du modèle dans le texte de la revue ; il n'est jamais inséré tout seul.
**Rien n'est envoyé à l'hébergeur avant que vous envoyiez la revue.**

L'Aperçu montre le brouillon comme une carte en haut de ce bloc, au-dessus des suggestions : la pastille du verdict, le texte de la revue, et trois
boutons.

- **✦ Utiliser le brouillon...** (`u`) accepte chaque suggestion ouverte comme commentaire en attente et ouvre *Terminer la revue* déjà
  rempli avec le texte et le verdict du brouillon. Vous le relisez, le changez et l'envoyez vous-même ; annuler le
  formulaire garde le brouillon. Les limites de l'hébergeur sont respectées : GitLab n'a pas *Demander des modifications*, donc le formulaire le dit et
  retombe sur *Commenter* ; sur Gerrit le verdict devient un vote Code-Review de +1 ou -1, seulement si le serveur
  permet cette valeur pour vous (sinon aucun vote n'est posé et le formulaire le dit).
- **Modifier le texte...** (`t`) change le texte de revue du brouillon avant que vous l'utilisiez.
- **Abandonner le brouillon** (`N`) jette le texte, le verdict et les suggestions restantes.

Le brouillon est gardé par demande, donc il est encore là quand vous fermez et rouvrez la revue. Quand la tête de
la demande a bougé depuis que le modèle l'a écrit, la carte dit *La demande a changé depuis l'écriture de ce brouillon.*
Envoyer la revue dépense le brouillon.

## Tout a une touche et une ligne de palette

| Action | Touche | Palette |
|---|---|---|
| Aperçu / Fichiers / Commits | `Alt+1` / `Alt+2` / `Alt+3` | *Revue : Afficher l'aperçu*, *Afficher les fichiers*, *Afficher les commits* |
| Commenter la ligne ou la plage | `c` | *Revue : Commenter la ligne...* |
| Commenter la discussion | `c` sur l'Aperçu, `C` | *Revue : Commenter la discussion...* (ouvre la zone de l'Aperçu), *Conversation : Nouveau commentaire* |
| Ouvrir le code d'un fil (Aperçu) / revenir au commentaire | `G` / `Esc` | *Conversation : Ouvrir le code du fil de discussion*, *Revue : Retour au commentaire dans l'aperçu* |
| Répondre au fil | `r` | *Revue : Répondre au fil de discussion...* |
| Résoudre ou rouvrir | `x` | *Revue : Résoudre ou rouvrir le fil de discussion* |
| Replier ou déplier un fil | `z` | *Revue : Replier ou déplier le fil de discussion* |
| Modifier / supprimer un commentaire en attente | `e` / `d` | *Revue : Modifier le commentaire en attente...*, *Supprimer le commentaire en attente* |
| Marquer le fichier comme vu | `v` | *Revue : Marquer le fichier comme vu* |
| Unifié / Côte à côte | `V` | *Revue : Disposition unifiée ou côte à côte* |
| Fichier entier / hunks seulement | `H` | *Revue : Hunks seulement ou fichier entier* |
| Modifications d'espaces seulement | `S` | *Revue : Afficher ou masquer les modifications d'espaces* |
| Retour à la ligne | `W` | *Revue : Activer ou désactiver le retour à la ligne* |
| Hunk suivant / précédent | `J` / `K` | *Revue : Hunk suivant*, *Hunk précédent* |
| Fichier suivant / précédent | `]` / `[` | *Revue : Fichier suivant*, *Fichier précédent* |
| Terminer / abandonner la revue | `f` | *Revue : Terminer la revue...*, *Abandonner la revue en attente...* |
| Actualiser | `F5`, `Ctrl+R` | *Revue : Actualiser* |
| Ouvrir dans le navigateur / basculer sur la branche | `o` / `b` | *Revue : Ouvrir dans le navigateur*, *Basculer sur la branche* |
| Assignés / relecteurs / étiquettes / jalon | `a` / `p` / `l` / `m` | *Revue : Modifier les assignés...* et ses trois sœurs |
| Choisir le patch set (Gerrit) | `P` | *Revue : Choisir le patch set...* |
| Comparer avec un patch set plus ancien (Gerrit) | `D` | *Revue : Comparer des patch sets...* |
| Soumettre / abandonner le change (Gerrit) | `U` / `Q` | *Revue : Soumettre le change...*, *Revue : Abandonner le change...* |
| ✦ Rédiger une revue... (IA activée ; ouvre les options de la revue) | `s` | *Revue : ✦ Rédiger une revue...* |
| Accepter / modifier et accepter / écarter une suggestion | `y` / `w` / `n` | *Revue : Accepter la suggestion*, *Modifier et accepter la suggestion...*, *Écarter la suggestion* |
| Accepter / écarter toutes les suggestions | `A` / `N` | *Revue : Accepter toutes les suggestions*, *Écarter toutes les suggestions* |
| Utiliser le résumé IA dans le texte de la revue | `i` | *Revue : Utiliser le résumé IA dans le texte de la revue* |
| Utiliser le brouillon ✦ (remplit Terminer la revue) | `u` | *Revue : ✦ Utiliser le brouillon...* |
| Modifier le texte du brouillon | `t` | *Revue : Modifier le texte du brouillon ✦...* |
| Abandonner le brouillon ✦ | `N` | *Revue : Abandonner le brouillon ✦* |
| Fermer | `Esc` | *Revue : Fermer* |

Ouvrez la palette (`Ctrl+P`) pendant que la vue est ouverte : ses lignes agissent sur elle, et disent pourquoi l'une ne fait rien
pour l'instant (« placez d'abord le curseur sur une ligne de code »).

## Relire sans clone

Une demande d'un dépôt qui n'est pas sur votre machine peut quand même être relue. Depuis le Radar, **Relire**
(`e`) sur sa ligne demande comment (voir [Le Radar](radar.md#relire-une-demande)) ; **Relire via l'API** ouvre
cette même vue dans la fenêtre Accueil, par-dessus le Radar. Tout ce que dit cette page fonctionne pareil, avec ces
différences :

- Le code (les fichiers modifiés, le diff de chaque fichier, les commits de la demande, le diff d'un commit, le fichier entier) est
  lu depuis l'API de l'hébergeur, donc le premier coup d'œil à une grosse demande prend un moment, et un fichier pour lequel l'hébergeur
  n'envoie pas de diff (un binaire ou un très gros fichier) le dit au lieu d'en montrer un.
- Vos commentaires en attente sont gardés dans `remote-reviews.toml` dans le dossier de configuration, classés sous l'hébergeur
  et le projet (`host/project`), au lieu de l'état propre d'un dépôt. Ils sont encore là quand vous
  fermez et rouvrez la revue, et envoyer la revue les dépense.
- Il n'y a pas de choix d'encodage (*Indisponible sans clone*) ni de **Basculer sur la branche** (il dit de cloner d'abord le
  dépôt).
- `Esc` ou le `✕` dans le coin quitte la vue et rend le Radar. La palette (`Ctrl+P`) atteint les lignes de
  la vue comme dans l'onglet d'un dépôt.

Pour garder le clone à la place, choisissez **Cloner, puis relire** (ou *Radar : Cloner et relire...* dans la palette) :
la boîte Cloner s'ouvre sur le compte connecté de la demande avec son dépôt sélectionné, et la revue s'ouvre dans l'onglet du nouveau dépôt une fois le clone arrivé.

## En ligne de commande et pour les agents

La même revue est disponible avec `gitpoulpi hosting review show|threads|comment|reply|resolve|submit|fetch|suggest`
(et les outils MCP qui en sont tirés) : voir la [référence CLI](cli-reference.md). `threads` liste aussi les
verdicts déjà envoyés (`reviews`). `suggest <n>` interroge le modèle (IA activée) et affiche son `verdict`, son `body`, son résumé et ses
suggestions en JSON sans rien publier, si bien qu'un script ou un agent peut choisir les commentaires à passer à
`submit --comment`.

Rien de tout cela ne nécessite un clone : `--project owner/name` (avec `--id` quand plusieurs intégrations sont configurées)
lance `show`, `threads`, `comment`, `reply`, `resolve`, `submit` et `suggest` sur un projet dont le répertoire
courant ne sait rien. `hosting review diff <n>` affiche le diff avec ses hunks (`--api` le lit depuis
l'hébergeur même dans un clone), et `hosting file <path> --project P [--ref R]` lit un fichier à une révision, par
exemple le `CLAUDE.md` du projet. Les deux lisent l'API de l'hébergeur : GitHub, GitLab, Gitea, Forgejo et Gerrit.
