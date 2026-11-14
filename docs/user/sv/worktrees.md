[English](../worktrees.md) · [Čeština](../cs/worktrees.md) · [Deutsch](../de/worktrees.md) · [Español](../es/worktrees.md) · [Français](../fr/worktrees.md) · [Italiano](../it/worktrees.md) · [日本語](../ja/worktrees.md) · [한국어](../ko/worktrees.md) · [Nederlands](../nl/worktrees.md) · [Polski](../pl/worktrees.md) · [Português (Brasil)](../pt-BR/worktrees.md) · [Português (Portugal)](../pt-PT/worktrees.md) · [Русский](../ru/worktrees.md) · **Svenska** · [Türkçe](../tr/worktrees.md) · [Українська](../uk/worktrees.md) · [简体中文](../zh-CN/worktrees.md)

# Worktrees

En worktree är en andra utcheckad kopia av repositoryt: en annan mapp, på en
annan gren, med samma historik. Avsnittet WORKTREES i Refs-panelen listar dem;
den huvudsakliga är markerad `(main)`.

## Visa avsnittet

Avsnittet är dolt i ett repository som ännu inte har någon länkad worktree. För
att visa det ändå (det står då ”Ingen worktree än: + skapar en”), använd
palettkommandot **Sidopanel: Visa avsnittet Worktrees**, kryssa i **WORKTREES**
i sidopanelens lista över avsnitt (högerklicka på en avsnittsrubrik) eller kör
något av skapa-kommandona nedan. GitPoulpi minns valet för det repositoryt (se
**Avsnitt i sidopanelen** i [Inställningar](settings.md)); ett annat repository
utan worktrees börjar fortfarande med avsnittet dolt.

## Skapa en worktree

**+** på rubriken WORKTREES (tangentbord: `+` i avsnittet; palett: **Refs:
Skapa worktree...**) öppnar en meny:

- **Från aktuell gren**: grenen är utcheckad i huvudmappen, så dialogrutan
  föreslår en ny gren på den (`<gren>-wt`).
- **Från vald commit**: nedtonad tills en commit är vald i Log. Worktreen
  startar frånkopplad där om du inte namnger en ny gren.
- **Välj en gren...**: dialogrutan öppnas på sin lista **Ref**.

Paletten har samma tre (**Refs: Skapa worktree från aktuell gren**,
**Git: Skapa worktree vid vald commit...**, **Refs: Skapa worktree och välj en
gren**). De äldre posterna fungerar fortfarande: **Skapa worktree från** i en
gren-, tag- eller commit-meny.

Dialogrutan föreslår allt och låter dig ändra det:

| Fält | Betydelse |
|---|---|
| **Namn** | Mappnamnet byggs av det. Föreslås från grenen eller det korta id:t. |
| **Mapp** | En syskonmapp till repositoryt, `<repository>-<namn>`. Den följer Namn tills du skriver i Mapp; **Bläddra...** väljer var. |
| **Ref** | Grenen eller commiten som worktreen startar från (en lista; skriv för att söka). Att ändra den föreslår ett nytt Namn. |
| **Ny gren** | Tomt checkar ut Refs gren där (eller frånkopplar på en commit). Krävs när grenen redan är utcheckad. |
| **Öppna den nya worktree som flik** | På som standard. |

En mapp som inte är tom, ett ogiltigt namn eller en gren som redan finns
rapporteras på dialogrutans egen rad, och **Skapa** förblir nedtonad.

## Radens meny

Högerklicka på en worktree (eller markera den och tryck på menytangenten,
`Skift+F10` eller `.`):

- **Öppna i ny flik**: som ett dubbelklick eller `Enter`.
- **Öppna i**: posterna i [Öppna i](open-in.md) vars ruta **Kan öppna en
  worktree** är ikryssad (Inställningar > Öppna i; varje post är ikryssad som
  standard). De körs med worktreens mapp som `${cwd}`.
- **Merge <gren> i...**: välj grenen att merga in i (som standard den
  huvudsakliga worktreens gren). Den vanliga merge körs i den worktree där den
  grenen är utcheckad, med dina merge-val; en konflikt löses där med det
  vanliga konfliktflödet. En gren som inte är utcheckad någonstans nekas med
  orsaken. Erbjuds inte för en frånkopplad worktree.
- **Ta bort worktree...**: aldrig den huvudsakliga.

Paletten har samma åtgärder på den markerade raden (**Refs: Öppna worktree
i...**, **Refs: Merge worktreens gren i...**, **Refs: Ta bort worktree...**,
**Refs: Öppna worktree i flik**).

## Ta bort en worktree

En ren worktree får en enkel bekräftelse. Om dess gren redan är mergad (in i
sin upstream, den huvudsakliga worktreens gren eller `main` / `master`) är
rutan **Radera också grenen <b> (efter merge i ...)** ikryssad som standard; en
ej mergad gren anger att den finns kvar. **Merge i... och ta bort** mergar
först och tar bara bort worktreen när mergen gick igenom utan konflikter.

Med ocommittade ändringar (spårade eller ospårade) listar dialogrutan dem
(antal, rullningsbar) och erbjuder:

- **Commit och ta bort**: ett meddelandefält i dialogrutan; varje ändring
  stagas (`.gitignore` respekteras) och commitas på worktreens gren, sedan tas
  worktreen bort. En frånkopplad worktree namnger en gren för commiten i
  samma dialogruta.
- **Stash och ta bort**: varje ändring, ospårade filer inkluderade, stashas
  med ett redigerbart meddelande (standard ”WIP på <mapp>”), sedan tas
  worktreen bort. Stash-listan delas av alla worktrees: stashen ligger efteråt
  i huvudrepositoryts STASHES, redo att poppas.
- **Öppna för commit**: öppnar worktreen i en flik med commit-meddelandet
  fokuserat; inget tas bort.
- **Ta bort ändå**: förkastar ändringarna efter en andra, uttrycklig
  bekräftelse.

## Ångra

Att ta bort en worktree går inte att ångra: mappen är borta. Det som sparades
först finns kvar: commiten ligger på sin gren, stashen i listan. Grenraderingen
som **Radera också grenen** lägger till är den vanliga journalförda: **Ångra**
ger tillbaka grenen (inte mappen). Att skapa en worktree ingår inte heller i
historiken.

## Från ett skript eller en agent

`gitpoulpi worktree changes <path>` listar de ocommittade ändringarna i en
worktree. `gitpoulpi worktree remove <path> [--force] [--commit-message M
[--commit-branch B]] [--stash-message M] [--delete-branch]` tar bort den;
MCP-verktygen är `worktree_changes` och `worktree_remove`. Se
[CLI-referens](cli-reference.md).
