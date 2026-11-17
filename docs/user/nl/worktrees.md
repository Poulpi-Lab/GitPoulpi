[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · **Nederlands** · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Een worktree is een tweede uitgecheckte kopie van de repository: een andere map, op een
andere branch, met dezelfde geschiedenis. De sectie WORKTREES van het Refs-paneel toont ze;
de hoofdworktree is gemarkeerd met `(hoofd)`.

![Het Refs-paneel van een repository met twee gekoppelde worktrees: de sectie WORKTREES toont main (hoofd), hotfix/axis-labels en feature/legend, met de sectie SUBMODULES eronder](../images/worktrees.png)

## De sectie tonen

De sectie is verborgen in een repository die nog geen gekoppelde worktree heeft. Om haar toch
te zien (ze zegt dan "Nog geen worktree: + om er een te maken") gebruik je het palet-commando
**Linkerpaneel: Worktrees-sectie tonen**, vink je **WORKTREES** aan in de toon/verberg-lijst
van het linkerpaneel (klik met rechts op een sectiekop), of voer je een van de
maak-commando's hieronder uit. GitPoulpi onthoudt het voor die repository (zie **Secties van
het linkerpaneel** in [Instellingen](settings.md)); een andere repository zonder worktrees
begint nog steeds met de sectie verborgen.

## Een worktree maken

De **+** op de kop WORKTREES (toetsenbord: `+` in de sectie; palet: **Refs: Worktree
maken...**) opent een menu:

- **Van de huidige branch**: de branch is uitgecheckt in de hoofdmap, dus het dialoogvenster
  stelt een nieuwe branch erop voor (`worktree-<id>`).
- **Van de geselecteerde commit**: grijs tot er een commit is geselecteerd in het Log. De
  dialoogvenster stelt een nieuwe branch `worktree-<id>` voor die daar begint; maak het
  branchveld leeg voor een losgekoppelde worktree.
- **Een branch kiezen...**: het dialoogvenster opent op zijn lijst **Worktree-branch**, om een
  branch te openen die al bestaat.

![Het dialoogvenster Worktree maken: de velden Naam, Map, Ref en Worktree-branch, de regel "nieuwe branch vanaf main", het vak De nieuwe worktree als tabblad openen en de knoppen Annuleren en Maken](../images/worktree-create.png)

Het palet heeft dezelfde drie (**Refs: Worktree maken van huidige branch**, **Git: Worktree
maken op geselecteerde commit...**, **Refs: Worktree maken met een gekozen branch**). De
oudere items werken nog: **Worktree maken van...** in het menu van een branch, tag of commit.

Het dialoogvenster stelt alles voor en laat je het wijzigen. De **Naam** en de nieuwe branch
beginnen allebei als `worktree-<id>`, waarbij `<id>` 8 willekeurige hexadecimale cijfers zijn
(voor beide dezelfde), zodat twee worktrees nooit botsen.

| Veld | Betekenis |
|---|---|
| **Naam** | De mapnaam wordt eruit opgebouwd. **Naam naar branch** (`Alt+N`) zet hem op `worktree-<branch>`: schuine strepen worden `-` en het woord wordt niet verdubbeld (`worktree-foo` en `feature/worktree-x` houden hun eigen naam, `feature/login` geeft `worktree-feature-login`). |
| **Map** | Een naaste van de repository, `<repository>-<naam>`. Hij volgt de Naam tot je in de Map typt; **Bladeren** kiest waar. |
| **Ref** | Waar een *nieuwe* branch begint: een branch of een commit (een lijst; typ om te zoeken). Hij wordt niet gebruikt, en dat staat erbij, zolang een bestaande branch is gekozen. |
| **Worktree-branch** | Eén veld, twee betekenissen, zoals eronder staat. Een naam die geen lokale branch is, maakt een nieuwe branch vanaf de Ref (**nieuwe branch vanaf main**). Een lokale branch die nergens anders is uitgecheckt (kies hem in de lijst of typ zijn naam) wordt daar ongewijzigd uitgecheckt (**checkt deze branch uit**). De eerste rij van de lijst, of een leeg veld, checkt de branch van de Ref uit (of koppelt los op een commit); dat wordt geweigerd als die branch al is uitgecheckt. |
| **Opnieuw** | `Alt+R`. Trekt een nieuwe `<id>` en vervangt de Naam en de branch erdoor, behalve wat je zelf typte (een Naam of branch die je schreef blijft) en een bestaande branch (wordt nooit vervangen). Een Naam van **Naam naar branch** of van de AI telt als gegenereerd. |
| **Naam met AI** | Alleen als AI aan staat (zie hieronder). |
| **De nieuwe worktree als tabblad openen** | Standaard aan. |

Een map die niet leeg is, een ongeldige naam, een branchnaam die git weigert of een branch die
een andere worktree heeft wordt gemeld op de eigen regel van het dialoogvenster, en **Maken**
blijft grijs.

### Een naam geven met AI

