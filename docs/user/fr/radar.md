[English](../radar.md) · [Čeština](../cs/radar.md) · [Deutsch](../de/radar.md) · [Español](../es/radar.md) · **Français** · [Italiano](../it/radar.md) · [日本語](../ja/radar.md) · [한국어](../ko/radar.md) · [Nederlands](../nl/radar.md) · [Polski](../pl/radar.md) · [Português (Brasil)](../pt-BR/radar.md) · [Português (Portugal)](../pt-PT/radar.md) · [Русский](../ru/radar.md) · [Svenska](../sv/radar.md) · [Türkçe](../tr/radar.md) · [Українська](../uk/radar.md) · [简体中文](../zh-CN/radar.md)

# Le Radar

Le Radar est un onglet à part, juste après Accueil. C'est une liste unique de ce qui
vous attend sur **plusieurs dépôts** : les pull (ou merge) requests que vous avez écrites,
celles qui demandent votre revue, les tickets qui vous sont assignés ou que vous avez
créés, et les dépôts où vous avez laissé du travail en cours. Il s'agit de la fenêtre
Accueil avec la vue Radar par-dessus, pour que rien en dessous ne puisse voler un
clic ou une touche.

Il demande un profil d'hébergeur de code (voir
[Hébergeurs et suivi de tickets](hosting-integration.md) ; les listes d'un seul dépôt sont dans [Pull requests et tickets](pull-requests-and-issues.md), et la discussion d'une ligne s'ouvre comme une [conversation](conversations.md)) et des dépôts liés à
celui-ci, ou un profil dont les listes à l'échelle du compte sont activées (voir « Au-delà du périmètre » plus bas).
Un dépôt qui n'est pas lié n'est jamais interrogé : il affiche à la place une ligne « Lier... » (voir « Dépôts qui ont quelque chose à dire » plus bas).

![Le Radar sur deux dépôts : une demande épinglée, puis À relire par moi, Mes pull requests, Qui me sont assignées, des tickets et du travail en cours](../images/radar.png)

**Relire...** sur une ligne de pull ou merge request (ou `e`) ouvre le dépôt avec la revue intégrée de la demande
([Relire une pull request](review.md)). Pour une demande d'un dépôt que vous avez cloné mais jamais lié à l'hébergeur, le clone est
d'abord lié au projet de la demande (sauf s'il a déjà son propre lien de pull request), si bien que la revue démarre dans son onglet.

### La colonne Actions

À droite d'une ligne de demande ou de ticket, de petits boutons font ce que fait une touche sur la ligne
sélectionnée (un clic sélectionne la ligne et lance l'action ; posez le pointeur sur un bouton pour voir son nom
et sa touche) :

```
Large :   #58  Round invoice totals  feature/login → main   alice   2d   Relire  [globe]  [talk]  ⊕  ×  ⇣  ⋮
Étroit :  #58  Round invoice totals  feature/login → main                 [rev]  [globe]  [talk]  ⋮
Ticket :  #12  Crash on empty list                         bob     5h   Démarrer  [globe]  [talk]  ×  ⋮
```

