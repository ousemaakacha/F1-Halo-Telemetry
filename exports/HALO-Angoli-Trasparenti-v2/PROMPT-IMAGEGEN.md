# Prompt di lavoro — angolo sinistro come riferimento

Strumento: ImageGen integrato. Il file METODO.md descrive la correzione finale del canale alpha applicata direttamente al PNG originale.

## Richiesta iniziale

MINIMAL LOCAL EDIT: use the clean, unannotated HUD as the target. The image with a red circle identifies the approved LEFT lower terminal corner.

Keep the LEFT corner completely unchanged. Correct ONLY the far-RIGHT terminal corner so it is the exact horizontal reflection of the circled LEFT corner about the vertical centerline of the speedometer. Copy the same taper angle, both adjoining edge slopes, corner sharpness, tip height, distance from the center, and transparency feather. Do not estimate a new angle or use the current right corner as reference. Reflect the LEFT geometry literally: a point (x,y) on the left contour corresponds to (2172-x,y) on the right contour in this 2172-pixel-wide image. Both tips must sit at the SAME vertical height.

The right corner must also inherit the LEFT corner's dark charcoal/navy tone and identical alpha fade. No visible outline, border stroke, bevel, hard cap, gray haze, or added glow. Preserve the fading orange rail at the terminal point.

Everything outside this tiny right-terminal adjustment remains unchanged: entire left half, middle, typography, every text/value, right-side gears and RPM, colors, upper white line, pedal bars, orange thickness, existing framing and transparent background. Do not regenerate or redesign the whole HUD. All text remains forward-reading. Remove the red annotation circle from the output. One full panoramic transparent PNG, same dimensions and margins.

## Vincoli geometrici di rifinitura

Refine ONLY the far-right end cut of this HUD. Keep the left end entirely unchanged.

The LEFT terminal contour is the master. Its main side edge leans approximately 14 degrees away from vertical, outward toward the bottom. The right terminal contour must have exactly the same 14-degree lean in the opposite horizontal direction, NOT the current steeper/nearly vertical right cut.

Match the actual left alpha silhouette by horizontal reflection around the canvas midpoint. At this 2172x724 size, the reflected right-side 50%-alpha boundary should pass through approximately (1887,440), (1892,460), (1897,480), (1902,500), (1907,520). These are invisible geometric targets, not text to draw. Match the lower right tapered vertex to the left vertex at the same height. Match the angle where the orange lower curve meets the side cut. Copy the same feather width and dark fade as the left side.

No visible contour stroke or border. Use genuine transparency only. Preserve the entire left half, all text and numbers, all colors, both pedal bars, center dial, upper white line, orange rail, geometry elsewhere and canvas exactly. Do not move or mirror the gears, do not add annotations, do not redesign. Output the whole unchanged HUD with only this small right alpha-contour correction. Transparent background retained.
