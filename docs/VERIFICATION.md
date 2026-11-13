# Local verification

## Select doppio e distacco posteriore live — 13 novembre 2026

Due pressioni entro 400 ms di View/Back Xbox oppure Share/Create PlayStation
alternano entrambi gli HUD. Testati fronti di pressione, rilascio obbligatorio,
limite 400/401 ms, pressione mantenuta, sequenze successive, riconnessione,
cambio dispositivo/sorgente e batch HID con tutte le transizioni. I report
DualSense semplici/USB/Bluetooth e DualShock 4 USB/Bluetooth distinguono
Share/Create da Options, stick e touchpad; XInput usa soltanto Back/View.

Il distacco legge l'intervallo anteriore della vettura nella posizione successiva
di classifica dallo stesso Lap Data. Fixture indipendenti coprono slot locali
0/23, indici avversari non contigui, minuti e millisecondi, zero, giro invalido,
ultima posizione, inseguitore inattivo, dati malformati, riordino, pausa,
scadenza a 500 ms, cambio avversario, sessione, sorgente e flashback. Il test
UDP reale verifica contemporaneamente cronologia e distacco da 2536 ms.

Passati **82 test** (45 unit, 5 game data, 29 protocollo, 3 UDP), rustfmt,
Clippy `--all-targets -D warnings`, build Release e build degli esempi.
Tre esecuzioni native isolate di 5 secondi, con UDP sintetico e una
ricreazione del device ciascuna, hanno prodotto zero errori di presentazione:

- [Wide, intervallo aggiornato da 1234 a 2345 ms](../artifacts/select-gap-wide-smoke.json):
  274 frame, telemetria LIVE e distacco finale 2345 ms.
- [Classic, inseguitore diventato inattivo](../artifacts/select-gap-classic-smoke.json):
  280 frame, telemetria LIVE e distacco finale assente.
- [Modern, solo Lap Data interrotto](../artifacts/select-gap-modern-smoke.json):
  201 frame, telemetria LIVE e distacco finale scaduto.

Entrambe le finestre conservano visibilità, topmost, click-through e mancata
attivazione. Configurazioni e porta sono separate dalle preferenze utente.
Non sono stati verificati una sessione reale di F1 25 o gesti su controller fisici.

## Straight Mode più compatto — 11 novembre 2026

SM OFF, SM ON, SM READY e countdown metri usano un carattere dedicato da
**19 px**, rispetto ai precedenti 22 px. Le quote delle righe restano condivise
con FUEL e delta. Passati **77 test**, rustfmt, Clippy e build Release.
Controllati con render nativo i tre stati e countdown da 700 e 65535 metri
su Wide, Classic e Modern: le scritte restano nel contorno.
[Anteprima countdown](../artifacts/hud-sm-countdown-preview.png).
Non è stata eseguita una sessione nel gioco.

## Due righe curve comuni per SM, delta e FUEL — 11 novembre 2026

FUEL usa ora la stessa curva di SM; il valore carburante usa quella del delta.
Ogni modello definisce una sola coppia di quote, condivisa dai due gruppi.
FUEL e valore seguono la curva glifo per glifo, con le dimensioni precedenti;
le quattro curve sono precalcolate alla creazione delle risorse grafiche.

Passati **77 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release.
Il nuovo test verifica che i centri delle scritte SM e FUEL appartengano agli
stessi due archi Modern. Il render nativo Direct2D è stato controllato con dati
assenti, +2.3, -0.8 e +12.3 nei tre modelli.
[Anteprima senza dati](../artifacts/fuel-rows-modern-waiting.png),
[anteprima con dati simulati](../artifacts/fuel-rows-modern-positive.png).
Non è stata eseguita una sessione nel gioco.

## Allineamento FUEL al bordo Halo — 11 novembre 2026

FUEL e valore carburante condividono la tangente del bordo inferiore nel punto
occupato dal riquadro. Classic ricava l'angolo dalla stessa Bézier usata per
disegnare il bordo; Modern riusa la tangente del proprio arco. Wide mantiene
la tangente già esistente. Rimossi gli angoli fissi di Classic e Modern.

Passati **76 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release.
Il nuovo test confronta la tangente Classic con la derivata della curva,
inclusa la posizione FUEL. Controllati visivamente tutti e tre i modelli.
Ogni Demo isolata dura **45 secondi**, con una ricreazione del device e zero
errori di presentazione: [Classic](../artifacts/fuel-alignment-classic-smoke.json),
[Modern](../artifacts/fuel-alignment-modern-smoke.json),
[Wide](../artifacts/fuel-alignment-wide-smoke.json).
Non è stata eseguita una sessione nel gioco.

## Fondi indipendenti, testi curvi, delta giro e marcia — 10 novembre 2026

Applicati i criteri del meta prompt: modifiche comuni ai tre modelli, riuso
delle impostazioni esistenti, separazione di fondo e contenuti, curve ricavate
dalla geometria reale e confronto basato soltanto su tempi validi della sessione.
Confermati dall'utente fondo 0–100% e velocità grande con marcia sopra.

