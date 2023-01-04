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

4. À partir du dossier `tools/mpeg2dec`, en exécutant `./src/mpeg2dec -v ../../videos/elementary/pendulum.m2v -l -o null`, nous obtenons un fichier `tvid.log` contenant les métadatas par frame pour une vidéo donnée.

5. Implémentation en Rust dans le dossier `src/` du projet.

6. On a choisi de render en PPM ainsi que de rendre à l'écran.

7. L'option `--fps [fps]` permet de spécifier le nombre d'images par seconde à afficher.

8. Comme la question 4) avec le frame_period de chaque séquence en plus.

9. Implémenté.

10. Implémenté.

## Partie B - Jouer un flux vidéo de chaîne d’infos américaine assez notoire

1. Le PID du flux vidéo est 0x1422, celui du flux audio est 0x1423.

2. Nous avons utilisé les commandes suivantes pour générer les PGMs à partir d'un PID de MPEG-TS :

-   `ffplay videos/ts/cnn.ts`
-   `./tools/mpeg2dec/src/mpeg2dec videos/ts/cnn.ts -t 0x1422 -o pgm`

3. `cargo run --release -- --pathdir="./videos/ts/cnn_pgm"`

4. La moitié des frames sont progressives, les autres sont entrelacées.

5. Le flag progressive n'apparaît pour aucune des séquences alors que la moitié des frames sont progressives.

6. Nous pouvons forcer le désentrelaceur pour chaque vidéo. Cela n'aura que très peu d'impact sur celles dont les séquences sont progressives.

## Partie C - Jouer un flux vidéo de chaînes de divertissement asiatiques

1. Le PID du troisième flux vidéo est 0x3fd.

2. Fait

3. On remarque sur la séquence des effets que la vidéo est ralentie alors que les effets ne le sont pas.

4. Le PID du premier flux vidéo est 0x3e9.

5. Fait

6. TODO
