# CastIt — INIT.md

## 1. Visión del Producto

**CastIt** es un launcher de comandos para desarrolladores en Linux. Un punto único de entrada al sistema operativo — rápido, bonito, offline-first y respetuoso con la privacidad.

**No es** un reemplazo genérico de Rofi. Es una **herramienta de productividad para devs** que combina:
- Lanzamiento de aplicaciones (`.desktop` files)
- Herramientas de desarrollo integradas (Base64, JWT, JSON, timestamps)
- IA bajo demanda (local o con tu propia API key)
- Gestión rápida de Docker

**Posicionamiento**: El Raycast que Linux nunca tuvo. Ni un launcher minimalista en texto plano, ni una app web disfrazada de escritorio.

---

## 2. Análisis del Mercado

| Herramienta | Stack | UI Rica | Rendimiento | Extensibilidad | Estado |
|---|---|---|---|---|---|
| Rofi / Wofi / Fuzzel | C | ❌ Texto plano | ✅ ~10MB RAM | ⚠️ Scripts Bash | Estable, estancado |
| Ulauncher | Python/GTK | ⚠️ GTK3, anticuado | ⚠️ ~80MB RAM | ✅ Extensions Python | Estable, lento |
| Albert | C++/Qt | ⚠️ Qt clásico | ✅ ~30MB RAM | ✅ Plugins C++ | Estable |
| Kunkun | Tauri/TS | ✅ Web-like | ⚠️ ~100MB RAM | ✅ JS Extensions | Nuevo, inestable |
| Flare | Varies | ✅ Intenta copiar Raycast | ⚠️ Variable | ⚠️ En desarrollo | Nuevo, inestable |

**El hueco**: No existe un launcher en Linux que combine rendimiento nativo (<30MB RAM), UI moderna con renderizado GPU, y herramientas de desarrollo integradas.

---

## 3. Principios de Diseño (No Negociables)

Estos principios guían TODA decisión técnica y de producto. Si una feature los viola, no entra.

1. **Keyboard-first**: Toda acción se completa sin tocar el ratón. El ratón es un *fallback*, no el flujo principal.
2. **Zero-friction**: De pulsar el atajo a tener el resultado, máximo 2 pasos. Si algo requiere 3 clics, el diseño está mal.
3. **Offline-first**: Toda funcionalidad core funciona sin internet. La IA es el único módulo que puede requerir red (y solo si el usuario elige un proveedor cloud).
4. **Privacy by default**: Cero telemetría, cero datos a terceros. El usuario trae su propia API key o usa un modelo local.
5. **Native performance**: El launcher debe sentirse parte del SO, no una app montada encima. Arranque instantáneo, uso de RAM imperceptible.
6. **Developer-centric**: Cada decisión de UX se toma pensando en un desarrollador que usa terminal, Docker, Git y APIs a diario.

---

## 4. Stack Técnico

### 4.1 Lenguaje: Rust

**Por qué Rust y no otra cosa:**
- Control de memoria sin GC → arranque instantáneo, ~15-25MB RAM
- Ecosistema Wayland maduro (smithay, wayland-client, layer-shell)
- Binarios estáticos, sin runtime → distribución trivial
- El estándar de facto para herramientas de sistema en Linux (ripgrep, fd, alacritty, helix, zed, cosmic-desktop)

### 4.2 Framework UI: Iced + `iced_layershell`

**Iced** es un framework GUI declarativo para Rust inspirado en Elm (The Elm Architecture — TEA).

**Por qué Iced:**
- Renderizado GPU via `wgpu` (Vulkan/OpenGL) → animaciones fluidas, gradientes, sombras
- Arquitectura TEA (Model → View → Message → Update) → estado predecible y testeable
- System76 lo usa para todo COSMIC Desktop → mantenimiento activo y soporte Wayland maduro
- `iced_layershell`: crate que permite crear superficies `wlr-layer-shell` (overlays, paneles, launchers) → integración nativa con Wayland sin hacks