**Background %** e Shift + rotella ora modificano soltanto i fondi di Halo e
Tyres. Scritte, numeri, bitmap auto e indicatori restano nitidi anche a 0%.
SM e delta seguono la curva sinistra glifo per glifo in Wide, Classic e Modern.
La marcia R/N/1–8 sostituisce KM/H sopra il numero grande della velocità.

Il delta sostituisce AERO STRAIGHT/CORNER: ultimo giro completato meno miglior
giro valido precedente, con tre decimali. Rosso se più lento, verde se più
rapido, neutro al pareggio. Giri invalidi, primo riferimento assente e dati
scaduti mostrano `--`. Un record nuovo conserva il confronto con quello vecchio.
Parser Session History verificato sulla specifica EA v1.2 e sulle strutture
ufficiali. Sono coperti vettura locale, record parziale, flag di validità,
conteggi, ultimo slot, tempi nulli, reset sessione, flashback e attesa del nuovo
giro. Il test UDP invia telemetria e cronologia sulla socket reale e verifica
il delta di 230 ms e la cancellazione al cambio sorgente.

Passati **75 test**, rustfmt, Clippy `--all-targets -D warnings`, build Release
e build degli esempi. Controllati visivamente i tre modelli, segni positivo e
negativo, pareggio, marcia centrale, fondi 0/100 e pannello in finestra ridotta.
Demo isolata di **248 secondi**: **14.625 frame**, una ricreazione del device,
zero errori di presentazione. [Report](../artifacts/halo-refinement-ui-smoke.json).
Non è stata eseguita una sessione nel gioco.

## Opacità percentuale e fondo Tyres uniforme — 10 novembre 2026

In **Halo & layout**, i campi **Opacity %** permettono di regolare Halo e Tyres
separatamente dal 15% al 100%. Valori maggiori riducono la trasparenza. Restano
compatibili i file impostazioni esistenti; applicare un valore non modificato
conserva la precisione precedente alla visualizzazione arrotondata.

Il disegno auto ora usa `assets/tyre-car-transparent.png`: il fondo originale
è rimosso con imagegen e il pannello fornisce una sola tonalità. Il PNG originale
è conservato. [Prompt e provenienza](TYRE-CAR-IMAGEGEN.md).

Passati **70 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release.
Estesa la verifica del pannello nativo con percentuali indipendenti, limiti
15/100, virgola decimale, valori non validi, salvataggio e precisione invariata.
Controllati visivamente campi e fondo uniforme al 100% e al 45%.

Demo isolata di **152 secondi**: **9.093 frame**, una ricreazione del device
e zero errori di presentazione. Il comando **Apply and save** ha salvato Halo
al 100% e Tyres al 45% nella configurazione di prova.
[Report](../artifacts/opacity-background-ui-smoke.json).
Non è stata eseguita una sessione nel gioco.

## Logo monoposto e telemetria — 10 novembre 2026

Il PNG scelto è copiato senza modifiche in `assets/app-logo.png`. La conversione
ICO conserva trasparenza e proporzioni e include 10 dimensioni da 16 a 256 px.
Le risorse Windows sono incorporate nell'eseguibile: il nuovo simbolo compare
nel pannello, nella barra del titolo, nelle icone delle finestre e nella tray.
Gli handle creati sono rilasciati alla chiusura e il logo resta disponibile
quando il pannello ricrea il tema al cambio DPI.

Passati **70 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release
GNU x64. Il nuovo test carica dal modulo tutte le 10 dimensioni e verifica
dimensioni effettive e profondità colore a 32 bit. Verificata nel PE la presenza
di 10 risorse ICON e una GROUP_ICON. Controllato visivamente il logo nel pannello
e nella barra del titolo dell'eseguibile distribuito.

Avvio Demo isolato di **33 secondi**: **1.962 frame** e zero errori di
presentazione. [Report](../artifacts/logo-smoke.json). La verifica usa una
configurazione temporanea. Non sono stati provati MSVC e una sessione nel gioco.

## Halo nelle visuali laterali — 10 novembre 2026

Il filtro della visuale posteriore ora nasconde Halo anche spostando lo stick
destro a destra o a sinistra. Halo torna al centro; i passaggi diagonali fra
retro e lati mantengono il filtro attivo. Il pannello mostra **Hide Halo: rear /
sides**. La preferenza esistente `hide_halo_rear_view` conserva il proprio valore.

Passati **69 test**, rustfmt, Clippy `--all-targets -D warnings` e build Release.
I controlli coprono entrambi gli assi DualSense USB/Bluetooth e XInput, soglie
55%/35%, rilascio, input scaduto o rimosso, direzioni diagonali e conservazione
delle preferenze Halo e pneumatici. Stick sinistro e clic R3 non attivano il filtro.

