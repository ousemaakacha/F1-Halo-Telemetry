# f1 tv

Modello Halo nativo Direct2D con gli indicatori del riferimento McLaren e
geometria simmetrica basata sulle curve di Shift. Si seleziona in
**Halo & layout → Halo model → f1 tv → Apply and save**.

![Anteprima nativa con dati simulati](../artifacts/f1-tv-preview.png)

Arco bianco e quadrante condividono il centro della finestra Halo. Le ali
usano lo stesso raggio di Shift, estremi speculari, spessore e cornici uguali.
Testi, divisori e barra RPM seguono la medesima curvatura. La classifica ha
due righe equidistanti; nomi lunghi terminano con `...` mantenendo la dimensione
dei caratteri. A destra, GEARS, numeri delle marce, barra e didascalia RPM hanno
quattro righe dedicate, con margini separati.

Il quadrante mostra km/h e mph, convertiti dalla stessa velocità UDP.
La scritta centrale indica la modalità ERS ricevuta: **NONE**, **MEDIUM**,
**HOTLAP** o **BOOST**. OVERTAKE si evidenzia quando disponibile o attivo.
La batteria, centrata sul quadrante compreso il terminale laterale, mostra
la percentuale al posto del fulmine. La percentuale usa la capacità nominale
letta dal gioco; senza capacità o status valido mostra `--%`.

Le fasce **RCH** (recharge) e **DPL** (deploy) seguono il cerchio e affiancano undici segmenti per
ciascun pedale, con le etichette **BRK** e **THR**. Il riempimento blu indica
la quota ancora disponibile nel giro, senza valori numerici in MJ.
Le due sigle sono centrate nei contenitori, su una sola riga
con spaziatura naturale. RCH sottrae l'energia MGU-K già raccolta dal limite UDP
del giro, che può cambiare durante la sessione. DPL sottrae l'energia
già usata dal limite nominale `DeployLimitMGUK` letto dai file installati.
Capacità batteria e limiti per giro sono denominatori distinti. Al reset dei
contatori UDP nel nuovo giro, le fasce tornano piene. Il residuo non scende
sotto zero; dati mancanti o scaduti lasciano il riempimento spento e la sigla attenuata.
Il pedale acceleratore usa il verde menta del riferimento quando il colore
globale è quello predefinito; un colore personalizzato viene rispettato.

Il pannello sinistro mostra l'auto immediatamente davanti nella riga superiore
e quella immediatamente dietro nella riga inferiore. Ciascuna riga contiene
posizione in classifica, nome e distacco dalla propria vettura in una colonna
dedicata, per esempio **1.234s**. I nomi provengono da Participants, mentre
posizioni, identità degli avversari e intervalli provengono da Lap Data.
I nomi restano validi dieci secondi
fra gli invii Participants, documentati ogni cinque secondi. Posizioni,
pedali e stati rapidi scadono dopo 500 ms. Nomi vuoti, UTF-8 non valido,
caratteri di controllo e dati assenti producono `--`. La riga relativa all'auto
davanti rimane senza dati quando si è primi; lo stesso vale per quella dietro
quando si è ultimi. I distacchi mostrano tre decimali sotto 100 secondi,
uno sotto 1000 e secondi interi per intervalli maggiori, per mantenere leggibile
la colonna. Un distacco valido di zero viene mostrato come **0.000s**.

La colonna dei distacchi occupa lo spazio del precedente riquadro TURN.
ACTIVE AERO mostra due tratti orizzontali in Straight Mode e inclinati in
Corner Mode. Un bordo aderente circonda singolarmente ciascun tratto, estremità
comprese: verde quando l'aerodinamica è attiva, rosso quando è inattiva.
La sola disponibilità non rende verde il
bordo; dati assenti, scaduti o non applicabili mantengono il bordo spento.
Il pannello destro evidenzia la marcia ricevuta, compresa R. La scritta sotto
la barra mostra i giri attuali per intero, per esempio **RPM 10000**; senza
telemetria valida mostra **RPM --**. La barra resta relativa al limite
effettivo della vettura.

Posizione, scala e trasparenza seguono i normali controlli Halo. L'allineamento
con la vettura richiede una regolazione sulla visuale e sul FOV usati nel gioco.
I profili Halo consentono di conservare tale regolazione per ciascuna visuale.

Il nuovo modello conserva gli altri modelli e il pannello pneumatici corrente.
La modalità Demo rimane identificata con **DEMO · SIMULATED DATA**.