**Versiones verificadas**: `iced` 0.14 + `iced_layershell` 0.18.1

**Limitación conocida**: `iced_layershell` usa el protocolo `wlr-layer-shell-unstable-v1`, que funciona en compositores wlroots (Sway, Hyprland, River). No funciona en GNOME/Mutter ni KDE/KWin sin un fallback a ventana winit estándar.

### 4.3 Crates Clave del Ecosistema

| Responsabilidad | Crate | Notas |
|---|---|---|
| UI Framework | `iced` 0.14 | Core del rendering y widgets |
| Wayland Layer Shell | `iced_layershell` 0.18.1 | Overlay nativo en Wayland |
| Fuzzy Search | `nucleo` | El motor de Helix editor. Smith-Waterman, ultrarrápido |
| .desktop Files | `freedesktop-desktop-entry` | Parsing + scan de paths XDG del sistema |
| XDG Paths | `xdg` | Localización de config, data, cache |
| Serialización | `serde` + `serde_json` | JSON tools, config, API Ollama |
| HTTP Client | `reqwest` | Peticiones a Ollama / APIs externas |
| Async Runtime | `tokio` | Runtime async (Iced lo soporta) |
| Clipboard | `arboard` | Acceso al portapapeles del sistema |
| JWT Decode | `jsonwebtoken` | Decodificación de tokens JWT |
| Base64 | `base64` | Encode/decode Base64 |
| Timestamps | `chrono` | Conversión de Unix timestamps |
| Config File | `toml` + `serde` | Configuración del usuario en TOML |
| Logging | `tracing` + `tracing-subscriber` | Logging estructurado |
| Error Handling | `thiserror` / `anyhow` | Errores tipados (lib) / ergonómicos (app) |
| IPC (Toggle) | `interprocess` o socket Unix raw | Comunicación daemon ↔ CLI |

---

## 5. Arquitectura

### 5.1 Patrón General: Hexagonal (Ports & Adapters)

La Clean Architecture de Kotlin/Android se traduce en Rust como **Hexagonal Architecture**:

- **Domain**: Modelos puros, traits (interfaces), lógica de negocio. **Cero dependencias externas.**
- **Infrastructure (Adapters)**: Implementaciones concretas de los traits del dominio — acceso a ficheros `.desktop`, clipboard, API de Ollama, Docker socket, IPC.
- **Presentation (UI)**: La capa Iced — theme, widgets, views. Consume el dominio, nunca la infra directamente.

```
┌─────────────────────────────────────────────────┐
│                  Presentation                   │
│         (Iced TEA: State → View → Msg)          │
│                                                 │
│   ┌─────────┐  ┌──────────┐  ┌──────────────┐  │
│   │ Launcher │  │  Tools   │  │   AI View    │  │
│   │  View    │  │  View    │  │              │  │
│   └────┬─────┘  └────┬─────┘  └──────┬───────┘  │
│        │             │               │          │
├────────┴─────────────┴───────────────┴──────────┤
│                    Domain                       │
│              (Traits + Models)                  │
│                                                 │
│  ┌──────────────┐ ┌────────────┐ ┌───────────┐  │
│  │ AppRepository│ │ AiProvider │ │ Clipboard  │  │
│  │   (trait)    │ │  (trait)   │ │  (trait)   │  │
│  └──────┬───────┘ └─────┬──────┘ └─────┬─────┘  │
│         │               │              │        │
├─────────┴───────────────┴──────────────┴────────┤
│                Infrastructure                   │
│               (Adapters)                        │
│                                                 │
│  ┌──────────────┐ ┌────────────┐ ┌───────────┐  │
│  │DesktopFiles  │ │  Ollama    │ │  Arboard   │  │
│  │  Parser      │ │  Client    │ │  Clipboard │  │
│  └──────────────┘ └────────────┘ └───────────┘  │
└─────────────────────────────────────────────────┘
```