Avvio Demo di 3 secondi: **167 frame**, registrazione Raw Input riuscita e zero
errori di presentazione. [Report](../artifacts/halo-side-view-smoke.json).
Non sono stati provati un controller fisico e una sessione nel gioco.

## Responsive Settings window — 9 November 2026

Settings now supports dragging native window edges and corners, maximizing,
restoring and Windows snapping. Initial size fits the monitor under the pointer.
Controls and GDI decoration use the same responsive geometry: general controls
move below the settings in portrait-width windows, categories move to the top
in compact windows, and setting tiles stack when necessary. Fonts retain their
DPI size; scrolling and focus visibility keep all controls accessible.

All **67 tests**, rustfmt, Clippy `--all-targets -D warnings` and Release build
passed. Layout checks cover nine viewport shapes down to 320 logical pixels.
The native test resizes all six categories across landscape, portrait and narrow
windows, including 1920 × 1040, 1080 × 1800, 720 × 1100 and 360 × 700, with
96/144/192 DPI paths. It checks control bounds and overlaps, keyboard focus
visibility, diagnostic scroll preservation and retention of unsaved settings.

In the Windows app, dragged the bottom-right corner and left edge to narrow
portrait-shaped windows, reached all bottom commands with the scrollbar, and
maximized back to the full landscape screen. The earlier hover regression
checks still pass. Physical monitor rotation and a live F1 driving session
were not performed.

## Titolo, usura generale e colonne simmetriche — 9 November 2026

Il titolo è ora **TYRES**, da 14 a 18 px. La legenda inferiore è rimossa e
la percentuale in alto sostituisce l'etichetta della modalità temperature.
Il valore è la media delle quattro gomme e diventa `--%` senza usura recente.
Etichette, percentuali e temperature sono centrate in colonne da 82 unità,
con margini speculari rispetto all'immagine centrale.

Build Release, rustfmt, Clippy `--all-targets -D warnings` e **65 test** passati.
Il nuovo test copre media, estremi e dati assenti. Verificata la finestra nativa
senza telemetria e con UDP sintetico: 13%, 15%, 18% e 22% producono **17%**.
Prova di 75 secondi: 707 frame, una ricreazione del device e zero errori
di presentazione. [Report](../artifacts/tyre-layout-smoke.json).
Non è stata eseguita una sessione nel gioco.

## Immagine auto nel mini HUD pneumatici — 9 November 2026

Sostituita la sagoma geometrica con il PNG verticale fornito, incorporato
nell'eseguibile senza modificare il file originale. L'immagine conserva le
proporzioni e lascia leggibili usura e temperature dei quattro pneumatici.

Build Release, rustfmt, Clippy `--all-targets -D warnings` e **64 test** passati.
Controllata la finestra nativa al 100%, anche dopo la ricreazione delle risorse.
Demo di 90 secondi al 170%: **5.367 frame**, una ricreazione del device,
zero errori di presentazione. [Report](../artifacts/tyre-car-smoke.json).
La verifica usa dati Demo; non include una sessione nel gioco.

## Settings hover and repaint loop — 9 November 2026

Fixed the Settings hover redraw: moving within an unchanged control no longer
invalidates the panel. Push buttons use the native owner-draw callback, and
combobox content and chrome share one paint cycle with DC state restored.
Checkboxes retain their native keyboard and accessibility behaviour.

The native regression test failed before the fix when mouse movement over
Halo X invalidated the whole panel. It now checks inputs, buttons, switches
and comboboxes, repeated mouse moves, and printed control skin/captions.
All **64 tests**, rustfmt, Clippy `--all-targets -D warnings` and Release build
passed. Checked category navigation and opening/closing the temperature menu
in the Windows app.

Also fixed a repeating paint queue from hidden composition HUDs by completing
their `BeginPaint` / `EndPaint` cycle. A diagnostic run counted **49,098**
overlay paint messages in three seconds before the fix; the fixed five-second
run processed **two**, one per HUD. In the same hidden-HUD Demo scenario,
sampled CPU fell from approximately **99% to 1.6% of one logical core**.
The temporary tracing code was removed before packaging.
[Verification record](../artifacts/settings-hover-verification.json).

A live F1 driving session was not run for this UI correction.

## Navy / cyan Settings UI — 8 November 2026

Restyled the native Settings panel with a category sidebar, outlined settings
panels, cyan selections and switches, and a persistent general-controls pane.
All existing fields and commands remain available across six categories.
The palette and panel arrangement follow the supplied visual reference.

Release build, rustfmt, Clippy `--all-targets -D warnings` and all **64 tests**
passed. The new native panel test clicks every category through the Win32
command queue, checks which controls are visible, changes DPI through 144,
192 and 96, and verifies that unsaved IP, temperature, colour, shortcut,
profile name and rear-view values survive. Checked the Layout page visually
in the Windows application, including custom comboboxes and switches.