| Bouton | Touche | Palette |
|---|---|---|
| **Relire** (une demande qui attend votre revue ; demande comment quand elle n'est pas clonée) | `e` | `Radar : Relire la pull request...` |
| **Démarrer** (un ticket cloné) | `n` | `Radar : Commencer à travailler sur un ticket...` |
| le globe : ouvrir la page sur l'hébergeur | `o` | `Radar : Ouvrir dans le navigateur` |
| le glyphe à lignes : ouvrir la conversation | `i` | `Radar : Ouvrir la conversation du ticket` |
| `»` **Marquer comme prête** (une demande en brouillon) | `y` | `Radar : Marquer comme prête` |
| `⊕` **Merge** (une demande) | `m` | `Radar : Merge la pull request...` |
| `×` **Fermer** (une demande ou un ticket) | `x` | `Radar : Fermer...` |
| `⇣` **Cloner** (une ligne dont le dépôt n'est pas cloné) | `g` | `Radar : Cloner le dépôt...` |

Une ligne sans action laisse sa cellule vide, de sorte que chaque bouton reste dans la même colonne sur toutes les
lignes. Quand la fenêtre est étroite, les mots Relire et Démarrer deviennent des glyphes avant qu'aucune autre
colonne disparaisse ; les boutons Marquer comme prête, Merge, Fermer et Cloner quittent la colonne avant le titre,
mais restent dans le menu de la ligne (`⋮`, clic droit, ou `Shift+F10`), où une action qui ne peut pas s'exécuter est
grisée avec sa raison. La colonne est automatique (ce n'est pas une des colonnes dimensionnables). Un double-clic ou
`Enter` sur une demande qui n'est pas clonée demande comment la relire, comme **Relire**.

### Les branches d'une demande

Une ligne de demande affiche `source → cible` après son titre (`alice:feature/login → main` quand la branche source est
dans un fork). Quand la place manque, la cible est coupée en premier (une ellipse), le titre garde l'essentiel de la
place, et dans une fenêtre étroite les branches ne sont pas affichées ; la colonne Actions n'est jamais poussée dehors.
Posez le pointeur sur un texte coupé pour le voir en entier. Un hébergeur qui n'a pas nommé les branches n'affiche rien
là. Une ligne de ticket nomme son dépôt dans la même pastille qu'une ligne de demande (le nom de dossier du clone, ou
`owner/name` quand il n'est pas cloné).

### Agir sur l'hébergeur

**Merge**, **Fermer** et **Marquer comme prête** fonctionnent sur GitHub, GitLab, Gitea et Forgejo (les autres hébergeurs
les grisent avec « Non pris en charge par ... » ; les tickets peuvent être fermés sur les mêmes hébergeurs).

- **Merge** demande d'abord : la boîte nomme la demande, son projet et ses branches, et propose la méthode (un commit de
  merge, squash, et rebase là où l'hébergeur l'a ; GitLab fait un merge ou un squash). Il est refusé avec sa raison pour un
  brouillon, une demande en conflit avec sa cible, ou des vérifications en échec ; un refus de l'hébergeur (pas de
  permission, une règle de la branche) s'affiche sur la ligne d'état.
- **Fermer** demande d'abord et ferme sans merge.
- **Marquer comme prête** sort un brouillon de l'état de brouillon (GitLab et Gitea : le marqueur `Draft:` / `WIP:` quitte
  le titre). Il ne demande rien : c'est réversible sur l'hébergeur.
- **Cloner** ouvre la boîte Cloner ordinaire sur le projet de la ligne (l'URL est remplie, le sélecteur de dossier est le
  sélecteur habituel). Une fois le clone créé, le Radar place la ligne dessus à la prochaine lecture des distants (rouvrez-le ou
  Actualisez), et Cloner indique alors où se trouve le clone.

Merge et Fermer modifient l'hébergeur et **ne peuvent pas être annulés depuis GitPoulpi** (la boîte le dit) ; rouvrez une
demande fermée sur l'hébergeur. Après un appel réussi, les listes sont redemandées : une ligne mergée ou fermée disparaît et
une ligne prête perd sa pastille `brouillon`.

## L'ouvrir

- **`Alt+R`** (réaffectable : `global.radar`), de n'importe où.
- La palette : `Radar : Ouvrir`.
- L'onglet **Radar**, juste après Accueil dans la barre d'onglets.
- La ligne **Radar** du menu **`+`**, tant que l'onglet est fermé.

Depuis un onglet de dépôt, cela vous amène à la fenêtre Accueil avec le Radar
ouvert ; `Esc` ou **Retour** ramène à la liste des dépôts, et le `⌂` fait de même.

## Fermer l'onglet, ou désactiver le Radar

L'onglet se ferme comme un onglet de dépôt : son **x**, un clic du milieu, `Ctrl+W` pendant qu'il est affiché, ou la
commande de palette `Radar : Fermer l'onglet`. Accueil est le seul onglet qui ne peut pas se fermer. Un onglet
fermé reste fermé pour cette session, même d'un lancement à l'autre, jusqu'à ce que vous rouvriez le Radar
(`Alt+R`, `Radar : Ouvrir`, ou le menu `+`) ; il revient alors juste après Accueil. Le fermer ne change aucun
paramètre.

