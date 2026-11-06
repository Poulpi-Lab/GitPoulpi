[English](../ai.md) · **Čeština** · [Deutsch](../de/ai.md) · [Español](../es/ai.md) · [Français](../fr/ai.md) · [Italiano](../it/ai.md) · [日本語](../ja/ai.md) · [한국어](../ko/ai.md) · [Nederlands](../nl/ai.md) · [Polski](../pl/ai.md) · [Português (Brasil)](../pt-BR/ai.md) · [Português (Portugal)](../pt-PT/ai.md) · [Русский](../ru/ai.md) · [Svenska](../sv/ai.md) · [Türkçe](../tr/ai.md) · [Українська](../uk/ai.md) · [简体中文](../zh-CN/ai.md)

# Asistence AI

GitPoulpi nikdy nevyžaduje účet a **AI je ve výchozím stavu vypnutá**. Když je
AI zapnutá, umí psát zprávy commitů a stashů, vysvětlovat změny, psát popisy
pull requestů, navrhovat řešení konfliktů merge a rozdělit vaše pracovní změny
do několika commitů. Model je váš: klíč API, který vlastníte, místní server nebo
již nainstalované CLI agenta (například `claude -p`).

Každý ovládací prvek AI ukazuje fialové `✦`. Dokud je AI vypnutá:

- Nic se nikam neodesílá.
- Nezobrazuje se žádný ovládací prvek AI: žádné tlačítko `✦`, žádný příkaz
  `AI:` v paletě, žádný řádek nabídky AI a žádná nápověda klávesy AI. Klávesy
  AI nedělají nic.
- Stránka AI repozitáře ukazuje jen „AI je vypnuta (Nastavení > AI)“.
- Server MCP nevypisuje nástroje `ai_*`. Příkazy CLI `gitpoulpi ai ...` odmítnou
  provedení se stejnou zprávou.

**Zapnout AI** se projeví okamžitě. Restart není potřeba.

AI s GitPoulpi lze používat dvěma způsoby:

| | Vestavěná AI (tato stránka) | Externí agent |
|---|---|---|
| Co | GitPoulpi žádá model o zprávu commitu, vysvětlení, ... | Claude Code, Cursor, Codex... ovládají GitPoulpi jako nástroj |
| Nastavení v | Nastavení > AI (`[ai]`) | konfigurace MCP agenta, limity v `[mcp]` |
| Průvodce | níže | [MCP a agenti](mcp-and-agents.md) |

## Konfigurace AI

1. `Ctrl+,` > **AI** > zaškrtněte **Zapnout AI**.
2. Profily jsou seznam na kartě **Profily (připojení)**, vlevo na stránce (na úzkém terminálu nad ní). Profil
   s názvem *Default* existuje vždy. Vyberte profil a upravujete ho vpravo
   (`Enter` přejde na jeho řádky; příkaz z palety **AI: Upravit profil...**
   skočí na seznam). **+ Přidat** vytvoří další, **Duplikovat** zkopíruje
   vybraný (bez klíče) a **Odebrat** se nejdřív zeptá. Zvolte
   **Poskytovatele** a vyplňte jeho pole:

   | Poskytovatel | Pole | Výchozí hodnoty |
   |---|---|---|
   | **Lokální agent** (bez klíče, bez vlastního účtu) | **Typ agenta** (o jakého agenta jde), **Příkaz** (přesný program: na vašem `PATH`, alias nebo wrapper skript, nebo úplná cesta), **Argumenty**, díky nimž běží bez interakce, a volitelné **proměnné prostředí**. Prompt jde na jeho stdin, odpověď se čte ze stdout. | Claude Code: `claude`, `-p --model haiku` |
   | **Anthropic** | **Klíč API**, volitelný **Model** a **Základní URL** | `claude-sonnet-5`, `https://api.anthropic.com/v1` |
   | **OpenAI** | **Klíč API**, Model, Základní URL | `gpt-5`, `https://api.openai.com/v1` |
   | **OpenRouter** | **Klíč API**, Model, Základní URL | `openai/gpt-5`, `https://openrouter.ai/api/v1` |
   | **Kompatibilní s OpenAI** (Ollama, LM Studio, vLLM, firemní brána) | **Základní URL** (`.../v1`), **Model**, volitelný klíč | `http://localhost:11434/v1` (Ollama; LM Studio je `http://localhost:1234/v1`), `llama3` |

   **Předvolby** (vždy můžete zadat jinou hodnotu):

   ![Rozbalovací seznam Typ agenta u profilu lokálního agenta s položkami Claude Code, OpenAI Codex CLI, Gemini CLI, opencode a dalšími CLI agentů](../images/settings-ai-presets.png)

   - **Lokální agent > Typ agenta** je rozbalovací seznam běžných CLI agentů,
     nejpoužívanější první: Claude Code, OpenAI Codex CLI, Gemini CLI,
     opencode, Cursor Agent, GitHub Copilot CLI, Pi, Qwen Code, Aider, Goose,
     Crush, Amp, nebo *Jiný (bez návrhů)*. Typ je **oddělený od příkazu**: jen
     určuje, jaké návrhy dostanete. Výběr typu předvyplní **Příkaz** (dokud je
     ještě výchozí hodnotou předchozího typu) a **Argumenty** bezobslužným
     voláním (`claude -p`, `codex exec -`, `gemini -p ...`, ...). **Příkaz** je
     volný text: napište `/opt/bin/work-claude`, wrapper skript nebo cíl aliasu
     a typ s jeho možnostmi, seznamem modelů a proměnnými zůstane. Je-li **Typ
     agenta** prázdný, GitPoulpi ho odvodí z příkazu.
   - **Možnosti argumentů** přidávají nebo odebírají volby vhodné pro
     jednorázové použití v GitPoulpi, každou s jednořádkovým důvodem: žádné
     nástroje, žádná uložená relace, výstup jako prostý text, nízká úroveň
     uvažování, režim jen pro čtení, žádné dotazy na oprávnění, žádná kontrola
     aktualizací ani analytika. Seznam se liší podle agenta, protože agenti se
     liší (jen někteří umějí vypnout nástroje z příkazové řádky; u opencode je
     to proměnná prostředí). **Model** vypíše modely daného agenta a zapíše
     správný přepínač (`--model haiku` pro Claude Code, `-m gpt-6-luna` pro
     Codex, `--model provider/model` pro opencode); *(výchozí - bez
     přepínače)* ho odstraní. Amp přepínač modelu nemá. `aider` čte prompt z
     `/dev/stdin`, takže funguje jen v Linuxu a macOS.
   - **Proměnné prostředí** se nastavují jen pro proces daného agenta, nikdy pro
     GitPoulpi ani jiné profily. **Přidat proměnnou prostředí** nabízí
     zdokumentované proměnné, které pro tohoto agenta něco znamenají, každou s
     popisem (vypnutí telemetrie a kontroly aktualizací, úroveň úsilí, složka
     konfigurace, klíč API, základní URL nebo místní hostitel, ...); můžete také
     napsat `NAME` nebo `NAME=value`. Každá proměnná pak má vlastní řádek **Env
     NAME**: její hodnota je tlačítko (stisknutím napíšete novou; uloží se po
     opuštění pole) a tlačítko **[x]** proměnnou smaže. Hodnota může být `${env:NAME}`, čímž si půjčí
     jednu z vlastních proměnných GitPoulpi (praktické u klíčů API, aby se
     nezapisovaly do `config.toml`). Hodnoty vypadající jako tajemství (názvy
     končící na `_KEY`, `_TOKEN`, `_SECRET`, ... nebo známé předpony klíčů) se
     zobrazují jako tečky a zadávají se od začátku. Hodnoty se nikdy nelogují.
   - **Poskytovatelé HTTP > Předvolba poskytovatele** vyplní základní URL a model
     pro OpenAI, Anthropic, OpenRouter, Google Gemini, xAI, DeepSeek, Mistral,
     Groq, Together, Fireworks, Cerebras, Perplexity, Moonshot, Qwen
     (DashScope), Zhipu (GLM) a místní servery Ollama, LM Studio, llama.cpp,
     vLLM, Jan a LocalAI. **Model** je rozbalovací seznam modelů daného
     poskytovatele, volný text je povolen.

