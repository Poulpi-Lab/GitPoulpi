En **sekvens** är en namngiven lista med kommandon som körs efter varandra:
till exempel *Stage alla ändringar*, *AI: Generera commit-meddelande*, *Commit*,
*Push* som en enda **Backup**-tangent. Varje steg är ett kommando i
[kommandopaletten](keybindings.md); en sekvens är en egen palettrad
(`Sekvens: <namn>`), kan bindas till en tangent och redigeras under
**Inställningar > Sekvenser** eller i `config.toml`.

## Sidan Sekvenser

**Inställningar > Sekvenser** är en [listsida](settings.md#listor): sekvenserna
till vänster, den valda sekvensens rader till höger. Varje ändring tillämpas och
skrivs direkt, och paletten och tangentkartan följer med utan omstart.

```
 Sekvenser
+- Sekvenser -------------+  Redigerar sekvens 1/2
| Denna installation      |  Namn         Backup
| > Backup        Alt+B   |  Id           backup
|   Worktree och agent    |  Beskrivning  Stage allt, ...
| Detta repository        |  Omfång       Denna installation
|   Release               |  Genväg       [ Alt+B ] [ Rensa ]
+-------------------------+  Steg         Stage alla > AI: ... > Commit > Push
| [+ Lägg till v] [Dup]   |              [ Redigera steg (4) ] [ Kör nu ]
| [Ta bort] [Upp] [Ned]   |
+-------------------------+
```

* **+ Lägg till** erbjuder **Tom sekvens** och de tre förinställningarna,
  **Från förinställning > Säkerhetskopia**, **Komponera och push** och
  **Worktree och agent**. En förinställning är en startpunkt: inget är aktivt
  förrän du lägger till den, och det du lägger till är en kopia du kan
  redigera. Förinställningarna får namn på gränssnittsspråket när de läggs
  till.
* **Duplicera**, **Ta bort** (frågar först), **Upp** / **Ned** eller en
  dragning ordnar om listan (`Insert`, `Ctrl+D`, `Delete`, `Alt+Upp`,
  `Alt+Ned`). Listans ordning är ordningen i `config.toml`.
* En rad med ett problem markeras i rött och säger varför (inga steg, ett steg
  som den här versionen inte känner till, en paus `Aldrig` som saknar det
  alternativ den behöver). En sekvens med problem **kan sparas**, okända
  framtida steg behålls som de är, men dess palettrad förblir nedtonad tills
  problemet är åtgärdat.

### Raderna i en sekvens

| Rad | Vad den gör |
|---|---|
| **Namn** | Det paletten visar: `Sekvens: <namn>`. |
| **Id** | Det korta namnet som används i `config.toml`, på kommandoraden och som genvägens nyckel (`a-z`, `0-9`, `_`, `-`). Om du byter namn flyttas genvägen med, i samma skrivning. |
| **Beskrivning** | Valfri, för dig själv. |
| **Omfång** | Se nedan. |
| **Genväg** | En knapp som visar tangenten. `Enter` eller ett klick öppnar samma tangentinspelning som **Tangentbord & mus**: konflikter och terminalvarningar visas medan du trycker, och en tangent som en annan åtgärd använder avvisas på samma sätt. **Rensa** tar bort den. Tangenten sparas som `[keybindings.sequence] <id>`. |
| **Steg** | Stegen i ordning. **Redigera steg** öppnar dem. |
| **Kör nu** | Kör sekvensen i aktuellt fönster. |

### Omfång

Varje sekvens har ett omfång som sparas med den:

| Omfång | Var den sparas | Vem som ser den |
|---|---|---|
| **Denna installation** (standard) | `[[sequences]]` i `config.toml`, med den här installationen som ägare | Bara den här installationen av GitPoulpi. |
| **Alla installationer på den här datorn** | `[[sequences]]` i `config.toml`, utan ägare | Varje installation som delar filen. |
| **Bara detta repository** | `.gitpoulpi/sequences.toml` i aktuellt repository | Alla som har repositoryt. Den räknas först när du litar på den (se nedan). |

Om du ändrar omfånget flyttas definitionen: att välja **Bara detta repository**
skriver den till repositoryts fil och tar bort den ur `config.toml`, och
tvärtom. Ett repositoryomfång kräver att ett repository är öppet.

### Repositorysekvenser och förtroende

Filen `.gitpoulpi/sequences.toml` kan checkas in, så den kan komma från andra
personer, och en sekvens kör kommandon på din dator. Ett repositorys sekvenser
**erbjuds därför inte förrän du litar på dem**, på samma sätt som kommandona i
en incheckad arbetsflödesfil.

När ett repository har en sådan fil som ingen litat på än visas ett meddelande
över Log: *Detta repository definierar 2 sekvenser* med **Granska**, **Lita på**
och **Ignorera**. Det blockerar aldrig något.

- **Granska** listar varje sekvens med varje steg, dess paus och dess
  alternativ, skrivskyddat. **Lita på** där litar på det som listades;
  **Stäng** (eller `Esc`) litar inte på något. Om listan är längre än fönstret
  erbjuds inte Lita på där (förstora fönstret): du litar aldrig på det du inte
  kunde läsa.
- **Lita på** (knappen, eller paletten *Sekvens: Lita på detta repositorys
  sekvenser*) litar på filen exakt som den är nu.
- **Ignorera** döljer meddelandet tills filen ändras.
- Från tangentbordet: paletten har *Sekvens: Granska / Lita på / Ignorera detta
  repositorys sekvenser*.

Det som registreras är en kontrollsumma av id:n, kommandona, pauserna och
alternativen, lagrad i repositoryts lokala tillstånd (aldrig i repositoryt). Om
något av dem ändras, till exempel efter en pull, kommer meddelandet tillbaka som
*Detta repositorys sekvenser har ändrats* och sekvenserna lämnar paletten och
tangenterna tills du litar på dem igen. Att byta namn på en sekvens eller
redigera dess beskrivning ändrar ingenting. En sekvens du skriver själv i
Inställningar förblir betrodd när filen redan var betrodd (eller tom) innan du
redigerade den.

När de är betrodda visas repositorysekvenser som `Sekvens: <namn>` i paletten
som dina egna och kan få en genväg i Inställningar. En sekvens med samma id som
en av dina är en krock: din vinner och den andra rapporteras som dubblett.

Från kommandoraden: `gitpoulpi sequence list` visar filens sekvenser med
`trusted` och ett `repo_trust`-tillstånd (`untrusted`, `changed`, `trusted`);
`gitpoulpi sequence trust` (och `untrust`, `--repo PATH` för ett annat
repository) är till för en person att köra efter att ha läst `sequence show`.
`sequence run` avvisar en repositorysekvens som man inte litar på. AI-agenter
kan inte lita på något: det finns inget MCP-verktyg för det.

## Steg

**Redigera steg** ersätter listan med sekvensens steg (rubriken säger **Steg i
<namn>**) och raderna med det valda stegets. **Tillbaka** (korset överst bland
raderna) eller `Esc` återvänder till sekvenserna.

* **+ Lägg till** öppnar **Lägg till ett steg**: varje kommando en sekvens får
  köra, under sitt palettnamn (AI-kommandona med gnistan), filtrerat medan du
  skriver med palettens matchning. Ett steg som den här versionen inte kan köra
  är nedtonat med skälet. `Enter` eller ett klick lägger till det sist; `Esc`
  eller korset stänger popupen.
* **Duplicera**, **Ta bort**, **Upp** / **Ned** och dragning fungerar som för
  sekvenserna.
* Ett destruktivt kommando (ta bort en gren, förkasta ändringar...) erbjuds
  aldrig: en sekvens kan inte köra ett.

### Paus

Varje steg har en **Paus**:

| Paus | Betydelse |
|---|---|
| **Standard** | Steget stannar bara när det behöver ett svar, i sin vanliga dialog (ett grennamn, ett commit-meddelande som fortfarande är tomt...). |
| **Alltid** | Stanna före detta steg och fråga om det ska fortsätta. |
| **Aldrig** | Stanna aldrig. Stegets alternativ måste ge det allt det behöver, så ett steg som frågar efter ett namn behöver sitt alternativ **Namn** ifyllt. |

Ett steg som alltid frågar innan det körs (en force push) erbjuder inte
**Aldrig**: raden säger varför. **Aldrig** på AI-meddelandesteget är tillåtet
som ett uttryckligt val.

### Alternativ

Raderna under **Paus** är stegets alternativ: en text **Namn** eller
**Meddelande**, en **Gren**, en kryssruta som **Öppna i en ny flik**, eller en
rullgardinslista fylld från din konfiguration (en **Remote**, en **AI-profil**,
en **Öppna i**-post). Texterna godtar variablerna `${branch}`, `${date}`,
`${time}` och `${n}` (ett nummer som gör namnet unikt). Om du tömmer en text
tas alternativet bort.

## Skapa och redigera med AI

Med [AI påslaget](ai.md#sekvensutkast) (Inställningar > AI) visas två poster:
**Lägg till > Skapa med AI...** (även palettraden `Sekvens: Skapa med AI...`,
som öppnar Inställningar på den här sidan med dialogen) och **Redigera med
AI...** bredvid **Kör nu** för den valda sekvensen. Ingen av dem finns medan AI
är av.

```
+- Skapa en sekvens med AI --------------------------------- x -+
| Beskriv sekvensen du vill ha                                  |
| +-----------------------------------------------------------+ |
| | stage allt, skriv commit-meddelandet med AI, commit och   | |
| | sedan push                                                | |
| +-----------------------------------------------------------+ |
| Enter lägger till en rad. Ctrl+Enter skickar. Esc stänger.    |
| [ ] Skicka sammanhang (remotenamn, AI-profiler, Öppna i-...)  |
|    Av: bara din beskrivning skickas.                          |
| AI:n föreslår bara. Inget sparas innan du godkänner...        |
| [ Skicka ] [ Avbryt ]                                         |
+---------------------------------------------------------------+
```

Du skriver vad du vill med egna ord; **Skicka** (eller `Ctrl+Enter`, `Ctrl+S`)
frågar modellen, via samma profil, samma jobbindikator och samma `Esc` som varje
annan AI-åtgärd (och samma fråga ”Det som skickas”). Svaret är ett **förslag**
som du läser innan något händer:

* namnet, id:t, beskrivningen och stegen med sin paus och sina alternativ;
* för **Redigera med AI** en diff mot aktuell sekvens: `+` ett steg tillagt,
  `-` ett steg borttaget, `~` ett steg vars paus eller alternativ ändrats,
  `~ Namn: ”a” -> ”b”` för ett fält;
* **Utelämnat**: ett steg som modellen föreslog men som inte är tillåtet
  (okänt, destruktivt, en sekvens i en sekvens, en omöjlig paus), med skälet;
  de giltiga stegen behålls;
* modellens **anmärkningar** och en **föreslagen genväg** som bara visas, med
  uppgift om tangenten redan är upptagen. Den binds aldrig: använd knappen
  **Genväg** efteråt.

```
+- Föreslagen ändring -------------------------------------- x -+
| + tillagt   - borttaget   ~ ändrat                            |
| ~ Namn: ”Daglig” -> ”Daglig 2”                                |
| Steg (3)                                                      |
| + 1. Git: Fetch alla remotes                                  |
|   2. Git: Stage alla ändringar                                |
| - Git: Commit...                                              |
| ~ 3. Git: Push: standard -> Paus: Alltid                      |
| Utelämnat                                                     |
| steg 4 `reset_hard`: destruktiva steg är inte tillåtna        |
| [ Godkänn ] [ Redigera före sparande ] [ Avvisa ] [ Tillbaka ]|
+---------------------------------------------------------------+
```

* **Godkänn** skriver det med en enda sparning (en ny sekvens, eller ersättning
  av den redigerade, som behåller sitt id, sitt omfång och sin tangent) och
  markerar den i listan.
* **Redigera före sparande** gör detsamma och öppnar dess steg så att du kan
  justera raderna (den här sidan sparar varje ändring direkt, så sekvensen är
  redan sparad).
* **Avvisa** skriver ingenting. **Tillbaka** återvänder till beskrivningen för
  att skicka igen.
* **Godkänn** är otillgänglig när det inte finns något att godkänna: inget
  användbart steg, ett problem som sidan skulle flagga, eller en redigering som
  inte ändrar något.

Inget skrivs före **Godkänn**, och en sekvens körs aldrig härifrån. Att stänga
dialogen (korset, `Esc`, ett klick utanför) frågar ”Förkasta ändringar?” medan
en beskrivning är skriven eller ett förslag är öppet; att stänga Inställningar
frågar också.

## config.toml

```toml
[[sequences]]
id = "backup"
name = "Backup"
scope = "installation"
owner = "<den här installationens id>"

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

Nycklar som den här versionen inte känner till, och steg den inte känner till,
behålls när sidan sparar.

## Köra en sekvens

En sekvens är en namngiven lista med steg, vart och ett ett kommando i paletten (stage allt, skriv commit-meddelandet med AI, commit, push...). Kör den från paletten (`Sekvens: <namn>`), med tangenten du gav den i Inställningar > Tangentbord & mus (`[keybindings.sequence]` i `config.toml`) eller från kommandoraden (`gitpoulpi sequence run`).

Medan den körs visar en remsa överst över Log `<namn> k/n`, steget och vad den väntar på. Körningen visas också i panelen Jobb som `Sekvens <namn> k/n: <steg>`. En sekvens körs åt gången; om du startar en andra anges vilken som körs.

**Var den stannar och frågar.** Varje steg har en inställning `pause` (`default`, `always`, `never`):

| Steg | Standard | `pause = "never"` |
|---|---|---|
| AI-commit-meddelande | meddelandet hamnar i meddelandefältet; remsan säger ”Granska meddelandet och välj sedan Fortsätt” | går vidare direkt |
| Commit | ett tomt meddelande stannar alltid: fältet får tangentbordet, och din commit (eller Fortsätt) går vidare | inte tillgängligt när det inte finns något meddelande att committa |
| AI-composer för commits | granskningen öppnas; Tillämpa går vidare, att stänga den avbryter | tillämpar förslaget som det är |
| Skapa worktree | dialogen öppnas med namnet ifyllt | skapar den från `options.name` utan dialogen |
| Push | när `git.push_behavior = ask` öppnas väljaren ”Push till...” | pushar till `options.remote` |
| Force push med lease, stage av filer som ser ut som hemligheter | frågar alltid | inte tillåtet |

`pause = "always"` på ett valfritt steg visar ”Nästa: <steg>” med **Fortsätt** och **Avbryt** innan det körs. Om pushen avvisas eftersom grenen har divergerat öppnas den vanliga dialogen och remsan säger ”Push avvisades: välj i dialogen”; att göra pull och sedan push (eller en force push med lease) fortsätter körningen, att stänga dialogen avbryter den.

**Fortsätt** är remsans knapp, `Sekvens: Fortsätt` i paletten eller tangenten bunden till `sequence.continue`. **Avbryt** är remsans stängkors, `Sekvens: Avbryt` (`sequence.cancel`), panelen Jobb eller `Esc` när inget annat vill ha den (en öppen dialog stängs först, vilket också avbryter). Att avbryta behåller de steg som redan körts och säger det: ”<namn> avbröts efter steg k/n (<steg>); tidigare steg finns kvar”.

**När ett steg misslyckas** stannar körningen där: statusraden anger sekvensen, steget och felet, remsan blir rött med **Detaljer** (posten i Aktivitetsloggen, som listar stegets alternativ) och ett stängkors, och de tidigare stegen förblir utförda.

**Worktrees.** Ett steg som skapar en worktree med `open_tab = true` flyttar körningen till den nya worktreens flik: de följande stegen (`tab_dir_terminal`, `tab_open_agent_here`, `tab_open_in`) verkar där. Om en annan flik visas säger remsan ”väntar på fliken <namn>” med **Gå dit**. Om den fliken stängs avbryts körningen. En körning överlever inte att GitPoulpi avslutas.

### En agent som föreslår en körning

Med *Låt AI-agenter föreslå ändringar som jag bekräftar* påslaget (`[mcp]
allow_ui_propose`) kan en agent be ditt fönster köra en sekvens
(`gitpoulpi ui sequence run <id>`, MCP `ui_sequence_run`). En banner visas:
*✦ Föreslaget av <agent>: kör sekvensen <namn>* med **Kör** och **Avvisa**.
Inget körs förrän du trycker på Kör (klicka på den, eller `Ctrl+N` och sedan
`Enter`); Avvisa, `Esc` (när inget annat vill ha den) eller paletten *Agent:
Avfärda väntande vy* tar bort den. Körningen går sedan precis som om du hade
startat den och pausar där sekvensen säger. Förslaget avvisas när sekvensen
inte finns, är ogiltig eller är en repositorysekvens du inte litat på.

## Ångra en körning

Varje steg som ändrar repositoryt (stage, commit, tag, gren, stash, worktree...) lämnar en **egen vanlig Ångra-post**, märkt med körningen, så `Ctrl+Z` ångrar fortfarande ett steg i taget och [Ångrahistoriken](undo-redo.md#ångrahistoriken) visar `⟳ <namn>` på dem. För att ta tillbaka hela körningen finns **Ångra hela sekvensen**: Ångrahistorikens knapp (`x`), palettraden `Redigera: Ångra hela sekvensen` eller `gitpoulpi undo --sequence`. Den ångrar körningens poster nyaste först, var och en med sina egna kontroller och frågor (en commit som redan pushats frågar först), stannar vid första avslaget med angivet steg och lämnar de äldre i journalen. AI-meddelandet och en push lämnar ingen post: en push ångras aldrig. Bara aktuellt repositorys journal berörs; steg som gjorts i en worktree ångras från det repositoryt. Gör om förblir per post. Detaljer: [Sekvenskörningar](undo-redo.md#sekvenskörningar).
