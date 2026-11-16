[English](../hosting-integration.md) · [Čeština](../cs/hosting-integration.md) · [Deutsch](../de/hosting-integration.md) · [Español](../es/hosting-integration.md) · [Français](../fr/hosting-integration.md) · [Italiano](../it/hosting-integration.md) · [日本語](../ja/hosting-integration.md) · [한국어](../ko/hosting-integration.md) · [Nederlands](../nl/hosting-integration.md) · **Polski** · [Português (Brasil)](../pt-BR/hosting-integration.md) · [Português (Portugal)](../pt-PT/hosting-integration.md) · [Русский](../ru/hosting-integration.md) · [Svenska](../sv/hosting-integration.md) · [Türkçe](../tr/hosting-integration.md) · [Українська](../uk/hosting-integration.md) · [简体中文](../zh-CN/hosting-integration.md)

# Integracje z hostingiem i trackerami zgłoszeń

GitPoulpi łączy się z tymi produktami:

- **Hosting** (pull requesty, zgłoszenia, klonowanie z twojego konta, klucze SSH):
  GitHub, GitHub Enterprise, GitLab, GitLab Self-Managed, Azure DevOps,
  Azure DevOps Server, Bitbucket, Bitbucket Data Center, Gitea i Forgejo
  (także Codeberg). **Gerrit Code Review** jest hostingiem tylko do przeglądu: jego zmiany to
  pull requesty, nie ma zgłoszeń.
- **Trackery zgłoszeń** (tylko zgłoszenia): Jira Cloud, Jira Data Center, Trello,
  Linear, YouTrack (Cloud i Server) oraz Redmine.
- **Lokalne notatki Markdown** (tylko zgłoszenia, bez tokena, bez sieci): folder z notatkami Markdown, na przykład folder vaultu Obsidian, zob. [Zgłoszenia lokalne i Obsidian](local-issues.md).

Każda integracja używa **osobistego tokenu**. Nie ma konta GitPoulpi ani
logowania OAuth. Integracje są opcjonalne i **włączane osobno dla każdego
repozytorium**: nic nie jest pobierane, dopóki nie powiążesz repozytorium. Twój
token jest przechowywany w menedżerze poświadczeń, nigdy w `config.toml`. Zwykłe
operacje `git` (fetch, pull, push) nie potrzebują integracji: używają
poświadczeń, które twój `git` już ma.

## 1. Utwórz token na hoście

Formularz połączenia ma odnośnik *Wygeneruj token na <host>*. Otwiera stronę
tokenów hosta z już zaznaczonymi właściwymi zakresami. Aby zrobić to ręcznie:

| Host | Gdzie | Zakresy |
|---|---|---|
| GitHub, GitHub Enterprise | Settings > Developer settings > Personal access tokens (classic) > Generate new token | `repo` (repozytoria, pull requesty, zgłoszenia), `read:user` (kim jesteś), `admin:public_key` (tylko do dodawania lub usuwania kluczy SSH) |
| GitLab, GitLab Self-Managed | User settings > Access tokens > Add new token | `api`, `read_user` |
| Azure DevOps | User settings > Personal access tokens, pod adresem `dev.azure.com/<organization>/_usersSettings/tokens` | `Code (Read & write)`, `Work Items (Read & write)`. Utwórz token **dla tej organizacji**, a nie dla „All accessible organizations”: tokeny globalne przestają działać 2026-12-01. |
| Azure DevOps Server | Ta sama strona na twoim serwerze (`https://<server>/<collection>/_usersSettings/tokens`) | Te same dwa zakresy |
| Bitbucket | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token with scopes | `read:user:bitbucket`, `read:workspace:bitbucket`, `read:repository:bitbucket`, `write:repository:bitbucket`, `read:pullrequest:bitbucket`, `write:pullrequest:bitbucket`. Dla kluczy SSH także `read:ssh-key:bitbucket`, `write:ssh-key:bitbucket`, `delete:ssh-key:bitbucket`. |
| Bitbucket Data Center | Profile picture > Manage account > HTTP access tokens | `Repository (Read & write)`, `Projects (Read)` |
| Gitea, Forgejo (Codeberg) | Settings > Applications > Generate New Token, pod adresem `<host>/user/settings/applications` | Odczyt `user`; odczyt i zapis `repository`, `issue` i `user` (ostatni dla kluczy SSH). Gitea nie pokazuje zakresów po fakcie, więc GitPoulpi nie może ostrzec o brakującym: odrzucone wywołanie o tym mówi. |
| Linear | Settings > Account > Security & access > Personal API keys > New API key (`linear.app/settings/account/security`) | Read, write, create issues. Klucz działa jako ty. |
| YouTrack (Cloud i Server) | Twój profil > Account Security > New token, pod adresem `<host>/users/me?tab=account-security` | Zakres `YouTrack`. Stały token działa jako ty. |
| Gerrit | Settings > HTTP Credentials > Generate new token (nowsze serwery; starsze: Generate new password), pod adresem `<host>/settings/#HTTPCredentials` | Brak. Token to **hasło HTTP** i działa jako ty; obok niego podaje się nazwę użytkownika. |
| Redmine | My account > API access key > Show, pod adresem `<host>/my/account`. Administrator musi włączyć **Administration > Settings > API > Enable REST web service**. | Brak. Klucz działa jako ty. |
| Jira Cloud | `id.atlassian.com/manage-profile/security/api-tokens` > Create API token (klasyczny rodzaj, bez zakresów) | Brak. Token działa jako ty. |
| Jira Data Center | Profile > Personal Access Tokens | Brak. Token działa jako ty. |
| Trello | Najpierw **klucz API** z `trello.com/apps/admin` (utwórz Power-Up, potem wygeneruj klucz API). Potem **token** z `trello.com/1/authorize?...&key=<your key>`. | `read`, `write` |

Nadaj tokenowi nazwę, np. „GitPoulpi”, i datę wygaśnięcia. Skopiuj token: host
pokazuje go tylko raz.

## 2. Połącz go

1. Naciśnij `Ctrl+,` i wybierz **Integracje**.
2. Wybierz produkt po lewej. Produkty są w dwóch grupach: **Hosting** i
   **Trackery zgłoszeń**.
3. Naciśnij **Połącz**. Aby dodać drugie konto tego samego produktu, użyj
   **+ Dodaj profil**.

   ![Strona Integracje: produkty po lewej i przycisk Połącz dla GitHub](../images/settings-integrations.png)

4. Wypełnij formularz i wklej token. Token jest pokazywany jawnym tekstem
   podczas wpisywania, więc upewnij się, że nikt nie patrzy. Formularz wylicza
   zakresy, których potrzebuje.

   ![Formularz połączenia: pole Osobisty token dostępu, zakresy repo, read:user i admin:public_key, odnośnik do wygenerowania tokenu na github.com oraz przyciski Połącz i Anuluj](../images/settings-connect.png)

5. Naciśnij **Połącz**. GitPoulpi sprawdza token z hostem w tle. Strona pokazuje
   potem kartę z twoim imieniem i loginem, *Połączono* na zielono oraz sekcję
   klucza SSH. Token bez jednego z oczekiwanych zakresów nadal się łączy, a
   brakujące zakresy są wymienione na czerwono.

Na karcie: **Edytuj** ponownie otwiera formularz. **Rozłącz** usuwa token i
zachowuje profil. **Usuń** usuwa oba, po potwierdzeniu.

Klawiatura na stronie Integracje: `Tab` / `Shift+Tab` przechodzą między listą
produktów, paskiem profili i formularzem. `a` dodaje profil, `e` go edytuje,
`d` lub `Delete` go usuwa, `o` otwiera stronę tokenu, `Esc` cofa o jeden
krok.

### O co prosi każdy formularz

Odnośnik *Wygeneruj token* działa dopiero po wpisaniu hosta lub organizacji.

- **GitHub, GitLab:** tylko token. **GitHub Enterprise** i
  **GitLab Self-Managed** proszą też o **Domena hosta**
  (`gitlab.example.com`; https, chyba że wpiszesz `http://`). Nazwa bez kropki
  (usługa Dockera albo maszyna w sieci lokalnej) wymaga portu lub `http://`: `gitea:3000`,
  `http://nas`. Serwer hostowany samodzielnie może
  działać po zwykłym http, na przykład lokalny kontener pod `http://localhost:8080`. Token wysłany
  zwykłym http jest przesyłany bez szyfrowania, więc używaj http tylko w sieci
  lokalnej lub zaufanej.
- **Azure DevOps:** **Organizacja**, czyli nazwa w
  `dev.azure.com/<organization>`. Adres `https://<org>.visualstudio.com` też
  jest przyjmowany. Dwie organizacje wymagają dwóch profili. Repozytorium
  używa profilu organizacji wskazanej przez jego zdalne repozytorium. **Azure DevOps Server**
  prosi o **URL kolekcji**, na przykład `tfs.corp/tfs/DefaultCollection`.
  GitPoulpi nie używa systemowego menedżera poświadczeń dla Azure DevOps: Git
  Credential Manager przechowuje tam token logowania, a nie osobisty token
  dostępu.
- **Bitbucket:** **E-mail konta Atlassian** i token API. Jeśli e-mail jest
  pusty, token jest wysyłany jako token typu bearer: użyj tego dla tokenu
  dostępu workspace lub repozytorium. GitPoulpi znajduje twoje workspace'y na
  podstawie tokenu. **Bitbucket Data Center:** **URL serwera** i token dostępu
  HTTP.
- **Gitea, Forgejo:** **Domena hosta** i token. **Forgejo** zaczyna od
  `https://codeberg.org`: zastąp ją dla serwera na własnym hostingu. Gitea nie ma
  publicznej wartości domyślnej. Adres `http://localhost:3000` działa dla serwera lokalnego.
  Oba rodzaje dzielą jednego klienta (Forgejo to fork Gitea z tym samym
  `/api/v1`); token jest wysyłany jako `Authorization: token <token>`. Systemowy
  menedżer poświadczeń nie jest dla nich używany.
- **Jira Cloud:** **URL witryny** (`acme.atlassian.net`), twój **E-mail konta
  Atlassian** i klasyczny token API. Tokeny Jira z zakresami nie są obsługiwane.
  **Jira Data Center:** **URL serwera** i osobisty token dostępu.
- **Trello:** **Klucz API** i **token**. Klucz API jest publiczny, więc jest
  przechowywany w `config.toml` (jako `account`). Token jest tajny i trafia do
  menedżera poświadczeń. Formularz ma dwa odnośniki: *Pobierz klucz API w Trello*
  i, po wpisaniu klucza, *Wygeneruj token*. Klucz i token są wysyłane w
  nagłówku, nigdy w adresie URL.

- **Linear:** tylko **klucz API**: Linear ma jeden publiczny host, więc nie ma
  adresu do wpisania. Klucz jest wysyłany bez zmian w nagłówku `Authorization`
  (Linear nie przyjmuje `Bearer` dla kluczy osobistych). Aby odwołania `ENG-12`
  w opisach commitów otwierały właściwą stronę, zapisz URL profilu razem ze swoim
  obszarem roboczym (`https://linear.app/acme`, z CLI: `--url`); same wiersze zgłoszeń
  zawsze niosą dokładny adres.
- **YouTrack:** **URL witryny** (`acme.youtrack.cloud` dla Cloud, w pozostałych
  przypadkach twój własny serwer, z jego ścieżką kontekstu, jeśli ją ma, na przykład
  `host/youtrack`) i stały token, wysyłany jako `Authorization: Bearer`. Ten sam
  klient obsługuje Cloud i Server (`<url>/api`).

