# xsfx — Manual de usuario

> **xsfx** — Empaquetador de ejecutables autoextraíbles

## Requisitos previos

- Una plataforma compatible (Linux, macOS o Windows)
- No se requieren dependencias en tiempo de ejecución

## Instalación

### Opción 1: Binario precompilado

Descargue desde [GitHub Releases](https://github.com/ratushnyi-labs/xsfx/releases):

| Plataforma | Archivo |
|------------|---------|
| Linux x64 (estático) | `xsfx-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (estático) | `xsfx-aarch64-unknown-linux-musl.tar.gz` |
| macOS x64 | `xsfx-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (Apple Silicon) | `xsfx-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `xsfx-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `xsfx-aarch64-pc-windows-msvc.zip` |

```bash
# Linux / macOS
curl -sSfL https://github.com/ratushnyi-labs/xsfx/releases/latest/download/xsfx-x86_64-unknown-linux-musl.tar.gz \
    | tar xzf - -C /usr/local/bin
```

### Opción 2: Compilar desde el código fuente

```bash
git clone https://github.com/ratushnyi-labs/xsfx.git
cd xsfx
cargo build --release --bin xsfx
```

## Uso

### Empaquetar un binario

```bash
xsfx <entrada> <salida> [--target <triple>]
```

- `entrada` — binario a empaquetar (use `-` para stdin)
- `salida` — ruta del archivo SFX (use `-` para stdout)
- `--target` — plataforma destino (por defecto: la actual)

### Ejemplos

```bash
# Empaquetado básico
xsfx myapp myapp-sfx
chmod +x myapp-sfx
./myapp-sfx

# Empaquetar para otra plataforma
xsfx myapp myapp-sfx.exe --target x86_64-pc-windows-msvc

# Listar destinos disponibles
xsfx
```

### Soporte de tuberías

Use `-` para stdin o stdout:

```bash
# Leer desde stdin
cat myapp | xsfx - myapp-sfx

# Escribir en stdout
xsfx myapp - > myapp-sfx

# Tubería completa
cat myapp | xsfx - - > myapp-sfx

# Empaquetar y desplegar por SSH
xsfx myapp - --target x86_64-unknown-linux-musl | ssh server 'cat > myapp && chmod +x myapp'
```

### Ejecutar el SFX empaquetado

El binario resultante se ejecuta como cualquier ejecutable normal. Todos los argumentos de línea de comandos se reenvían a la carga útil:

```bash
./myapp-sfx --verbose --config /etc/myapp.conf
```

## Destinos compatibles

| Destino | Arquitectura | Ejecución en memoria |
|---------|-------------|----------------------|
| `x86_64-unknown-linux-musl` | x64 | `memfd_create` + `execveat` |
| `aarch64-unknown-linux-musl` | ARM64 | `memfd_create` + `execveat` |
| `x86_64-apple-darwin` | x64 | `NSCreateObjectFileImageFromMemory` |
| `aarch64-apple-darwin` | ARM64 | `NSCreateObjectFileImageFromMemory` |
| `x86_64-pc-windows-msvc` | x64 | Cargador PE en proceso |
| `aarch64-pc-windows-msvc` | ARM64 | Cargador PE en proceso |

## Formato del binario

El binario SFX consta de tres partes:

```
+------------------------+
| Stub                   |  cargador específico de plataforma (<100 KB)
+------------------------+
| Carga comprimida       |  flujo LZMA2/XZ
+------------------------+
| Tráiler (16 bytes)     |  payload_len (u64 LE) + magic (u64 LE)
+------------------------+
```

No se escriben archivos temporales durante la extracción. La carga se descomprime y ejecuta completamente en memoria.

## Verificación

Después de empaquetar, verifique que el SFX funciona correctamente:

```bash
# Empaquetar
xsfx myapp myapp-sfx

# Ejecutar el original
./myapp --version

# Ejecutar el SFX — debe producir el mismo resultado
./myapp-sfx --version
```

## Solución de problemas

| Problema | Causa | Solución |
|----------|-------|----------|
| `"Invalid SFX magic marker"` | Binario SFX corrupto | Reempaquetar desde el original |
| `"File too small to contain trailer"` | Archivo SFX truncado | Descargar o reempaquetar |
| `Permission denied` | Falta permiso de ejecución | `chmod +x <sfx>` |
| `memfd_create: Operation not permitted` | El kernel restringe memfd en contenedor | Asegurar `SYS_PTRACE` o kernel >= 3.17 |
| Windows: `"Failed to load DLL"` | Falta DLL en tiempo de ejecución | Instalar Visual C++ redistributable |
| macOS: `"Failed to create object file image"` | Problema de firma de código | Firmar el SFX o permitir ejecución sin firma |