A 15-second Release Demo run rendered **882 unique telemetry frames**, with
one forced device recreation and **zero presentation failures**. The saved
test configuration remained byte-for-byte unchanged, and both HUDs retained
topmost, click-through and no-activation flags.
[Runtime report](../artifacts/settings-ui-smoke.json).

A live F1 driving session and physical DPI transitions between monitors were
not checked for this Settings-only change.

## Modern pedal inclination — 8 November 2026

Modern Brake and Throttle bars now rotate by -9° and +9° around centres at
equal distances from the central dial. Segment dimensions and vertical pivots
are identical; labels rotate with the bars. Pedal values remain independent.
Release build, rustfmt, Clippy `--all-targets -D warnings` and all **63 tests**
passed. Checked the regenerated native-renderer preview; no live driving run
was performed for this visual adjustment.

## Modern Halo model — 8 November 2026

Added **modern** to **Halo model**, alongside Wide and Classic. It follows the
attached reference with one shaded charcoal ribbon, curved ends, a thin upper
arch, an orange lower edge, a larger central dial and integrated fuel/SM/Aero
and gears/RPM. The gradient brush is cached per surface. All telemetry remains
live and shares the existing freshness, missing-data and ERS/SM rules.

Release build, rustfmt, Clippy `--all-targets -D warnings` and **63 automated
tests** passed. The new geometry check covers symmetry, curved end caps,
canvas bounds and room for gear labels; the existing settings round trip now
includes Modern. The regenerated Wide and Classic exports match their previous
PNG pixels exactly.

Selected modern from the native dropdown and used Apply and save on an isolated
configuration. Model persisted as Modern; Halo/tyre layouts, opacity and named
profiles were retained. Relaunch restored Modern with no presentation errors.
[Restart report](../artifacts/halo-modern-restart.json).

A 20-second Release Demo run rendered **1,183 unique telemetry frames** with
one forced device recreation and **zero presentation failures**. Both HUDs
retained click-through, topmost and no-activation flags.
[Runtime report](../artifacts/halo-modern-smoke.json).

The 4000 × 1200 PNG preview comes directly from the same Direct2D renderer with
fixed example telemetry. A live driving session and alignment with the user's
camera remain unverified.

## ERS mode, Straight Mode countdown and fuel margin — 8 November 2026

Release build, rustfmt, Clippy `--all-targets -D warnings`, all examples and
**62 automated tests** passed. Tests cover the signed MFD fuel field at offset
13, invalid floating-point values, deploy mode captions, negative zero,
SM activation/availability priority, distance handling and demo round trips.

Checked both Wide and Classic in the Windows app using synthetic UDP on a
separate port and an isolated settings file:

- The battery caption shows NONE, MEDIUM, HOTLAP or BOOST. The old ERS caption
  beneath Active Aero is absent.
- SM OFF updates from 350 m to 125 m; availability shows SM READY and activation
  shows SM ON with a green background.
- The small FUEL pod displays + 2.3, - 0.8 and + 12.3. The right gears/RPM band
  remains available. Missing status clears fuel and ERS; expired telemetry
  clears all live values and displays STALE DATA.
- Settings, model choices, context menus, status messages and edit hints are
  English. Switching Wide to Classic preserves X, Y, scale and opacity.

The end-to-end run accepted 27,678 packets with zero malformed, unsupported
or reordered packets, and zero presentation failures.
[Diagnostic report](../artifacts/hud-fuel-sm-update-smoke.json).
The check used synthetic data; a live F1 25 driving session remains unverified.

## Profili Halo con nome — 6 novembre 2026

Aggiunti il campo modificabile **Profilo Halo** e i comandi **Salva profilo** e
**Carica profilo**. Ogni nome conserva X, Y e Scala; salvare un nome esistente lo
aggiorna. La scelta è manuale, persistente e compatibile con i due modelli Halo.
I valori salvati non vengono sovrascritti spostando il layout corrente.

Build Release, fmt, Clippy `--all-targets -D warnings` e **59 test automatici**
passati. I quattro nuovi test verificano compatibilità dei JSON precedenti,
isolamento dei profili, richiamo delle sole geometrie, aggiornamento del nome
senza duplicati per maiuscole/spazi, persistenza su disco e rifiuto di nomi,
scale o riferimenti non validi senza modificare i profili salvati.

Verificati nell'app Windows su configurazione e porta separate:

- Creazione **Ferrari** a (455, 537), scala effettiva 1,037, e **Red Bull** a
  (300, 450), scala 0,82. Valori presi dai campi Halo e applicati subito.
- Scelta Ferrari dal menu e Carica: ritorno alle coordinate salvate; pneumatici,
  modello, colori e comando retrovista conservati.
- Riavvio: nome Ferrari ripristinato e richiamo Red Bull ancora funzionante.
  Un'opacità modificata nel campo ma non applicata resta nel pannello, senza
  essere applicata o scartata dal caricamento della geometria.
