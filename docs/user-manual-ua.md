# xsfx — Посібник користувача

> **xsfx** — Пакувальник самовидобувних виконуваних файлів

## Передумови

- Підтримувана платформа (Linux, macOS або Windows)
- Додаткові залежності не потрібні

## Встановлення

### Варіант 1: Готовий бінарний файл

Завантажте з [GitHub Releases](https://github.com/ratushnyi-labs/xsfx/releases):

| Платформа | Файл |
|-----------|------|
| Linux x64 (статичний) | `xsfx-x86_64-unknown-linux-musl.tar.gz` |
| Linux ARM64 (статичний) | `xsfx-aarch64-unknown-linux-musl.tar.gz` |
| macOS x64 | `xsfx-x86_64-apple-darwin.tar.gz` |
| macOS ARM64 (Apple Silicon) | `xsfx-aarch64-apple-darwin.tar.gz` |
| Windows x64 | `xsfx-x86_64-pc-windows-msvc.zip` |
| Windows ARM64 | `xsfx-aarch64-pc-windows-msvc.zip` |

```bash
# Linux / macOS
curl -sSfL https://github.com/ratushnyi-labs/xsfx/releases/latest/download/xsfx-x86_64-unknown-linux-musl.tar.gz \
    | tar xzf - -C /usr/local/bin
```

### Варіант 2: Збірка з вихідного коду

```bash
git clone https://github.com/ratushnyi-labs/xsfx.git
cd xsfx
cargo build --release --bin xsfx
```

## Використання

### Запакувати бінарний файл

```bash
xsfx <вхід> <вихід> [--target <трійка>]
```

- `вхід` — бінарний файл для пакування (використовуйте `-` для stdin)
- `вихід` — шлях для SFX-файлу (використовуйте `-` для stdout)
- `--target` — цільова платформа (за замовчуванням — поточна)

### Приклади

```bash
# Базове пакування
xsfx myapp myapp-sfx
chmod +x myapp-sfx
./myapp-sfx

# Пакування для іншої платформи
xsfx myapp myapp-sfx.exe --target x86_64-pc-windows-msvc

# Показати доступні цілі
xsfx
```

### Підтримка конвеєрів

Використовуйте `-` для stdin або stdout:

```bash
# Читання з stdin
cat myapp | xsfx - myapp-sfx

# Запис у stdout
xsfx myapp - > myapp-sfx

# Повний конвеєр
cat myapp | xsfx - - > myapp-sfx

# Пакування та розгортання через SSH
xsfx myapp - --target x86_64-unknown-linux-musl | ssh server 'cat > myapp && chmod +x myapp'
```

### Запуск запакованого SFX

Вихідний бінарний файл запускається як звичайний виконуваний файл. Усі аргументи командного рядка передаються корисному навантаженню:

```bash
./myapp-sfx --verbose --config /etc/myapp.conf
```

## Підтримувані цілі

| Ціль | Архітектура | Виконання в пам'яті |
|------|-------------|---------------------|
| `x86_64-unknown-linux-musl` | x64 | `memfd_create` + `execveat` |
| `aarch64-unknown-linux-musl` | ARM64 | `memfd_create` + `execveat` |
| `x86_64-apple-darwin` | x64 | `NSCreateObjectFileImageFromMemory` |
| `aarch64-apple-darwin` | ARM64 | `NSCreateObjectFileImageFromMemory` |
| `x86_64-pc-windows-msvc` | x64 | Завантажувач PE у процесі |
| `aarch64-pc-windows-msvc` | ARM64 | Завантажувач PE у процесі |

## Формат бінарного файлу

SFX-бінарний файл складається з трьох частин:

```
+------------------------+
| Stub                   |  платформо-специфічний завантажувач (<100 КБ)
+------------------------+
| Стиснуте навантаження  |  потік LZMA2/XZ
+------------------------+
| Трейлер (16 байт)      |  payload_len (u64 LE) + magic (u64 LE)
+------------------------+
```

Тимчасові файли під час видобування не створюються. Навантаження розпаковується та виконується повністю в пам'яті.

## Перевірка

Після пакування переконайтеся, що SFX працює коректно:

```bash
# Запакувати
xsfx myapp myapp-sfx

# Запустити оригінал
./myapp --version

# Запустити SFX — результат має бути ідентичним
./myapp-sfx --version
```

## Усунення проблем

| Проблема | Причина | Рішення |
|----------|---------|---------|
| `"Invalid SFX magic marker"` | Пошкоджений SFX-файл | Перепакуйте з оригінального файлу |
| `"File too small to contain trailer"` | Обрізаний SFX-файл | Завантажте або перепакуйте знову |
| `Permission denied` | Відсутній дозвіл на виконання | `chmod +x <sfx>` |
| `memfd_create: Operation not permitted` | Ядро обмежує memfd у контейнері | Додайте `SYS_PTRACE` або ядро >= 3.17 |
| Windows: `"Failed to load DLL"` | Відсутня DLL | Встановіть Visual C++ redistributable |
| macOS: `"Failed to create object file image"` | Проблема з підписом коду | Підпишіть SFX або дозвольте непідписані |
