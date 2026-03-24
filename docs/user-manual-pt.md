# xsfx — Manual do utilizador

> **xsfx** — Empacotador de executáveis autoextraíveis

## Pré-requisitos

- Uma plataforma suportada (Linux, macOS ou Windows)
- Sem dependências de runtime necessárias

## Instalação

### Opção 1: Binário pré-compilado

Descarregue a partir de [GitHub Releases](https://github.com/ratushnyi-labs/xsfx/releases):

| Plataforma | Ficheiro |
|------------|----------|
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

### Opção 2: Compilar a partir do código-fonte

```bash
git clone https://github.com/ratushnyi-labs/xsfx.git
cd xsfx
cargo build --release --bin xsfx
```

## Utilização

### Empacotar um binário

```bash
xsfx <entrada> <saída> [--target <triplo>]
```

- `entrada` — binário a empacotar (use `-` para stdin)
- `saída` — caminho do ficheiro SFX (use `-` para stdout)
- `--target` — plataforma de destino (predefinido: a plataforma atual)

### Exemplos

```bash
# Empacotamento básico
xsfx myapp myapp-sfx
chmod +x myapp-sfx
./myapp-sfx

# Empacotar para outra plataforma
xsfx myapp myapp-sfx.exe --target x86_64-pc-windows-msvc

# Listar destinos disponíveis
xsfx
```

### Suporte de pipes

Use `-` para stdin ou stdout:

```bash
# Ler a partir de stdin
cat myapp | xsfx - myapp-sfx

# Escrever para stdout
xsfx myapp - > myapp-sfx

# Pipe completo
cat myapp | xsfx - - > myapp-sfx

# Empacotar e implementar via SSH
xsfx myapp - --target x86_64-unknown-linux-musl | ssh server 'cat > myapp && chmod +x myapp'
```

### Executar o SFX empacotado

O binário resultante executa-se como qualquer executável normal. Todos os argumentos da linha de comandos são reencaminhados para o payload:

```bash
./myapp-sfx --verbose --config /etc/myapp.conf
```

## Destinos suportados

| Destino | Arquitetura | Execução em memória |
|---------|------------|---------------------|
| `x86_64-unknown-linux-musl` | x64 | `memfd_create` + `execveat` |
| `aarch64-unknown-linux-musl` | ARM64 | `memfd_create` + `execveat` |
| `x86_64-apple-darwin` | x64 | `NSCreateObjectFileImageFromMemory` |
| `aarch64-apple-darwin` | ARM64 | `NSCreateObjectFileImageFromMemory` |
| `x86_64-pc-windows-msvc` | x64 | Carregador PE em processo |
| `aarch64-pc-windows-msvc` | ARM64 | Carregador PE em processo |

## Formato do binário

O binário SFX é composto por três partes:

```
+------------------------+
| Stub                   |  carregador específico da plataforma (<100 KB)
+------------------------+
| Payload comprimido     |  fluxo LZMA2/XZ
+------------------------+
| Trailer (16 bytes)     |  payload_len (u64 LE) + magic (u64 LE)
+------------------------+
```

Não são criados ficheiros temporários durante a extração. O payload é descomprimido e executado inteiramente em memória.

## Verificação

Após o empacotamento, verifique que o SFX funciona corretamente:

```bash
# Empacotar
xsfx myapp myapp-sfx

# Executar o original
./myapp --version

# Executar o SFX — deve produzir um resultado idêntico
./myapp-sfx --version
```

## Resolução de problemas

| Problema | Causa | Solução |
|----------|-------|---------|
| `"Invalid SFX magic marker"` | Binário SFX corrompido | Reempacotar a partir do original |
| `"File too small to contain trailer"` | Ficheiro SFX truncado | Descarregar ou reempacotar |
| `Permission denied` | Permissão de execução em falta | `chmod +x <sfx>` |
| `memfd_create: Operation not permitted` | Kernel restringe memfd no contentor | Adicionar `SYS_PTRACE` ou kernel >= 3.17 |
| Windows: `"Failed to load DLL"` | DLL de runtime em falta | Instalar Visual C++ redistributable |
| macOS: `"Failed to create object file image"` | Problema de assinatura de código | Assinar o SFX ou permitir execução não assinada |
