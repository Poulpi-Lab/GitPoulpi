[English](../branching-workflows.md) · [Čeština](../cs/branching-workflows.md) · [Deutsch](../de/branching-workflows.md) · [Español](../es/branching-workflows.md) · **Français** · [Italiano](../it/branching-workflows.md) · [日本語](../ja/branching-workflows.md) · [한국어](../ko/branching-workflows.md) · [Nederlands](../nl/branching-workflows.md) · [Polski](../pl/branching-workflows.md) · [Português (Brasil)](../pt-BR/branching-workflows.md) · [Português (Portugal)](../pt-PT/branching-workflows.md) · [Русский](../ru/branching-workflows.md) · [Svenska](../sv/branching-workflows.md) · [Türkçe](../tr/branching-workflows.md) · [Українська](../uk/branching-workflows.md) · [简体中文](../zh-CN/branching-workflows.md)

# Workflows de branches

Un workflow de branches dit à GitPoulpi comment votre équipe utilise les branches : lesquelles vivent
pour toujours (`main`, `develop`, `production`), lesquelles sont des branches thématiques de courte durée
(`feature/...`, `hotfix/...`), comment elles sont nommées, et d'où chacune part et où elle se termine. C'est
Git Flow rendu personnalisable : des noms fixes, des préfixes et des suffixes, ou des modèles avec des
variables, de sorte que les branches qu'un bot crée à partir d'un ticket sont reconnues comme les vôtres.

Cette page couvre le modèle, le fichier, les modèles prédéfinis, la détection et l'import, les commandes
en lecture seule, les actions qui démarrent, terminent, synchronisent et promeuvent des branches (CLI, MCP et
application), et les écrans de l'application : l'éditeur des paramètres, la barre latérale groupée, le menu
`Flow` et ses fenêtres.

## Concepts

- Une **branche de base** est durable. Une base peut avoir un **parent** (`develop` dans `main`) ; un tronc n'en
  a pas. Une base peut lister plusieurs noms possibles (`main` ou `master`, `develop` ou `dev`) : celui qui
  existe est utilisé.