3. Stiskněte **Otestovat připojení**: *Poslední test* zní `připojeno: model
   odpověděl` nebo `selhalo: <důvod>`.
4. Při více profilech zvolte dole **Výchozí profil**; každá akce může stále
   pojmenovat svůj vlastní.

![Blok Profily na stránce AI pro lokálního agenta: Typ agenta (Claude Code), Příkaz, Argumenty, Model, dva řádky Env, Přidat a Odebrat proměnnou prostředí, Otestovat připojení a Výchozí profil](../images/settings-ai-agent.png)

**Klíče API** se zadávají klávesou `Enter` na řádku **Klíč API** a ukládají se do
systémového credential helperu (Git Credential Manager, libsecret / `pass`,
Keychain) pod `ai:<profile id>`, nikdy do `config.toml`. Bez helperu to GitPoulpi
řekne, místo aby předstíralo, že klíč drží. *Nastavení > SSH a přihlášení >
Zapomenout všechna jména a hesla* je odstraní.

Totéž v `config.toml` (vše ostatní je v [referenční příručce
konfigurace](config-reference.md#ai-vestavěná-ai-ve-výchozím-stavu-vypnuta)):

```toml
[ai]
enabled = true
default_profile = "claude-code"

[[ai.profiles]]
id = "claude-code"
name = "Claude Code"
kind = "local_agent"      # open_ai | anthropic | open_router | open_ai_compatible | local_agent
command = "claude"        # free: an alias, a wrapper script, an absolute path
harness = "claude"        # which agent it is; empty = inferred from `command`
args = ["-p", "--model", "haiku"]

[ai.profiles.env]         # this agent's process only; ${env:NAME} borrows ours
CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC = "1"
ANTHROPIC_API_KEY = "${env:ANTHROPIC_API_KEY}"
```

## Režim spuštění: bezobslužný nebo virtuální terminál

Profil lokálního agenta spouští svého agenta jedním ze dvou způsobů, podle
řádku **Režim spuštění** (zobrazuje se u Claude Code, OpenAI Codex CLI a Gemini
CLI):

- **Bezobslužný (neinteraktivní)**, výchozí: jednorázový režim agenta
  (`claude -p`), prompt na stdin, odpověď na stdout.
- **Interaktivní (virtuální terminál)**: GitPoulpi spustí vlastní obrazovku
  agenta ve skrytém terminálu, napíše do ní požadavek jako člověk, počká na
  odpověď a agenta zavře. Někteří agenti účtují nebo omezují bezobslužný režim
  jinak než interaktivní; co platí pro vás a co o obou říkají podmínky
  poskytovatele, určuje poskytovatel, takže si je u něj ověřte.

Každý režim má své vlastní **Argumenty**, **Možnosti argumentů** a **Model**:
přepnutím režimů nic neztratíte a interaktivní režim nabízí jen možnosti, které
dávají smysl bez bezobslužného přepínače. Další dva řádky platí pro oba režimy:

- **Pracovní složka**: *Vyhrazená složka* je prázdná složka, kterou vlastní
  GitPoulpi, takže agent nevidí žádný projekt a o důvěru se zeptá jen jednou.
  *Aktuální repozitář* dá agentovi vlastní kontext projektu (například jeho
  `CLAUDE.md`) a o důvěru se zeptá jednou pro každý repozitář. *Výchozí* je
  vyhrazená složka v interaktivním režimu a v bezobslužném režimu místo, kde
  běží GitPoulpi, jako dřív.
- **Časový limit (sekundy)**: jak dlouho může agent trvat, než to GitPoulpi
  vzdá a zastaví ho. Prázdné znamená 120 sekund bezobslužně a 180 interaktivně.

**Na otázky při prvním spuštění odpovídáte vy.** Agent, který ve složce ještě
nikdy neběžel, se nejdřív na něco zeptá (důvěřovat této složce, přihlásit se,
vybrat motiv). GitPoulpi za vás nikdy neodpovídá: požadavek se zastaví se
zprávou a **Otevřít agenta...** (u profilu, nebo *AI: Otevřít agenta* v paletě
příkazů) jednorázově otevře agenta ve viditelném terminálu v pracovní složce,
kterou použije. Odpovězte na otázku, zavřete agenta a zkuste to znovu; odpověď
si agent pamatuje.

Odpověď se u Claude Code čte z přepisu relace agenta (přesný text, poté se
smaže) a u Codexu a Gemini z obrazovky mezi dvěma řádky se značkami, o které
požadavek žádá. Najednou běží nejvýše dvě spuštění ve virtuálním terminálu a
stisknutí Esc během požadavku agenta zastaví.

```toml
[[ai.profiles]]
id = "claude-terminal"
kind = "local_agent"
command = "claude"
harness = "claude"
args = ["-p"]                       # the headless set, kept
run_mode = "terminal"               # headless (default) | terminal
terminal_args = ["--effort", "low"] # the interactive set
workdir = "dedicated"               # default | dedicated | repository
timeout_secs = 180                  # 0 or absent: 120 headless, 180 terminal
```

## Instrukce

Text, který přidáte k tomu, co se modelu řekne. Vestavěný prompt každé akce
(který určuje formát odpovědi) je vždy první a nelze ho upravit; pak následují
čtyři úrovně, od nejobecnější po nejkonkrétnější, každá jen když není prázdná:

1. **Globální instrukce** (každá akce). Čipy *Rychlého vložení* v editoru
   vkládají běžné věty: *Odpovídat v jazyce <language>*, *Stručné odpovědi*,
   *Rozkazovací způsob*, *Technický tón*, *Neformální tón*.
2. **Instrukce pracovního prostoru** (každá akce, každý repozitář
   [pracovního prostoru](workspaces.md#instrukce-ai-pracovního-prostoru)): pro všechny akce
   a volitelně pro každou akci zvlášť. Upravují se ve formuláři Upravit pracovního prostoru.
3. **Instrukce repozitáře** (každá akce, jen tento repozitář), s volbou *Spojit s
   globálními*: **Přidat ke globálním** (za ně) nebo **Nahradit globální**
   (místo nich).
4. **Instrukce pro akci** (*Generování zprávy commitu*, *Vysvětlení změn*,
   *Generování zprávy stashe*, *Pull request*, *Composer commitů*, *Řešení
   konfliktů*, *Kontrola kódu*), každý blok s vlastními čipy v editoru (např. *Formát conventional
   commits*, *Limit 50 znaků*) a vlastním rozbalovacím seznamem **Profil**
   (`Výchozí (<name>)` sleduje výchozí profil).

`Enter` na řádku *Vlastní instrukce* otevře víceřádkový editor (`Ctrl+S` uloží,
`Esc` zruší). Nad textem je řádek *Rychlé vložení* s čipy dané sekce: kliknutím
na čip vložíte jeho větu na pozici kurzoru. Z klávesnice `Tab` přejde na řádek,
`Left`/`Right` vybírají čip, `Enter` nebo `Space` ho vloží a vrátí se do textu
(`Tab` nebo `Esc` se vrátí bez vložení). Čip, jehož věta už v instrukcích je,
je ztlumený a další vložení nic neudělá. Text je plnohodnotný editor: kliknutí
umístí kurzor, tažení nebo `Shift`+šipky vybírají, `Ctrl+A` vybere vše,
`Ctrl+C` / `Ctrl+X` zkopírují a vyjmou výběr, `Ctrl`+šipky skáčou po slovech,
`Home` / `End`, `PageUp` / `PageDown` a kolečko myši procházejí dlouhý text.
Samotná stránka ukazuje u každé instrukce jen náhled.

![Stránka AI: globální instrukce, poté bloky Generování zprávy commitu a Vysvětlení změn s rozbalovacím seznamem profilu a vlastními instrukcemi](../images/settings-ai.png)

**Pro repozitář**: *Nastavení > Nastavení repozitáře > AI* (při otevřeném
repozitáři) obsahuje instrukce repozitáře a pro každou akci profil a instrukce,
které přebíjejí globální. Prázdné pole používá globální nastavení (v něm se
zobrazuje zašedle); **Obnovit globální** přepsání odstraní. Ukládá se do
`.gitpoulpi/repo-state.toml` repozitáře a respektuje to i CLI.

![Stránka AI repozitáře: Instrukce repozitáře s rozbalovacím seznamem Spojit s globálními, poté profil a instrukce pro každou akci](../images/settings-repo-ai.png)

### Jak se úrovně spojují

Pořadí odeslání: vestavěný prompt, **globální**, **pracovního prostoru**, **repozitáře**
a nakonec instrukce samotné akce. Režim úrovně rozhoduje, co se stane s úrovněmi nad ní:

- **Přidat** (výchozí): text úrovně se přidá za obecnější úrovně.
- **Nahradit**: text úrovně se použije *místo* všech obecnějších úrovní (globální
  a pracovního prostoru, když nahrazuje repozitář). Úroveň bez textu nikdy nic nenahrazuje.

Instrukce samotné akce se řídí stejnou myšlenkou: globální pro tu akci, potom instrukce
pracovního prostoru pro tu akci (přidané za ně, nebo je nahrazující) a vlastní přepsání
akce v repozitáři vyhrává nad oběma. Použitý pracovní prostor je aktivní pracovní prostor
okna, pokud obsahuje repozitář, jinak první pracovní prostor, který ho obsahuje; na příkazové
řádce je to ten v `GITPOULPI_WORKSPACE`, jinak první.

Co přesně se odesílá, úroveň po úrovni, ukáže
`gitpoulpi ai prompt --action <action> --repo .` (`base`, `global`, `workspace`,
`repository`, `task` a výsledný `system_prompt`). Nevolá žádný model a funguje i s vypnutou AI.
Instrukce pracovního prostoru nastavíte ze skriptu příkazem
`gitpoulpi workspace set-ai-instructions <id> [--action <a>] [--mode
append|replace] --text ...` (nebo `--clear`).

### Proměnné promptu

Každý text instrukcí (globální, pracovního prostoru, repozitáře, pro akci) smí obsahovat proměnné
`${name}`. Těsně před odesláním promptu se nahradí svou hodnotou (uložený text
si `${...}` ponechá). Název, který není v tabulce, se odešle tak, jak je
napsán, a editor na to upozorní; pro doslovné `${` napište `$${`.

| Proměnná | Rozbalí se na | Příklad |
|---|---|---|
| `${uilang}` | Jazyk rozhraní jeho anglickým názvem | `French` |
| `${repo}` | Název složky repozitáře | `gitpoulpi` |
| `${branch}` | Aktuální větev | `main` |
| `${ref}` / `${commit}` | Krátké sha commitu, kterého se akce týká (*Vysvětlení*), jinak HEAD | `4c0443d` |
| `${commit_subject}` | Předmět tohoto commitu | `feat: add search` |
| `${date}` | Místní datum, ISO | `2026-10-03` |
| `${time}` | Místní čas, `HH:MM` | `14:05` |
| `${datetime}` | Místní datum a čas | `2026-10-03 14:05` |
| `${user}` / `${email}` | `git config user.name` / `user.email` | `Ada Lovelace` |
| `${remote}` | Výchozí remote (`origin`, jinak první) | `origin` |
| `${host}` | `github`, `gitlab`, `other` (jiný hostitel) nebo `none` | `github` |
| `${files}` | Počet souborů v diffu akce | `3` |
| `${action}` | Spouštěná akce | `commit_message` |

V editoru instrukcí napište `${` a vyskočí seznam odpovídajících proměnných
(`Up`/`Down` vybírají, `Enter` nebo `Tab` vloží, `Esc` ho zavře); nebo použijte
rozbalovací seznam **Proměnné** vlevo dole (`Ctrl+T`, nebo *AI: Vložit proměnnou
promptu* v paletě příkazů), který vkládá na pozici kurzoru. Každá položka má
popis, zobrazený v rozbalovacím seznamu a na řádku nad tlačítky, když na ni
najedete nebo ji zvýrazníte.

## Co umí

| Akce | V okně | CLI |
|---|---|---|
| **Zpráva commitu** | Přidejte změny do indexu, stiskněte `✦` vpravo od pole Souhrn, `Ctrl+G i` nebo příkaz z palety `AI: Vygenerovat zprávu commitu`. Souhrn a popis se vyplní, abyste je mohli upravit; nic se necommituje. | `ai commit-message` (vypíše), `commit --ai` (commituje) |
| **Vysvětlení změn** | `[✦]` vpravo nahoře v panelu pracovních změn (`Ctrl+G w`: vybraný commit, jinak pracovní změny), nebo položka nabídky commitu *Vysvětlit změny (AI)*. Vyskakovací okno: šipky posouvají, `c` kopíruje, `Esc` zavře. | `ai explain [rev] [--wip]` |
| **Zpráva stashe** | `Ctrl+G m` otevře výzvu stash; `Ctrl+G` uvnitř ní požádá model. Příkaz z palety `AI: Vygenerovat zprávu stashe` udělá obojí. | `stash push --ai` |
| **Název a popis pull requestu** | *Vygenerovat název a popis* ve formuláři Vytvořit pull request (commity a diff větve se čtou lokálně). | |
| **Řešení konfliktů merge** | V editoru konfliktů merge: tlačítko panelu nástrojů `✦ Navrhnout`, nabídka konfliktu pod pravým tlačítkem *✦ Navrhnout řešení (AI)*, `Alt+I` nebo příkaz z palety `AI: Navrhnout řešení konfliktu merge`. Návrh nahradí oblast *aktuálního konfliktu* ve Výstupu jedinou úpravou (označeno jako upravené, jeden krok zpět); nic se neuloží ani nepřidá do indexu, dokud neuložíte. | `ai resolve-conflict <path> [--conflict N]` (vypíše) |
| **Kontrola kódu** | V [kontrole pull nebo merge requestu](review.md#-koncept-kontroly-od-ai): tlačítko `✦ Koncept kontroly...`, `s` nebo příkaz z palety `Kontrola: ✦ Koncept kontroly...` (otevře možnosti kontroly, `Enter` spustí). Model čte diff, popis a existující komentáře žádosti (a ve výchozím stavu také `CLAUDE.md` a `AGENTS.md` projektu: viz [Kontext kontroly](#kontext-kontroly)) a navrhuje komentáře; každý přijmete, upravíte nebo odmítnete. Nic nedorazí k hostiteli, dokud kontrolu neodešlete. | `hosting review suggest <n>` (vypíše JSON, nic neodešle), `ai review-context` (výchozí hodnoty) |
| **Skript pro přepis historie** | V [nástroji pro přepis historie](history-rewrite.md#napsat-skript-pomocí-ai): tlačítko `✦ Napsat pomocí AI...` v editoru skriptu nebo příkaz z palety `Přepis: Napsat skript pomocí AI...`. Dialog ukáže přesně, co se odešle; skript se objeví v editoru, označený, a nikdy se za vás nespustí ani neprovede. | `ai rewrite-script "<instrukce>" [--sample N]` (vypíše) |
| **Pravidla pro přepis historie** | V [nástroji pro přepis historie](history-rewrite.md#vytvořit-pravidla-pomocí-ai): tlačítko `✦ Vytvořit pravidla pomocí AI...`, `Alt+I` nebo příkaz z palety `Přepis: Vytvořit pravidla pomocí AI...`. Víceřádkový dialog ukáže přesně, co se odešle; pravidla a nastavení se objeví v nástroji a nic se za vás nezobrazí v náhledu ani neprovede. | `ai rewrite-rules "<popis>" [--context]` (vypíše) |
| **Composer commitů** | Nabídka řádku WIP *Sestavit commity (AI)...*, `Ctrl+G C`, příkaz z palety `AI: Sestavit commity z rozpracovaných změn...` | |
| **Rozdělení commitu** | Nabídka commitu *Rozdělit commit (AI)...* nebo příkaz z palety `AI: Rozdělit vybraný commit...`. Otevře se editor rozdělení s vyplněným seskupením od modelu; viz níže. | `ai split-plan <rev>` (vypíše) |

`Esc` zruší běžící požadavek (proces lokálního agenta se ukončí).

### Řešení konfliktů merge

Model dostane cestu a jazyk souboru, základní, aktuální (A) a příchozí (B) blok
konfliktu s popisky, které editor ukazuje, až 20 řádků již sloučeného textu před
ním a za ním a vaše instrukce (globální, repozitáře a instrukce *Řešení
konfliktů* v Nastavení > AI). Odpoví vyřešeným textem tohoto jediného konfliktu
v bloku kódu; odpověď se značkami konfliktu nebo povídáním místo kódu se jednou
pošle zpět, pak se odmítne. Konflikt nad zhruba 40 KB se neodesílá. Ovládací
prvky jsou skryté, dokud je AI vypnutá, a návrh nikdy nic neuloží, nepřidá do
indexu ani nevyřeší: zkontrolujte Výstup, upravte ho a uložte jako obvykle.

![Formulář commitu po stisku tlačítka s jiskrou: souhrn a popis napsané modelem, připravené k úpravě](../images/ai-commit-message.png)

![Vyskakovací okno Vysvětlení pracovních změn: vysvětlení a seznam věcí k dvojí kontrole](../images/ai-explain.png)

### Composer commitů

Model rozdělí pracovní změny do několika commitů (každý změněný soubor přesně do
jednoho; neplatný plán se jednou pošle zpět, pak se odmítne). Plán zkontrolujete,
než se cokoli commituje:

![Kontrola v composeru: čtyři navržené commity s jejich soubory a tlačítka Obnovit, Zrušit a Vytvořit 4 commity](../images/composer.png)

| Klávesa | Myš | Co dělá |
|---|---|---|
| `Up` / `Down` (`k` / `j`) | klik na řádek | pohyb po commitech a souborech |
| `Alt+Up` / `Alt+Down` (`K` / `J`), `m` | šipky, přetažení | přesune commit; přesune soubor do jiného commitu |
| `Right` / `Left`, `*` | šipka řádku | rozbalí / sbalí commit, všechny (počáteční stav určuje `ui.composer_files`) |
| `e` / `Enter` | tužka | upraví zprávu |
| `d` / `Delete` | `drop` / `obnovit` | vynechá commit (jeho soubory zůstanou necommitnuté) |
| `n` | `+ vlastní commit` | přesune soubor do vlastního commitu |
| `r` | `Obnovit` | zpět na plán modelu |
| `Ctrl+S` / `a` | `Vytvořit N commitů` | vytvoří je v pořadí |
| `Esc` | `Zrušit` | odejde, nic se neprovede |

Vytvoření přidá soubory každého commitu do indexu a postupně je commituje;
pracovní strom se nikdy nedotkne. Soubor v indexu jen zčásti se commituje celý.
Pokud krok selže, zastaví se a oznámí, kolik commitů vzniklo.

### Rozdělení commitu

Model přečte změny commitu, očíslované tak, jak je čísluje [editor rozdělení](log-and-history.md#rozdělení-commitu), a navrhne několik commitů, každý s vlastní zprávou; umí seskupovat hunky i jednotlivé řádky. Odpověď, která nějakou změnu vynechá, přiřadí dvakrát nebo pojmenuje něco neexistujícího, se odmítne. Editor rozdělení se otevře v každém případě: buď s vyplněným návrhem, nebo, když model selhal, s důvodem v poznámce, abyste mohli rozdělit ručně. Nic se nerozdělí, dokud nestisknete **Rozdělit** (`Ctrl+S`); pokud seskupení nesedí, nejdřív změny přesuňte, commity přejmenujte nebo přidejte další. Profil a instrukce jsou ty pro *Rozdělení commitu* v Nastavení > AI.

### Skript pro přepis historie

Popíšete, co má přepis udělat („posunout každý commit vytvořený v noci na následující
ráno“, „nahradit můj starý e-mail novým, jen před rokem 2024“), a model napíše
[skript v Lua](history-rewrite.md#skripty-v-lua) pro pravidlo Skript, s hlavičkou,
`rewrite_commit(c, ctx)` a pomocnými funkcemi izolovaného prostředí. Daemon odpověď
zkompiluje; když se nezkompiluje, chyba se modelu vrátí jednou. Co zbyde, se ukáže
v editoru: přečtěte si to, vyzkoušejte a teprve pak spusťte náhled. Podrobnosti a dialog
jsou v části [Přepis historie](history-rewrite.md#napsat-skript-pomocí-ai). Profil a instrukce
jsou ty pro *Skript pro přepis historie* v Nastavení > AI.

### Pravidla pro přepis historie

Vlastními slovy popíšete, jak se má historie přepsat („nahradit můj starý e-mail a@b.org adresou
c@d.org“, „odstranit secrets.txt z celé historie“), a model navrhne rozsah, některá nastavení a
seznam [pravidel přepisu](history-rewrite.md#vytvořit-pravidla-pomocí-ai) ve slovníku samotného
nástroje. **Jen připravuje**: odpověď vyplní nástroj, nic se nezobrazí v náhledu ani neprovede,
a položka, kterou nelze použít, se vypíše s důvodem, zatímco platné zůstanou. Pravidlo Skript se
takto nikdy nevytvoří. Volitelně se přidá rozsah, názvy větví a tagů a autoři nejnovějších
commitů (zaškrtávací políčko, ve výchozím stavu vypnuté, viditelné v podokně „co se odešle“). Profil
a instrukce jsou ty pro *Pravidla pro přepis historie* v Nastavení > AI.

## Kam jde klíč

Klíč API (OpenAI, Anthropic, OpenRouter, server kompatibilní s OpenAI) je navázán na adresu, pro kterou byl uložen: původ základní URL profilu, zaznamenaný v `trust.toml` v soukromé datové složce jen pro uživatele. Pokud základní URL později ukazuje jinam, požadavek se odmítne dřív, než se cokoli odešle, a klíč zůstane, kde je. Spusťte `gitpoulpi ai trust <profile> --accept-origin <url>` (nebo to přijměte v aplikaci), abyste klíč záměrně přesunuli na novou adresu.

- Prosté `http://` je povoleno na tomto počítači (Ollama, LM Studio na `localhost`); kdekoli jinde vyžaduje `gitpoulpi ai trust <profile> --allow-insecure-http on`.
- Vlastní server, jehož certifikát nevede k žádné známé autoritě, se přijme jen pro otisk klíče, který připnete pomocí `--accept-certificate <sha256>`; TLS handshake stále dokazuje, že server tento klíč drží.
- Profil, který měl klíč už před existencí této kontroly, se při prvním použití tiše zaznamená z aktuální základní URL.
- Lokální agent (`claude -p` a podobní) nemá klíč ani HTTP, takže není co navázat.

## Kontext kontroly

Koncept kontroly kódu (`✦` nebo `hosting review suggest`) čte diff, název, popis a existující komentáře
žádosti. Může dostat i vlastní konvence projektu, v bloku **Project conventions** umístěném před diffem, a místní
agent může pracovat uvnitř repozitáře. Výchozí hodnoty:

| Nastavení | Výchozí | Význam |
|---|---|---|
| `run_in_repository` | `profile` | `profile`: rozhoduje *Pracovní složka* profilu AI. `yes`: místní agent pro kontrolu běží v repozitáři, ať profil říká cokoli. `no`: ne tam (profil nastavený na *Repozitář* použije místo toho vyhrazenou složku). Profily HTTP pracovní složku nemají, takže se pro ně nic nemění. |
| `include_claude_md` | zapnuto | Předat modelu soubor `CLAUDE.md` repozitáře. |
| `include_agents_md` | zapnuto | Předat modelu soubor `AGENTS.md` repozitáře. |
| `extra_file` | žádný | Ještě jeden soubor: cesta relativní ke kořeni repozitáře, absolutní cesta nebo odkaz. |

Vrství se jako instrukce, přičemž vyhrává to nejkonkrétnější, pole po poli: globální tabulka `[ai.review]`,
pak vlastní hodnoty repozitáře (nenastavené pole se zdědí), pak volby jednoho běhu. Repozitář, který není
naklonovaný, uchovává své vlastní hodnoty podle `host/project` v `remote-reviews.toml`, vedle svých konceptů
kontrol.

Repozitář může globální další soubor také **vypnout**: jeho `no_extra_file = true` (zaškrtávací pole **Žádný
další soubor** v *Nastavení repozitáře > AI*, v dialogu voleb kontroly ✦ a přepínač `--no-extra-file` příkazu
`ai review-context`) znamená „pro tento repozitář žádný další soubor“, ať `[ai.review]` určuje cokoli. **Prázdné**
pole **Další soubor** to není totéž: zdědí globální soubor. Soubor repozitáře a přepínač se vylučují (nastavení
jednoho vymaže druhý) a přepínač vyhrává, pokud ruční úprava souboru obsahuje obojí. Starší `extra_file = ""`
repozitáře stále znamená žádný.

Odkud soubory pocházejí:

- `CLAUDE.md`, `AGENTS.md` a relativní další soubor se čtou z **kořene** repozitáře ve větvi, na kterou
  žádost míří, nikdy z pracovního stromu (který může být na jiné větvi nebo mít lokální úpravy). S klonem
  ze základního commitu žádosti; bez něj z API hostitele.
- Absolutní cesta se čte z tohoto počítače.
- Odkaz na soubor na hostiteli kódu (GitHub `.../blob/<branch>/<path>`, GitLab `.../-/blob/<branch>/<path>`,
  Gitea a Forgejo `.../src/branch/<branch>/<path>` nebo odkaz na raw) se čte přes profil integrace, jehož
  hostitel odpovídá, takže soukromý repozitář funguje s tokenem, který jste už zadali. Odkaz, který
  neodpovídá žádnému profilu, nebo jakákoli jiná adresa se stáhne anonymně. Název větve s lomítkem nelze
  z odkazu přečíst: použijte cestu nebo odkaz na větev bez lomítka.
- Chybějící `CLAUDE.md` nebo `AGENTS.md` se tiše přeskočí; chybějící další soubor se ohlásí
  (`context.problems` v JSON příkazové řádky).
- Každý soubor se ořízne na **8 KB** a všechny dohromady na **16 KB**, s viditelnou poznámkou
  `[CLAUDE.md cut: showing ... of ... bytes]` v tom, co model čte.

Když místní agent běží v repozitáři, prompt mu říká, že soubory ve checkoutu nemusí odpovídat
hlavě žádosti a že rozhoduje diff.

V zobrazení kontroly `s` otevře dialog **✦ Možnosti kontroly** s vyplněnými hodnotami: hodnotami repozitáře nad
globálními. Změňte je jen pro tento běh, nebo zaškrtněte *Uložit jako výchozí pro tento repozitář*, aby se
zachovaly (viz [Možnosti kontroly](review.md#možnosti-kontroly)). Výchozí hodnoty lze upravit i v *Nastavení > AI* (blok
*Kontrola kódu*, čtyři řádky) a pro každý repozitář v *Nastavení repozitáře: AI*, kde každý řádek může
*použít globální hodnotu* a tlačítko *Obnovit globální* zahodí vlastní hodnoty repozitáře; hledání v Nastavení je
najde (`Zahrnout AGENTS.md`, `Další soubor`). Cokoli, co model nemohl dostat (chybějící další soubor), nebo co se
oříznulo, aby se to vešlo, je ukázáno na kartě konceptu.

Z příkazové řádky:

```sh
gitpoulpi ai review-context                      # globální výchozí hodnoty a co kontrola dostane
gitpoulpi ai review-context --agents-md off      # uložit globální výchozí hodnotu
gitpoulpi --repo ~/work/app ai review-context --claude-md off --context-file docs/REVIEW.md
gitpoulpi ai review-context --project github.com/acme/app --context-file https://github.com/acme/rules/blob/main/review.md
gitpoulpi ai review-context --repo ~/work/app --reset   # opět se řídit globálními hodnotami
gitpoulpi hosting review suggest 12 --claude-md off --context-file docs/REVIEW.md --run-in-repo no
```

`ai review-context` nepotřebuje model a funguje i při vypnuté AI. `ai prompt --action code_review` vypíše
účinný kontext jako `review_context`. Stejná čtyři nastavení jsou `ai.review.*` v
`config.toml` ([reference](config-reference.md#ai-vestavěná-ai-ve-výchozím-stavu-vypnuta)).

## Soukromí

- Nic se neodesílá, dokud nestisknete tlačítko nebo nespustíte příkaz. Žádná AI
  na pozadí, žádná telemetrie, žádný účet GitPoulpi.
- Co se odesílá: prompt (vestavěný, pak vaše instrukce) a diff, na který jste se
  ptali, oříznutý na zhruba 60 KB (s poznámkou o tom); u pull requestu také
  předměty commitů větve.
- Požadavek na skript pro přepis historie odešle vaši instrukci a pevný kontrakt skriptu
  (API izolovaného prostředí pro skripty). Autor, datum a předmět až 20 commitů odejdou
  **jen** tehdy, když v dialogu zaškrtnete *Odeslat vzorek* (ve výchozím stavu vypnuto;
  CLI vyžaduje `--sample N`). Obsah souborů a tajné hodnoty se neodesílají nikdy.
- Jde to jen k profilu zvolenému pro danou akci: k hostiteli daného API, nebo
  na stdin příkazu vašeho lokálního agenta (který pak dělá, co dělá ten agent).
- Klíče API používá samotný proces `gitpoulpi` a nikdy se nedostanou k daemonu,
  k ostatním klientům okna ani ke klientovi MCP.
- Vygenerovaný text se nikdy sám necommituje, nepushuje ani neaplikuje.

### Co se z požadavku odstraní

Každý požadavek, ať ho odesílá kterákoli akce a ať ho přijímá kterýkoli poskytovatel
(hostované API nebo lokální agent), nejprve projde stejným krokem čištění:

- **Tajné hodnoty se skryjí** (`[ai.privacy] redact_secrets`, ve výchozím stavu zapnuto). Token
  se známou předponou, blok soukromého klíče, JWT, heslo v URL a řádek
  `password = value`, jehož hodnota vypadá náhodně, se nahradí
  `[REDACTED:<rule>]`. Není to vyčerpávající: najde běžné tvary (stejný
  skener jako `rewrite scan-secrets`).
- **Citlivé soubory se vynechají** (`[ai.privacy] exclude_sensitive_files`, ve výchozím stavu
  zapnuto). Diff souboru odpovídajícího `[privacy] sensitive_patterns` (`.env`
  a `.env.*` kromě souborů `.example`, `.sample` a `.template`, `*.pem`,
  `*.key`, soukromé klíče SSH, `*.p12`, `*.pfx`, `*.jks`, `*.keystore`, `*.crt`,
  `*.cer`, `*.der`, `*.kdbx`) není součástí požadavku. Rozdělený kus nebo seznam
  souborů composeru, který takový soubor jmenuje, se ořízne stejně, takže plán pro něj
  může být odmítnut jako neúplný: tento soubor přidejte do indexu nebo rozdělte sami.

Vypnutí kteréhokoli z nich je vaše volba a Nastavení > AI pod ním ukazuje červený řádek,
dokud je vypnuté.

**Co se odesílá, než se to odešle.** Dříve než požadavek odejde, TUI ukáže dialog
**Co se odešle**: instrukce a požadavek po vyčištění (stejné podokno jako v dialozích přepisu
historie, posouvatelné, `Alt+W` ho ukáže nebo skryje), kolik tajných hodnot bylo nahrazeno a které
soubory byly vynechány (pravidlo a maskovaný náhled jako `ghp_****abcd`, nikdy hodnota), zaškrtávací
pole **Příště nezobrazovat pro <akce>** a **Odeslat** / **Zrušit** (`Esc`, zavírací křížek a klik
mimo ruší; nic neodejde, dokud nestisknete Odeslat). Pole zapíše
`[ai.sent_preview] <action> = false`; Nastavení > AI > *Zobrazit, co se odesílá, před...*
má jeden přepínač na akci a **Zobrazit vše znovu**. Oba dialogy přepisu historie
už svůj požadavek ukazují, takže přepínač nemají.

**Na příkazové řádce** každá odpověď `ai ...` (a `hosting review suggest`)
nese `privacy: {redactions: N, excluded_files: [...]}` a `--print-sent`
vypíše vyčištěný systémový a uživatelský text jako `{sent: [{system, user}], privacy,
dry_run: true}`, aniž by volal model (vyžaduje `ai.enabled = true`, žádný klíč
a žádnou síť).

**Omezení.** Lokální agent, jehož pracovní složkou je repozitář
(`workdir = repository`, nebo *Spustit agenta v repozitáři* u kontroly), si může
soubory přečíst sám: čištění pokrývá to, co mu GitPoulpi pošle, ne to, co
otevře sám. Použijte pracovní složku `dedicated`, aby zůstal mimo
repozitář.

## Řešení potíží

| Příznak | Řešení |
|---|---|
| Nikde není `✦` | AI je vypnutá: *Nastavení > AI > Zapnout AI*. |
| `✦` je zašedlé | Nic není v indexu (zpráva commitu). |
| *Otestovat připojení*: příkaz nenalezen | **Příkaz** lokálního agenta není na `PATH`, který GitPoulpi vidí: spusťte GitPoulpi tam, kde funguje `claude --version`, nebo zadejte úplnou cestu. |
| Lokální agent visí nebo ukazuje interaktivní obrazovku | V bezobslužném režimu spuštění přidejte do **Argumentů** jeho bezobslužný přepínač (`-p` u Claude Code), nebo znovu zvolte jeho **Typ agenta**, aby se obnovilo bezobslužné volání. Interaktivní obrazovka je to, k čemu je určen režim *Interaktivní (virtuální terminál)*. |
| "Use "Open the agent"..." | Agent položil otázku při prvním spuštění (důvěra ke složce, přihlášení). Použijte u profilu **Otevřít agenta...**, odpovězte, zavřete agenta a zkuste to znovu. |
| Požadavek ve virtuálním terminálu vyprší | Zpráva končí posledními řádky obrazovky agenta: ukáže se tam žádost o oprávnění, přihlášení nebo otázka na aktualizaci. Odpovězte na ni přes **Otevřít agenta...**, nebo zvyšte **Časový limit**. |
| `selhalo:` 401 | Špatný klíč nebo klíč jiného poskytovatele: zadejte ho znovu. |
| `selhalo:` 404 | **Model** tam neexistuje: vymažte ho a použije se výchozí. |
| `selhalo:` chyba připojení (Ollama, LM Studio) | Server neběží, nebo **Základní URL** má špatný port či chybí `/v1`. |
| Klíč po zadání ukazuje `(nenastaven)` | Není credential helper: nakonfigurujte jeden (`git config --global credential.helper ...`) a zadejte klíč znovu. |
| Špatný jazyk nebo styl | Přidejte instrukce; čipy Rychlého vložení v editoru to urychlí. |
| Composer hlásí, že plán je neplatný | Model přeskočil nebo zopakoval soubor: zkuste to znovu nebo použijte silnější model. |

## Starší konfigurační soubory

Starší `config.toml` s `[[ai.presets]]` a `[ai.defaults]` se stále načte:

- Výchozí předvolba každé akce se stane jejím `[ai.actions.<action>]`
  (`system_prompt` se stane `instructions`).
- Každá stará předvolba se uchová v `[[ai.legacy_presets]]`. Nastavení > AI je
  ukazuje pod *Staré předvolby (před redesignem)* s tlačítkem *Smazat archiv*.
- Dřívější `[preferences.ai_presets]` repozitáře se stane přepsáním repozitáře.

Vestavěný prompt je nyní vždy první. Pokud starý prompt zkopíroval vestavěný
prompt, odstraňte zkopírovanou část a ponechte jen to, co chcete přidat.