- **Gerrit:** **URL serwera** (ze ścieżką kontekstu, jeśli ją ma), twoja **Nazwa użytkownika** i hasło
  HTTP jako token. Podróżują jako uwierzytelnianie HTTP basic; GitPoulpi rozmawia z API REST `/a/`
  serwera i ignoruje linię `)]}'`, od której zaczyna się każda odpowiedź. Nazwa użytkownika nie jest tajna i jest przechowywana w
  `config.toml` (jako `account`), hasło trafia do menedżera poświadczeń. Repozytorium jest powiązane z *projektem*
  Gerrit w sekcji PULL REQUESTS (tam wypisane są jego zmiany); Gerrit nie ma zgłoszeń, a zmianę tworzy się
  wypychając (zob. *Wypchnięcie do przeglądu* w [Przeglądzie](review.md#gerrit)).
- **Redmine:** **URL witryny** twojego Redmine (z podścieżką, jeśli ją ma,
  na przykład `host/redmine`) i twój klucz API, wysyłany jako `X-Redmine-API-Key`. Jeśli
  Redmine odpowiada 403 lub stroną HTML, usługa REST jest wyłączona (poproś
  administratora albo włącz ją w **Administration > Settings > API**).

Jira, Trello, Linear, YouTrack i Redmine to wyłącznie trackery zgłoszeń: bez pull requestów, bez
repozytoriów do sklonowania i bez kluczy SSH. Bitbucket nie ma zgłoszeń
(Atlassian wycofał Bitbucket Issues). Zamiast tego powiąż sekcję ZGŁOSZENIA
repozytorium Bitbucket z projektem Jira.

## 3. Gdzie przechowywany jest token

Token jest przechowywany w systemowym **menedżerze poświadczeń** (Git Credential
Manager w Windows, libsecret lub `pass` w Linuksie, Keychain w macOS). Używa
prywatnego wpisu, którego nie da się pomylić z poświadczeniami prawdziwego
hosta. Nigdy nie jest zapisywany w `config.toml` ani wypisywany.

Jeśli żaden menedżer poświadczeń nie jest skonfigurowany albo odrzuca token,
GitPoulpi trzyma token **w pamięci tylko na tę sesję** i cię o tym informuje.
Po restarcie połącz ponownie.

GitPoulpi szuka tokenu profilu w tej kolejności:

1. Zmienna środowiskowa `GITPOULPI_TOKEN_<PROFILE ID>`. Użyj jej, by podać
   token bez przechowywania, na przykład w CI.
2. Token zapisany dla profilu (`token_ref`).
3. Tylko dla profili GitHub i GitLab: systemowy menedżer poświadczeń dla hosta,
   gdy żaden inny włączony profil nie używa tego samego hosta.

Każdy profil to wpis `[[integrations]]` w `config.toml`. Zob.
[Opis konfiguracji](config-reference.md#integrations-hosting-i-trackery-zgłoszeń).

## Kontrola bezpieczeństwa

Zapisany token jest tak bezpieczny, jak adres, na który jest wysyłany. Dlatego GitPoulpi zapamiętuje dla każdej integracji, z czym była połączona, i zatrzymuje się, gdy to się zmieni:

- **Adres.** Przy pierwszym użyciu integracji (lub połączeniu) jej adres jest zapisywany w `trust.toml`, w prywatnym folderze danych dostępnym tylko dla użytkownika (`<data>/private/`), a nie w `config.toml`: edycja konfiguracji nigdy nie poszerzy tego, czemu zaufano. Jeśli adres w konfiguracji nie jest już zapisanym, nic nie jest do niego wysyłane, a token nie jest nawet odczytywany. Różne zapisy jednego adresu (`HTTPS://Git.Example.com:443/`) to ten sam adres; host z podobną nazwą nie.
- **Konto.** Po połączeniu tokenu, a potem mniej więcej raz dziennie (i raz na uruchomienie dla tokenu pochodzącego z `GITPOULPI_TOKEN_*` lub z helpera poświadczeń), GitPoulpi pyta hosta, do kogo należy token, i porównuje **id** konta, a nie login. Token, który teraz należy do kogoś innego, jest zatrzymywany.
- **Certyfikat.** Serwery, których certyfikat nie łączy się ze znanym urzędem (prywatny urząd certyfikacji, serwer z certyfikatem samopodpisanym), są akceptowane tylko dla dokładnie tego klucza, który zaakceptowałeś, a uzgadnianie TLS nadal dowodzi, że serwer ma ten klucz. Przy włączonym ścisłym przypinaniu klucz musi pasować nawet wtedy, gdy certyfikat łączy się z publicznym korzeniem; ceną jest to, że każda rotacja klucza pyta ponownie, dlatego jest to opcja do włączenia. Hosty publiczne (github.com, gitlab.com, codeberg.org, bitbucket.org, dev.azure.com, `*.atlassian.net`, ...) nie mają zapisu certyfikatu.
- **Zwykłe http.** Adres `http://` inny niż pętla zwrotna jest odrzucany, chyba że zezwolono na niego dla tej integracji.

Każde żądanie jest też ograniczone do własnego adresu integracji (a dla hosta publicznego do jego hosta API, takiego jak `api.github.com`): żądanie gdziekolwiek indziej jest odrzucane przed wysłaniem, a przekierowanie na inny adres nie jest śledzone.

**Integracje połączone, zanim ta kontrola istniała**, dostają swój zapis po cichu, z konfiguracji w jej dawnym kształcie, przy pierwszym użyciu po aktualizacji: zachowują dzisiejszy poziom zaufania i nic o nic nie pyta.

### Z wiersza poleceń

Zatrzymane polecenie kończy się kodem wyjścia 2 i wypisuje, co się zmieniło (zob. [Opis CLI](cli-reference.md)). `gitpoulpi hosting integrations trust <id>` pokazuje, czemu zaufano; `--accept-origin <url>`, `--accept-account <id>`, `--accept-certificate <sha256>`, `--strict-tls on|off` i `--allow-insecure-http on|off` akceptują po jednej rzeczy, wskazując dokładną wartość. Po zaakceptowaniu nowego adresu wpisz token ponownie. Rozłączenie (`hosting logout`) lub usunięcie integracji kasuje jej token, zapis zaufania i to, co z niej zapisano w pamięci podręcznej (zapisane listy repozytoriów okna Klonuj, listy kont Radaru i robocze wersje zdalnych przeglądów jej hosta).

Agent (MCP) niczego nie może zaakceptować: dostaje informację, że kontrola bezpieczeństwa wymaga ciebie.

### W interfejsie terminala

Zatrzymane żądanie otwiera okno **Kontrola bezpieczeństwa integracji** nad tym, co akurat robiłeś. Mówi, co się zmieniło (stary i nowy adres, stare i nowe konto z ich id albo stary i nowy certyfikat jako mała tabela z odciskiem SHA-256 w grupach po cztery, podmiotem, wystawcą i ważnością) oraz że token nie został wysłany. Odpowiedzi są ponumerowane:

1. **Rozłącz** kasuje zapisany token, zapis zaufania i to, co z integracji zapisano w pamięci podręcznej.
2. **Zaufaj** nowemu adresowi, kontu lub certyfikatowi (adres bez szyfrowania: **Zezwól na zwykłe http**).
3. **Anuluj** niczego nie zmienia.

Żadna odpowiedź nie jest podświetlona, dopóki nie naciśniesz strzałki lub cyfry, więc przypadkowy Enter niczego nie akceptuje. Esc i krzyżyk zamknięcia anulują; kliknięcie poza oknem nic nie robi. Długa treść przewija się kółkiem, Page Up/Page Down lub paskiem przewijania. Kilka kontroli ustawia się w kolejce (jedna na integrację i rodzaj zmiany) i są zadawane jedna po drugiej. Anulowana kontrola nie wyskakuje ponownie, dopóki się nie zmieni; **Integracje: Przejrzyj kontrolę bezpieczeństwa** w palecie poleceń ją przywraca.

Po zaufaniu nowemu **adresowi** formularz integracji otwiera się z aktywnym polem tokenu i podpowiedzią „Zalecane jest ponowne wpisanie tokenu”: zapisany token nigdy tam nie został wysłany. W formularzu adres zmieniony celowo jest akceptowany razem z wpisanym dla niego tokenem; bez tokenu formularz o niego prosi.

Formularz ma też dwa przełączniki. **Ścisłe przypinanie certyfikatu** (dla adresu https) przypina klucz certyfikatu, który serwer przedstawia przy łączeniu. **Zezwól na zwykłe http (bez szyfrowania)** jest oferowane tylko dla adresu `http://` innego niż ten komputer, z czerwonym ostrzeżeniem, że token jest przesyłany otwartym tekstem. Karta połączonej integracji pokazuje przypięty odcisk.

### Co może token i do kiedy

Połączenie pyta też hosta, co token może robić. GitHub (tokeny klasyczne), GitLab i Bitbucket (gdy przesyła zakresy) raportują swoje zakresy: karta pokazuje żółty wiersz „Ten token ma więcej uprawnień, niż potrzebuje GitPoulpi” z listą zakresów ponad to, czego GitPoulpi używa, a te wrażliwe (takie jak `delete_repo`, `admin:org`, `sudo`) na czerwono, z radą, by wybrać token o precyzyjnych uprawnieniach z datą wygaśnięcia. Klasyczny token GitHub (`ghp_...`) dostaje tę radę nawet wtedy, gdy jego zakresy są dokładne. O uprawnienia tokenu GitHub o precyzyjnych uprawnieniach nie da się zapytać, więc pokazywana jest tylko jego data wygaśnięcia. Karta pokazuje datę wygaśnięcia, gdy host ją podaje. Token bez daty wygaśnięcia nie pokazuje przypomnienia, a inne produkty nie raportują zakresów i nie wywołują ostrzeżenia. Te same pola są w JSON polecenia `hosting integrations login` i `test` (`excess_scopes`, `token_flavor`, `recommend_fine_grained`, `expires_at`). Są odczytywane przy połączeniu, więc karta pokazuje je dla sesji, która się połączyła.

## 4. Powiąż repozytorium

Powiązanie jest jawne i dla każdego repozytorium osobne. Gdy profil jest
połączony, panel boczny Refs pokazuje po TAGS sekcję **PULL REQUESTS** i
sekcję **ZGŁOSZENIA**. Każda sekcja ma dwie listy: dostawcę i projekt.

- Lista dostawcy **proponuje** konta pasujące do zdalnych repozytoriów
  (najpierw `origin`), jako wiersze typu `owner/name · origin · GitHub`. Jeden
  wybór ustawia i dostawcę, i projekt.
- Nic nie jest powiązywane automatycznie. Jira, Trello, Linear, YouTrack i Redmine nigdy nie są
  proponowane, bo żadne zdalne repozytorium ich nie wskazuje: wybierz je sam.
- **Brak (odłącz to repozytorium)** na górze listy odłącza sekcję. Nic nie
  powiąże jej ponownie samo.
- Powiązanie jest zapisywane w pliku `.gitpoulpi/repo-state.toml` repozytorium.

Obie sekcje są powiązywane **osobno**. Na przykład pull requesty mogą pochodzić
z Azure DevOps, a zgłoszenia z Jira.

| Dostawca | Projekt PULL REQUESTS | Projekt ZGŁOSZENIA |
|---|---|---|
| GitHub, GitLab | `owner/name`, `group/sub/proj` | to samo |
| Azure DevOps | `Project/Repo` (organizacja pochodzi z profilu) | `Project` (elementy pracy) |
| Gitea, Forgejo | `owner/name` | to samo |
| Bitbucket | `workspace/slug` (Data Center: `PROJ/slug`) | brak: powiąż Jirę |
| Jira | brak | klucz projektu, na przykład `ABC` |
| Trello | brak | tablica |
| Linear | brak | klucz zespołu, na przykład `ENG` (zespoły są wypisane) |
| YouTrack | brak | krótka nazwa projektu, na przykład `ABC` |
| Gerrit | nazwa projektu Gerrit, na przykład `team/tool` | brak |
| Redmine | brak | identyfikator projektu, na przykład `bench` (ostatnia część adresu projektu) |

Listę projektów ZGŁOSZENIA można przeszukiwać. Można też wpisać projekt, którego
lista nie pokazuje.

**Dodatkowe zapytanie.** Źródło zgłoszeń ma opcjonalny wiersz filtra pod listą
projektu. Jest łączony (AND) z projektem:

- Jira: fragment **JQL** (`component = UI AND labels = backend`).
- Azure DevOps: warunek **WIQL** (`[System.AssignedTo] = @Me`).
- YouTrack: **zapytanie YouTrack** (`#Unresolved Type: Bug`, `tag: ui or tag: ux`).
  Jest ujmowane w nawiasy, więc `or` nie może poszerzyć projektu. Twoje
  `sort by:` zastępuje domyślne `sort by: updated desc`.
- Redmine: brak (filtry stanu i „przypisane do mnie” w sekcji wystarczą).
- Linear: brak (filtry stanu i „przypisane do mnie” w sekcji
  wystarczą).
- Trello: lista rozwijana, na której zaznaczasz **listy** Trello do pokazania. Bez zaznaczenia
  pokazuje każdą otwartą listę.
- GitHub i GitLab go ignorują.

`Enter` zachowuje wpisane zapytanie, `Esc` je odrzuca. Paleta:
*Zgłoszenia: Wybierz filtr...*.

Z CLI: `gitpoulpi hosting link pulls --id <profile>` (lub `issues`),
`--clear` odłącza, `--filter` dla dodatkowego zapytania. Zob.
[Opis CLI](cli-reference.md#polecenia).

### Klonowanie z połączonego konta

Połączony profil (GitHub, GitLab, Gitea, Forgejo, Azure DevOps, Bitbucket, Gerrit) jest też źródłem w oknie **Klonuj** (przycisk **Klonuj** na karcie Home albo tam `c`). Każdy profil jest wymieniony pod **Klonuj z URL** jako osobne źródło (`GitHub · name`, `GitLab · name`); GitHub i GitLab nigdy się nie mieszają. Wybierz jedno, a załadują się jego repozytoria, pogrupowane według właściciela: najpierw twoje konto, potem jedna sekcja na organizację lub grupę (`quarry/tools` dla podgrupy GitLaba), każda pod nagłówkiem z liczbą. Kłódka oznacza repozytorium prywatne.

![Okno Klonuj na połączonym koncie GitHub: jedna sekcja na właściciela i kłódka przy repozytorium prywatnym](../images/clone-hosted.png)

Pole wyszukiwania filtruje we wszystkich sekcjach (nazwa, pełna ścieżka i opis); sekcja bez dopasowań znika. `Enter` na wierszu wypełnia klon adresem HTTPS tego repozytorium i nazwą folderu.

![Okno Klonuj z wyszukiwaniem zawężającym listę do jednej organizacji](../images/clone-hosted-search.png)

Lista profilu, który już otwierano, pokazuje się od razu z zapisanej kopii, podczas gdy host jest pytany ponownie (*odświeżanie...* obok etykiety); gdy odpowiedź dotrze, nowe repozytoria się pojawiają, a nieistniejące znikają, przy czym podświetlone i wybrane repozytorium zostają na miejscu. Jeśli odświeżenie się nie powiedzie, zapisana lista zostaje z adnotacją, a przyczyna jest w Dzienniku aktywności.

Jeśli host odrzuci token, okno zamiast listy pokazuje *nie można wylistować repozytoriów* z przyczyną (a zapisana kopia jest usuwana); pozostałe źródła nadal działają.

## 5. Pull requesty i zgłoszenia w panelu bocznym

### PULL REQUESTS

Sekcja ma nagłówek z licznikiem i `+` (utwórz pull request), listy dostawcy
i projektu, pole wyszukiwania oraz listę stanu (*Otwarte*, *Merged / closed*,
*Wszystkie*). Potem są cztery grupy, każda ze swoim licznikiem:

- **Moje pull requesty**: jesteś autorem.
- **Przypisane do mnie**.
- **Czekają na mój przegląd**: jesteś wskazanym recenzentem.
- **Wszystkie pull requesty**: na początku zwinięte.

Pole wyszukiwania filtruje każdą grupę (tytuł, numer, autor, gałąź). Każda
zmiana listy stanu ponownie pyta hosta.

Wiersz zaczyna się od znaku stanu: `●` otwarty (zielony), `○` szkic, `◆`
scalony (fioletowy), `✕` zamknięty (czerwony). Żądanie bieżącej gałęzi jest
pogrubione. Wiersze wyglądają jak `#12  title` (GitHub, Bitbucket) lub
`!53  title` (GitLab, Azure DevOps).

### ZGŁOSZENIA

Tytuł sekcji wskazuje dostawcę: `GITHUB ZGŁOSZENIA`, `JIRA ZGŁOSZENIA` i tak dalej.
Dla Azure DevOps brzmi `... ELEMENTY PRACY`, a dla Trello `... KARTY`. Pod
listami: pole wyszukiwania, lista stanu (*Otwarte*, *Przypisane do mnie*,
*Zamknięte*, *Wszystkie*) i zgłoszenia. Wiersz Jira wygląda jak
`ABC-12  title`, z przyciemnionym typem i statusem, gdy panel boczny jest
dość szeroki.

### Akcje na wierszu

- `Enter` lub dwukrotne kliknięcie otwiera wiersz: pull request w [przeglądzie](review.md) (GitHub, GitLab, Gitea,
  Forgejo, Gerrit), zgłoszenie oraz żądanie w Azure DevOps lub Bitbucket jako [rozmowę](#5c-rozmowy) w oknie,
  a wszystko inne w przeglądarce. **Otwórz w przeglądarce** w menu wiersza zawsze otwiera przeglądarkę.
- Prawy przycisk myszy lub `.` otwiera menu wiersza:
  - **Otwórz rozmowę** (zgłoszenia i żądania, które nie mają widoku przeglądu), **Otwórz w przeglądarce**,
    **Kopiuj URL**, **Kopiuj numer**.
  - Dla pull requesta: **Edytuj na <dostawca>**, **Checkout gałęzi
    źródłowej**, **Pokaż commity gałęzi źródłowej**. Dla żądania z forka
    checkout brzmi **Checkout pull request (fork)**, a **Pokaż** jest
    wyszarzone.
  - Dla zgłoszenia: **Utwórz gałąź ze zgłoszenia**. Otwiera okno nowej gałęzi
    z nazwą typu `12-fix-the-thing` (lub `ABC-123-fix-the-thing` dla Jira).
  - W GitHub, GitLab, Gitea i Forgejo wiersz pull requesta ma też **Przejrzyj...**
    (także **Enter** na wierszu): przegląd w aplikacji, zob.
    [Przegląd pull lub merge requesta](review.md).
  - W GitHub, GitLab, Gitea i Forgejo oba wiersze mają też **Edytuj przypisanych...**,
    **Edytuj etykiety...** i **Ustaw kamień milowy...** (oraz **Edytuj recenzentów...** przy
    pull requeście): zob.
    [Przypisani, recenzenci, etykiety i kamień milowy](#przypisani-recenzenci-etykiety-i-kamień-milowy).
- **Edytuj na <dostawca>** ma trzy inne wejścia: ołówek `✎` na prawym końcu
  wiersza pull requesta pod kursorem lub zaznaczonego, `e` na zaznaczonym
  wierszu oraz polecenie palety *Integracje: Edytuj powiązany pull request na
  hoście* (dla żądania bieżącej gałęzi).
- Jeśli gałęzi źródłowej nie ma jeszcze w repozytorium, GitPoulpi o tym mówi.
  Najpierw zrób fetch (`Ctrl+G f`).

**Checkout pull requesta.** GitPoulpi pobiera żądanie do lokalnej gałęzi
`pr/<n>` (`mr/<n>` w GitLab), która je śledzi, i przełącza się na nią. W GitHub,
GitLab, Gitea i Forgejo działa to także dla forków (GitPoulpi pobiera specjalny ref head
hosta). Azure DevOps i Bitbucket pobierają gałąź źródłową żądania. Żądanie z
forka jest tam odrzucane: dodaj fork jako zdalne repozytorium i sam zrób checkout jego
gałęzi. Cofnięcie wraca do poprzedniej gałęzi. CLI: `gitpoulpi hosting
checkout-pr <n>`.

**Przegląd pull requesta z wiersza poleceń (i przez agenta).** W GitHub
(także Enterprise), GitLab (także self-managed), Gitea i Forgejo żądanie można
czytać i przeglądać bez opuszczania GitPoulpi: `hosting review show <n>` (pliki,
stan scalenia, kontrole), `hosting review threads <n>` (wątki przeglądu i
dyskusja), `hosting review comment`, `hosting review reply`, `hosting review resolve`
oraz `hosting review submit <n> --event approve|request-changes|comment
--comment path:line[:side]:text`. Diff pochodzi z twojego repozytorium, nie z
hosta: `hosting review fetch <n>` pobiera head żądania do
`refs/gitpoulpi/pr/<n>`, a jego gałąź docelową do `refs/gitpoulpi/pr/<n>-base`
(żadne z nich nie jest gałęzią, nic nie jest wyewidencjonowywane). Numery linii są z tego
diffa: `new` to linia w head, `old` linia w bazie scalenia. GitLab
nie ma stanu przeglądu „request changes”: jest tam odrzucany, nic nie jest wysyłane.
Gitea i Forgejo przyjmują komentarz tylko do jednej linii (bez zakresów); wątek to
komentarze do jednej linii pliku. Gitea potrafi rozwiązać wątek, Forgejo 11 nie
(host nie ma do tego punktu końcowego: wywołanie o tym mówi). Odpowiedź na serwerze
bez punktu końcowego odpowiedzi jest publikowana jako jednokomentarzowy przegląd tej samej linii.
Gitea odmawia zaakceptowania lub odrzucenia twojego własnego żądania.
Bez klonu: dodaj `--project owner/name` (GitLab: `group/project`) do `review show`, `threads`,
`comment`, `reply`, `resolve`, `submit` i `suggest`, aby przejrzeć projekt, który nie jest bieżącym repozytorium,
`hosting review diff <n> --project P` czyta diff (hunki każdego pliku) z API hosta, a
`hosting file <path> --project P [--ref R]` czyta plik w danej rewizji (GitHub, GitLab, Gitea, Forgejo i Gerrit; w Gerricie diff
patch setu jest czytany plik po pliku, a plik pochodzi z gałęzi albo, dla id commita, z commita). Plik, którego host
nie pokazuje w diffie (za duży), jest oznaczony jako `omitted`.
Narzędzia MCP to te same polecenia (`hosting_review_show`, `hosting_review_submit`, `hosting_review_diff`,
`hosting_file`...).

### Przypisani, recenzenci, etykiety i kamień milowy

W GitHub (także Enterprise), GitLab (także self-managed), Gitea i
Forgejo możesz zmienić, komu przypisano pull request lub zgłoszenie, kto je przegląda, jego etykiety
i kamień milowy bez opuszczania GitPoulpi. Inni dostawcy nie pokazują nic z tego.

- **Menu wiersza**: **Edytuj przypisanych...**, **Edytuj recenzentów...** (tylko pull requesty),
  **Edytuj etykiety...**, **Ustaw kamień milowy...**.
- **Paleta**: *Hosting: Edytuj przypisanych...*, *Hosting: Edytuj recenzentów...*,
  *Hosting: Edytuj etykiety...*, *Hosting: Ustaw kamień milowy...* najpierw wypisują
  żądania i zgłoszenia z paska bocznego do wyboru. W
  [Radarze](radar.md) te same cztery działają na zaznaczonym wierszu.
- Selektor otwiera się od razu i wczytuje od hosta osoby lub etykiety projektu (oraz to, co
  element ma teraz). Wiersz ze spinnerem pojawia się po około 150 ms.
  Wpisz, aby filtrować (dopasowanie rozmyte), `Up` / `Down` przesuwają, `Space` zaznacza lub
  odznacza, `Enter` stosuje, `Esc` anuluje, `Tab` dociera do przycisków. Kliknięcie
  wiersza go zaznacza, a kółko przewija. Osoby mają plakietkę z inicjałami,
  etykiety swój kolor.
- Kamień milowy to lista rozwijana (z **Brak kamienia milowego**, aby go wyczyścić).
- Po **Zastosuj** host jest aktualizowany, a listy i Radar pokazują
  zmianę po następnym odświeżeniu, które zaczyna się od razu.

Formularze tworzenia (pull requesta, zgłoszenia) mają te same wybory jako rząd
przycisków: `Przypisani: brak`, `Recenzenci: brak`, `Etykiety: brak`,
`Kamień milowy: brak`. W GitHub, GitLab, Gitea i Forgejo przycisk Etykiety zastępuje wpisywane
pole etykiet formularza zgłoszenia. Jeśli host przyjmie nowy pull request, ale
odrzuci niektóre jego etykiety lub osoby, mówi o tym pasek stanu.

CLI: `gitpoulpi hosting members|labels|milestones`, `hosting pr-edit <n>`,
`hosting issue edit <n>` oraz `--assignee`, `--reviewer`, `--label`,
`--milestone` przy `hosting pr` i `hosting issue create`; zob.
[Opis CLI](cli-reference.md).

### Klawisze w sekcjach

| Klawisz | Akcja |
|---|---|
| `Enter` / dwukrotne kliknięcie | otwiera rozmowę w GitPoulpi, gdy host ją ma (pull request w GitHub, GitLab, Gitea lub Gerrit otwiera przegląd), w przeciwnym razie przeglądarkę |
| `o` | otwiera wiersz w przeglądarce |
| `.` / prawy przycisk | menu wiersza |
| `e` | edytuje zaznaczony pull request na hoście |
| `n` | nowy pull request (PULL REQUESTS) lub nowe zgłoszenie (ZGŁOSZENIA) |
| `v` | otwiera listę stanu |
| `r` | odświeża |
| `Enter` / `Space` na nagłówku | zwija lub rozwija sekcję |

### Ponowne połączenie i tryb offline

Gdy brakuje tokenu, został on odrzucony albo brakuje mu zakresu, pierwszy wiersz sekcji
brzmi `Połącz ponownie (token odrzucony)`. Kliknij go lub naciśnij `Enter`, aby
otworzyć profil w Ustawienia > Integracje.

Gdy host jest nieosiągalny, ostatnie listy zostają na ekranie, a nagłówek
pokazuje ich wiek (`· 5m old`). Ostatnia odpowiedź jest też buforowana na dysku
(`.gitpoulpi/hosting-cache.json`), więc listy pojawiają się, zanim odpowie sieć.

## 5b. Tablica zgłoszeń

Przycisk **Tablica** w nagłówku ZGŁOSZENIA (paleta: **Zgłoszenia: Otwórz tablicę**) pokazuje zgłoszenia powiązanego źródła jako tablicę Kanban
nad oknem. Kolumny zależą od źródła:

| Źródło | Kolumny | Przenoszenie karty |
|---|---|---|
| Lokalne notatki Markdown | statusy (`statuses` profilu, potem każdy inny w użyciu) | przepisuje `status` notatki |
| GitHub, GitLab, Gitea, Forgejo | **Otwarte**, potem wybrane przez ciebie etykiety, potem **Zamknięte** | podmienia etykietę statusu; do **Zamknięte** zamyka zgłoszenie, do kolumny znów je otwiera |
| Trello | otwarte listy tablicy, od lewej do prawej | przenosi kartę na listę |
| Redmine | statusy zgłoszeń | zmienia status zgłoszenia (obowiązują reguły workflow Redmine) |
| Jira (Cloud i Data Center) | statusy projektu, najpierw do zrobienia, na końcu zrobione | uruchamia przejście workflow prowadzące do kolumny (`GET`, potem `POST .../transitions`); gdy takiego przejścia nie ma, karta wraca, a pasek stanu mówi `no transition from X to Y` |
| Azure DevOps (Services i Server) | stany typów elementów pracy projektu, według kategorii stanu | ustawia `System.State`; element pracy, którego typ nie ma stanu kolumny, wraca z powodem (`a Bug has no state Done`) |

W projekcie GitHub lub GitLab **Kolumny** (albo `c`, albo **Tablica: Ustaw etykiety kolumn...**) pytają o etykiety, które są
kolumnami, po kolei, na przykład `status::todo, status::doing, status::review`. Proponują etykiety projektu, które
wyglądają jak statusy. Wybór jest zapisywany dla repozytorium. Bez etykiet tablica to Otwarte / Zamknięte. Przeniesienie karty
dotyka tylko wymienionych przez ciebie etykiet statusu: jej inne etykiety zostają. Tablica hosta wypisuje otwarte zgłoszenia i 60 ostatnio
zamkniętych.

Przejście w Jira, które wymaga wymaganych pól (na przykład rozwiązania), jest odrzucane przez Jirę, a karta wraca z jej własnym
komunikatem: zrób to jedno na stronie.

Ta sama tablica jest dostępna dla skryptów i agentów: `gitpoulpi hosting board` wypisuje kolumny z kartami, a
`gitpoulpi hosting board move <card> <column>` przenosi jedną (zob. [opis CLI](cli-reference.md)).

Karta pokazuje numer, tytuł, przypisanych jako inicjały, etykiety i kamień milowy. Przeciągnij ją na inną kolumnę,
albo `Shift+Left` / `Shift+Right`, albo **Tablica: Przenieś kartę do...** (`m`). Przenosi się od razu; po około 150 ms spinner
mówi, że host jeszcze pracuje, a jeśli host odmówi, karta wraca, a pasek stanu mówi dlaczego. `Enter` otwiera
zgłoszenie na hoście (notatkę, dla zgłoszeń lokalnych); `e` edytuje przypisanych, etykiety i kamień milowy; `n` zaczyna nowe zgłoszenie w
zaznaczonej kolumnie; `a`, `l`, `M` i `/` filtrują według przypisanego, etykiety, kamienia milowego i tekstu, `x` czyści filtry; `r`
czyta hosta ponownie. Tablica czyta ponownie, gdy listy paska bocznego się odświeżają. Radar nie ma tablicy.

## 5c. Rozmowy

**Rozmowa** to wszystko, co powiedziano o jednym zgłoszeniu, elemencie pracy, karcie, zmianie lub pull / merge
requeście: opis, komentarze, przeglądy, wątki w kodzie i zdarzenia (etykiety, przypisania, zmiany stanu,
pushe). GitPoulpi czyta ją z hosta, pokazuje sformatowaną i pozwala odpowiedzieć bez opuszczania okna.

### Otwieranie

| Skąd | Jak |
|---|---|
| Wiersz ZGŁOSZENIA | `Enter` lub dwukrotne kliknięcie; **Otwórz rozmowę** w menu wiersza; paleta *Hosting: Otwórz rozmowę zgłoszenia...* (lista zgłoszeń na ekranie). **Otwórz w przeglądarce** w menu wiersza zostawia przeglądarkę. |
| Wiersz PULL REQUESTS w Azure DevOps lub Bitbucket | `Enter` lub **Otwórz rozmowę**. W GitHub, GitLab, Gitea, Forgejo i Gerrit wiersz otwiera [przegląd](review.md), którego strona Przegląd ogólny jest rozmową żądania. |
| Karta [tablicy zgłoszeń](#5b-tablica-zgłoszeń) | `Enter` lub dwukrotne kliknięcie. |
| [Radar](radar.md) | `i` na wierszu zgłoszenia lub żądania, **Otwórz rozmowę** w menu wiersza albo *Radar: Otwórz rozmowę zgłoszenia*. |
| Odwołanie w innej rozmowie | Kliknij je (albo `l`, potem wybierz): `#7`, `!12`, `ABC-12` powiązanego źródła otwierają się w aplikacji i trafiają na stos Wstecz. |
| Agent | `gitpoulpi ui show issue <n> --view conversation` (także `ui show pr`), zob. [opis CLI](cli-reference.md). |

**Repozytoria, które nie są sklonowane.** Wiersz Radaru z repozytorium, którego nie masz sklonowanego, też otwiera
swoją rozmowę: widok potrzebuje tylko profilu integracji i projektu, więc Radar wyświetla go nad własnym oknem,
a `Esc` oddaje Radar z powrotem. Pominięte jest to, co wymaga klonu: odwołanie do commita otwiera się na hoście
zamiast w Logu, a *Pozwól AI czytać repozytorium* jest wyszarzone z powodem (zob. [rozdział o AI](ai.md#pomoc-w-pisaniu-na-hoście)).
Żądanie sklonowanego repozytorium ma zamiast tego swój przegląd (`e`), a **Otwórz rozmowę** w Radarze mówi, że nie
ma tu dla niego rozmowy do odczytu; zgłoszenie sklonowanego repozytorium otwiera ją na karcie tego repozytorium.

Zgłoszenie hosta, który żadnych nie ma do odczytu (Bitbucket i Gerrit nie mają zgłoszeń), otwiera przeglądarkę jak
dawniej, a build bez funkcji `conversation` robi tak wszędzie ([własne buildy](custom-builds.md)).

### Widok

Widok na całe okno: nagłówek (stan, kto i kiedy to otworzył, etykiety, liczba komentarzy, **Pokaż zdarzenia**,
**Wstecz**, **Odśwież**, **Otwórz w przeglądarce**), linia **Publikowanie jako**, oś czasu, a pod nią pole komentarza.

- Oś czasu czyta się od najstarszego do najnowszego i zaczyna od najnowszego okna (najwyżej 300 wpisów). **Wczytaj
  starsze** na jej górze (`m`, paleta *Rozmowa: Wczytaj starsze wpisy*) czyta okno wcześniejsze, z liczbą wpisów,
  które jeszcze zostały, gdy host ją podaje.
- Komentarz pokazuje autora, datę, znacznik `edytowano`, znacznik `bot` dla botów, `wewnętrzny` dla notatek
  prywatnych oraz przyciski **Odpowiedz**, **Cytuj**, **Edytuj**, **Usuń** i **Linki**, na które pozwala zaznaczony
  wpis. Wątek w kodzie pokazuje `ścieżka:linia`, `rozwiązany` lub `nierozwiązany`, `nieaktualny` i swoje
  komentarze. Przegląd pokazuje swój werdykt.
- Zdarzenia to po jednej przygaszonej linii (*carol dodał(a) etykietę lfs*). **Pokaż zdarzenia** (`v`) je ukrywa;
  wybór jest pamiętany dla repozytorium. Hosty bez historii zdarzeń (YouTrack, elementy pracy Azure DevOps,
  notatki lokalne) nie mają takiego pola wyboru.
- Tekst jest renderowanym Markdownem: nagłówki, listy zagnieżdżone i z zadaniami, cytaty, alerty w stylu GitHub,
  podświetlony kod, tabele, linki, obrazy jako `[image: alt]`. Tekst, który przyszedł we własnych znacznikach hosta,
  jest najpierw konwertowany (zob. [Konwersja znaczników](#konwersja-znaczników)). Blok kodu `mermaid` dostaje
  przycisk **Otwórz diagram**, który pokazuje źródło diagramu w przewijanym podglądzie (`Enter` je kopiuje); paleta
  ma *Rozmowa: Otwórz diagram wpisu*. Wymaga to buildu z funkcjami `conversation` i `mermaid`.
- Widok czyta ponownie co 60 sekund i po `F5` (**Odśwież**).

**Klawisze.** Każdy ma przycisk, wiersz menu lub wiersz palety (*Rozmowa: ...*), a `Ctrl+N` numeruje przyciski.

| Klawisz | Akcja |
|---|---|
| `Up`, `Down`, `Ctrl+J`, `Ctrl+K`, `j`, `k` | Poprzedni / następny wpis |
| `PageUp`, `PageDown`, `Home`, `End`, kółko, pasek przewijania | Przewijanie |
| `Tab`, `Shift+Tab` | Oś czasu, pole komentarza, potem przyciski nagłówka |
| `r` | Odpowiedź: do wątku zaznaczonego wpisu albo z jego tekstem w cytacie, gdy host nie ma wątków |
| `Q` | Cytuje zaznaczony wpis w nowym komentarzu |
| `c` | Nowy komentarz |
| `e`, `d` | Edytuje, usuwa jeden z twoich komentarzy (przed usunięciem pada pytanie) |
| `l`, `Enter` | Linki wpisu: `Enter` idzie za jedynym albo otwiera ich menu |
| `y`, `o` | Kopiuje adres wpisu, otwiera go w przeglądarce |
| `n` | Następny nieprzeczytany wpis |
| `v`, `m`, `F5` | Pokaż lub ukryj zdarzenia, Wczytaj starsze, Odśwież |
| `Alt+Left`, `Backspace` | Wstecz po stosie Wstecz |
| `q`, `Esc` | Zamyka (pyta najpierw, gdy pole zawiera tekst) |

### Pisanie

**Nowy komentarz** (`c` albo `Tab` do pola), **Odpowiedz** (`r`), **Cytuj** (`Q`) i **Edytuj** (`e`) używają jednego
edytora Markdown: **Pisanie**, **Podgląd** i **Podział** (`Alt+P` przełącza Pisanie i Podgląd, `Alt+L` włącza Podział
i wymaga okna szerokiego na 100 kolumn). `Ctrl+Enter` (także `Ctrl+S` i `Alt+Enter`) lub **Wyślij** publikuje,
**Anuluj** odrzuca pole, a `Alt+E` otwiera **Pisz z AI**, gdy AI jest włączone ([rozdział o AI](ai.md#pomoc-w-pisaniu-na-hoście)).
Podgląd jest układany tym samym rendererem co oś czasu, z tymi samymi odwołaniami. Gdy host nie przyjmuje Markdown,
stopka podaje, w jakich znacznikach tekst jest wysyłany (*Wysłano jako Jira wiki markup*, *Wysłano jako Textile*).

- Komentarz jest dodawany do osi czasu od razu i zajmuje swoje miejsce, gdy host odpowie. Jeśli host odmówi, wpis
  znika, a pole otwiera się ponownie z twoim tekstem i powodem.
- Tekst, którego nie wysłano, jest przechowywany dla każdego elementu (`remote-conversations.toml` w folderze
  konfiguracji) i czeka przy następnym otwarciu tego pola; wysłanie lub odrzucenie go usuwa. Zamknięcie pola z
  tekstem najpierw pyta.
- **Odpowiedz** w wątku (wątek przeglądu, dyskusja GitLab, komentarz Linear, wątek Bitbucket lub Azure DevOps)
  publikuje do tego wątku. Gdy host ma tylko płaską listę, **Odpowiedz** zaczyna nowy komentarz z zaznaczonym
  wpisem w cytacie (`> **autor**`), tak samo jak **Cytuj**.
- **Notatka wewnętrzna** (pole wyboru, `Alt+I`): w GitLab i Redmine komentarz widzą tylko członkowie projektu.
  Jest pokazywana tylko na hostach, które mają takie notatki, i tylko przy nowym komentarzu.
- Publikowanie jest działaniem na zewnątrz: nic nie jest zapisywane w dzienniku i **z GitPoulpi nie da się niczego
  cofnąć**. Bez połączenia pole zostaje, **Wyślij** jest wyszarzone z powodem, a nagłówek mówi, że pokazano
  zapisaną kopię.

**Kto może edytować lub usuwać.** Tylko twoje własne komentarze mają **Edytuj** i **Usuń**. Pierwsze słowo ma
host, gdy je daje (`viewerCanUpdate` i `viewerCanDelete` w GitHub, `can_edit` gdzie indziej), potem twój login
porównany z autorem komentarza, bez rozróżniania wielkości liter; komentarze bota App muszą też pochodzić z konta
bota. Dopóki login bota GitHub App nie jest znany, liczą się tylko komentarze opublikowane przez tę instalację
(repozytorium zachowuje ich id): nic nie jest zgadywane z nazwy. W wątku **Edytuj** i **Usuń** działają na ostatnim
komentarzu wątku, który jest twój. Host, który nie umie edytować ani usuwać (zob. [tabelę](#czego-nie-potrafi-każdy-host)),
pokazuje przyciski wyszarzone z powodem.

### Publikowanie jako

Linia pod nagłówkiem mówi, jako kto są publikowane twoje wiadomości, a ta sama linia jest w każdym polu:

| Linia | Znaczenie |
|---|---|
| *Publikowanie jako ann (token osobisty)* | Token profilu: twój własny login (`GET /user`). |
| *Publikowanie jako ci-bot (polecenie tokenu repozytorium)* | Do API hostingu używane jest [polecenie tokenu](#13d-działanie-jako-bot-github-app) repozytorium. |
| *Publikowanie jako my-app[bot] (bot GitHub App)* | Token instalacji. Nazwa pochodzi z **Publikuj jako**, a w razie braku z hosta. |
| *Publikowanie jako bot GitHub App (jego nazwa nie jest jeszcze znana: ustaw „Publikuj jako” w Ustawieniach)* | Nie skonfigurowano nazwy i jeszcze żadnej nie poznano. |
| *Publikowanie jako ... (bot projektu)* | Token dostępu projektu lub grupy GitLab. |
| *Pisanie jako kai (notatka lokalna)* | Folder notatek Markdown: `local.me` profilu, a w razie braku podłączony użytkownik. |
| *Nieznane jeszcze, jako kto publikujesz. Ustaw „Publikuj jako” w Ustawieniach.* | Nic nie dało się ustalić. |

**Publikuj jako** to nazwa: dla GitHub App jest to `my-app[bot]`. Ustaw ją dla każdego repozytorium w **Ustawienia >
Integracje > Publikuj jako** (`general.hosting_post_as`) albo dla jednego repozytorium w **Ustawienia > Ustawienia
repozytorium > SSH i poświadczenia > Hosting**, co ją nadpisuje. Gdy jest pusta, GitPoulpi pyta GitHub (GraphQL
`viewer { login }`, na co token instalacji może, ale nie musi odpowiedzieć), a w razie braku odpowiedzi poznaje
nazwę z autora pierwszego komentarza, który opublikuje, i przechowuje ją z repozytorium. Token osobisty nigdy jej
nie potrzebuje. Zob. [Działanie jako bot GitHub App](#13d-działanie-jako-bot-github-app).

### Odwołania krzyżowe

Odwołania w rozmowie są podkreślone i klikalne. Najpierw dopasowywane są powiązane źródła ZGŁOSZEŃ i PULL REQUESTS
repozytorium, więc `ABC-12` powiązanego projektu Jira otwiera się w aplikacji nawet wewnątrz komentarza GitHub;
to, co dodaje własna gramatyka hosta, otwiera jego stronę po potwierdzeniu linku. Id commita otwiera się w Logu,
gdy repozytorium jest sklonowane, a id ma co najmniej 7 znaków, w przeciwnym razie na hoście. Nic wewnątrz
fragmentu kodu, bloku ogrodzonego, linku Markdown, adresu e-mail ani URL nie jest odwołaniem, podobnie jak `C#1`,
`UTF-8`, `SHA-256` i `ISO-8601`.

| Host | Rozpoznawane |
|---|---|
| GitHub | `#7`, `GH-7`, `owner/repo#7`, `owner/repo@sha`, `@login`, id commitów |
| GitLab | `#7`, `!7`, `group/project#7`, `group/project!7`, `project#7` (projekt siostrzany), `@user`, `~label` i `%milestone` jako chipy, id commitów |
| Gitea, Forgejo | `#7`, `!7`, `owner/repo#7`, `owner/repo@sha`, `@user`, id commitów |
| Azure DevOps | `#7`, `!7`, `AB#7`, `@user` |
| Bitbucket | `@user`, wzmianki `{uuid}`, id commitów |
| Gerrit | wartości `Change-Id` (`I` i 40 cyfr szesnastkowych), id commitów |
| Redmine | `#7`, rewizje `r123`, `commit:abc1234` |
| Jira, Linear, YouTrack | `ABC-12`, tylko dla projektu lub zespołu powiązanego źródła |
| Trello | adresy kart (`trello.com/c/...`) |
| Notatki lokalne | `[[note]]`, `[[note\|alias]]` i względne linki `.md` do innej notatki folderu |

`gitpoulpi hosting markdown --body T --id I` wypisuje, co aplikacja robi z tekstem, wraz z odwołaniami.

### Konwersja znaczników

Hosty, które nie mówią Markdownem, są konwertowane w obie strony, więc wszędzie czytasz i piszesz Markdown. To, czego
konwerter nie rozumie, pozostaje czytelnym tekstem.

- **Jira** (Cloud i Data Center) przechowuje komentarze jako wiki markup przez API v2. To, co czytasz, jest
  konwertowane na Markdown; to, co piszesz, jest konwertowane na wiki markup, a stopka mówi *Wysłano jako Jira wiki
  markup*.
- **Redmine** formatuje tekst jako Textile lub Markdown, czego API nie podaje: mówi o tym GitPoulpi wiersz profilu
  **Format tekstu serwera** (Ustawienia > Integracje, `markup`, domyślnie Textile). Textile jest konwertowany na
  Markdown do czytania i z powrotem na Textile do pisania.
- **Azure DevOps Server** zwraca komentarze elementów pracy jako HTML i przyjmuje je jako HTML: są pokazywane jako
  Markdown skonwertowany z niego i zapisywane jako HTML. Azure DevOps Services i wątki pull requestów przyjmują
  Markdown.
- **Gerrit** to zwykły tekst: nie jest renderowany jako Markdown, a to, co piszesz, jest wysyłane tak, jak wpisano.
- Każdy importer najpierw usuwa sekwencje sterujące i ucieczki, więc komentarz nie może poruszyć terminalem.

### Zapisana kopia, Wczytaj starsze, nieprzeczytane

- Każda otwarta rozmowa jest zapisywana (`<katalog stanu>/conversations/<integracja>/<projekt>/<rodzaj>-<n>.json`,
  tylko dla użytkownika, bez tokenu, najwyżej 2 MB na plik, zachowywanych jest 50 najnowszych plików). Maluje się od
  razu po otwarciu elementu i jest odświeżana w tle. Gdy host jest nieosiągalny, zapisana kopia zostaje na ekranie
  pod *Offline: pokazano kopię zapisaną 3 min temu (powód)*, a Wyślij jest wyszarzone.
- **Nieprzeczytane:** to, co widziałeś ostatnio, jest pamiętane dla każdej rozmowy (zachowywane jest 200 najnowszych).
  Wpisy innych osób, które są nowsze, dostają pasek akcentu i znacznik `nowe`, a `n` przeskakuje do następnego.
  Rozmowa otwierana po raz pierwszy nie ma nic nowego.

### Czego nie potrafi każdy host

Każde `nie` poniżej to prawdziwe ograniczenie: kontrolka jest ukryta lub wyszarzona z powodem, a CLI odpowiada
błędem z nazwą hosta, zanim cokolwiek zostanie wysłane.

| Host | Elementy | Odpowiedź w wątku | Edycja i usuwanie własnych | Rozwiązanie wątku | Zdarzenia | Notatki wewnętrzne | Zapisywane jako |
|---|---|---|---|---|---|---|---|
| GitHub | zgłoszenia, pull requesty | pull requesty | oba | pull requesty | tak | nie | GitHub Markdown |
| GitLab | zgłoszenia, merge requesty | tak (dyskusje) | oba | merge requesty | tak | tak | GitLab Markdown |
| Gitea | zgłoszenia, pull requesty | pull requesty | oba | pull requesty | tak | nie | Markdown |
| Forgejo | zgłoszenia, pull requesty | pull requesty | oba | nie (host nie ma endpointu) | tak | nie | Markdown |
| Azure DevOps | elementy pracy, pull requesty | pull requesty | oba | nie (status pokazany, niezmieniany) | tylko pull requesty | nie | Markdown (HTML w elementach pracy Server) |
| Bitbucket (Cloud, Data Center) | tylko pull requesty | tak | oba | nie (pokazane, niezmieniane) | tak | nie | Markdown |
| Gerrit | tylko zmiany | tak | nie (żadnego nie da się edytować ani usunąć) | tak | tak | nie | zwykły tekst |
| Jira | zgłoszenia | nie (płaska lista) | oba | nie | tak (changelog) | nie (wymaga roli w projekcie) | wiki markup |
| Linear | zgłoszenia | tak (odpowiedzi na komentarz) | oba | nie | tak | nie | Markdown |
| YouTrack | zgłoszenia | nie | oba | nie | nie | nie | Markdown |
| Redmine | zgłoszenia | nie | tylko edycja (Redmine 5 i nowszy), nigdy usuwanie | nie | tak (dzienniki) | tak (notatki prywatne) | Markdown lub Textile |
| Trello | karty | nie | oba | nie | tak (przeniesienia) | nie | Markdown |
| Notatki lokalne | notatki | nie | własne lokalnego autora | nie | nie | nie | Markdown |

Ponadto: żaden host nie pozwala zacząć z rozmowy nowego wątku przy linii kodu (rób to w przeglądzie), a reakcje są
pokazywane tylko w GitHub. Tekst dłuższy, niż przyjmuje host, jest odrzucany (GitHub 65 536 znaków, GitLab 1 000 000,
Azure DevOps 150 000, Jira 32 767, Trello 16 384): CLI sprawdza to najpierw, w aplikacji pokazywany jest własny
komunikat hosta, a twój tekst zostaje. Notatka lokalna trzyma swoje komentarze w końcowej sekcji `## Comments` samego
pliku notatki, po jednym wpisie `### autor - czas`, więc widzą je także Obsidian i powłoka.

W wierszu poleceń te same rozmowy to `gitpoulpi hosting issue comments`, `issue comment`, `comment-edit`
i `comment-delete` oraz `hosting review conversation` (zob. [opis CLI](cli-reference.md)); dla agentów są to
narzędzia MCP o tych samych nazwach (zob. [MCP i agenci](mcp-and-agents.md)).

## 6. Utwórz pull request

Otwórz formularz przez `+` w nagłówku PULL REQUESTS, `n` w sekcji albo
polecenie palety *Pull Requesty: Utwórz pull request...*. Menu gałęzi ma też
**Rozpocznij pull request z <gałąź>...**.

Formularz zawiera:

- Jedną kartę na każdy połączony profil, który ma pull requesty (`Left` /
  `Right` na pasku kart).
- Listy **Z repozytorium** / **Gałąź** i **Do repozytorium** / **Gałąź**.
  Długie listy można przeszukiwać. Można wpisać gałąź lub repozytorium, których
  listy nie pokazują.
- **Tytuł**, wypełniony tematem commita na końcu gałęzi.
- **Opis**, w wielu liniach (`Enter` dodaje nową linię, `Tab` opuszcza pole).
- Przyciski **Przypisani**, **Recenzenci**, **Etykiety** i **Kamień milowy** (GitHub,
  GitLab, Gitea i Forgejo), które otwierają selektor.
- **Wyślij jako szkic**.
- Przy włączonym [AI](ai.md): **Wygeneruj tytuł i opis**.

Formularz otwiera się na profilu, repozytorium i gałęziach bieżącej gałęzi
(`feature -> main`). Inne repozytorium źródłowe tworzy żądanie z forka.

**Utwórz pull request** (lub `Ctrl+S`) tworzy żądanie na hoście. Wynik otwiera
się w przeglądarce, a pasek stanu go nazywa.

## 7. Utwórz zgłoszenie

Gdy sekcja ZGŁOSZENIA ma dostawcę **i** projekt, jej nagłówek ma `+` (*Nowe
zgłoszenie*). Także: `n` w sekcji albo polecenie palety
*Hosting: Nowe zgłoszenie...*.

Formularz ma **Tytuł** (wymagany), **Opis** (wiele linii) i **Etykiety**
(po przecinku, na przykład `bug, ui`; w GitHub, GitLab, Gitea i Forgejo
przyciski Etykiety, Przypisani i Kamień milowy otwierają zamiast tego selektor). Jira i Azure DevOps
dodają listę **Typ**. Trello dodaje listę **Lista**. Linear i YouTrack
dodają listę **Status** (stan workflow albo wartość `State`, w jakim zgłoszenie
zaczyna). Redmine dodaje listę **Typ** z trackerami projektu (Bug, Feature,
Support...); nie ma etykiet, więc nie ma dla niego przycisku Etykiety, a etykieta
podana w wierszu poleceń jest odrzucana. Formularz wypełnia te
listy z hosta przy otwarciu.

**Utwórz zgłoszenie** (lub `Ctrl+S`) tworzy je na hoście. `Esc` lub **Anuluj**
zamyka formularz. Jeśli host odmówi (odrzucony token, brak zakresu, nieznany
projekt, nieznana etykieta), formularz pokazuje powód i zachowuje wpisane
dane. Po sukcesie pasek stanu nazywa nowe zgłoszenie, a sekcja ZGŁOSZENIA jest
odświeżana.

CLI: `gitpoulpi hosting issue create --title ... [--body ... | --body-file ...]
[--label ...] [--type ...] [--list ...]`.

## 8. Utwórz na własnej stronie WWW hosta

Formularz nowego zgłoszenia i formularz tworzenia pull requesta mają przycisk
**Utwórz na <dostawca>** w prawym górnym rogu. Klawisz to `Alt+W`. Otwiera on
w przeglądarce formularz WWW hosta, wypełniony tym, co dotąd wpisano. Okno
pozostaje otwarte. Nie potrzebuje tokenu i nie wywołuje API, więc działa też
po błędzie API.

- GitHub nie ma parametru URL dla *szkicu*, więc pole szkicu nie jest
  przenoszone. GitLab dostaje prefiks tytułu `Draft: `.
- GitLab przyjmuje etykiety tylko po id, więc są dodawane jako szybka akcja
  `/label ~"x"` na końcu opisu.
- Bardzo długi opis jest ucinany, aby URL zmieścił się w 6000 znaków, z
  adnotacją `[...truncated: paste the rest]`.
- Bitbucket Data Center nie ma adresu formularza WWW, więc nie ma przycisku
  **Utwórz na <dostawca>**.

CLI: `gitpoulpi hosting issue create ... --web` i `gitpoulpi hosting pr --web`
wypisują ten sam URL.

## 9. Znacznik na pasku narzędzi

Na końcu paska narzędzi (domyślnie po prawej) znacznik pokazuje żądanie bieżącej gałęzi:
`GH #12` (GitHub), `GL !53` (GitLab), `AZ !12` (Azure DevOps) lub `BB #12`
(Bitbucket). Jego kolor pokazuje stan: zielony otwarty, szary szkic, fioletowy
scalony, czerwony zamknięty.

- Najedź na niego, by zobaczyć tytuł, stan i `source → target`.
- Kliknij go lub naciśnij `Ctrl+G g`, aby otworzyć żądanie w przeglądarce.
- Kliknij go prawym przyciskiem dla **Otwórz w przeglądarce**, **Kopiuj URL**,
  **Odśwież** i **Zmień profil**.

Wypchnięta gałąź bez żądania pokazuje przyciemniony znacznik `+ PR`. Kliknij go,
aby rozpocząć pull request. Znacznika nie ma, gdy żaden profil nie pasuje, gdy
gałąź nie jest wypchnięta albo gdy host był nieosiągalny. W wąskim oknie
znacznik jest ukrywany, aby zostawić miejsce na przyciski paska narzędzi.

## 10. Kiedy listy się odświeżają

Listy odświeżają się:

- gdy otwierasz repozytorium;
- gdy zmienia się przełączona gałąź;
- po push lub fetch;
- co *Odświeżanie: pull requesty i zgłoszenia (minuty)*
  (`general.hosting_refresh_minutes`, domyślnie 5, `0` wyłącza licznik;
  Ustawienia > Integracje);
- gdy zamykasz Ustawienia;
- gdy naciśniesz `r`, uruchomisz *Integracje: Odśwież pull requesty i zgłoszenia*
  albo zmienisz filtr.

Żądania nigdy nie blokują okna.

Hosty ograniczają liczbę wywołań. GitPoulpi odsyła `ETag` ostatniej odpowiedzi,
więc niezmieniona lista kosztuje niewiele (nic w GitHubie). Gdy host zgłasza, że
zostało niewiele wywołań, mówi o tym pasek stanu. Gdy nie zostało żadnego,
GitPoulpi przestaje pytać ten host do czasu zresetowania limitu, a listy
zostają bez zmian.

## 11. Terminologia

GitHub, Azure DevOps i Bitbucket mówią „pull request”. GitLab mówi „merge
request”. *Terminologia żądań przeglądu* (`ui.pr_terminology`, w
Ustawienia > Integracje) wybiera słowo dla każdej etykiety, menu, polecenia
palety i komunikatu:

- **Auto (wg hosta)**: idzie za powiązanym dostawcą repozytorium („Pull
  Request”, dopóki nic nie jest powiązane).
- **Pull Request** lub **Merge Request**: zawsze to słowo.

Zmiana działa od razu. CLI i MCP zawsze mówią „pull/merge request”. Ta strona
mówi „pull request”.

## 12. Odwołania w opisach commitów i nazwach gałęzi

Gdy zgłoszenia lub pull requesty repozytorium są powiązane, odwołania w opisie
commita w szczegółach commita są podkreślone. Kliknięcie otwiera powiązaną
stronę. Klawiatura: polecenie palety
*Integracje: Otwórz odwołanie z commita (#zgłoszenie / !żądanie)...* (menu, gdy
opis ma kilka odwołań). Repozytorium bez powiązań nie ma odnośników.

Co jest odwołaniem, zależy od dostawcy sekcji ZGŁOSZENIA:

| ZGŁOSZENIA powiązane z | W opisie | W nazwie gałęzi |
|---|---|---|
| GitHub, GitLab, Gitea, Forgejo | `#123` | `123-fix-login` |
| Azure DevOps | `#123` i `AB#123` | `123-fix-login` |
| Jira | `ABC-123`, tylko gdy `ABC` jest kluczem powiązanego projektu. Tekst taki jak `UTF-8`, `SHA-256` lub `ISO-8601` nigdy nie jest odnośnikiem. | `ABC-123` w dowolnym miejscu nazwy (`feature/ABC-123-x`), wielkimi literami |
| Linear | `ENG-123`, tylko gdy `ENG` jest kluczem powiązanego zespołu (albo zespołu z listy zespołów profilu). `UTF-8` nigdy nie jest odnośnikiem. | własne `branchName` Linear, gdy tworzysz gałąź ze zgłoszenia; `ENG-123` w dowolnym miejscu nazwy jest odczytywane z powrotem |
| YouTrack | `ABC-123`, tylko gdy `ABC` jest krótką nazwą powiązanego projektu (albo projektu z listy projektów profilu) | `ABC-123` w dowolnym miejscu nazwy |
| Redmine | `#123`, tylko gdy Redmine jest powiązanym źródłem ZGŁOSZENIA (źródło pull requestów zachowuje własny znak) | `123-fix-login` |
| Trello | adres `https://trello.com/c/<shortLink>`; `#12` tylko gdy buforowana karta ma numer 12 | `12-fix-login` |

| PULL REQUESTS powiązane z | W opisie |
|---|---|
| GitLab, Azure DevOps | `!45` |
| GitHub, Bitbucket, Gitea, Forgejo | `#45` |

Gdy pasują obie listy `#N`, wygrywa zgłoszenie. CLI:
`gitpoulpi hosting refs --message TEXT | --commit REV | --branch NAME`.

**W rozmowach.** Tekst zgłoszenia lub żądania jest czytany z gramatyką jego hosta, szerszą niż tabela powyżej:
`owner/repo#7`, `!7`, `GH-7`, `@login`, id commitów i inne formy wymienione w
[Odwołaniach krzyżowych](#odwołania-krzyżowe). Najpierw dopasowywane są powiązane źródła repozytorium, więc klucz Jira
w komentarzu GitHub nadal otwiera swoje zgłoszenie w aplikacji. `gitpoulpi hosting markdown --body TEXT --id I`
pokazuje, co aplikacja znajduje w tekście.

### Wstaw odwołanie do zgłoszenia

W kompozytorze commita przycisk `#▾` na prawym końcu Podsumowania otwiera
filtrowaną listę powiązanych zgłoszeń. Zgłoszenie wskazane przez bieżącą
gałąź jest pierwsze, oznaczone *(z gałęzi)*. `Enter` lub kliknięcie wstawia
jego odwołanie (`ABC-123`, `#12`, `AB#12`) w miejscu kursora. Klawiatura:
`Alt+R` w kompozytorze. Paleta: *Commit: Wstaw odwołanie do zgłoszenia...*.
Bez powiązanej listy zgłoszeń przycisk jest wyszarzony i mówi *najpierw
powiąż listę zgłoszeń na pasku bocznym*. GitPoulpi nigdy nie dodaje odwołania
samo.

## 13. Klucze SSH

### Klucz SSH na profil

Połączony profil ma sekcję **Klucz SSH** pod swoją kartą. Bez klucza wybierz
jedno z:

- **Klucz domyślny**: twój ssh-agent lub `~/.ssh`. Nic się nie zmienia.
- **Wygeneruj nowy klucz**: opcjonalny tytuł (domyślnie
  `GitPoulpi <host> <date>`). Przy zaznaczonym **Dodaj klucz publiczny do
  <host>** GitPoulpi go wysyła, jeśli host jeszcze go nie ma. Klucz jest
  zapisywany bez hasła w `~/.ssh/gitpoulpi_<profile id>_ed25519` i `.pub`.
  Nie jest potrzebne `ssh-keygen`. Istniejący plik nigdy nie jest nadpisywany.
- **Własny klucz**: wpisz ścieżkę lub użyj **Przeglądaj**, aby wskazać plik
  klucza prywatnego, potem naciśnij **Użyj tego klucza**.

Gdy klucz jest dołączony, sekcja pokazuje `title (SHA256:...)` z:

- **Usuń**: odłącza klucz i usuwa go na hoście, jeśli dodał go GitPoulpi.
- **Usuń pliki**: to samo, a dodatkowo usuwa pliki klucza, po potwierdzeniu
  wymieniającym obie ścieżki.

### Klucz SSH dla jednego repozytorium

Aby jedno repozytorium używało określonego klucza, otwórz menu karty (`Ctrl+T`
lub prawy przycisk na karcie) i wybierz **SSH i poświadczenia...** albo uruchom
polecenie palety *Repozytorium: SSH i poświadczenia...*, a pod SSH wybierz
**Plik klucza** (lub **Klucz konta** dla klucza profilu integracji). Wybór jest
przechowywany w prywatnym folderze GitPoulpi: nic nie jest zapisywane w
repozytorium ani w konfiguracji gita. Zob.
[Ustawienia](settings.md#ustawienia-repozytorium-ssh-i-poświadczenia).

Repozytorium, które nadal ma własne `core.sshCommand` z dawnego przypięcia „Klucz
SSH dla tego repozytorium”, pokazuje je na tej stronie, tylko do odczytu, z
przyciskiem **Przenieś do GitPoulpi**.

CLI i MCP: `gitpoulpi repo-auth` (wybór dla repozytorium) oraz, dla samego
przypięcia `core.sshCommand`, `gitpoulpi ssh-command show | set --key <file> |
clear`.

Klucz globalny z Ustawienia > SSH i poświadczenia (*Ścieżka klucza
prywatnego* lub *Generuj parę kluczy*) obowiązuje każde repozytorium bez
własnego wyboru. Plik klucza lub przypięcie repozytorium wygrywa z nim.

## 13b. Ochrona gałęzi przy przepisywaniu historii

Gdy przepisujesz historię ([warsztat](history-rewrite.md)), GitPoulpi pyta hosta, które gałęzie chroni, aby
domyślnie pominąć je w przepisywaniu. Pyta tylko o repozytorium **powiązane** (sekcja pull requestów lub
zgłoszeń w panelu bocznym albo przypięty profil), tokenem tego profilu, a token, który nie może tego odczytać,
daje po prostu „nieznane”: nic się nie psuje. Co odpowiada każdy dostawca i czego potrzebuje token:

| Host | Co jest czytane | Czego potrzebuje token |
|---|---|---|
| GitHub (i Enterprise Server) | flaga `protected` gałęzi oraz aktywne rulesety gałęzi (wzorce `~DEFAULT_BRANCH`, `~ALL` i `refs/heads/...`) | klasyczny token z `repo` (`public_repo` dla publicznego) albo token szczegółowy z Metadata (read). Ruleset, którego token nie może odczytać, jest zgłaszany, a reguły klasyczne nadal się liczą |
| GitLab (gitlab.com i własny serwer) | `protected_branches` (nazwy i wzorce z `*`) | `read_api` oraz rola, która widzi ustawienia repozytorium |
| Azure DevOps (Services i Server) | włączone, blokujące polityki gałęzi repozytorium (zakresy `Exact`, `Prefix` i `DefaultBranch`) | Code (read). Uprawnienie „force push” to ustawienie zabezpieczeń i nie jest czytane |
| Bitbucket Cloud | ograniczenia gałęzi, które zatrzymują push (`push`, `force`, `delete`) | `repository:admin`. Ograniczenie z modelu gałęzi nie jest rozwiązywane |
| Bitbucket Data Center | uprawnienia gałęzi, które zatrzymują force push (`fast-forward-only`, `pull-request-only`, `read-only`) | administrator repozytorium |

Strona ustawień ochrony (oraz gałęzi domyślnej i usuwania repozytorium) jest budowana z adresu zdalnego
repozytorium dla GitHub, GitLab, Azure DevOps i Bitbucket, nawet gdy repozytorium nie jest powiązane,
i pokazywana z trzema drogami wyjścia, gdy host odrzuci push.

## 13c. Listy obejmujące całe konto w Radarze

Radar zwykle obserwuje sklonowane przez ciebie repozytoria. Profil może też
wypisać to, co cię dotyczy na całym hoście, sklonowane lub nie, trzema przełącznikami w
swojej tabeli `[integrations.radar]` (wszystkie domyślnie wyłączone; zob.
[opis konfiguracji](config-reference.md#integrationsradar-listy-obejmujące-całe-konto)):

- **żądania do przeglądu**: otwarte żądania, które proszą o twój przegląd;
- **otwarte żądania**: każde otwarte żądanie twojej własnej przestrzeni nazw i każdej organizacji lub
  grupy, do której należysz, albo przestrzeni nazw, które podasz. Nigdy cały host;
  **Przestrzenie nazw** profilu (po jednej w wierszu w Ustawienia > Integracje > Radar, opcjonalnie
  jako wyrażenia regularne) ograniczają wszystkie trzy listy: wiersz pasuje do pełnej ścieżki
  repozytorium, gdy jest jej równy albo jest jej prefiksem kończącym się na `/` (`ptm` zachowuje
  `ptm/produits/x`, nie `ptmx/y`), i jest stosowany do wierszy zwracanych przez host, a także po
  stronie serwera, gdy wyszukiwanie hosta na to pozwala (kwalifikatory `user:` / `repo:` GitHuba);
- **przypisane zgłoszenia**: otwarte zgłoszenia przypisane do ciebie.

Odpowiadają GitHub, GitLab (także własny serwer), Gitea, Forgejo, Azure DevOps (Services
i Server), Bitbucket Data Center i Gerrit. Jak pyta się każdy z nich:

| Host | Żądania | Zgłoszenia |
|---|---|---|
| Azure DevOps | aktywne pull requesty całej organizacji (`searchCriteria.reviewerId` dla tych czekających na ciebie; żądanie, na które już zagłosowałeś, jest pomijane); gdy serwer odrzuci trasę na poziomie organizacji, pytane są po kolei pierwsze 20 projektów. `namespaces` to nazwy projektów | elementy pracy przypisane do ciebie, które nie są Done, Closed ani Removed |
| Bitbucket Data Center | twój pulpit: rola `REVIEWER` jeszcze bez akceptacji dla żądań do przeglądu, `AUTHOR` i `PARTICIPANT` dla otwartych żądań; `namespaces` to klucze projektów | brak (Data Center nie ma trackera zgłoszeń; mówi o tym notka) |
| Gerrit | `is:open reviewer:self -owner:self -is:ignored`; otwarte żądania to `is:open projects:<prefix>` dla każdej przestrzeni nazw (prefiks ścieżki projektu), albo twoje własne i przeglądane zmiany, gdy żadnej nie podano; zmiana work-in-progress to szkic | brak |

Bitbucket Cloud nie ma wierszy obejmujących całe konto: nie oferuje punktu końcowego, który wypisuje
żądania z wielu repozytoriów, a pytanie każdego repozytorium z osobna oznaczałoby zbyt wiele
wywołań. Jego sklonowane repozytoria zachowują własne listy, a `gitpoulpi hosting
radar --id <bitbucket>` tak mówi. Gdy Gerrit ma więcej zmian, niż wynosi limit, który
zwraca, lista jest oznaczona jako obcięta, z notką.

Szkice są pomijane, chyba że je zachowasz. Każda lista jest ograniczona: 300 wierszy na rodzaj, z
notką mówiącą, ile było więcej, a w GitHubie kilka wywołań wyszukiwania na
rundę, ponieważ jego API wyszukiwania dopuszcza tylko 30 na minutę. Przestrzeń nazw, której token
nie może odczytać, staje się notką, a pozostałe nadal się wczytują; odrzucony token lub
limit zapytań kończy błędem tylko ten profil. Odpowiedzi są używane ponownie przez pięć minut
(`radar-account.json` w folderze konfiguracji).

W Ustawienia > Integracje połączony profil hosta kodu ma blok **Radar**
pod swoim kluczem SSH: jedno pole wyboru na listę (*Żądania czekające na mój przegląd*,
*Każde otwarte żądanie*, *Uwzględnij szkice*, *Zgłoszenia przypisane do mnie*, ostatnie
nieobecne dla hostów bez zgłoszeń), pole *Przestrzenie nazw* (jeden wpis na wiersz: nazwa pasująca do tej grupy lub właściciela i wszystkiego pod nią,
albo wyrażenie regularne w wierszu przy zaznaczonym *Wyrażenia regularne*; puste oznacza twoją własną
i każdą, do której należysz) oraz wybór *Żądanie z
repozytorium, które nie jest sklonowane*: *Zapytaj*, *Sklonuj* lub *Przejrzyj przez API*.
Wszystko jest zapisywane w trakcie zmiany. Hosty bez zapytania obejmującego całe konto (Bitbucket
Cloud) pokazują zamiast tego jedną linię z tą informacją. Paleta poleceń dociera do bloku
przez *Ustawienia: Opcje Radaru profilu...*, a wyszukiwarka Ustawień znajduje każdą
kontrolkę po jej etykiecie.

Z wiersza poleceń: `gitpoulpi hosting integrations radar <id> --review-requests on`
włącza listę, `gitpoulpi hosting radar --json` wypisuje listy (a
`--refresh` pomija pamięć podręczną). Agenci AI mają te same dwa polecenia przez MCP
(`hosting_radar`, `hosting_integrations_radar`).

## 13d. Działanie jako bot GitHub App

Projekt, który nie może pokazywać czyjegoś imienia, może robić wszystko na hoście przez **GitHub App**: każdy
komentarz, przegląd, zamknięcie lub ponowne otwarcie i zmiana etykiety pojawia się wtedy jako
`<app-slug>[bot]`. Działa to dla jednego repozytorium naraz i zostawia twoje pozostałe repozytoria na twoim
własnym tokenie.

1. Utwórz GitHub App dla organizacji z uprawnieniami **Issues** i **Pull requests** ustawionymi na *Read & write*
   (oraz *Contents: read*, aby można było wyświetlać i przeglądać pull requesty) i zainstaluj ją w repozytorium.
2. Napisz mały skrypt, który wypisuje **token instalacji** na swoim standardowym wyjściu (klucz prywatny App
   podpisuje krótkotrwały JWT, który jest wymieniany na token ważny przez godzinę). GitPoulpi nigdy nie widzi
   klucza.
3. W **Ustawienia > Ustawienia repozytorium > SSH i poświadczenia** wybierz **Polecenie tokenu** w HTTPS, wpisz
   skrypt jako **Polecenie** i host (`github.com`) w **Hosty**, i zaznacz **Użyj do API hostingu (zgłoszenia,
   pull requesty)**. Zapisz i potwierdź polecenie (człowiek zawsze
   je potwierdza, wraz z jego SHA-256). Z wiersza poleceń: `gitpoulpi repo-auth set --https token-command --hosts
   github.com --use-for-hosting --yes --command ./app-token.sh`.

Co się zmienia dla tego repozytorium:

- Jego wywołania zgłoszeń i pull requestów (wyświetlanie, tworzenie, komentowanie, przeglądanie, merge, zamykanie,
  etykiety) używają tokenu polecenia, z tym samym 30-sekundowym limitem, pamięcią podręczną w pamięci i redakcją co
  własne logowanie gita. Gdy host odpowiada 401, bo token wygasł, polecenie jest uruchamiane ponownie jeden raz.
  Jeśli polecenia nie da się uruchomić, wywołanie jest odrzucane: twój osobisty token nigdy nie jest rozwiązaniem
  awaryjnym.
- Nie ma „mnie”: token instalacji nie może zapytać, kim jest. Nagłówek sekcji nazywa profil z **(bot aplikacji)**,
  a filtry zbudowane na „mnie” (*Przypisane do mnie*, *Moje*, *Czekają na mój przegląd*) nie mają czego dopasować. Nic
  nie jest pokazywane jako błąd.
- **Publikuj jako.** Każda wiadomość, którą GitPoulpi publikuje, jest pokazywana z nazwą, jako którą jest
  publikowana (*Publikowanie jako my-app[bot] (bot GitHub App)* pod nagłówkiem [rozmowy](#publikowanie-jako) i w jej
  polu komentarza; `posted_as` w odpowiedzi CLI). Z tokenem osobistym jest to twój login (`GET /user`). Z tokenem
  App jest to nazwa **Publikuj jako**, na przykład `my-app[bot]`: ustaw ją dla każdego repozytorium w
  **Ustawienia > Integracje** (`general.hosting_post_as`, także
  `gitpoulpi config set general.hosting_post_as my-app[bot]`) albo dla jednego repozytorium w **Ustawienia >
  Ustawienia repozytorium > SSH i poświadczenia > Hosting**, co wygrywa. Gdy jest pusta, GitPoulpi pyta GitHub
  (GraphQL `viewer { login }`), a w razie braku odpowiedzi poznaje nazwę z autora pierwszego komentarza, który
  opublikuje, i przechowuje ją z repozytorium. Ta nazwa służy też do rozpoznawania twoich własnych komentarzy,
  więc edycja i usuwanie są oferowane tylko na nich (komentarze App muszą też pochodzić z konta bota); dopóki nie
  jest znana, można zmieniać tylko komentarze opublikowane przez tę instalację.
- App nie można przypisać. Połącz tę konfigurację z blokadą przypisywania opisaną niżej.
- Włączenie lub wyłączenie **Użyj do API hostingu** wymaga człowieka, tak jak nowe polecenie.

Tylko profile GitHub używają tokenu repozytorium w ten sposób; inne hosty zachowują własny token profilu. (Token
dostępu projektu lub grupy GitLab to także bot: jest tokenem profilu, pokazywanym jako *bot projektu*, i nic nie
trzeba ustawiać.)

### Blokowanie przypisywania w repozytorium

**Ustawienia > Ustawienia repozytorium > SSH i poświadczenia** ma wiersz **Przypisywanie** (**Dozwolone** domyślnie
albo **Zablokowane**; zapisywane od razu). Gdy jest zablokowane, przypisywanie w zgłoszeniach i pull requestach tego
repozytorium jest niemożliwe, kto by o nie nie prosił:

- wybór przypisanych jest pominięty w formularzach nowego zgłoszenia i nowego pull requesta (z notatką, dlaczego),
  polecenie palety *Hosting: Edytuj przypisanych...* jest wyszarzone z powodem, a menu wiersza mówi o tym zamiast je
  otwierać;
- każdy klient odrzuca przypisanie, zanim cokolwiek zostanie wysłane, także przypisanych podanych przy tworzeniu
  zgłoszenia lub pull requesta: CLI odpowiada `{"refused": {"reason": "assignment_blocked", ...}}` z kodem wyjścia
  2, a agent przez MCP dostaje to samo odrzucenie;
- istniejący przypisani pozostają widoczni (tylko do odczytu).

`gitpoulpi hosting assignment` pokazuje ustawienie, `--block` je włącza, a `--allow` wyłącza z powrotem; ponownie
zezwolić może tylko człowiek (serwer MCP nie może).

## 14. Z wiersza poleceń

```
echo $TOKEN | gitpoulpi hosting login --id my-gitlab --token-stdin
gitpoulpi hosting integrations list --json
gitpoulpi hosting prs --mine --json
gitpoulpi hosting edit-url --branch my-branch --open
gitpoulpi hosting open pr 12 --browser
gitpoulpi hosting pr-edit 12 --add-reviewer ann --add-label bug --milestone v1.0
gitpoulpi hosting radar --what review --json
gitpoulpi hosting assignment --block
```

Wszystkie polecenia opisuje [Opis CLI](cli-reference.md#polecenia). Token nigdy nie
jest przyjmowany jako argument, a serwer MCP nie oferuje
`hosting login`, więc token nigdy nie przechodzi przez agenta. `hosting login`
wypisuje `stored: "os"`, gdy token trzyma systemowy magazyn poświadczeń, albo
`stored: "session"` i `store_error`, gdy magazyn odmówił, a token żyje w pamięci
tylko na czas tej sesji.

## Rozwiązywanie problemów

| Objaw | Rozwiązanie |
|---|---|
| *Połącz* kończy się błędem uwierzytelniania | Token jest zły, wygasł lub został unieważniony. Utwórz nowy. |
| Zakres jest wymieniony na czerwono | Token go nie ma. Połączenie działa, ale ta funkcja nie. Dodaj zakres na hoście, potem **Edytuj** i wklej nowy token. |
| Panel boczny nie ma sekcji PULL REQUESTS | Żaden włączony profil nie jest połączony. Połącz jeden, potem powiąż repozytorium na liście sekcji. |
| Wiersz *Połącz ponownie* na liście | Token zginął (restart bez menedżera poświadczeń) albo wygasł. Kliknij wiersz, aby otworzyć profil. |
| Instancja self-managed zawodzi | Sprawdź, czy **Domena hosta** nie ma ścieżki i czy twój komputer ufa certyfikatowi instancji. |

## Znane ograniczenia

- **Pull requesty z forków:** Azure DevOps i Bitbucket nie mogą zrobić checkout
  żądania z forka (zob. [Akcje na wierszu](#akcje-na-wierszu)). Azure DevOps nie
  może utworzyć pull requesta z forka.
- **Trello** pokazuje najwyżej pierwsze 1000 kart tablicy. Użyj filtra listy,
  aby zawęzić. **Jira**, **Linear** i **YouTrack** wypisują najwyżej 500
  zgłoszeń na odświeżenie. Użyj fragmentu JQL lub zapytania YouTrack, aby zawęzić.
- **Linear** przyjmuje jedną osobę przypisaną na zgłoszenie: jeśli w selektorze wybierzesz kilka,
  wysyłana jest pierwsza. Jego „kamień milowy” to **cykl** zespołu (kamienie milowe
  projektu nie są używane). W **YouTrack** to pole `Fix versions`; wersja nie ma
  numeru, więc selektor numeruje niewydane według ich pozycji w projekcie.
  Żaden z tych trackerów nie ma pull requestów.
- **Redmine** przyjmuje jedną osobę przypisaną na zgłoszenie (pierwszą z selektora), nie ma etykiet
  i wypisuje najwyżej 500 zgłoszeń na odświeżenie. Przypisani to członkowie projektu
  (osoby i grupy), rozróżniani po nazwie wyświetlanej. Kolumny tablicy Kanban
  to statusy zgłoszeń, a przeniesienie karty zmienia status; obowiązują domyślne
  reguły workflow Redmine (rola, która nie może wykonać tego przejścia, dostaje
  odmowę, a karta wraca). „Resolved” nie jest statusem zamkniętym w domyślnych
  danych Redmine: kolumna Zamknięte to statusy „Closed” i „Rejected”.
- **Bitbucket Data Center** nie ma odnośników w przeglądarce dla nowych pull
  requestów, gałęzi ani commitów. Strony pull requestów otwierają się normalnie.
- **Azure DevOps** nie pokazuje awatarów autorów. Filtry „moje” i „czeka na
  przegląd” potrzebują punktu końcowego, który może odrzucać osobiste tokeny
  dostępu. W takim przypadku profil łączy się bez nazwy użytkownika, a te dwa
  filtry są niedostępne. Nieprawidłowy token dostaje stronę logowania zamiast
  błędu; GitPoulpi zgłasza to jako odrzucony token.
- **Gitea i Forgejo:** szkic to żądanie, którego tytuł zaczyna się od `WIP:`
  (serwer nie ma flagi szkicu przy tworzeniu). Żądanie wypisuje jako recenzentów
  każdego, kto już je przejrzał, a wycofanie prośby o przegląd usuwa
  tylko oczekujące. Pull requesty z lokalnej gałęzi forka robią
  checkout przez `refs/pull/<n>/head`, jak GitHub. Wzmianki są wyszukiwane
  filtrem serwera `mentioned_by`. Serwer nie pokazuje zakresów tokenu, więc
  brakujący zakres wychodzi na jaw dopiero, gdy wywołanie zostanie odrzucone.
- **Tablica zgłoszeń** przenosi karty w każdym źródle zgłoszeń. W Jira przeniesienie wykonuje przejście workflow prowadzące do kolumny;
  przejście, które wymaga pól (na przykład rozwiązania), jest odrzucane przez Jirę, a karta wraca z jej komunikatem. W Azure DevOps
  work item typu bez stanu docelowego wraca wraz z powodem.
- **Bitbucket:** hasła aplikacji już nie istnieją; użyj tokenu API.
  Repozytoria są wyliczane workspace po workspace.
- **Jira Cloud:** działają tylko klasyczne tokeny API (bez tokenów z zakresami,
  bez OAuth). Nowe zgłoszenie Jira dostaje opis jako zwykły tekst.
- **Rozmowy:** najpierw ładuje się najnowszych 300 wpisów (**Wczytaj starsze** dla reszty); komentarz, edycja lub
  usunięcie opublikowane z GitPoulpi to działanie na zewnątrz i nie można ich z niego cofnąć; to, czego nie potrafi
  każdy host, jest w [tabeli](#czego-nie-potrafi-każdy-host). W Gerrit wiadomości ani komentarza nie można edytować
  ani usunąć, a tekst Gerrit jest zwykły.
- Utworzenie pull requesta, elementu pracy, zgłoszenia lub karty odbywa się na
  hoście i **nie można tego cofnąć** z GitPoulpi. Checkout żądania można cofnąć
  normalnie.