### 5.2 Patrón UI: The Elm Architecture (TEA)

Iced impone TEA de forma natural. El flujo es:

```
State (struct) ─── view() ──→ UI (widgets)
     ↑                           │
     │                      user interaction
     │                           │
     │                           ▼
     └──── update(msg) ◄─── Message (enum)
```

- **State**: Un struct con todo el estado de la aplicación (query actual, resultados, vista activa, estado de loading de la IA…).
- **Message**: Un enum que representa TODOS los eventos posibles (QueryChanged, ResultSelected, AiResponseReceived…).
- **update()**: Función pura que recibe el estado + mensaje y devuelve el nuevo estado + efectos secundarios (Commands).
- **view()**: Función pura que recibe el estado y devuelve la UI.

> **Nota**: TEA sustituye a los ViewModels de Android/MVVM. No hay ViewModels. No hay LiveData. No hay StateFlow. El `update()` ES tu ViewModel.

### 5.3 Patrón de Sistema: Daemon + IPC (Toggle)

```
┌──────────────────┐     Unix Socket      ┌───────────────────┐
│  castit toggle   │ ──────────────────▶  │   CastIt Daemon   │
│  (CLI invocation)│                       │  (Iced app loop)  │
│                  │                       │                   │
│  Ejecutado por   │                       │  Recibe "toggle"  │
│  el atajo del DE │                       │  → show/hide      │
└──────────────────┘                       │     ventana       │
                                           └───────────────────┘
```

**Flujo**:
1. El usuario configura en su DE (Sway/Hyprland) un atajo: `Super+Space → castit toggle`
2. `castit toggle` envía un byte por un Unix Domain Socket
3. El daemon (que ya está corriendo) recibe la señal y alterna la visibilidad de la ventana layer-shell
4. Cuando la ventana pierde el foco → se oculta automáticamente

---

## 6. Estructura de Directorios

```
castit/
├── Cargo.toml                 # Dependencias y metadata del proyecto
├── Cargo.lock
├── INIT.md                    # Este documento
├── README.md
├── LICENSE                    # MIT
│
├── src/
│   ├── main.rs                # Entry point: parseo de CLI args + arranque daemon/toggle
│   ├── app.rs                 # Implementación del Application de Iced (TEA root)
│   ├── config.rs              # Carga de configuración (~/.config/castit/config.toml)
│   │
│   ├── domain/                # === CAPA DE DOMINIO (pura, sin deps externas) ===
│   │   ├── mod.rs
│   │   ├── models.rs          # AppEntry, ToolResult, AiQuery, AiResponse...
│   │   ├── ports.rs           # Traits: AppRepository, ClipboardPort, AiProvider...
│   │   └── search.rs          # Lógica de búsqueda y scoring (usa nucleo)
│   │
│   ├── infra/                 # === ADAPTADORES (implementaciones concretas) ===
│   │   ├── mod.rs
│   │   ├── desktop.rs         # Parser de .desktop files + scan XDG paths
│   │   ├── clipboard.rs       # Implementación de ClipboardPort con arboard
│   │   ├── ipc.rs             # Unix socket: daemon listener + toggle sender
│   │   ├── docker.rs          # Docker Engine API via socket local
│   │   └── ai/
│   │       ├── mod.rs
│   │       ├── ollama.rs      # Implementación de AiProvider para Ollama
│   │       └── openai.rs      # Implementación de AiProvider para API compatible OpenAI
│   │
│   ├── tools/                 # === HERRAMIENTAS DEV (funciones puras) ===
│   │   ├── mod.rs
│   │   ├── base64.rs          # Encode/decode Base64
│   │   ├── jwt.rs             # Decode JWT + mostrar claims
│   │   ├── json_fmt.rs        # Format / minify JSON
│   │   └── timestamp.rs       # Unix timestamp ↔ fecha legible
│   │
│   └── ui/                    # === CAPA DE PRESENTACIÓN (Iced) ===
│       ├── mod.rs
│       ├── theme.rs           # Design tokens, paleta, tipografías
│       ├── components/        # Widgets reutilizables
│       │   ├── mod.rs
│       │   ├── search_input.rs
│       │   ├── result_item.rs
│       │   └── preview_panel.rs
│       └── views/             # Pantallas/modos de la paleta
│           ├── mod.rs
│           ├── launcher.rs    # Vista de lanzamiento de apps
│           ├── tools.rs       # Vista de herramientas dev
│           └── ai.rs          # Vista de respuesta IA
│
└── resources/
    └── icons/                 # Iconos propios de la app (si los hay)
```

