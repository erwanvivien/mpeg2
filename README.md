# MPEG-2 Decoder

Ce projet a été réalisé dans le cadre du cours de TVID à l'EPITA.

## Partie A - Décoder une vidéo MPEG2

### 1. Visualiser une vidéo MPEG2

```bash
cargo run --release -- --pathdir="./videos/elementary"
```

### 2. Comment utiliser l'application

Il faut d'abord générer les PGMs à partir de la vidéo MPEG2.

```bash
$ ./tools/mpeg2dec/src/mpeg2dec videos/elementary/pendulum.m2v -o pgm -l -v
```

Les différents paramètres sont :

```bash
$ cargo run --release -- --help

MPEG2 Decoder

Usage: mpeg2.exe [OPTIONS]

Options:
  -p, --pathdir <PATHDIR>  [default: videos/pendulum]
  -f, --fps <FPS>
  -m, --mode <MODE>
  -h, --help               Print help information
  -V, --version            Print version information
```

Pour lancer l'application avec les paramètres :

```bash
cargo run --release -- --pathdir="."
```