- Errore di scrittura provocato bloccando soltanto il JSON di prova: file e
  mappa dei profili invariati, larghezza Halo ripristinata a 1037 pixel dopo il
  tentativo di applicare scala 0,82. Nessun errore grafico.
  [Diagnostica dopo il ripristino](../artifacts/halo-profiles-ui.json).

L'allineamento dei singoli profili alla visuale delle diverse vetture va regolato
nel gioco. Le coordinate sopra sono soltanto dati della prova locale.

## Correzione del comando: stick destro verso il basso

Il comando concordato è lo **stick destro verso il basso, senza clic R3**.
Aggiornati i decoder Raw Input USB/Bluetooth e il fallback XInput (`sThumbRY`).
L'asse sinistro e i clic degli stick non attivano più il filtro. La diagnostica
esporta `right_stick_down`. Soglie, rilascio e visibilità dei due HUD conservati.

Aggiornato il test dei report per verificare il movimento destro e ignorare
asse sinistro e clic in tutti e quattro i formati supportati. La verifica fisica
in gioco con il controller dell'utente resta necessaria.

## Prima implementazione: sospensione con stick sinistro verso il basso

Aggiunta **Nascondi Halo dietro**, attiva per default e salvabile dal pannello.
L'asse sinistro verso il basso applica un filtro temporaneo soltanto al Halo;
il rilascio conserva tutte le preferenze manuali. La funzione richiede guida
live, Race Mode e F1 25 in primo piano. Demo, Edit, pausa, dati scaduti o perdita
del dispositivo ripristinano il layout previsto dalle preferenze manuali.

Build Release, fmt, Clippy `--all-targets -D warnings` e **55 test automatici**
passati. I sei nuovi test coprono formati DualSense USB/Bluetooth, asse corretto
indipendente da L3 e stick destro, report multipli e malformati, soglie con
isteresi, rilascio, input scaduto/rimosso, combinazioni di visibilità manuale e
migrazione/salvataggio del nuovo flag. Nessuna nuova dipendenza Rust.

Verificati nell'app Windows la casella inizialmente selezionata e il salvataggio
di `false` con Applica e salva. Modello Classico, geometrie, precisione di scala
e opacità dei due HUD conservati nel JSON di prova, su porta separata dal gioco.

Smoke test Release Demo di 12 secondi: **699 frame**, 583 frame telemetrici
distinti, una ricreazione delle risorse e zero errori di presentazione. Entrambi
gli HUD restano topmost, click-through e senza attivazione. Registrazione Raw
Input riuscita; nessun campione controller ricevuto durante la prova. Demo
correttamente non attiva il filtro. [Report](../artifacts/halo-rear-view-smoke.json).

La pressione fisica del DualSense, il mantenimento della visuale posteriore e
il ritorno davanti in una sessione reale restano da verificare sul controller
dell'utente. I test del decoder usano report sintetici, non una cattura hardware.

## Selezione dei modelli Ampio e Classico

Nel pannello è disponibile **Modello Halo**, con scelta **Ampio** e **Classico**.
Il modello Classico riprende il secondo riferimento, mantenendo tutte le
telemetrie aggiornate del primo: km/h, ERS centrale azzurro in zona Overtake,
pedali, marce/RPM, **SM ACTIVE** / **SM IN-ACT** e Active Aero. La logica dei dati
è condivisa. Non vengono ripristinate mph o le vecchie targhette decorative.

Scelta persistente nel profilo; i file precedenti aprono Ampio. Canvas e centro
del quadrante sono comuni ai due modelli. I decimali di scala/opacità impostati
con il mouse vengono conservati quando i relativi campi non cambiano.

Build Release, fmt, Clippy `--all-targets -D warnings` e **49 test automatici**
passati. Verificato nell'app il passaggio Ampio/Classico/Ampio tramite il menu,
seguito da Applica e salva: scelta salvata, posizione, scala e opacità conservate.
Verificati nel Classico ERS azzurro, stato SM inattivo e attivo e assenza delle
vecchie telemetrie. Le anteprime usano traffico sintetico su una porta separata.
[Ampio](../artifacts/halo-model-wide-preview.png),
[Classico](../artifacts/halo-model-classic-preview.png).

Controlli al riavvio di 12 secondi per modello, inclusa ricreazione delle risorse:

- Ampio: 700 frame telemetrici distinti, zero errori, una ricreazione.
  [Report](../artifacts/halo-model-Wide-smoke.json).
- Classico: 698 frame telemetrici distinti, zero errori, una ricreazione.
  [Report](../artifacts/halo-model-Classic-smoke.json).

La prova nella propria visuale di gioco resta manuale.

## Arcata ampliata sulle guide gialle

Sostituito il profilo precedente con un arco più ampio. Il bordo superiore passa
da 712 a 875 unità di larghezza; le estremità sono meno ripide. Canvas 1000 × 300,
con spazio laterale aggiunto senza ingrandire quadrante o caratteri. Marce,
scala RPM e pannello SM seguono i nuovi bordi. Testo inattivo: **SM IN-ACT**.