Si vous ne vous en servez jamais, désactivez-le dans **Paramètres > Général > Activer le Radar**
(`general.radar_enabled`, voir la [référence de configuration](config-reference.md)). Désactivé, l'onglet, ses
commandes de palette et `Alt+R` disparaissent et GitPoulpi ne récupère plus rien pour lui. Cela s'applique
immédiatement, sans redémarrage.

## Ce qu'il couvre : le périmètre

La liste déroulante **Périmètre** choisit quels dépôts sont interrogés :

- un **espace de travail**, par son nom (ses dépôts, même ceux sans onglet ouvert) ;
- **Onglets ouverts** : les dépôts ouverts en onglets de cette session ;
- **Dossier de clonage par défaut** et **Dossier de navigation par défaut** : tous les dépôts trouvés dans le dossier
  défini dans les [Paramètres](config-reference.md) (`repos.clone_root`, `repos.browse_root`), qu'ils aient un onglet ou non.

Il s'ouvre sur le dernier périmètre que vous avez choisi ici (gardé dans `radar.toml`), tant qu'il existe encore ;
avant votre premier choix, ou quand cet espace de travail a été supprimé, sur l'espace de travail actif de la session,
sinon sur les onglets ouverts. Le choix ne vaut que pour cette vue : il ne change pas l'espace de travail actif de la
fenêtre. Quand deux dépôts ont le même nom de dossier, le dossier parent est ajouté à la pastille pour les distinguer.

### Les deux périmètres de dossier

Un périmètre de dossier cherche des dépôts **sur deux niveaux** : les dossiers placés directement dedans, et les dossiers
à l'intérieur de ceux qui ne sont pas eux-mêmes des dépôts (`~/src/work/app`). Les dossiers cachés sont ignorés, on n'entre
jamais dans un dépôt, et 200 dépôts au plus sont listés. La recherche se fait en arrière-plan, jamais sur le thread qui
dessine, et son résultat est gardé jusqu'à l'actualisation : le bouton Actualiser, `r`, `F5` (la touche d'actualisation de la fenêtre, réassignable) ou `Radar : Actualiser` relance
la recherche dans le dossier (un dépôt cloné entre-temps apparaît alors).

L'hébergeur reste facultatif pour chaque dépôt : un dépôt du dossier qui n'est **pas lié** à un hébergeur n'est jamais
interrogé. Il n'apporte que sa ligne de travail en cours, a une ligne `non lié` avec **Lier...** sous Dépôts (elle ouvre la
section des pull requests de ce dépôt, où vous choisissez le fournisseur et le projet), et la ligne d'état les compte
(`3 non liés (travail en cours seulement)`).