- Un **type de branche** est une sorte de branche de courte durée (feature, bugfix, release, hotfix, support...).
  Il a un **modèle de nom**, un **parent** (d'où elle part et dans quoi elle est terminée) et une façon de
  se terminer. Une branche de ce type est une **branche thématique**.
- Chaque branche est **classée** : une base (d'après son nom), une branche thématique d'un type (d'après un
  modèle), ou non classée.
- Le **parent d'une branche** est, dans l'ordre : celui enregistré quand GitPoulpi a créé la branche
  (`git config branch.<nom>.gitpoulpi-parent`), sinon `git-town-branch.<nom>.parent` de Git Town s'il est
  présent, sinon le parent de son type.

## Où c'est stocké

| Quoi | Où | Commité |
|---|---|---|
| Le workflow partagé | `.gitpoulpi/workflow.toml` dans le dépôt | Oui, par vous : GitPoulpi ne l'écrit que sur votre demande |
| Vos ajustements | `[workflow]` dans `.gitpoulpi/repo-state.toml` | Non (ce fichier est ignoré) |

Le dossier d'état s'ignore lui-même (`.gitpoulpi/.gitignore`), sauf `workflow.toml`, qui est fait pour être
commité. Le workflow effectif est le fichier partagé avec votre surcharge par-dessus : une base de même nom, ou
un type de même id, est remplacé ; les nouveaux sont ajoutés ; `disable_bases` et `disable_topics` retirent
des éléments partagés pour vous seul.

Enregistrer par-dessus un fichier existant conserve les clés que cette version ne connaît pas (celles d'un
GitPoulpi plus récent, ou d'un collègue), mais pas les commentaires ni l'ordre des clés.

## Le fichier

```toml
template = "git-flow"          # git-flow | github-flow | gitlab-flow | trunk | custom (informatif)

[[base]]
name = "main"
names = ["master"]             # autres écritures de la même branche
protected = true               # jamais supprimée, les commits directs sont suspects

[[base]]
name = "develop"
names = ["dev"]
parent = "main"
merge = "merge"                # merge | squash | rebase | fast-forward
tag_on_merge = false
# environment = true           # une branche d'environnement (GitLab Flow) : promue en aval

[[topic]]
id = "feature"
label = "Feature"
pattern = "feature/{slug}"
patterns = ["features/{slug}"] # d'autres noms qui correspondent aussi ; jamais utilisés pour créer
# regex = "^feat-(?P<slug>.+)$"  # correspond aussi ; les groupes nommés deviennent des champs
parent = "develop"             # d'où elle part, et la cible de fin par défaut
finish = [{ into = "develop", merge = "squash" }]
finish_mode = "merge-request"  # local-merge | merge-request | ask
delete_after_finish = true

[hooks]                        # commandes shell facultatives, toujours montrées avant de s'exécuter
after_start = []
before_finish = ["cargo test"]

[issue_flow]                   # flux pilotés par l'hôte, voir plus bas
mode = "local"                 # local | host-creates-branch | bot
```

`merge`, `finish`, `finish_mode` et `hooks` pilotent les [actions](#actions) ci-dessous ; `issue_flow` pilote
les [flux pilotés par l'hôte](#flux-pilotés-par-lhôte).

### Modèles de nom

Du texte avec des `{variables}` :

| Variable | Correspond à | Produite à partir de |
|---|---|---|
| `{slug}` | du texte libre ; peut contenir `/` en dernier | le texte, en minuscules, sans accents, avec `-` entre les mots, 50 caractères au plus |
| `{issue}` | des chiffres (`123`) | `123` ou `#123` |
| `{issue_key}` | une clé de suivi (`PROJ-123`) | telle quelle |
| `{user}` | un nom sans `/` | transformé en slug |
| `{date}` | `2026-10-16` ou `20261016` | telle quelle |
| `{version}` | `1.2.0`, `v1.2`, `1.2.0-rc1` | telle quelle |
| `{n}` | des chiffres | telle quelle |

Tout le reste est littéral : les préfixes et les suffixes sont donc du simple texte :
`feature/{slug}`, `{user}/{issue}-{slug}-wip`, `bot/{issue_key}`. Chaque variable ne peut apparaître qu'une
fois, et deux variables ont besoin de texte entre elles. Un modèle doit produire des noms de branche valides
(les règles de `git check-ref-format`) : pas d'espaces, pas de `~ ^ : ? * [ \`, pas de `..`, aucun composant qui
commence par `.` ou se termine par `.lock`.

Une `regex` ne sert qu'à reconnaître (elle est ancrée aux deux bouts) ; utilisez-la pour des noms qu'aucun
modèle ne sait exprimer. Les groupes nommés deviennent les champs de la branche.

### Quel type l'emporte

1. Une branche nommée comme une base (l'un quelconque de ses noms) est cette base.
2. Sinon, chaque modèle de chaque type qui correspond est candidat : `pattern`, puis `patterns`, puis `regex`,
   type par type dans l'ordre du fichier. Celui qui fixe le plus de caractères littéraux l'emporte
   (`release/{version}` bat `{user}/{slug}`, qui bat un simple `{slug}`) ; une regex compte son préfixe littéral.
   En cas d'égalité, le premier du fichier gagne.
3. Sinon la branche est non classée.

Un modèle ou une regex cassé est signalé comme avertissement et ignoré ; le reste continue de fonctionner.

## Modèles prédéfinis

| Modèle | Bases | Types de branche |
|---|---|---|
| `git-flow` | `main` ou `master` ; `develop` ou `dev` (parent main) | `feature/` et `bugfix/` (depuis develop), `release/` (depuis develop, dans main et develop, avec un tag), `hotfix/` (depuis main, dans main et develop, avec un tag), `support/` (depuis main) ; les pluriels (`features/`, `releases/`...) correspondent aussi |
| `github-flow` | `main` | `feature/` ou n'importe quel nom, depuis main, demande de fusion dans main, squash, supprimée ensuite |
| `gitlab-flow` | `main` ; `pre-production` et `production` (branches d'environnement, chaînées main, pre-production, production) | `feature/` ou n'importe quel nom depuis main ; `release/{version}` comme cible de cherry-pick |
| `trunk` | `main` | `{user}/{slug}` de courte durée depuis main, rebasé |
| `custom` | aucune | aucun |

Les branches d'environnement correspondent aussi sous les noms `preprod`, `staging` et `prod`. Un modèle est un
point de départ : affichez-le avec `flow config init`, puis modifiez le fichier.

## Détection et import

`gitpoulpi flow detect` classe les modèles d'après les branches du dépôt, avec un score sur 100 et les noms
concrets trouvés (`master` pour `main`, `dev` pour `develop`, `features/` pour `feature/`). Les bases comptent
pour 60 points, la part des autres branches que reconnaissent les modèles spécifiques du modèle pour 40, et une
branche durable d'un autre modèle (un `develop` dans un candidat GitHub Flow) en retire 25.

`gitpoulpi flow import --from git-flow` lit les paramètres classiques de git-flow (`gitflow.branch.master`,
`gitflow.branch.develop`, `gitflow.prefix.feature|bugfix|release|hotfix|support|versiontag`) dans le
`.git/config` propre au dépôt ; `--from git-town` lit `git-town.main-branch` et `git-town.perennial-branches`
(le parent de chaque branche, `git-town-branch.<nom>.parent`, est repris comme parent enregistré). Rien n'est
appliqué en silence : la commande affiche le TOML, et `--write` crée `.gitpoulpi/workflow.toml` seulement s'il
n'existe pas.

## Commandes

Toutes acceptent `--json` ; le serveur MCP les expose comme outils (`flow_status`, `flow_templates`,
`flow_detect`, `flow_config_show` lisent ; les actions sont décrites plus bas).

| Commande | Ce qu'elle fait |
|---|---|
| `gitpoulpi flow status` | Chaque branche locale et distante : sorte (base, branche thématique, non classée), type et champs, parent et son origine (`recorded`, `git_town`, `type`, `base`), et pour les branches locales de combien de commits elle est en avance (`ahead`) et en retard (`behind`) sur son parent. Aussi les avertissements trouvés dans la définition, la détection classée et les paramètres d'un autre outil qu'on pourrait importer. |
| `gitpoulpi flow templates` | Les modèles prédéfinis ci-dessus, avec leur définition complète. Pas besoin de dépôt. |
| `gitpoulpi flow detect` | Les modèles classés avec les bases et les types trouvés. |
| `gitpoulpi flow config [--show-effective]` | Le fichier partagé (son chemin, s'il existe, le workflow analysé, les clés inconnues), ou avec `--show-effective` le workflow selon lequel vos branches sont classées. `flow config show` est identique. |
| `gitpoulpi flow config init --template <t> [--write]` | Affiche le TOML d'un modèle ; `--write` crée `.gitpoulpi/workflow.toml` et refuse d'en écraser un. |
| `gitpoulpi flow import --from git-flow\|git-town [--write]` | Comme décrit plus haut. |

Les compteurs d'avance et de retard sont calculés pour 300 branches locales au plus par appel.

Un fichier de workflow illisible (TOML invalide, valeur d'un mauvais type) est une erreur qui nomme le fichier ;
elle n'est jamais ignorée en silence.

## Actions

`start`, `finish`, `sync`, `promote` et `set-parent` agissent sur le dépôt. Chacune commence par **planifier** :
elle liste chaque étape git qu'elle va faire, avec la commande git correspondante, les avertissements, ce qui
ne peut pas être annulé et tout ce qui la bloque. Rien ne change avant votre confirmation. Chaque exécution est
**une entrée de l'historique d'annulation** (`gitpoulpi undo` reprend ensemble les branches, le tag, la branche
supprimée, le parent enregistré et le checkout).

| Commande | Ce qu'elle fait |
|---|---|
| `gitpoulpi flow start <type> [nom=valeur ...] [--from <ref>] [--publish]` | Nomme la branche d'après le modèle du type (`flow start feature "Login form"` crée `feature/login-form` ; `flow start release version=1.2.0` crée `release/1.2.0` ; un mot seul est le `slug` ; `user` et `date` valent par défaut vous et aujourd'hui), la crée depuis le parent du type ou `--from` (une branche, une branche distante ou un tag), la bascule et enregistre son parent. `--publish` la pousse avec `-u` (non annulé par `undo`). |
| `gitpoulpi flow finish [<branche>]` | Pour chaque cible `finish` du type, dans l'ordre : bascule sur la cible et merge la branche avec la stratégie de la cible (`merge` : toujours un commit de merge ; `squash` : un seul commit intitulé `--title` ou d'après le nom de la branche ; `rebase` : rebase la branche sur la cible, puis fast-forward ; `fast-forward`), crée le tag si la cible en a un (`tag = "v{version}"`, annoté), puis bascule sur le parent et supprime la branche locale quand le type le dit (`--delete` / `--keep` le remplacent ; `--delete-remote` la supprime aussi sur le distant, non annulable). Avec `finish_mode = "merge-request"` (ou `--mode merge-request`), elle ne fait que pousser la branche et ouvrir la pull ou merge request vers le parent, avec `--title`, `--draft`, `--assignee`, `--reviewer`, `--label`, `--milestone` et `--integration` (non annulable ; rien n'est fusionné ni supprimé en local). `--mode local-merge` remplace un type `merge-request`, et un type qui dit `ask` demande l'un des deux. |
| `gitpoulpi flow sync [<branche>] [--stack] [--rebase\|--merge]` | Amène les commits du parent dans la branche : un merge, ou un rebase quand la stratégie de la branche est `rebase` (ou `--rebase`). Avec `--stack`, aussi les branches dont le parent est celle-ci (enregistré avec `set-parent`), récursivement, dans l'ordre. Revient à la branche où vous étiez. |
| `gitpoulpi flow promote <branche-env>` | Branches d'environnement de GitLab Flow : merge dans la branche la branche dont elle est en aval (`main` dans `pre-production`, `pre-production` dans `production`), en une étape, puis revient. Fait un fast-forward quand c'est possible. |
| `gitpoulpi flow set-parent <branche> <parent>` / `--clear` | Écrit (ou retire) `branch.<nom>.gitpoulpi-parent`. Refuse un cycle. Annulable. |

Une branche démarrée ailleurs se termine là : quand le parent enregistré (par `--from` ou `set-parent`) diffère de
celui du type, la première cible de fin est le parent enregistré.

### Confirmer, planifier, scripter

| Option | Sens |
|---|---|
| `--dry-run` | Affiche le plan en JSON (`{"plan": ...}`) et s'arrête. Code de sortie 1 quand quelque chose le bloque. |
| `--yes` | Exécute sans demander. |
| ni l'un ni l'autre | Dans un terminal : le plan est affiché et on vous demande. Sinon (un script, `--json`, un agent) le plan revient sous la forme `{"needs_confirmation": {"plan": ...}}` avec le code de sortie 3 et rien de changé. `start` sans `--publish` ni hooks, `set-parent` et `--abort` n'ont pas besoin de confirmation. |
| `--run-hooks` / `--no-hooks` | Voir [hooks](#hooks). |
| `--autostash` | Met de côté (stash) les modifications locales qui bloquent un checkout ou un merge et les remet ensuite, quel que soit `general.auto_stash`. |

Un plan bloqué en donne la raison : une branche de base là où il faut une branche thématique, un nom qui existe
déjà, un champ manquant (`release/{version}` demande `version=...`), un tag qui existe, une branche cible en
retard sur son distant (`--allow-behind` continue quand même), un merge ou un rebase déjà en cours, un workflow
non configuré.

Avec le paramètre d'**auto-stash** (`general.auto_stash`), les modifications locales qui bloquent un checkout ou
un merge sont mises de côté et restaurées autour de l'action exactement comme pour un checkout simple ; avec
`never`, l'action s'arrête avant de rien changer.

### Conflits

Un merge ou un rebase qui s'arrête sur des conflits laisse l'état de conflit habituel (le bandeau de conflit,
`gitpoulpi conflicts ...`) et le flux est mémorisé. Code de sortie 2, `status: "conflicted"`, les chemins dans
`conflicts`. Résolvez les fichiers et marquez-les résolus, puis :

- `gitpoulpi flow finish --continue` commite le merge (ou continue le rebase, ou commite le squash avec son
  message) et exécute les étapes restantes ;
- `gitpoulpi flow finish --abort` abandonne le merge ou le rebase, revient à la branche où le flux a commencé et
  oublie le reste. Les étapes déjà terminées (la première cible d'une double fin, avec son tag) restent : `undo`
  les reprend.

Un flux arrêté par un hook en échec reprend de la même façon (`--continue` relance cette étape). L'exécution en
conflit et sa suite sont deux entrées d'annulation.

### Demandes de fusion

Le démon pousse la branche ; ouvrir la demande exige votre jeton d'hébergeur, donc la CLI le fait avec le même
code que `gitpoulpi hosting pr` (une intégration qui correspond au distant, sinon la page web préremplie de
l'hébergeur, affichée sous forme d'URL). Le résultat porte `merge_request` (ce qui a été demandé) et `opened`
(ce que l'hébergeur a répondu), ou `open_error` quand la branche est poussée mais que la demande n'a pas pu être
ouverte.

### Hooks

`[hooks]` dans le fichier de workflow liste des commandes shell (`after_start`, `before_finish`). Elles viennent
d'un fichier que votre équipe partage : elles ne **s'exécutent donc jamais sans votre accord** : `--run-hooks`, ou
oui à la question qu'un terminal pose après les avoir listées. Le plan les liste toujours, avec `hooks_run: false`
quand elles ne s'exécuteront pas. Un hook s'exécute dans un shell avec le dépôt comme dossier de travail et
`GITPOULPI_FLOW_ACTION` / `GITPOULPI_FLOW_BRANCH` définies ; sa commande exacte, son code de sortie et toute sa
sortie sont dans le résultat. Un hook qui sort avec un code non nul arrête le flux à cet endroit (un hook
`before_finish` s'exécute avant que rien n'ait changé, c'est donc une simple erreur ; après un hook
`after_start`, la branche existe et le flux peut être continué ou abandonné). `--no-hooks` les ignore sans
demander. Un hook peut tout faire : `undo` ne reprend donc que les changements git.

### Pour les agents (MCP)

Les outils sont `flow_start`, `flow_finish`, `flow_sync`, `flow_promote` et `flow_set_parent` (finish, sync et
promote sont annotés destructifs, donc `allow_mutations` les contrôle). Ils fonctionnent en planifier puis
exécuter : appelez avec `dry_run: true` pour obtenir le plan (un résultat normal), puis de nouveau avec
`yes: true` pour l'exécuter. Sans l'un ni l'autre, l'appel est refusé et porte le plan, donc rien ne s'exécute
par accident. Les hooks ne s'exécutent qu'avec un `run_hooks: true` explicite ; le plan les liste (`hooks`,
`hooks_run`) pour que l'agent puisse montrer d'abord les commandes à l'utilisateur. Après un conflit,
`continue_: true` (l'option est `--continue`) ou `abort: true`.

### Annulation

| Action | Ce que fait `undo` |
|---|---|
| start | Supprime la branche, oublie son parent, rebascule sur la branche précédente. |
| finish | Restaure d'un coup chaque branche déplacée, le tag, la branche supprimée et le checkout. Une branche poussée, une suppression distante et une demande de fusion ouverte restent (le plan le dit). |
| sync | Remet chaque branche mise à jour, fichiers compris pour la branche extraite. |
| promote | Remet la branche d'environnement. |
| set-parent | Restaure le parent enregistré précédent. |

## Flux pilotés par l'hôte

Pour les équipes où un ticket lance le travail : le ticket est écrit à partir d'un modèle, et l'hôte (GitLab,
GitHub, Gitea, Forgejo) ou un bot crée la branche et la merge request. GitPoulpi crée le ticket, attend ce que
l'hôte fait, et le reprend : une branche locale qui suit la branche distante, son parent enregistré (la cible de
la merge request), extraite. Cela demande que la section ISSUES soit liée à un suivi
(`gitpoulpi hosting link issues`) ; le lien PULL REQUESTS dit où vivent les branches et les demandes (le projet
d'ISSUES quand il n'est pas lié).

```toml
[issue_flow]
mode = "bot"                         # local | host-creates-branch | bot
issue_title = "[{type}] {title}"     # {type} {title} {user} {date}
issue_labels = ["workflow::{type}"]  # un bot se déclenche souvent sur une étiquette
description_template = ".github/ISSUE_TEMPLATE/feature.md"   # un fichier du dépôt
branch_from_issue = "{type}/{issue}-{slug}"   # le nom que l'hôte ou le bot donne à la branche
wait_for = "branch-or-mr"            # branch | mr | branch-or-mr
timeout_secs = 600                   # combien de temps `wait` interroge
merge_request = true                 # host-creates-branch : aussi une demande brouillon
assignees = []
```

`{type}` dans `branch_from_issue` est l'id du type de branche ; les autres sont les variables habituelles des
[modèles de nom](#modèles-de-nom) (`{issue}`, `{slug}`, `{issue_key}`...). Une branche appartient au ticket 12
quand elle correspond à ce modèle avec `{issue}` égal à 12 (`feature/12-login` oui, `feature/120-x` non).

| Mode | Après la création du ticket |
|---|---|
| `local` | Rien sur l'hôte. Avec `--pick-up`, un `flow start` du type avec le numéro et le titre du ticket (annulable comme tout start). |
| `host-creates-branch` | GitPoulpi crée la branche **sur l'hôte** depuis le parent du type (GitLab : l'API des branches ; GitHub : `createLinkedBranch`, donc le ticket la montre sous Development ; Gitea/Forgejo : l'API des branches), et une demande **brouillon** qui dit `Closes #12`. Un hôte qui refuse une demande vide (GitHub : pas encore de commits) laisse la branche et le signale par un avertissement. |
| `bot` | Rien : un bot (un gestionnaire de webhook, un job de CI, une GitHub App) crée la branche et la demande. `--wait` interroge jusqu'à ce que `wait_for` soit satisfait. |

Ce que regarde `wait`, pour chaque fournisseur : les liens propres à l'hôte (GitLab
`issues/:iid/related_merge_requests` et `closed_by` ; GitHub les branches liées et les pull requests `connected` /
`cross-referenced` de la chronologie), les branches dont le nom correspond à `branch_from_issue`, et les demandes
ouvertes dont la branche source y correspond. Il interroge avec une pause croissante (1, 2, 3, 5, 8, puis toutes
les 10 s) jusqu'à ce que `wait_for` soit satisfait ou que `timeout_secs` s'écoule, et un hôte lent ou brièvement
injoignable après une réponse est réessayé. Une source que l'hôte ne propose pas (un Gitea n'a pas de liens de
tickets) est ignorée ; quand toutes les sources échouent, la première erreur est la réponse.

| Commande | Ce qu'elle fait |
|---|---|
| `gitpoulpi flow issue new --type <t> --title "..." [--body T \| --body-file F] [--label L] [--assignee A] [--milestone M] [--wait] [--pick-up] [--timeout S] [--integration ID] [--dry-run]` | Crée le ticket à partir du modèle (`issue_title`, `issue_labels`, le fichier de description, `--body` à sa place) et poursuit selon le mode. `--dry-run` affiche ce qui serait créé (`{"plan": ...}`) et ne crée rien. `--pick-up` attend d'abord en mode `bot`. Code de sortie 4 : l'attente a expiré (le ticket existe ; `flow issue wait` continue d'attendre). |
| `gitpoulpi flow issue wait <#12> [--type t] [--for branch\|mr\|branch-or-mr] [--timeout S] [--once] [--pick-up]` | Interroge ce que l'hôte a créé pour un ticket, avec la progression sur stderr dans un terminal ; Ctrl+C annule. `--once` regarde une seule fois. Code de sortie 4 : rien pour l'instant. |
| `gitpoulpi flow pick-up <#12 \| !5 \| branche> [--remote R] [--parent B] [--no-fetch] [--dry-run]` | Fait un fetch, crée la branche locale qui suit `origin/<branche>`, enregistre son parent et la bascule. `#12` cherche sur l'hôte la branche ou la demande du ticket, `!5` une demande, tout le reste est un nom de branche. Le parent est la cible de la demande (`--parent` la remplace), sinon le parent du type. Pas besoin de `--yes` : c'est local et annulable, comme `start`. Un `--dry-run` fait quand même le fetch (il ne déplace que des refs de suivi distantes, et le plan doit voir la branche). |

`pick-up` est une entrée de l'historique d'annulation : `gitpoulpi undo` supprime la branche, oublie le parent et
rebascule sur la branche précédente (le fetch reste, comme tout fetch). Il est planifié comme les autres
actions : `--dry-run` affiche son plan (`create_branch`, `set_upstream`, `checkout`, `record_parent`) et les
blocages habituels s'appliquent (les modifications locales suivent le paramètre d'auto-stash). La requête est
`{"action": "pick_up", "branch": ..., "parent": ...}`.

Les tickets créés par un flux sont **surveillés** (`watched_issues` dans `.gitpoulpi/repo-state.toml`, jamais
commité) : après chaque fetch, l'application compare les branches distantes à `branch_from_issue` et vous dit
quand l'une est apparue pour un ticket surveillé. Une reprise le règle.

### Pour les agents (pilotés par l'hôte)

Les outils sont `flow_issue_new`, `flow_issue_wait` et `flow_pick_up` (écriture, réseau ; `allow_mutations` les
contrôle). `flow_issue_new` avec `dry_run: true` renvoie le plan ; `flow_issue_wait` bloque jusqu'à `timeout`
secondes, donnez donc une valeur courte à un agent et rappelez-le (`once: true` répond tout de suite). Les deux
renvoient du JSON avec `found` (`branches`, `requests`) et `timed_out`.

### Flux pilotés par l'hôte dans l'application

Avec un workflow et un suivi lié, le menu Flow gagne *Nouveau ticket <Type>...* (un par type ; palette
`Flow : Nouveau ticket <Type>...`, et `Flow : Nouveau ticket...`) et *Reprendre une branche créée par l'hôte...*
(palette `Flow : Reprendre une branche de l'hôte...`).

- **Nouveau ticket** demande le type (une liste déroulante) et le titre ; il montre d'où vient la description et
  qui crée la branche. *Reprendre la branche dès qu'elle existe* est coché par défaut. *Créer le ticket* le
  crée, puis en mode `bot` affiche la **carte d'attente** : un spinner, ce qu'elle attend, le temps écoulé et le
  moment où elle abandonne, et **Arrêter d'attendre** (`Esc`). Arrêter ne fait pas perdre le ticket : il reste
  surveillé. Quand la branche est là, elle est reprise et la ligne d'état le dit.
- **Une branche est apparue** est une carte qui s'ouvre après qu'un fetch a amené la branche d'un ticket
  surveillé (quand aucune autre fenêtre n'est ouverte) : *Reprendre* la prend en charge, *Plus tard* la ferme
  (elle n'est plus proposée avant le redémarrage de l'application).
- **Reprendre** accepte `#12`, `!5` ou un nom de branche et montre ce qu'il a trouvé avant de la reprendre.

Tout s'exécute en arrière-plan ; un appel qui dure plus de 150 ms affiche un spinner.

### L'essayer sur un banc d'essai

`scripts/fake-gitlab-bot.sh` est un bot de substitution pour le GitLab Docker (`scripts/test-bench.sh`) : il
surveille un projet pour les tickets portant une étiquette et crée `<type>/<iid>-<slug>` (avec un commit) et une
demande brouillon qui ferme le ticket. `cargo run -p xtask -- mock-host --flow-bare <dépôt bare>` sert la même
chose hors ligne (une API façon GitLab au-dessus d'un vrai dépôt bare, avec le bot).

## Dans l'application

![La barre latérale avec un workflow Git Flow : main et develop marquées comme branches de base, les branches feature dans un dossier Feature, le reste sous Autres](../images/workflow-sidebar.png)

![Démarrer Feature : le type, le slug, le point de départ et le nom de branche qui sera créé](../images/workflow-start.png)

![Paramètres > Dépôt > Workflow : le modèle, la détection et l'éditeur des branches de base](../images/workflow-settings.png)

Tout ce qui suit n'apparaît que pour un dépôt qui a un workflow (un `.gitpoulpi/workflow.toml`, ou une surcharge de
votre part). Sans workflow, rien ne change : la barre latérale est l'arbre ordinaire et il n'y a pas de bouton
`Flow`.

### Paramètres > Workflow

*Paramètres > Paramètres du dépôt > Workflow* modifie la définition. Elle travaille sur un brouillon et n'enregistre
rien avant que vous appuyiez sur un bouton Enregistrer.

- **Modèle** est une liste déroulante des modèles prédéfinis ; en choisir un remplace les bases et les types de
  branche ci-dessous par le modèle (vos hooks et votre flux de tickets restent).
- **Détecter d'après les branches** lit les noms de branches de ce dépôt et montre le modèle le plus proche avec
  son score, les noms de base trouvés (`master` pour `main`), combien de branches chaque type de branche
  reconnaît, avec des exemples, et le score de chaque modèle. **Utiliser le workflow détecté** le charge dans le
  brouillon.
- Quand le dépôt contient la configuration d'un autre outil (`gitflow.*` de Git Flow, `git-town.*` de Git Town),
  **L'importer** est proposé et charge la définition importée dans le brouillon. Rien n'est appliqué avant que
  vous appuyiez dessus.
- Les **Branches de base**, les **Types de branche** et leurs **Règles de fin** forment une seule
  [liste](settings.md#listes) en groupes sur l'onglet **Règles** ; l'onglet **Général** contient le reste
  (modèle, détection, hooks, flux de tickets). Sélectionnez un élément pour le modifier à droite (nom et autres noms, parent, protégée, stratégie de
  merge, tag au merge, branche d'environnement ; libellé, modèle de nom, autres modèles, expression régulière,
  parent, mode de fin, suppression après la fin). **+ Ajouter ▾** propose ce qu'on peut ajouter (une règle de
  fin appartient au type de branche sélectionné) ; ajouter, dupliquer, supprimer et glisser fonctionnent comme
  dans toutes les listes des Paramètres, et les éléments se déplacent au sein de leur groupe. La ligne **Modèle de nom** liste les variables (`{slug} {issue} {issue_key} {user}
  {date} {version} {n}`) ; dessous, un aperçu en direct montre les branches existantes que le modèle reconnaît
  et le nom qu'il créerait, et un modèle qui ne s'analyse pas en dit la raison en rouge, juste sous le champ
  (rien de ce qui est tapé n'est perdu).
- **Hooks** sont des commandes shell, une par ligne (`Après le démarrage`, `Avant la fin`), modifiées dans un
  éditeur multiligne (`Ctrl+S` enregistre, `Esc` annule). Elles exécutent des commandes sur votre machine : la page
  le dit, et les fenêtres les montrent avant qu'elles s'exécutent.
- **Flux de tickets** porte les champs qu'utilisent les [flux pilotés par l'hôte](#flux-pilotés-par-lhôte) (mode,
  titre et étiquettes du ticket, modèle de branche, ce qu'il faut attendre), enregistrés dans le fichier de
  workflow.
- **Problèmes** liste ce que la définition a de faux (un parent qui n'existe pas, un modèle cassé).
- Une barre fixée en bas de la page affiche **● Non enregistré** (ou **Aucun changement**). **Enregistrer dans le
  fichier partagé** écrit le brouillon dans `.gitpoulpi/workflow.toml` (commitez-le), en conservant les clés que
  cette version ne connaît pas, et y intègre votre propre surcharge. **Pour moi seul** ne conserve que la
  différence avec le fichier partagé dans l'état de votre dépôt (`.gitpoulpi/repo-state.toml`, non commité).
  **Abandonner les modifications** relit ce qui est enregistré. Fermer les Paramètres avec un brouillon non
  enregistré demande d'abord confirmation.

### La barre latérale

LOCALES et DISTANTES sont groupées d'après le workflow : d'abord les branches de base, chacune avec un badge
`[base]` (`[base, protégée]` quand elle est protégée), puis un dossier par type de branche, nommé d'après son
libellé avec le nombre de branches (`Feature (3)`), puis les branches qu'aucun type ne revendique sous `Autres`.
Les dossiers se replient comme n'importe quel dossier ; les arbres simples reviennent avec *Grouper les branches
par type* (la ligne du menu Flow) ou la commande de palette `Flow : Basculer le groupement des branches`. Survoler
une branche montre son type, son parent et de combien elle est en retard (et en avance) sur le parent.

### Le menu Flow et ses fenêtres

Le bouton **Flow** de la barre d'outils (`Ctrl+G Shift+W`, palette `Flow : Ouvrir le menu`) liste *Démarrer <Type>...*
pour chaque type de branche, *Terminer la branche courante...*, *Synchroniser avec le parent...*,
*Promouvoir...* (branches d'environnement de GitLab Flow), *Définir le parent...*, la bascule du groupement et
*Paramètres du workflow...*. La palette a les mêmes commandes (`Flow : Démarrer Feature...` est générée pour
chaque type).

- **Démarrer** est un formulaire construit d'après le modèle de nom du type : un champ par variable. `{issue}` et
  `{issue_key}` viennent avec une liste déroulante des tickets du suivi lié, qui remplit aussi `{slug}` d'après
  le titre du ticket ; `{version}` suggère la version qui suit votre plus haut tag de release ; `{user}` est
  votre profil actif ; `{date}` est aujourd'hui. Une ligne montre le nom de branche que le modèle produit (ou
  pourquoi il ne peut pas : une ref invalide, un nom déjà pris). *Partir de* est une liste déroulante (le parent
  du type d'abord), et *Publier* pousse la nouvelle branche.
- **Terminer**, **Synchroniser**, **Promouvoir** et **Définir le parent** demandent ce dont elles ont besoin, puis
  montrent l'**aperçu** que calcule le démon (le même plan qu'affiche `gitpoulpi flow ... --dry-run`) : chaque
  étape dans l'ordre, les avertissements (une cible en retard sur son distant, des branches empilées sur
  celle-ci), et les **blocages** (une cible qui n'existe pas, une cible en retard sur son distant) qui
  désactivent le bouton d'exécution et en disent la raison. Les étapes qu'on ne peut pas reprendre (un push, la
  suppression d'une branche distante, une demande de fusion) sont marquées `!` et colorées, avec une ligne
  « L'annulation ne reprendra pas : ... ». Les hooks qui s'exécuteraient sont listés sous un avertissement, avec
  une case *Exécuter les hooks* (décochée par défaut). Quand le mode de fin du type est `ask`, Terminer demande
  d'abord *Terminer par* : merge en local, ou ouvrir une pull ou merge request.
- L'action s'exécute ensuite avec une marque par étape (un spinner après 150 ms) et la fenêtre montre comment
  chaque étape s'est passée. Toute l'action est **une entrée d'annulation** : `Ctrl+Z` reprend d'un coup le
  démarrage, la fin, la synchronisation ou la promotion (l'Historique d'annulation affiche
  `Workflow finish feature/login`) ; les push, les suppressions distantes et les demandes de fusion restent, comme
  l'aperçu l'annonçait.
- Un **conflit** arrête l'action avec **Continuer** et **Abandonner**. La fenêtre peut être fermée pour résoudre le
  conflit avec les outils de conflit habituels, puis rouverte avec la commande de palette
  `Flow : Continuer ou abandonner l'action interrompue...` (aussi une ligne du menu Flow tant qu'une action est
  interrompue) ; Continuer refuse tant que des fichiers sont encore en conflit et garde les deux boutons.
  Abandonner revient à la branche où l'action a commencé ; les étapes déjà faites restent (l'annulation les
  reprend).
- Une fin par **demande de fusion** pousse la branche, puis ouvre le formulaire habituel *Créer une pull request*
  (*Créer une merge request* sur GitLab) prérempli : la cible est le parent de la branche, le titre vient de la
  branche (et du ticket). Les boutons de métadonnées (assignés, relecteurs, étiquettes, jalon) fonctionnent comme
  dans n'importe quelle pull request. Quand aucune intégration ne répond pour le distant, l'URL de comparaison
  est ouverte dans le navigateur à la place (et affichée sur la ligne d'état).
- Les textes des étapes et des blocages sont en anglais, comme ceux de la CLI ; le texte propre à la fenêtre est
  traduit. Un arbre de travail avec des modifications suit le paramètre `auto-stash`.

Chacune de ces actions est accessible au clavier et à la palette, et chaque liste déroulante est une combobox
(Enter l'ouvre, taper filtre, Enter choisit).

Les lignes pilotées par l'hôte (*Nouveau ticket <Type>...*, *Reprendre une branche créée par l'hôte...*) sont
décrites [plus haut](#flux-pilotés-par-lhôte-dans-lapplication).