Build Release, fmt, Clippy `--all-targets -D warnings` e **46 test automatici**
passati. Un test confronta 13 punti dei bordi con le guide gialle del riferimento,
con tolleranza di 8 pixel per il tratto a mano. Verificati nella finestra nativa
la scala 100% e 80%, ERS azzurro e fascia SM attiva/inattiva.

[Anteprima nativa con dati simulati](../artifacts/halo-wide-preview.png).
Smoke test Release di 12 secondi: 699 frame telemetrici distinti, una ricreazione
delle risorse grafiche, zero errori di presentazione. Verificate dimensioni
1000 × 300 e proprietà topmost, senza attivazione e click-through.
[Report runtime](../artifacts/halo-wide-smoke.json).

## ERS centrale, Straight Mode e curva del cockpit

Conservate le dimensioni Halo 840 × 300. Pannelli, numeri delle marce e scala RPM
seguono ora lo stesso profilo del bordo interno, più piatto al centro e più
inclinato alle estremità. Rimossi mph, HALO/26, targhetta LIVE, targhetta Overtake
e popup Straight Mode. ERS al centro sotto i km/h; stato SM nella fascia sinistra.

Build Release, fmt, Clippy `--all-targets -D warnings` e **45 test automatici**
passati. I test Halo coprono Overtake disponibile prima del boost, stato SM
indipendente dalla disponibilità, invalidazione dei dati e continuità del profilo.

Verificati nell'app con UDP sintetico su porta separata:

- ERS bianco fuori zona e azzurro con Overtake disponibile, senza boost attivo.
- **SM IN-ACTIVE** quando spento; **SM ACTIVE** con fascia verde quando attivo.
- ERS 0%, 50% e 100%; scala 100% e 80%, senza tagli delle scritte.
- Vettura pre-2026: **SM N/A**, Aero N/A e MJ quando la capacità non è nota.
- Dati scaduti: velocità, ERS e stato SM sostituiti da `--`, con avviso compatto.

[Anteprima nativa con dati simulati](../artifacts/halo-ers-sm-preview.png).
Smoke test Release di 12 secondi con entrambi gli HUD e ricreazione delle risorse
dopo 3 secondi: 698 frame telemetrici distinti, una ricreazione, zero errori di
presentazione. [Report runtime](../artifacts/halo-ers-sm-smoke.json).
La posizione rispetto al Halo reale va verificata nella propria visuale di gioco.

## Ridisegno Halo dal riferimento visivo

Halo portato a 840 × 300 unità, con quadrante centrale km/h e mph, ali inclinate,
pedali a segmenti e scala marce/RPM. Palette iniziale arancione, verde lime e
rosso; i colori restano modificabili. Il riquadro laterale indica la marcia
corrente, perché non è disponibile un dato sul numero della prossima curva.

Build Release, fmt, Clippy `--all-targets -D warnings` e **45 test automatici**
passati (15 unit, 5 asset/handling, 22 protocollo, 3 UDP). Aggiunto un controllo
della conversione mph: 350 km/h corrispondono a 217 mph arrotondate.

Verifiche nell'app Windows con telemetria sintetica su porta separata:

- Stato di attesa e dati scaduti: velocità, marcia, Aero e percentuali diventano
  `--`, senza conservare valori di guida precedenti.
- Scala 100% e 80%, marcia 8, folle e retromarcia: numeri e selezione della
  marcia restano allineati sulle ali inclinate.
- Passaggio Corner/Straight: cambia l'indicazione Aero e scompare il popup
  di disponibilità all'attivazione dello Straight Mode.
- Race Mode: clic sui pixel del tachimetro ricevuto dalla finestra probe
  sottostante, con contatore da 0 a 1.

Smoke test Release di 12 secondi, entrambi gli HUD visibili e ricreazione delle
risorse dopo 3 secondi: 701 frame telemetrici distinti, una ricreazione eseguita,
zero errori di presentazione. Finestre topmost, senza attivazione e click-through;
dimensioni Halo 840 × 300. [Report runtime](../artifacts/halo-redesign-smoke.json).
La verifica usa dati simulati; non include una nuova sessione di guida nel gioco.

## Aggiornamento 0.2.0: batteria, Straight Mode e visibilità

Build Release, fmt, Clippy `--all-targets -D warnings` e **44 test automatici**
passati (14 unit, 5 asset/handling, 22 protocollo, 3 UDP). Le fixture di handling
sono scritte per i test e non contengono asset EA.

- Rilevata automaticamente l'installazione Steam corrente. Letti 44 record di
  handling: 15 per il 2025 e 29 per il 2026, tutti a 4 000 000 J, nessun errore.
  L'esecuzione locale del lettore Release, inclusi avvio e output, ha richiesto
  circa 154 ms. [Risultato della lettura](../artifacts/game-battery-v0.2.0.json).