---

## 7. Historias de Usuario (MVP)

### Epic 1: Motor Gráfico y Sistema

> **US-1.1**: Como desarrollador, quiero pulsar un atajo de teclado y que el launcher aparezca centrado en mi pantalla en menos de 100ms, para no romper mi flujo de trabajo.
>
> **Criterios de aceptación:**
> - La ventana aparece centrada, sin bordes, con fondo semitransparente
> - El cursor está ya posicionado en el campo de búsqueda
> - Funciona en Sway, Hyprland (wlroots). Fallback winit para X11.

> **US-1.2**: Como desarrollador, quiero que el launcher desaparezca cuando pulso `Escape` o hago clic fuera de él, para volver a mi contexto inmediatamente.
>
> **Criterios de aceptación:**
> - `Escape` oculta la ventana y limpia el input
> - Clic fuera (pérdida de foco) oculta la ventana
> - El proceso sigue vivo en background (daemon)

> **US-1.3**: Como desarrollador, quiero que el launcher arranque automáticamente con mi sesión y consuma menos de 25MB de RAM en reposo.
>
> **Criterios de aceptación:**
> - Proporcionamos un `.desktop` file para autostart
> - En reposo (ventana oculta), consumo < 25MB RSS

### Epic 2: Lanzador de Aplicaciones

> **US-2.1**: Como desarrollador, quiero escribir el nombre de una aplicación y que aparezcan resultados relevantes mientras escribo (fuzzy search), para encontrar apps sin recordar el nombre exacto.
>
> **Criterios de aceptación:**
> - La búsqueda es fuzzy (ej: "firef" encuentra "Firefox")
> - Los resultados aparecen en <16ms tras cada pulsación
> - Se muestran el icono y nombre de la app

> **US-2.2**: Como desarrollador, quiero pulsar `Enter` sobre un resultado para lanzar la aplicación y que el launcher se oculte automáticamente.
>
> **Criterios de aceptación:**
> - La app seleccionada se lanza como proceso hijo independiente
> - El launcher se oculta tras lanzar
> - Navegación con flechas ↑↓ entre resultados

### Epic 3: Herramientas de Desarrollo Offline

> **US-3.1**: Como desarrollador, quiero pegar un JSON y formatearlo/minificarlo directamente en el launcher, para no abrir un navegador o una herramienta externa.
>
> **Criterios de aceptación:**
> - Detección automática de JSON al pegar, o prefijo `json:`
> - Botones "Format" / "Minify"
> - Resultado copiado al portapapeles con un atajo o `Enter`

> **US-3.2**: Como desarrollador, quiero decodificar un JWT pegándolo en el launcher, para inspeccionar claims sin salir de mi flujo.
>
> **Criterios de aceptación:**
> - Detección automática de tokens JWT, o prefijo `jwt:`
> - Muestra header y payload decodificados
> - Indica si el token está expirado

> **US-3.3**: Como desarrollador, quiero convertir timestamps Unix a fechas legibles y viceversa.
>
> **Criterios de aceptación:**
> - Detección de números que parecen timestamps (10+ dígitos)
> - Muestra la fecha en formato ISO 8601 y local
> - Permite copiar el resultado

