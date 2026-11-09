# Angolo destro ricavato dal sinistro

Riferimento: l'angolo inferiore sinistro cerchiato dall'utente nel modello HALO-Angoli-Trasparenti-v1.

Il risultato finale deriva direttamente dal PNG di partenza. È stata modificata soltanto la trasparenza dell'estremità destra, utilizzando il profilo di dissolvenza sinistro riflesso orizzontalmente. La copia usa x_destinazione = larghezza - 1 - x_origine e conserva l'inclinazione del taglio sinistro. Non vengono aggiunti contorni.

I caratteri chiari vengono conservati; il profilo considera il bordo esterno, evitando di importare le lettere o il contorno interno dell'arco. La copia può soltanto ridurre l'opacità e non rende più opaco lo sfondo.

Sono stati confrontati i pixel del PNG originale e di quello esportato: i valori RGB di tutta l'immagine sono invariati. Tutti i pixel fuori dall'estremità destra sono invariati anche nel canale alpha. Sono stati modificati 17.864 valori alpha. Dimensioni conservate: 2172 × 724.

Verifica del bordo del taglio nella zona dell'angolo, alla soglia alpha 128: sei campioni tra le righe 480 e 580 coincidono con il riflesso del bordo sinistro, con scarto di 0 pixel. I dettagli sono in VERIFICA.json.

Il PNG principale conserva lo sfondo trasparente. L'anteprima mostra lo stesso PNG su un fondo nero uniforme.

Strumenti: ImageGen integrato per l'elaborazione della richiesta; correzione finale geometrica del canale alpha con PowerShell e System.Drawing per conservare i pixel originali e verificare l'inclinazione.