Un dossier qui n'est pas défini est grisé dans la liste (`Dossier de navigation par défaut (à définir dans les Paramètres)`) :
le choisir ouvre sa ligne dans les Paramètres (Paramètres > Général) et laisse le périmètre comme il était. Le dossier de
clonage compte comme défini grâce à son défaut selon le système (le dossier sur lequel s'ouvre la boîte Cloner) tant que ce
dossier existe ; le dossier de navigation n'a pas de défaut. La liste des périmètres est relue dans les paramètres à chaque
ouverture : une modification faite là s'applique tout de suite.

## La liste

| Groupe | Ce qu'il contient |
|---|---|
| Épinglés | Tout ce que vous avez épinglé, quel que soit son type, dans l'ordre d'épinglage |
| À relire par moi | Les demandes qui demandent votre revue (et que vous n'avez pas écrites) |
| Mes demandes | Les demandes que vous avez écrites ; la pastille dit `brouillon` pour un brouillon |
| Qui me sont assignées | Les demandes qui vous sont assignées |
| Qui me mentionnent | Les demandes et tickets ouverts qui vous mentionnent et ne sont pas déjà dans un groupe ci-dessus (un ticket qui ne vous est ni assigné ni créé par vous n'apparaît ici que s'il vous mentionne) |
| Autres demandes | Les autres demandes ouvertes |
| Tickets qui me sont assignés | Les tickets ouverts qui vous sont assignés |
| Tickets créés par moi | Les tickets ouverts que vous avez créés |
| Travail en cours | Les dépôts du périmètre avec des modifications non commitées ou des commits non poussés |
| Reportés | Les éléments que vous avez masqués un moment ; repliés tant que vous ne les ouvrez pas |
| Dépôts | Les dépôts qui ont quelque chose à dire (hors ligne, jeton à renouveler, non lié) |

Les mots « pull request » et « merge request » suivent le réglage habituel de
terminologie et l'hébergeur.

Chaque ligne montre le type de l'élément (`⇄` demande, `●` ticket, `✎` travail en
cours), la pastille du dépôt (de la couleur de l'espace de travail), le numéro, le
titre, l'auteur, l'âge et, quand il y a de la place, les étiquettes. Une ligne de
travail en cours montre la branche et ce qui l'attend : `✎` fichiers modifiés, `+`
nouveaux fichiers, `↑` commits à pousser, `↓` commits à tirer.

Les nombres sont dans chaque titre. Dans un groupe, l'élément le plus récemment mis
à jour vient en premier.

La ligne d'état sous le filtre affiche un indicateur d'activité quand les
hébergeurs mettent plus d'un instant à répondre. Les listes en cache (le même cache
que celui de la barre latérale) s'affichent tout de suite et sont remplacées à
mesure que les réponses arrivent.

### Colonnes

La pastille du dépôt, le `#` (numéro et marques de vérification), l'auteur et l'âge sont des **colonnes dimensionnables**. Laissée en automatique, la colonne du dépôt est
aussi large que le nom le plus long à l'écran (jusqu'à 40 cellules, et au plus un tiers de la fenêtre), si bien que
`owner/name` et le `group/sub/project` d'une liste de compte ne sont pas coupés sur un terminal normal. Le titre prend
le reste. Un nom qui ne tient pas se termine par une ellipse (`group/sub...`), et :

- **Arrêtez le pointeur** sur un dépôt, un titre, un auteur ou une liste d'étiquettes coupés : une infobulle montre le
  texte entier. Sélectionnez la ligne et la ligne d'état sous le filtre nomme aussi le projet et le titre en entier, si
  bien que le clavier voit la même chose.
- **Faites glisser le séparateur** (la cellule juste après le nom du dépôt, après le `#`, ou juste avant l'auteur ou l'âge) pour dimensionner la
  colonne ; **double-cliquez** dessus pour l'ajuster de nouveau. La liste a une **ligne d'en-tête** (dépôt, `#`,
  titre, auteur, âge) et les séparateurs suivent `ui.column_resize` comme ceux du Log : avec `line` (la
  valeur par défaut) un `│` estompé repose dans le séparateur sur la ligne d'en-tête, et tant que le pointeur est sur le séparateur
  (sur l'en-tête ou sur n'importe quelle ligne de la liste), que la colonne est choisie avec les touches ou que vous la glissez, un trait fin d'accent est
  tracé sur toute la liste ; un glissement depuis n'importe quel point du trait redimensionne. Une fois relâché, le trait disparaît sauf si le
  pointeur est encore sur le séparateur (il part alors avec le pointeur) : un glissement à la souris ne laisse jamais une colonne choisie. Avec `header`, il n'y a ni repère ni trait : la première
  cellule après le séparateur, sur la ligne d'en-tête, s'allume et est le seul endroit d'où glisser. Le pointeur devient
  un pointeur de redimensionnement sur un séparateur dans les terminaux qui le gèrent. La pastille d'état a une
  largeur fixe, et l'âge aussi (l'âge le plus large affiché, tant que vous ne le dimensionnez pas ; un âge coupé par une colonne étroite montre sa
  date complète dans une infobulle), si bien que l'auteur commence à la même cellule sur chaque ligne.
- Ou utilisez les touches : **`C`** choisit une colonne (encore, ou `Tab`, pour la suivante), **`]`** l'élargit de deux
  cellules, **`[`** la réduit, **`0`** remet toutes les colonnes en automatique, `Esc` quitte. La palette a les mêmes
  commandes : `Radar : Choisir une colonne`, `Radar : Élargir la colonne`, `Radar : Réduire la colonne`, `Radar : Réinitialiser les colonnes`.

Les largeurs sont gardées dans `radar.toml` (`[columns]`, `repo`, `number`, `author` et `age`, en cellules) et survivent à un redémarrage.
Ce sont les mêmes gestes de dimensionnement que pour les colonnes du Log.

