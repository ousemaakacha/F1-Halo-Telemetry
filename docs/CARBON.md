# Carbon — Halo e pneumatici

Modello aggiunto il 16 novembre 2026 dai due riferimenti forniti dall'utente.
Selezionare **Halo model → Carbon (Halo + Tyres)** e **Apply and save**.
La singola scelta applica il design a entrambi gli HUD e viene conservata nel
file impostazioni. Gli altri tre modelli conservano il pannello gomme originale.

Gli HUD usano gli stessi canvas, layout, profili, scala e opacità dei modelli
precedenti. La grafica è nativa Direct2D/DirectWrite: geometrie e trama carbonio
sono preparate alla creazione del device, senza ricostruirle a ogni frame.
Bordi rossi, tipografia Arial Bold Italic, contorno argento e valori bianchi
riprendono i riferimenti. Le percentuali gomme indicano usura, non vita residua.
Temperature e usura mantengono soglie di allarme configurabili, modalità
Surface/Inner/Both e timestamp indipendenti. Il quadrante usa i 15 bit LED reali.
I valori assenti sono indicati da `--`; nessun dato illustrativo è incorporato
nella grafica. Il fondo grigio del riferimento è esterno agli overlay.

Le due sezioni esterne, Straight Mode e Behind, hanno una fascia più alta:
circa il 45% di spazio in più tra bordo superiore e inferiore nelle zone dei
valori. Le due righe sono più distanziate; quadrante, canvas e dimensioni dei
caratteri sono conservati.
Il profilo superiore segue una curva continua fino al quadrante. Alle estremità
le ali restano aperte: sono visibili soltanto il bordo superiore argento e quello
inferiore rosso, senza chiusure laterali.
Le estremità sono accorciate con due tagli obliqui speculari rispetto al centro.
Le scritte delle ali seguono ora il profilo della fascia lettera per lettera:
FUEL, Straight Mode, pedali, RPM e Behind usano curve ricavate dai bordi reali.
Barre, LED e separatori sono allineati allo stesso arco; i tracciati vengono
calcolati una volta alla creazione delle risorse grafiche.

## Asset monoposto

Asset finale: [`assets/tyre-car-carbon.png`](../assets/tyre-car-carbon.png).
Generato con il tool integrato **image_gen**, a partire dal riferimento del
pannello pneumatici. PNG RGBA 1024 × 1536; alpha conservata e verificata.
L'asset originale degli altri modelli rimane disponibile.

Prompt finale utilizzato:

```text
Use case: background-extraction. Asset type: transparent PNG car illustration for the centre of a live racing telemetry HUD. Input image is the edit target. Extract ONLY the complete silver, charcoal and red Formula racing car in the centre of the supplied HUD. Preserve its exact top-down orthographic orientation, nose pointing up, silhouette, four black slick tyres, suspension arms, front wing, rear wing, metallic silver highlights and thin red trim. Enlarge and clean the car into a crisp premium studio 3D product render, tall portrait composition, tightly framed with just 3 percent transparent padding around the whole car. True alpha transparency outside the car, including spaces between tyres, suspension and body; preserve opaque dark bodywork. Remove the entire HUD panel, all labels, percentages, temperature values, bars, border, grey background and shadows outside the car. No text, no logos, no floor, no outline glow, no checkerboard drawn into the image. Output a single car only on a genuinely transparent background, suitable to composite over a black panel.
```
