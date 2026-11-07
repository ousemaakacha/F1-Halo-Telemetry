# Architettura e scelte native

## Separazione del lavoro

`telemetry/protocol` legge buffer binari senza Windows o renderer. `telemetry/state`
gestisce sequenza, sessioni e timestamp separati. `udp` riceve su un thread
dedicato con buffer fisso da 65535 byte, timeout socket e retry del bind.
La sorgente Demo è selezionata tramite un comando e sostituisce esplicitamente
la sorgente UDP, svuotando lo stato precedente.
Ogni cambio sorgente incrementa un'epoca e svuota subito lo snapshot condiviso:
un aggiornamento ancora in volo della sorgente precedente viene scartato.

`game_data` individua l'installazione e legge soltanto indici ERPK e risorse BXML
di handling. La decompressione Zstandard è in Rust tramite `ruzstd`; dimensioni,
offset e finestra sono limitati prima di allocare. Il catalogo nominale resta sul
thread ricezione, si rinnova per sessione e fornisce un valore Copy allo snapshot.
Non vengono aperti handle con accesso alla memoria del processo di gioco.

Lo snapshot è una struttura Copy dentro un `RwLock`. Il writer copia lo stato
e segnala un evento Win32. Il renderer usa esclusivamente `try_read`: se il lock
non è disponibile conserva la copia precedente e ne verifica comunque la
scadenza. Non attende mai la ricezione né un lock del thread UDP. Questa scelta
mantiene il passaggio tra thread in codice Rust sicuro senza una nuova dipendenza
o un buffer lock-free scritto con unsafe. La contesa riguarda soltanto la copia
di una struttura piccola; le misure locali sono nel documento di verifica.

## Rendering e composizione

Un device Direct3D 11 hardware con BGRA support è condiviso dalle due superfici.
Ogni HUD ha una swap chain composition BGRA8, due buffer, alpha premoltiplicato,
flip sequential e latenza massima di coda 1. Direct2D disegna direttamente sul
backbuffer DXGI; DirectWrite gestisce il testo con antialias grayscale adatto
alla trasparenza. DirectComposition collega la superficie all'HWND e applica
l'opacità. Non viene usato `UpdateLayeredWindow` né un upload CPU del framebuffer.

Geometria Halo, segmenti curvi, formati testo e brush vengono creati una volta.
Il testo variabile viene formattato in buffer sullo stack. Il brush viene
riutilizzato cambiandone il colore. Gli oggetti COM vivono soltanto sul thread UI.
In caso di errore grafico vengono rilasciati i riferimenti ai backbuffer e
ricreate tutte le risorse; il loop esegue retry con backoff, non un ciclo pieno.

`Present(0)` consegna i frame al compositore DWM. Il pacing dell'app usa un timer
waitable ad alta risoluzione e scadenze assolute; evita due attese vsync seriali
per i due HUD. La frequenza configurata è un limite. Senza nuovi dati o modifiche
di stato il renderer non disegna; il message loop attende eventi, input o scadenze.
La modalità adattiva legge il refresh del monitor Halo e ne conserva il valore
fino a uno spostamento o cambio display, evitando query monitor per frame.

Riferimenti Microsoft:

