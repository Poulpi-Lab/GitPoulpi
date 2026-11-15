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
  föreslår en ny gren på den (`worktree-<id>`).
- **Från vald commit**: nedtonad tills en commit är vald i Log. Worktreen
  föreslår en ny gren `worktree-<id>` som startar där; töm grenfältet för en
  frånkopplad worktree.
- **Välj en gren...**: dialogrutan öppnas på sin lista **Worktree-gren**, för
  att öppna en gren som redan finns.

Paletten har samma tre (**Refs: Skapa worktree från aktuell gren**,
**Git: Skapa worktree vid vald commit...**, **Refs: Skapa worktree och välj en
gren**). De äldre posterna fungerar fortfarande: **Skapa worktree från** i en
gren-, tag- eller commit-meny.

Dialogrutan föreslår allt och låter dig ändra det. **Namn** och den nya grenen
börjar båda som `worktree-<id>`, där `<id>` är 8 slumpmässiga hexadecimala
siffror (samma för båda), så att två worktrees aldrig krockar.

| Fält | Betydelse |
|---|---|
| **Namn** | Mappnamnet byggs av det. **Namn efter gren** (`Alt+N`) sätter det till `worktree-<gren>`: snedstreck blir `-` och ordet dubbleras inte (`worktree-foo` och `feature/worktree-x` behåller sina egna, `feature/login` ger `worktree-feature-login`). |
| **Mapp** | En syskonmapp till repositoryt, `<repository>-<namn>`. Den följer Namn tills du skriver i Mapp; **Bläddra...** väljer var. |
| **Ref** | Var en *ny* gren startar: en gren eller en commit (en lista; skriv för att söka). Den används inte, och säger det, medan en befintlig gren är vald. |
| **Worktree-gren** | Ett fält, två betydelser, angivna under det. Ett namn som inte är en lokal gren skapar en ny gren från Ref (**ny gren från main**). En lokal gren som inte är utcheckad någon annanstans (välj den i listan eller skriv dess namn) checkas ut där som den är (**checkar ut den här grenen**). Listans första rad, eller ett tomt fält, checkar ut Refs gren (eller frånkopplar på en commit); det nekas när den grenen redan är utcheckad. |
| **Slumpa om** | `Alt+R`. Drar ett nytt `<id>` och ersätter Namn och grenen med det, utom det du skrev själv (ett Namn eller en gren du skrev står kvar) och en befintlig gren (ersätts aldrig). Ett Namn från **Namn efter gren** eller från AI:n räknas som genererat. |
| **Namnge med AI** | Bara när AI är på (se nedan). |
| **Öppna den nya worktree som flik** | På som standard. |

En mapp som inte är tom, ett ogiltigt namn, ett grennamn som git nekar eller
en gren som en annan worktree har rapporteras på dialogrutans egen rad, och
**Skapa** förblir nedtonad.

### Namnge med AI

Med [AI](ai.md) påslagen öppnar **Namnge med AI** (`Alt+A`) en ruta i
dialogrutan: *Beskriv funktionen, felet eller uppgiften*. **Namnge** ber
modellen om ett kort worktree-namn och ett grennamn som följer repositoryts
egna grennamn (`feature/`, `fix/`, gemener med bindestreck). De fyller Namn
(som `worktree-<namn>`) och grenfältet, fortfarande redigerbara; ett grennamn
som redan finns får suffixet `-2` så att svaret aldrig tyst checkar ut något.
Rutan visar upptagetläget, **Stoppa** (eller `Esc`) avbryter, och ett fel visas
i dialogrutan. Bara din beskrivning och de lokala grennamnen skickas.

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
MCP-verktygen är `worktree_changes` och `worktree_remove`. `gitpoulpi ai
worktree-name --describe "<uppgift>"` skriver ut worktree- och grennamnen som
AI:n föreslår (`{ name, worktree, branch }`) och skapar ingenting; MCP-verktyget
är `ai_worktree_name`. Se [CLI-referens](cli-reference.md).