- App nativa collegata a traffico UDP sintetico indipendente: 2 000 000 J di
  energia ricevuta, capacità automatica `game_files` di 4 000 000 J, display e
  diagnostica al 50%. Il vecchio valore manuale 8 500 000 J nel JSON di prova
  è stato ignorato e rimosso al salvataggio.
- Mini popup **STRAIGHT MODE DISPONIBILE** osservato sul Halo con disponibilità
  attiva; scomparsa osservata al passaggio in Straight Mode.
- `Ctrl+Shift+O` premuto da una finestra di un altro processo: entrambi gli HUD
  nascosti, poi ripristinati con le stesse geometrie. Focus rimasto al probe.
  Nei due report a distanza di 24,375 s i frame totali restano 6428, mentre i
  pacchetti accettati aumentano di 2439: la ricezione continua senza rendering.
  [Prima](../artifacts/features-hidden-a-v0.2.0.json),
  [dopo](../artifacts/features-hidden-b-v0.2.0.json).
- `Ctrl+Shift+H` ancora funzionante dopo il toggle. Click sui pixel del Halo
  ricevuto dal probe, contatore da 0 a 1 e focus conservato.
- Passaggio UDP / Demo / UDP verificato: provenienza `demo` separata, epoche
  0/1/2; ritorno a `game_files`, 50%, finestre topmost e click-through, nessun
  errore di presentazione.
  [Demo](../artifacts/features-demo-v0.2.0.json),
  [stato finale](../artifacts/features-v0.2.0.json).

Controllo prestazioni 0.2.0 separato dalle prove UI: UDP sintetico 60 Hz per 30 s,
pannello chiuso, due HUD visibili, nessuna acquisizione UI durante la misura.
Media 59,96 Hz; draw CPU p95 1,12 ms; intervallo p95 17,03 ms;
RX→Present submit p95 1,92 ms; GPU p95 1,93 ms; CPU media 0,52%;
working set finale 55,13 MiB; zero errori di presentazione.
[Report](../artifacts/udp-60-v0.2.0.json),
[campioni memoria](../artifacts/udp-60-v0.2.0-memory.csv).
Sono misure del processo HUD e traffico sintetico, non FPS del gioco o latenza
ottica. La lettura statica è della capacità nominale; una sessione di guida
reale e gli eventuali modificatori runtime restano da verificare.

## Ambiente e build iniziale 0.1.0

- Windows 11 Pro x64, build 10.0.26200.
- AMD Ryzen 5 7600, NVIDIA GeForce RTX 4070, desktop 1920 × 1080.
- Rust 1.99.1, target `x86_64-pc-windows-gnu`, linker Rust LLD, crate windows 0.62.2.
- Profilo Release con thin LTO, un codegen unit e debug info rimosse.
- Compilazione, `cargo fmt --check`, Clippy con `-D warnings` e 26 test automatici passati.

Le misure sono state eseguite sequenzialmente sul desktop corrente, con pannello
chiuso e due HUD visibili. Il traffico UDP proviene dal generatore esplicito del
progetto, su porte locali separate dal gioco. F1 25 non era in esecuzione.
Le normali applicazioni del desktop non sono state disattivate per isolare il PC.

## Risultati misurati della 0.1.0

| Scenario | Durata | Hz medi | Draw CPU p95 | Intervallo p95 | Generazione/RX→Present p95 | GPU p95 | CPU media processo | RAM finale |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Demo 60 Hz | 30 s | 60,00 | 1,10 ms | 17,04 ms | 13,59 ms | 0,50 ms | 0,81% | 55,83 MiB |
| UDP sintetico 60 Hz | 120 s | 60,00 | 1,05 ms | 17,01 ms | 17,29 ms | 0,38 ms | 0,72% | 55,43 MiB |
| UDP sintetico 120 Hz | 30 s | 120,00 | 1,03 ms | 8,62 ms | 1,53 ms | 0,47 ms | 1,32% | 55,50 MiB |
| Inattività | 8 s | — | 8,53 ms¹ | — | — | — | 0,23% | 52,32 MiB |
| Ricreazione risorse al secondo 3 | 10 s | 59,18 | 1,15 ms | 17,07 ms | 16,02 ms | 1,21 ms | 0,92% | 55,34 MiB |
| Interruzione UDP dopo 3 s | 8 s | 59,99² | 1,17 ms | 17,02 ms | 17,40 ms | 1,04 ms | 0,46% | 53,13 MiB |

¹ Soltanto **due frame complessivi** in otto secondi, all'avvio: il percentile
su due campioni include il costo iniziale di font e risorse. Non è un loop a FPS pieno.
² Frequenza durante il tratto attivo; lo stato finale è **DATI OBSOLETI**.
Anche gli scenari UDP ordinari interrompono il sender prima dell'uscita dell'app,
quindi possono terminare correttamente nello stato dati obsoleti.