- [CreateSwapChainForComposition](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_2/nf-dxgi1_2-idxgifactory2-createswapchainforcomposition)
- [Composizione senza copie di framebuffer CPU](https://learn.microsoft.com/en-us/archive/msdn-magazine/2014/june/windows-with-c-high-performance-window-layering-using-the-windows-composition-engine)
- [Extended window styles](https://learn.microsoft.com/en-us/windows/win32/winmsg/extended-window-styles)

## Input, focus e pannello

Race Mode usa WS_EX_LAYERED, WS_EX_TRANSPARENT, WS_EX_NOACTIVATE,
WS_EX_TOOLWINDOW e WS_EX_NOREDIRECTIONBITMAP. `SetLayeredWindowAttributes`
imposta l'alpha globale a 255 per il comportamento input Win32; i pixel e
l'opacità effettiva restano su DirectComposition. Edit Mode rende gli HUD
interattivi e individuabili nella navigazione tra finestre. Non installa hook
globali: le due scorciatoie usano RegisterHotKey. La registrazione è transazionale:
un conflitto conserva entrambe le combinazioni precedenti. Il comando globale di
visibilità applica un filtro ai layout senza sovrascrivere le preferenze salvate;
quando il filtro nasconde gli HUD non vengono effettuati draw o Present.

`input/rear_view` legge soltanto l'asse verticale destro del DualSense tramite
Raw Input HID (USB e i due formati Bluetooth), con fallback XInput per i pad
virtuali Steam Input. Il pannello registra joystick/gamepad con INPUTSINK e
DEVNOTIFY: l'input del gioco non viene consumato. I report vengono letti durante
WM_INPUT, prima di DefWindowProc; vengono accettati soltanto vendor/prodotto Sony
e formati di lunghezza nota. La coda UI conserva asse e identificatore dispositivo,
senza riferimenti ai buffer Windows. La rimozione invalida subito il campione.

La pressione verso il basso si attiva al 55% e si rilascia al 35%, evitando
sfarfallii vicino alla soglia. Un campione HID scade dopo 500 ms; input assente,
telemetria scaduta, pausa, perdita del focus, Demo ed Edit ripristinano la
visibilità. Il focus verifica soltanto nome e percorso del processo F1_25.exe.
Durante la guida il timer ricontrolla al massimo ogni 16 ms, anche con il solo
Halo nascosto, per rilevare il rilascio. La scansione degli slot XInput assenti
avviene al massimo una volta al secondo.

La sospensione è un filtro temporaneo soltanto sul layout Halo. Draw, applicazione
impostazioni e hotkey usano lo stesso filtro; i layout salvati e il mini HUD
pneumatici rimangono indipendenti. `hide_halo_rear_view` è attivo per default anche
nei profili precedenti, disattivabile dal pannello. La diagnostica esporta stato
del filtro, registrazione Raw Input, conteggio campioni, sorgente e ultimo asse.
Non viene dedotta la visuale dai flag UDP: il filtro segue il comando concordato
con l'utente, quindi richiede la stessa assegnazione nel gioco.

Riferimenti: [Raw Input](https://learn.microsoft.com/en-us/windows/win32/inputdev/raw-input),
[XInputGetState](https://learn.microsoft.com/en-us/windows/win32/api/xinput/nf-xinput-xinputgetstate),
[formati DualSense in SDL](https://github.com/libsdl-org/SDL/blob/main/src/joystick/hidapi/SDL_hidapi_ps5.c).

Drag e resize usano cattura del mouse e la posizione conservata nel messaggio
di input, non la posizione corrente che potrebbe essere già cambiata quando
si svuota la coda. Il resize conserva il rapporto del singolo HUD. Le window
procedure usano dati con interior mutability e accodano azioni: non rientrano
con un riferimento mutabile al controller durante le chiamate Win32.

Il pannello usa EDIT, BUTTON, COMBOBOX, STATIC e scrollbar Win32 standard.
Supporta tastiera tramite IsDialogMessage e layout DPI per monitor. È topmost
quando visibile per evitare che gli HUD coprano i controlli. Chiuso, non riceve
ridisegni periodici o aggiornamenti delle etichette. Non introduce un secondo
renderer, runtime UI o motore browser.

I profili Halo sono una mappa ordinata `halo_profiles` nel JSON, con nome libero
e valori X/Y/scala. `active_halo_profile` conserva l'ultimo nome salvato o caricato;
il layout corrente resta indipendente dai preset. Salva profilo legge soltanto
i tre campi geometrici Halo, conserva la precisione della scala e applica il
clamp DPI/area di lavoro prima di registrare le dimensioni effettive. Carica profilo
modifica soltanto quei valori, usando i normali filtri di visibilità globale e
visuale posteriore. Le modifiche tramite mouse non sovrascrivono i preset.

Entrambi i comandi operano su una copia delle impostazioni e salvano tramite la
sostituzione atomica già esistente. Un errore di salvataggio ripristina la geometria
precedente senza aggiornare la mappa in memoria. I nomi vengono normalizzati negli
spazi esterni e confrontati senza distinzione di maiuscole; nomi vuoti, caratteri
di controllo e scale non finite/fuori intervallo sono rifiutati prima dell'applicazione.
I profili precedenti senza questi campi rimangono compatibili. Il pannello offre
una combobox modificabile e i comandi espliciti Salva/Carica, senza cambio sorgente UDP.

La configurazione JSON usa serde e serde_json; ruzstd legge le risorse compresse
del gioco. Sono dipendenze Rust, senza un runtime UI aggiuntivo. Non sono stati
adottati backend grafici alternativi allo stack richiesto.
