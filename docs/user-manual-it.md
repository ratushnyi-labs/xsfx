# xsfx — Manuale utente

> **xsfx** — Impacchettatore di eseguibili autoestraenti

## Prerequisiti

- Una piattaforma supportata (Linux, macOS o Windows)
- Nessuna dipendenza runtime richiesta

## Installazione

### Opzione 1: Binario precompilato

Scarica da [GitHub Releases](https://github.com/ratushnyi-labs/xsfx/releases):

| Piattaforma | File |
|-------------|------|
| Linux x64 (statico) | `xsfx-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (statico) | `xsfx-aarch64-unknown-linux-musl.tar.gz` |
| macOS x64 | `xsfx-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (Apple Silicon) | `xsfx-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `xsfx-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `xsfx-aarch64-pc-windows-msvc.zip` |

```bash
# Linux / macOS
curl -sSfL https://github.com/ratushnyi-labs/xsfx/releases/latest/download/xsfx-x86_64-unknown-linux-musl.tar.gz \
    | tar xzf - -C /usr/local/bin
```

### Opzione 2: Compilare dal codice sorgente

```bash
git clone https://github.com/ratushnyi-labs/xsfx.git
cd xsfx
cargo build --release --bin xsfx
```

## Utilizzo

### Impacchettare un binario

```bash
xsfx <input> <output> [--target <tripla>]
```

- `input` — binario da impacchettare (usa `-` per stdin)
- `output` — percorso del file SFX (usa `-` per stdout)
- `--target` — piattaforma di destinazione (predefinita: la piattaforma corrente)

### Esempi

```bash
# Impacchettamento base
xsfx myapp myapp-sfx
chmod +x myapp-sfx
./myapp-sfx

# Impacchettare per un'altra piattaforma
xsfx myapp myapp-sfx.exe --target x86_64-pc-windows-msvc

# Elencare le destinazioni disponibili
xsfx
```

### Supporto pipe

Usa `-` per stdin o stdout:

```bash
# Leggere da stdin
cat myapp | xsfx - myapp-sfx

# Scrivere su stdout
xsfx myapp - > myapp-sfx

# Pipe completa
cat myapp | xsfx - - > myapp-sfx

# Impacchettare e distribuire via SSH
xsfx myapp - --target x86_64-unknown-linux-musl | ssh server 'cat > myapp && chmod +x myapp'
```

### Eseguire il SFX impacchettato

Il binario risultante si esegue come qualsiasi eseguibile normale. Tutti gli argomenti della riga di comando vengono inoltrati al payload:

```bash
./myapp-sfx --verbose --config /etc/myapp.conf
```

## Destinazioni supportate

| Destinazione | Architettura | Esecuzione in memoria |
|-------------|-------------|----------------------|
| `x86_64-unknown-linux-musl` | x64 | `memfd_create` + `execveat` |
| `aarch64-unknown-linux-musl` | ARM64 | `memfd_create` + `execveat` |
| `x86_64-apple-darwin` | x64 | `NSCreateObjectFileImageFromMemory` |
| `aarch64-apple-darwin` | ARM64 | `NSCreateObjectFileImageFromMemory` |
| `x86_64-pc-windows-msvc` | x64 | Caricatore PE in-process |
| `aarch64-pc-windows-msvc` | ARM64 | Caricatore PE in-process |

## Formato del binario

Il binario SFX è composto da tre parti:

```
+------------------------+
| Stub                   |  caricatore specifico per piattaforma (<100 KB)
+------------------------+
| Payload compresso      |  flusso LZMA2/XZ
+------------------------+
| Trailer (16 byte)      |  payload_len (u64 LE) + magic (u64 LE)
+------------------------+
```

Non vengono scritti file temporanei durante l'estrazione. Il payload viene decompresso ed eseguito interamente in memoria.

## Verifica

Dopo l'impacchettamento, verifica che il SFX funzioni correttamente:

```bash
# Impacchettare
xsfx myapp myapp-sfx

# Eseguire l'originale
./myapp --version

# Eseguire il SFX — deve produrre un risultato identico
./myapp-sfx --version
```

## Risoluzione dei problemi

| Problema | Causa | Soluzione |
|----------|-------|-----------|
| `"Invalid SFX magic marker"` | Binario SFX corrotto | Reimpacchettare dall'originale |
| `"File too small to contain trailer"` | File SFX troncato | Riscaricare o reimpacchettare |
| `Permission denied` | Permesso di esecuzione mancante | `chmod +x <sfx>` |
| `memfd_create: Operation not permitted` | Il kernel limita memfd nel container | Aggiungere `SYS_PTRACE` o kernel >= 3.17 |
| Windows: `"Failed to load DLL"` | DLL runtime mancante | Installare Visual C++ redistributable |
| macOS: `"Failed to create object file image"` | Problema di firma del codice | Firmare il SFX o consentire l'esecuzione non firmata |