### Vérifications et conflits

Une ligne de demande porte deux petites marques après son numéro quand l'hébergeur a dit quelque chose :

| Marque | Signification |
|---|---|
| `✓` | les vérifications du commit de tête ont réussi |
| `✗` | une vérification a échoué |
| `●` | des vérifications sont en file d'attente ou en cours |
| `!` (à côté) | la demande entre en conflit avec sa branche cible |

Posez le pointeur sur les marques et une infobulle le dit en mots (« Vérifications en échec », « En conflit avec main ») ;
sélectionnez la ligne et la ligne d'état sous le filtre dit la même chose, si bien que le clavier le voit aussi. Une demande
sans marque est une demande dont l'hébergeur n'a rien dit (pas de CI configurée, ou pas demandé).

- **GitLab** envoie l'état du pipeline et l'indicateur de conflit avec la liste des merge requests : chaque ligne a
  ses marques.
- **GitHub** ne le fait pas : la liste n'a ni l'un ni l'autre. GitPoulpi les demande par demande (la demande elle-même pour
  son état de merge, l'état combiné du commit de tête et ses check runs : trois appels), pour au plus
  **15 demandes ouvertes par dépôt**, les vôtres d'abord. Si GitHub refuse l'un de ces appels (une limite de débit),
  les autres restent sans marque jusqu'à la prochaine actualisation. Les réponses sont enregistrées avec le cache d'hébergement.