Met [AI](ai.md) aan opent **Naam met AI** (`Alt+A`) een vak in het dialoogvenster: *Beschrijf de
functie, bug of taak*. **Noem het** vraagt het model om een korte worktreenaam en een branchnaam
die de eigen branchnamen van de repository volgt (`feature/`, `fix/`, kleine letters met
streepjes). Ze vullen de Naam (als `worktree-<naam>`) en het branchveld, nog te bewerken; een
branchnaam die al bestaat krijgt het achtervoegsel `-2`, zodat het antwoord nooit stilzwijgend
iets uitcheckt. Het vak toont de bezig-status, **Stop** (of `Esc`) annuleert en een mislukking
wordt in het dialoogvenster getoond. Alleen je beschrijving en de lokale branchnamen worden
verstuurd.

## Het menu van de rij

Klik met rechts op een worktree (of selecteer hem en druk op de menutoets, `Shift+F10` of
`.`):

- **Openen in een nieuw tabblad**: zoals een dubbelklik of `Enter`.
- **Openen in**: de items van [Openen in](open-in.md) waarvan het vak **Kan een worktree
  openen** is aangevinkt (Instellingen > Openen in; elk item is standaard aangevinkt). Ze
  draaien met de map van de worktree als `${cwd}`.
- **Merge <branch> in...**: kies de branch waarin je mergt (standaard de branch van de
  hoofdworktree). De gewone merge draait in de worktree waar die branch is uitgecheckt, met
  jouw merge-opties; een conflict wordt daar opgelost met de gebruikelijke conflictflow. Een
  branch die nergens is uitgecheckt wordt geweigerd, met de reden. Niet aangeboden voor een
  losgekoppelde worktree.
- **Worktree verwijderen...**: nooit de hoofdworktree.

![Het rechtsklikmenu van een worktreerij: Openen in een nieuw tabblad, Openen in, Merge hotfix/axis-labels in... en Worktree verwijderen...](../images/worktree-menu.png)

**Openen in een nieuw tabblad** (of een dubbelklik, of `Enter`) opent de map van de worktree als
een eigen tabblad, met zijn eigen Log, status en branch:

![De worktree hotfix/axis-labels geopend als eigen tabblad naast het hoofdtabblad, met zijn commit en gewijzigde bestand](../images/worktree-tab.png)

Het palet heeft dezelfde acties op de geselecteerde rij (**Refs: Worktree openen in...**,
**Refs: Worktree-branch mergen in...**, **Refs: Worktree verwijderen...**, **Refs: Worktree
openen in tabblad**).

## Een worktree verwijderen

Een schone worktree krijgt een gewone bevestiging. Als zijn branch al is gemerged (in zijn
upstream, de branch van de hoofdworktree of `main` / `master`) is het vak **Ook branch <b>
verwijderen** standaard aangevinkt; bij een niet-gemergede branch staat dat hij blijft
bestaan. **Merge in... en dan verwijderen** mergt eerst en verwijdert pas als de merge zonder
conflicten is geland.

Bij niet-gecommitte wijzigingen (gevolgd of niet gevolgd) toont het dialoogvenster ze (aantal,
scrollbaar) en biedt het aan:

- **Committen en verwijderen**: een berichtveld in het dialoogvenster; elke wijziging wordt
  gestaged (`.gitignore` wordt gerespecteerd) en gecommit op de branch van de worktree, daarna
  wordt de worktree verwijderd. Een losgekoppelde worktree geeft in hetzelfde dialoogvenster
  een branch voor de commit een naam.
- **Stash en verwijderen**: elke wijziging, ook niet-gevolgde bestanden, gaat in een stash met
  een bewerkbaar bericht (standaard "WIP op <map>"), daarna wordt de worktree verwijderd. De
  stashlijst wordt door alle worktrees gedeeld: de stash staat daarna in de STASHES van de
  hoofdrepository, klaar om te poppen.
- **Openen om te committen**: opent de worktree in een tabblad met het commitbericht
  gefocust; er wordt niets verwijderd.
- **Toch verwijderen**: gooit de wijzigingen weg na een tweede, expliciete bevestiging.

## Ongedaan maken

Een worktree verwijderen kan niet ongedaan worden gemaakt: de map is weg. Wat eerst is
bewaard, blijft: de commit staat op zijn branch, de stash in de lijst. Het verwijderen van de
branch dat **Ook branch verwijderen** toevoegt, is het gewone gejournaalde: **Ongedaan maken**
brengt de branch terug (niet de map). Een worktree maken maakt ook geen deel uit van de
geschiedenis.

## Vanuit een script of een agent

`gitpoulpi worktree changes <path>` toont de niet-gecommitte wijzigingen van een worktree.
`gitpoulpi worktree remove <path> [--force] [--commit-message M [--commit-branch B]]
[--stash-message M] [--delete-branch]` verwijdert hem; de MCP-tools zijn `worktree_changes`
en `worktree_remove`. `gitpoulpi ai worktree-name --describe "<taak>"` drukt de worktree- en
branchnamen af die de AI voorstelt (`{ name, worktree, branch }`) en maakt niets; de MCP-tool is
`ai_worktree_name`. Zie de [CLI-referentie](cli-reference.md).