Nessuno scenario ha registrato errori di presentazione. Il test di ricreazione
ha registrato **una ricreazione completa**, con ripresa della visualizzazione.
La riduzione della media in quello scenario comprende la pausa di ricreazione.
Le due finestre risultavano topmost, senza attivazione e con stile transparent.

L'intervallo p50 del test UDP 60 è **16,64 ms**, p99 **17,08 ms**. I 16,67 ms
sono il riferimento a 60 Hz, non un valore identico garantito per ogni frame.
La latenza misurata raggiunge il target preferito sotto 35 ms p95 nei test
sintetici indicati; non è una misura ottica né una verifica di impatto sul gioco.

CPU è la media del processo normalizzata sui core logici, non l'uso di un singolo
core. I percentili e gli Hz usano gli ultimi 4096 campioni disponibili; il conteggio
totale e la CPU media dell'intero processo sono separati. La Demo misura dalla
generazione, UDP dal ritorno di recv_from. Il timer GPU esclude DWM e scansione
del monitor. Queste distinzioni impediscono di attribuire alla GPU o alla rete
una misura che comprende fasi diverse.

Report grezzi:

- [Demo 60](../artifacts/demo-60.json)
- [UDP 60, 120 secondi](../artifacts/udp-60-v0.1.0.json)
- [UDP 120](../artifacts/udp-120.json)
- [Inattività](../artifacts/idle.json)
- [Ricreazione device](../artifacts/recreate-device.json)
- [Interruzione UDP](../artifacts/udp-disconnect.json)

## Memoria e microbenchmark

Nel test UDP di 120 s il working set iniziale campionato era **53,31 MiB**,
il picco campionato **55,48 MiB**, il finale **55,43 MiB**. Dopo il riscaldamento,
da 40 a 110 s, i campioni ogni 10 s restano tra **55,42 e 55,46 MiB**. I private
bytes restano circa 62,27–62,39 MiB e gli handle tra 552 e 556 nello stesso tratto.
Non è stata osservata crescita continua in questo intervallo; una prova di due
minuti non dimostra l'assenza di ogni leak.

[Campioni RAM, private bytes e handle](../artifacts/udp-60-v0.1.0-memory.csv).

Il microbenchmark parser ha misurato **12,37 ns/pacchetto** su 2 000 000 parse
di un Car Telemetry già in cache, selezionando la vettura locale. È la media di
questa specifica prova: non include socket, copia kernel, parsing dei 24 record
completi o rendering. Il risultato non rappresenta il throughput UDP del sistema.

## Verifiche funzionali osservate

| Area | Verifica eseguita |
| --- | --- |
| DirectX e alpha | Entrambi gli HUD visibili; dati e sagoma pneumatici aggiornati |
| Dati Demo | Telemetry, Status e Aero circa 60 Hz; Damage circa 10 Hz |
| Drag | Gesto mouse reale: Halo da (650,686) a (760,591), coordinate salvate nel JSON |
| Resize | Gesto dall'angolo, scala 1,00 a 0,8548387, rapporto conservato, swap chain ridimensionata |
| Impostazioni | Scala 1,10, opacità 0,70, accento #FFB84D applicati live e riscontrati su disco |
| Temperature | Both, Inner e Surface selezionati live; valori e righe corrispondenti osservati |
| Pannello | Chiusura del pannello con aggiornamento degli HUD ancora attivo |
| Hotkey | Ctrl+Shift+H da un secondo processo alterna Edit e Race |
| Focus | Esc da un HUD attivo in Edit ripristina il focus alla finestra probe precedente |
| Click-through | Clic sui pixel di entrambi gli HUD ricevuti da un altro processo: contatore 0, 1, 2; focus conservato |
| Connessione | UDP reale su loopback, passaggio Demo/UDP, errore porta occupata e retry automatico |
| Protocollo | Fixture indipendenti, vettura 23, marcia R, offset 2026, gomme, ERS, lunghezze, NaN, sessioni e sequenza |

Il confronto tra sorgenti e i contatori di errore sono testati senza collegarsi
al gioco. Le fixture non sono catture di una sessione reale.

## Prove ancora necessarie sul gioco e su altro hardware

- Sessione reale di F1 25 con 2026 Season Pack, compresi flashback del gioco e cambi modalità.
- Impatto sugli FPS e sui frame time del gioco, con e senza overlay.
- Latenza ottica con strumenti di acquisizione adeguati.
- Reset reale del driver o rimozione device; la prova svolta ricrea volontariamente le risorse dell'app.
- Più monitor fisici con DPI diversi, HDR, altri driver e configurazioni fullscreen esclusive.
- Test prolungati di ore e profiling heap/GPU per una ricerca completa di leak.

Le API per DPI, clamp visibile, adattamento al refresh e recupero device sono
implementate. Le prove hardware sopra elencate restano esplicitamente non verificate.