- **Gitea et Forgejo** sont lus comme GitHub : la liste porte l'indicateur de conflit (`mergeable`), l'état des
  commits est demandé demande par demande (15 par dépôt au plus, les vôtres d'abord), et les tickets forment les mêmes
  groupes. Les mentions utilisent la recherche du serveur (`mentioned_by`).

### Mentions

**Qui me mentionnent** et le filtre **Afficher : Qui me mentionnent** utilisent ce que l'hébergeur sait des mentions. GitHub est
interrogé par sa recherche (`mentions:`), GitLab par votre liste de tâches : seules les mentions encore **en attente**
y comptent, si bien qu'une tâche que vous avez marquée comme faite quitte le groupe.

Les demandes fermées et mergées et les tickets fermés ne sont pas listés.

## Vues et filtres

Trois listes déroulantes et une zone de texte restreignent la liste. Chaque choix
est une liste déroulante : cliquez dessus, ou utilisez sa touche.

- **Vue** (`v`) : Tout, demandes, Tickets, WIP, Reportés.
- **Afficher** (`f`) : Tout le monde, Les miens (demandes que vous avez écrites,
  tickets que vous avez créés), Qui me sont assignés, Revue demandée, Qui me mentionnent.
- **Périmètre** (`w`) : voir plus haut.
- **Texte du filtre** (`/` ou `Ctrl+F`) : correspondance approximative sur le dépôt,
  le numéro, le titre, l'auteur, la branche et les étiquettes. `Enter` rend les
  touches à la liste et garde le texte, `Esc` l'efface.

`Ctrl+N` affiche un badge sur le filtre, chaque liste déroulante, les boutons Actualiser et Retour et chaque titre de section à l'écran. Appuyez sur le caractère d'un badge : le filtre reçoit le clavier, une liste déroulante s'ouvre, un bouton s'exécute et un titre reçoit la sélection (`Enter` ou `Space` le replie ensuite). `Esc` annule. Voir [Raccourcis clavier](keybindings.md).

## Actions

Sélectionnez une ligne et utilisez son menu (clic droit, le `⋮` au bout de la ligne,
`.` ou `Menu`), une touche, ou la palette (`Radar : ...`). Une action à la souris
a toujours une touche et une jumelle dans la palette.

| Action | Touche | Palette |
|---|---|---|
| Ouvrir le dépôt (son onglet s'ouvre, ou est activé ; une demande non clonée demande comment la relire) | `Enter` / double-clic | `Radar : Ouvrir le dépôt` |
| Relire... (une demande ; une demande non clonée demande comment) | `e` | `Radar : Relire la pull request...` |
| Relire une demande via l'API de l'hébergeur (sans clone) | `a` dans la question | `Radar : Relire via l'API` |
| Cloner le dépôt, puis relire la demande | `c` dans la question | `Radar : Cloner et relire...` |
| Ouvrir dans le navigateur | `o` | `Radar : Ouvrir dans le navigateur` |
| Basculer sur la branche (une demande) | `c` | `Radar : Basculer sur la branche` |
| Créer un worktree... (une demande) | `t` | `Radar : Créer un worktree...` |
| Afficher dans la liste du dépôt | `s` | `Radar : Afficher dans le dépôt` |
| Commencer à travailler sur un ticket... (un ticket) | `n` | `Radar : Commencer à travailler sur un ticket...` |
| Épingler en haut, ou désépingler | `p` / clic sur le `★` | `Radar : Épingler ou désépingler` |
| Modifier les assignés... / les relecteurs... / les étiquettes..., Définir le jalon... (GitHub, GitLab) | menu de la ligne | `Radar : Modifier les assignés...`, `Radar : Modifier les relecteurs...`, `Radar : Modifier les étiquettes...`, `Radar : Définir le jalon...` |
| Reporter... | `z` | `Radar : Reporter...` |
| Ne plus reporter | `u` | |
| Afficher ou masquer les demandes des dépôts non clonés | `a` | `Radar : Afficher les demandes non clonées` |
| Afficher ou masquer les brouillons des lignes à l'échelle du compte | `d` | `Radar : Afficher les brouillons` |
| Activer les demandes de relecture à l'échelle du compte d'un hébergeur | `T` / `Enter` sur sa ligne d'invite / **Activer** | `Radar : Activer les relectures du compte...` |
| Actualiser | `r`, `F5` | `Radar : Actualiser` |
| Le menu de la ligne | `.` | `Radar : Menu de la ligne...` |
| Retour à l'accueil | `Esc` | `Radar : Retour à l'accueil` |

Les actions qui ont besoin de la fenêtre du dépôt lui-même (basculer, worktree,
afficher, commencer à travailler, lier, les sélecteurs de métadonnées) passent d'abord à son onglet et s'exécutent
là une fois qu'il est chargé :

- **Basculer sur la branche** utilise le checkout de la barre latérale : la branche
  quand elle est connue localement, sinon la tête de la demande récupérée dans une
  branche locale (celle d'un fork, par exemple).
- **Créer un worktree...** ouvre la boîte de dialogue de création de worktree sur la
  branche de la demande.
- **Afficher dans le dépôt** sélectionne l'élément dans la liste PULL REQUESTS ou
  ISSUES.
- **Commencer à travailler sur un ticket...** ouvre l'invite Nouvelle branche avec un
  nom construit à partir du ticket (`12-fix-the-thing`).

### Épingler et reporter

Une **épingle** garde un élément en haut. Un **report** le masque pendant **1
heure**, **4 heures**, **jusqu'à demain** (8 h le lendemain matin, heure locale, même s'il n'est pas encore 8 h maintenant) ou **jusqu'à un changement**
(l'hébergeur signale une mise à jour de l'élément ou, pour une ligne de travail en
cours, ses compteurs bougent). Les deux sont gardés dans `radar.toml` dans votre
dossier de configuration, indexés par hébergeur, dépôt, type et numéro, et sont
partagés par toutes les sessions de la machine.

### Dépôts qui ont quelque chose à dire

Un dépôt qui a un problème a sa propre ligne à la fin de la liste, et ses éléments
en cache restent à l'écran :

- **hors ligne, il y a 5m** : l'hébergeur n'a pas pu être joint ;
- **Reconnecter** : le jeton est absent ou refusé ; le bouton ouvre Paramètres > Intégrations ;
- **en pause pour ce tour** : l'hébergeur a demandé de ralentir (ou a refusé le
  jeton), donc GitPoulpi ne l'a pas réinterrogé pour les autres dépôts de ce tour ;
- **non lié, GitHub suggéré** : le distant du dépôt correspond à un profil, mais
  l'hébergement est facultatif par dépôt, donc rien n'est demandé tant que vous ne
  l'avez pas lié. **Lier...** vous amène à la section PULL REQUESTS du dépôt, où
  vous choisissez le fournisseur et le projet.

Au plus quatre dépôts sont interrogés en même temps, et jamais sur le thread qui
dessine l'écran.

## Au-delà du périmètre : les lignes à l'échelle du compte

Le périmètre couvre les dépôts que vous avez clonés. Une demande qui attend votre relecture dans un dépôt que
vous n'avez jamais cloné y est invisible. Les **lignes à l'échelle du compte** interrogent l'hébergeur lui-même,
une fois par profil, et rejoignent la liste quel que soit le périmètre : elles parlent de vous, pas d'un dossier.

Elles sont **désactivées par défaut** (la recherche d'un hébergeur coûte du quota d'API). Tant qu'un hébergeur de
code connecté n'a aucune de ses listes activée, le haut de la liste montre une ligne pour lui :

`GitHub : lister les demandes qui attendent votre relecture, y compris dans les dépôts non clonés  [Activer]`

Cliquez sur **Activer**, appuyez sur `Enter` sur cette ligne, appuyez sur `T`, ou lancez `Radar : Activer les relectures du compte...` :
cela écrit `integrations.radar.review_requests = true` pour ce profil dans `config.toml` et
interroge aussitôt l'hébergeur. Les autres listes (toutes les demandes ouvertes de vos espaces de noms, les tickets qui vous sont assignés,
les brouillons, les espaces de noms) se règlent avec `gitpoulpi hosting integrations radar` ou dans les réglages du profil ; voir
la [référence de configuration](config-reference.md) et la [référence CLI](cli-reference.md). GitHub, GitLab,
Gitea, Forgejo, Azure DevOps, Bitbucket Data Center et Gerrit répondent ; Bitbucket Cloud n'a pas de point d'accès
à l'échelle du compte, donc ses demandes ne s'affichent que pour les dépôts liés du périmètre.

Les **Espaces de noms** du profil limitent toutes ses listes à certains dépôts (Paramètres > Intégrations > Radar : une zone de texte avec une entrée par ligne, et une case **Expressions régulières**). Une ligne correspond au chemin complet d'un dépôt quand elle lui est égale ou en est un préfixe qui s'arrête à un `/` : `orbit` garde `orbit/products/x` et écarte `orbitx/y` et `training/y`. Avec la case cochée, chaque ligne est une expression régulière appliquée au chemin complet tel que vous l'écrivez (`^orbit/(products|infra)/`) ; une ligne invalide s'affiche sous le champ et n'est pas enregistrée. Vide signifie pas de limite.

Comment les lignes sont placées :

- Une ligne que le périmètre liste déjà (même hébergeur, projet, type et numéro) apparaît une seule fois : **la
  ligne du périmètre l'emporte**.
- Une ligne dont vous avez cloné le projet quelque part (dépôts récents, espaces de travail, dossiers trouvés par les
  périmètres de dossiers ; reconnu par l'URL du distant) porte ce clone : **Enter**, basculer sur la branche, worktree et les
  autres l'ouvrent comme n'importe quelle ligne.
- Toute autre ligne est **non clonée** : la pastille du dépôt est atténuée et montre le projet (`acme/app`), et
  la ligne d'état dit `Non clonés` quand elle est sélectionnée. **Ouvrir dans le navigateur** fonctionne, **Relire** demande comment
  la relire (voir plus bas), et basculer sur la branche, worktree, afficher dans le dépôt et commencer à travailler disent
  `clonez d'abord le dépôt`.

### Relire une demande

**Relire** (`e`) sur une demande des dépôts du périmètre ouvre l'onglet de ce dépôt avec la revue
dedans. Sur une demande d'une ligne à l'échelle du compte, la façon de faire dépend de l'existence d'un clone connu et du réglage
du profil **Demande d'un dépôt non cloné** (Paramètres > Intégrations, `review_mode`
dans [la référence de configuration](config-reference.md)) :

| Réglage | Un clone est connu | Aucun clone n'est connu |
|---|---|---|
| Demander (par défaut) | demande : **Ouvrir le clone local** (`l`) ou **Relire via l'API** (`a`) | demande : **Cloner, puis relire** (`c`) ou **Relire via l'API** (`a`) |
| Le cloner | ouvre le clone | ouvre la boîte Cloner |
| Relire via l'API | relit via l'API | relit via l'API |

La question est une petite boîte de dialogue : les lettres choisissent, **Enter** choisit la ligne en surbrillance, `Esc`, **Annuler**,
le `✕` et un clic à l'extérieur la ferment, et **Mémoriser pour GitHub** (`r`) écrit le réglage du profil pour que la
question ne soit plus posée (la ligne des Paramètres le change à nouveau). `Ctrl+N` badge ses contrôles.

- **Relire via l'API** ouvre la revue dans la fenêtre Accueil, par-dessus le Radar, avec le code pris chez
  l'hébergeur (voir [Relire sans clone](review.md#relire-sans-clone)). `Esc` ou le `✕` rend
  le Radar ; les lignes *Revue : ...* de la palette y fonctionnent.
- **Cloner, puis relire** ouvre la boîte Cloner sur le compte connecté d'où vient la ligne (la même liste de dépôts,
  groupée par propriétaire), avec le dépôt de la demande sélectionné et le dossier de clone par défaut rempli ; changez
  l'un ou l'autre, puis clonez, avec la même connexion que n'importe quel clone depuis ce compte. Si le dépôt n'est pas dans la
  liste du compte (vous n'en êtes pas membre), la liste reste ouverte avec la recherche sur son nom et une note le dit ;
  pour un hébergeur qui ne sait pas lister les dépôts, la boîte s'ouvre sur **Cloner avec une URL** avec l'adresse remplie. Quand le clone est arrivé, le nouveau dépôt est lié à l'hébergeur
  (sa source PULL REQUESTS, et sa source ISSUES quand l'hébergeur a des tickets et qu'aucune n'est définie), son onglet s'ouvre et
  la revue de cette demande y est ouverte. Annuler la boîte, ou un clone qui échoue, ne lie rien.
- Les demandes Azure DevOps et Bitbucket n'ont pas de revue dans l'appli : **Relire** ouvre la page de la demande dans le
  navigateur et le dit sur la ligne d'état.

Les lignes de palette *Radar : Relire via l'API* et *Radar : Cloner et relire...* sautent la question.

Deux cases à côté du filtre (elles apparaissent dès qu'une liste à l'échelle du compte est demandée), gardées dans
`radar.toml` :

| Case | Touche | Par défaut | Effet |
|---|---|---|---|
| `[x] Non clonés` | `a` | activée | Liste les lignes des dépôts sans clone connu. Désactivée, seules les lignes des clones connus restent. |
| `[ ] Brouillons` | `d` | désactivée | Liste les demandes en brouillon des lignes à l'échelle du compte. Les brouillons des dépôts du périmètre sont toujours listés. |

Les deux sont sur les badges `Ctrl+N` et dans la palette (`Radar : Afficher les demandes non clonées`, `Radar : Afficher les brouillons`).

La réponse d'un profil est gardée dans `radar-account.json` pendant **5 minutes** : rouvrir le Radar dans ce
délai montre les lignes sans interroger l'hébergeur. **Actualiser** (`r` ou `F5`) interroge toujours : chaque dépôt et chaque liste à l'échelle du compte, quel que soit l'âge de la dernière réponse. Un hébergeur qui a demandé de ralentir
ou refusé le jeton n'est pas réinterrogé au même tour. Au plus 300 lignes sont gardées par profil et par type,
et une ligne sous **Dépôts** dit combien d'autres existaient, ou pourquoi un profil n'a pas pu être lu
(**Reconnecter** ouvre Paramètres > Intégrations).

## Limites

- Les lignes à l'échelle du compte viennent des profils GitHub, GitLab (y compris auto-hébergé), Gitea, Forgejo, Azure DevOps,
  Bitbucket Data Center et Gerrit (pas Bitbucket Cloud). Les tickets qui vous sont assignés viennent des cinq
  premiers ; Bitbucket et Gerrit n'ont pas de tickets.
- Les listes par dépôt utilisent les profils GitHub, GitLab (y compris auto-hébergé), Gitea, Forgejo, Azure DevOps,
  Bitbucket et Gerrit ; les vérifications, conflits et mentions ne sont lus que pour les quatre premiers.
- Les mentions GitLab ne viennent que des tâches en attente ; les vérifications GitHub, Gitea et Forgejo sont demandées pour 15 demandes par dépôt.
