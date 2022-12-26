# Réponses aux questions

## Partie A - Jouer un flux MPEG-2 élémentaire de test

1. Nous avons utilisé la commande `vlc -v [...]` pour visualiser les vidéos MPEG2.

2. Nous avons utilisé le binaire `mpeg2dec` pour décoder les vidéos MPEG2 en séquence d'image en format PGM (P5).

3. Les PGMs générés sont structuré en 3 parties, la chroma (Y) et les deux autres composantes (Cb et Cr). \
   Nous avons principalement des images YUV 4:2:0. Les premières rangées de pixels composent la composante Y, \
   puis une fois les 2/3 de la hauteur de l'image atteint, les rangées suivantes composent les composantes Cb et Cr \
    (les deux composantes sont mélangées dans une rangée).

Une image ressemble à ça :

```text
P5
4 3    <-- 4 largeur, 3 hauteur (reel_hauteur = hauteur * 2/3 = 2)
YYYY
YYYY   <-- 2 lignes de Y (reel_hauteur)
BBRR   <-- 1 ligne de Cb / Cr (hauteur * 1/3)
```

4. TODO

5. Implémentation en Rust dans le dossier `src/` du projet.

6. On a choisit de render en PPM ainsi que de rendre à l'écran

7. L'option `--fps [fps]` permet de spécifier le nombre d'images par seconde à afficher.

8. TODO

9. TODO

10. TODO