> **US-3.4**: Como desarrollador, quiero codificar/decodificar texto en Base64.
>
> **Criterios de aceptación:**
> - Prefijo `b64:` o detección de strings Base64 válidos
> - Muestra encode y decode lado a lado
> - Copiar resultado con atajo

### Epic 4: IA Integrada

> **US-4.1**: Como desarrollador, quiero escribir `ai: genera un regex para validar emails` y recibir directamente el código, sin interfaz de chat.
>
> **Criterios de aceptación:**
> - Prefijo `ai:` activa el modo IA
> - Se muestra un indicador de carga mientras se espera
> - La respuesta se muestra con syntax highlighting
> - Botón / atajo para copiar al portapapeles
> - Funciona con Ollama local y con APIs compatibles OpenAI

> **US-4.2**: Como desarrollador, quiero configurar mi proveedor de IA (Ollama local o API key propia) en un fichero de configuración, sin que la app envíe nada sin mi consentimiento.
>
> **Criterios de aceptación:**
> - Config en `~/.config/castit/config.toml`
> - Si no hay proveedor configurado, el prefijo `ai:` muestra un mensaje explicativo, no un error críptico

---

## 8. Requisitos No Funcionales

| Requisito | Objetivo | Medición |
|---|---|---|
| **Arranque visible** | < 100ms desde toggle hasta ventana pintada | Medido con `time castit toggle` + timestamp de primer frame |
| **RAM en reposo** | < 25MB RSS con ventana oculta | `ps -o rss` |
| **RAM activo** | < 50MB RSS con ventana visible y resultados | `ps -o rss` |
| **Latencia de búsqueda** | < 16ms por keystroke (60fps) | Profiling con `tracing` |
| **Tamaño del binario** | < 15MB (release, stripped) | `ls -lh` |
| **Compatibilidad** | Sway, Hyprland, River (wlroots). X11 fallback best-effort | Testing manual en cada compositor |

---

## 9. Riesgos Técnicos Identificados

| Riesgo | Impacto | Mitigación |
|---|---|---|
| `iced_layershell` no soporta GNOME/KDE | Alto — no funciona en esos DEs | Documentar como limitación. Evaluar `ext-layer-shell-v1` cuando se estandarice. Fallback winit para X11 |
| Curva de aprendizaje de Rust | Medio — ralentiza el desarrollo inicial | Empezar con el módulo `tools/` (funciones puras, sin UI). Ir de lo simple a lo complejo |
| Renderizado de iconos de apps | Bajo — algunos `.desktop` no tienen icono | Tener un icono placeholder por defecto |
| API de Ollama cambia | Bajo | Abstraer detrás del trait `AiProvider`, cambio localizado |

---

## 10. Roadmap de Fases

| Fase | Contenido | Estado |
|---|---|---|
| **Fase 0: Bootstrap** | Cargo init, ventana layer-shell, input + Escape | ✅ En progreso |
| **Fase 1: Launcher** | .desktop parser, fuzzy search, lanzar apps, autostart | Pendiente |
| **Fase 2: Dev Tools** | Base64, JWT, JSON, Timestamps | Pendiente |
| **Fase 3: IA** | Ollama client, prefijo `ai:`, copy to clipboard | Pendiente |
| **Fase 4: Polish** | Theme/animaciones, Docker manager, config avanzada | Pendiente |

---

## 11. Decisiones Tomadas

| Decisión | Elección | Razonamiento |
|---|---|---|
| Nombre del proyecto | CastIt | Fácil de recordar, guiño a Raycast |
| Licencia | MIT | Permisiva, compatible con uso comercial B2B |
| Lenguaje | Rust | Rendimiento nativo, ecosistema Wayland maduro |
| UI Framework | Iced + iced_layershell | TEA, GPU rendering, soporte layer-shell nativo |
| Target principal | Compositores wlroots (Sway, Hyprland) | Audiencia dev-first usa tiling WMs mayoritariamente |
