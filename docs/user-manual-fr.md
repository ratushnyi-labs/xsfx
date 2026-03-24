# xsfx — Manuel utilisateur

> **xsfx** — Empaqueteur d'exécutables auto-extractibles

## Prérequis

- Une plateforme prise en charge (Linux, macOS ou Windows)
- Aucune dépendance d'exécution requise

## Installation

### Option 1 : Binaire précompilé

Téléchargez depuis [GitHub Releases](https://github.com/ratushnyi-labs/xsfx/releases) :

| Plateforme | Fichier |
|------------|---------|
| Linux x64 (statique) | `xsfx-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (statique) | `xsfx-aarch64-unknown-linux-musl.tar.gz` |
| macOS x64 | `xsfx-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (Apple Silicon) | `xsfx-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `xsfx-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `xsfx-aarch64-pc-windows-msvc.zip` |

```bash
# Linux / macOS
curl -sSfL https://github.com/ratushnyi-labs/xsfx/releases/latest/download/xsfx-x86_64-unknown-linux-musl.tar.gz \
    | tar xzf - -C /usr/local/bin
```

### Option 2 : Compiler depuis les sources

```bash
git clone https://github.com/ratushnyi-labs/xsfx.git
cd xsfx
cargo build --release --bin xsfx
```

## Utilisation

### Empaqueter un binaire

```bash
xsfx <entrée> <sortie> [--target <triple>]
```

- `entrée` — binaire à empaqueter (utilisez `-` pour stdin)
- `sortie` — chemin du fichier SFX (utilisez `-` pour stdout)
- `--target` — plateforme cible (par défaut : la plateforme courante)

### Exemples

```bash
# Empaquetage basique
xsfx myapp myapp-sfx
chmod +x myapp-sfx
./myapp-sfx

# Empaqueter pour une autre plateforme
xsfx myapp myapp-sfx.exe --target x86_64-pc-windows-msvc

# Lister les cibles disponibles
xsfx
```

### Support des tubes (pipes)

Utilisez `-` pour stdin ou stdout :

```bash
# Lire depuis stdin
cat myapp | xsfx - myapp-sfx

# Écrire vers stdout
xsfx myapp - > myapp-sfx

# Tube complet
cat myapp | xsfx - - > myapp-sfx

# Empaqueter et déployer via SSH
xsfx myapp - --target x86_64-unknown-linux-musl | ssh server 'cat > myapp && chmod +x myapp'
```

### Exécuter le SFX empaqueté

Le binaire résultant s'exécute comme n'importe quel exécutable normal. Tous les arguments de ligne de commande sont transmis à la charge utile :

```bash
./myapp-sfx --verbose --config /etc/myapp.conf
```

## Cibles prises en charge

| Cible | Architecture | Exécution en mémoire |
|-------|-------------|----------------------|
| `x86_64-unknown-linux-musl` | x64 | `memfd_create` + `execveat` |
| `aarch64-unknown-linux-musl` | ARM64 | `memfd_create` + `execveat` |
| `x86_64-apple-darwin` | x64 | `NSCreateObjectFileImageFromMemory` |
| `aarch64-apple-darwin` | ARM64 | `NSCreateObjectFileImageFromMemory` |
| `x86_64-pc-windows-msvc` | x64 | Chargeur PE en processus |
| `aarch64-pc-windows-msvc` | ARM64 | Chargeur PE en processus |

## Format du binaire

Le binaire SFX se compose de trois parties :

```
+------------------------+
| Stub                   |  chargeur spécifique à la plateforme (<100 Ko)
+------------------------+
| Charge compressée      |  flux LZMA2/XZ
+------------------------+
| Bande-annonce (16 o)   |  payload_len (u64 LE) + magic (u64 LE)
+------------------------+
```

Aucun fichier temporaire n'est créé lors de l'extraction. La charge est décompressée et exécutée entièrement en mémoire.

## Vérification

Après l'empaquetage, vérifiez que le SFX fonctionne correctement :

```bash
# Empaqueter
xsfx myapp myapp-sfx

# Exécuter l'original
./myapp --version

# Exécuter le SFX — doit produire un résultat identique
./myapp-sfx --version
```

## Dépannage

| Problème | Cause | Solution |
|----------|-------|----------|
| `"Invalid SFX magic marker"` | Binaire SFX corrompu | Réempaqueter depuis l'original |
| `"File too small to contain trailer"` | Fichier SFX tronqué | Retélécharger ou réempaqueter |
| `Permission denied` | Permission d'exécution manquante | `chmod +x <sfx>` |
| `memfd_create: Operation not permitted` | Le noyau restreint memfd dans un conteneur | Ajouter `SYS_PTRACE` ou noyau >= 3.17 |
| Windows : `"Failed to load DLL"` | DLL manquante | Installer Visual C++ redistributable |
| macOS : `"Failed to create object file image"` | Problème de signature de code | Signer le SFX ou autoriser l'exécution non signée |
