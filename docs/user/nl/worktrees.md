[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · **Nederlands** · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · [Svenska](../sv/worktrees.md) · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

Een worktree is een tweede uitgecheckte kopie van de repository: een andere map, op een
andere branch, met dezelfde geschiedenis. De sectie WORKTREES van het Refs-paneel toont ze;
de hoofdworktree is gemarkeerd met `(hoofd)`.

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
  stelt een nieuwe branch erop voor (`<branch>-wt`).
- **Van de geselecteerde commit**: grijs tot er een commit is geselecteerd in het Log. De
  worktree begint daar losgekoppeld, tenzij je een nieuwe branch een naam geeft.
- **Een branch kiezen...**: het dialoogvenster opent op zijn lijst **Ref**.

Het palet heeft dezelfde drie (**Refs: Worktree maken van huidige branch**, **Git: Worktree
maken op geselecteerde commit...**, **Refs: Worktree maken met een gekozen branch**). De
oudere items werken nog: **Worktree maken van...** in het menu van een branch, tag of commit.

Het dialoogvenster stelt alles voor en laat je het wijzigen:

| Veld | Betekenis |
|---|---|
| **Naam** | De mapnaam wordt eruit opgebouwd. Voorgesteld op basis van de branch of de korte id. |
| **Map** | Een naaste van de repository, `<repository>-<naam>`. Hij volgt de Naam tot je in de Map typt; **Bladeren** kiest waar. |
| **Ref** | De branch of commit waar de worktree mee begint (een lijst; typ om te zoeken). Door hem te wijzigen wordt een nieuwe Naam voorgesteld. |
| **Nieuwe branch** | Leeg checkt de branch van de Ref daar uit (of koppelt los op een commit). Vereist als de branch al is uitgecheckt. |
| **De nieuwe worktree als tabblad openen** | Standaard aan. |

Een map die niet leeg is, een ongeldige naam of een branch die al bestaat wordt gemeld op de
eigen regel van het dialoogvenster, en **Maken** blijft grijs.

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
en `worktree_remove`. Zie de [CLI-referentie](cli-reference.md).
