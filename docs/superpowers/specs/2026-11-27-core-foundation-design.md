# Open Optimizer — Subprojeto 1: Núcleo, Interface e Motor de Reversão

**Data:** 2026-11-27 · **Status:** aprovado em brainstorming, aguardando revisão da spec
**Repositório:** https://github.com/LucianoBarral/OpenOptimizer (público)

## 1. Contexto e roadmap

Open Optimizer é um otimizador de PC para Windows 10/11, gratuito e open source. O produto inteiro foi decomposto em 5 subprojetos, cada um com spec → plano → implementação próprios:

| # | Subprojeto | Depende de |
|---|---|---|
| **1** | **Núcleo + interface + motor de reversão + auto-update** (este documento) | — |
| 2 | Desempenho & input lag + Modo Gaming (daemon) | 1 |
| 3 | Paridade com winutil (apps via winget, tweaks, features, config de Update) | 1 |
| 4 | Atualizador de drivers (Windows Update Agent API + endpoints públicos de GPU) | 1 |
| 5 | Memória & limpeza (standby list, working sets, temporários) | 1 |

### Princípios do produto (valem para todos os subprojetos)

1. **Tudo reversível.** Toda mudança grava o valor *original lido do sistema* antes de alterar; nunca um "padrão" hardcoded.
2. **Zero risco de ban.** O app **nunca** abre handle, injeta, lê ou escreve memória de processo de jogo. Atua apenas no SO via APIs públicas (registro, serviços, energia, MMCSS, rede, prioridade via `SetPriorityClass` do próprio Windows).
3. **Sem placebo.** Cada tweak tem justificativa técnica escrita no catálogo; o que não tem efeito mensurável não entra.
4. **Real, não simulado.** Todo número na interface vem de leitura real do sistema. Nada de valores fixos.
5. **Leve.** Instalador ≤ 15 MB, CPU em repouso ≈ 0%. RAM: o processo Rust ≤ 15 MB; o total com WebView2 tem piso de ~100 MB com a janela aberta (medido: 108 MB, Rust 7 MB) — por isso, minimizado, o WebView2 entra em modo de memória baixa. *(Meta original de 60 MB não tinha base em medição e foi corrigida em 2026-11-27.)*
6. **Driver updater sem engenharia reversa de terceiros.** Não usamos a API privada do Driver Booster (ToS + risco de segurança); usamos WUA (drivers assinados pela Microsoft) e lookups públicos de fabricantes de GPU.

## 2. Escopo deste subprojeto

**Entrega:** um app instalável que abre com o design aprovado, mostra métricas reais, aplica e desfaz um conjunto inicial de 5 tweaks reais via o motor de reversão, exibe o histórico com desfazer, e **se atualiza sozinho** a partir de Releases do GitHub (validado de ponta a ponta com v0.1.0 → v0.1.1).

**Fora de escopo:** Modo Gaming em segundo plano, drivers, winget, limpeza de memória. As telas correspondentes existem na navegação como "Em breve" (sem dados falsos).

## 3. Stack

| Camada | Escolha | Por quê |
|---|---|---|
| Shell | **Tauri 2** | Exe ~10 MB usando o WebView2 do Windows; updater oficial com assinatura |
| Backend | **Rust** (workspace Cargo), crate `windows` | Acesso nativo a Win32/COM/registro sem runtime |
| Frontend | **Svelte 5 + Vite + TypeScript**, CSS puro com tokens | Compila para JS sem runtime pesado (~30 KB); animações via CSS |
| Bindings | **ts-rs** | Tipos TS gerados das structs Rust (estável; tauri-specta ainda é RC) |
| i18n | Dicionários tipados `pt-BR` e `en` | Idioma detectado do SO, trocável nas configurações |
| CI/CD | GitHub Actions + `tauri-apps/tauri-action` | Build, assinatura e Release a partir de tag `v*` |

## 4. Estrutura do repositório

```
OpenOptimizer/
├─ Cargo.toml                 # workspace
├─ crates/
│  ├─ oo-core/                # motor: Tweak, Action, Journal, Engine (sem Tauri, testável)
│  ├─ oo-sys/                 # trait SystemBackend + impl Windows real + impl Fake (testes)
│  ├─ oo-metrics/             # leituras reais: CPU, memória, timer resolution, info de HW
│  └─ oo-catalog/             # catálogo declarativo de tweaks (TOML embutido) + tweaks em código
├─ app/                       # crate Tauri (src-tauri): comandos IPC, updater, manifest admin
├─ ui/                        # Svelte 5
│  ├─ src/lib/design/         # tokens.css, componentes (Switch, Button, Card, Ring, Sparkline…)
│  ├─ src/lib/i18n/           # pt-BR.ts, en.ts
│  ├─ src/lib/ipc/            # bindings gerados pelo ts-rs + wrapper tipado de invoke
│  └─ src/routes/             # overview, performance, history, settings, (em breve: gaming, memory, drivers, apps)
├─ .github/workflows/         # ci.yml (PR), release.yml (tag)
└─ docs/
```

**Fronteira principal:** `oo-core` não conhece Windows nem Tauri; ele fala com o sistema só através do trait `SystemBackend` de `oo-sys`. Isso permite testar todo o motor de reversão com um backend falso, sem tocar no registro real.

## 5. Motor de reversão (coração do projeto)

### 5.1 Modelo

- **Action** — operação atômica e primitiva sobre o sistema. Cada uma sabe **capturar o estado anterior** e **restaurá-lo**:
  - `RegistrySet { hive, path, name, value }` — captura: valor anterior *ou* "não existia" (revert apaga).
  - `RegistryDelete { … }` — captura: valor completo + tipo.
  - `ServiceStartType` — adiado para o subprojeto 3 (nenhum tweak inicial precisa).
  - `PowerSchemeDuplicateAndActivate { source_guid }` — captura: GUID ativo anterior; revert reativa o anterior e remove o duplicado.
- **Tweak** — unidade que o usuário vê (um toggle). Metadados: `id`, títulos/descrições i18n, `category` (Latência, FPS, Energia, Rede, Entrada, Privacidade…), `risk` (`safe` | `advanced`), `requires_reboot`, `rationale` (justificativa técnica). Possui `detect()` → `Applied | NotApplied | Partial | Unsupported` e uma lista de Actions.
- **Change** — registro no journal: `{ id (ULID), tweak_id, batch_id, timestamp (RFC 3339 UTC), actions: [{ action, before, after }], status: Pending | Applied | Reverted | Failed }`.

### 5.2 Journal (write-ahead)

- Local: `%ProgramData%\OpenOptimizer\journal.jsonl` (append-only; uma linha por evento).
- **Antes** de executar cada Action, grava a linha com `before` capturado e status `Pending`; depois grava `Applied`. Se o app cair no meio, na próxima abertura o motor encontra `Pending` e oferece reverter.
- Cada linha é escrita e seguida de `FlushFileBuffers`; a leitura tolera uma última linha truncada (descarta-a).

### 5.3 Execução em lote

- "Otimizar" aplica N tweaks como um **batch**. Se a Action k de um tweak falhar, o motor reverte automaticamente as Actions já aplicadas **daquele tweak** e o marca como `Failed`; os demais tweaks do batch continuam (falha isolada). O resultado volta para a UI tweak a tweak (eventos de progresso).
- Antes do primeiro batch de uma sessão: tenta criar **Ponto de Restauração do Windows** (WMI `SystemRestore.CreateRestorePoint`). Se o Windows recusar (limite de 24h, proteção desativada), registra aviso e segue — o journal próprio é a garantia primária.

### 5.4 Reverter

- Por tweak (toggle off), por batch ("desfazer esta otimização") ou **tudo** ("Restaurar tudo ao original").
- Revert aplica os `before` em ordem inversa. Se o valor atual já não for o `after` que gravamos (usuário/outro app mudou), **não sobrescreve às cegas**: marca conflito e pergunta na UI.

### 5.5 Catálogo

- Tweaks simples (só registro/serviço) são **declarativos em TOML** embutido no binário — mesmo modelo que permite portar o catálogo do winutil no subprojeto 3 com esforço linear.
- Tweaks complexos (ex.: plano de energia) são implementados em Rust e registrados no mesmo catálogo.

### 5.6 Tweaks iniciais (validam o motor; todos seguros e anti-cheat-neutros)

| Tweak | Mecanismo | Categoria |
|---|---|---|
| Desativar power throttling | `HKLM\SYSTEM\CurrentControlSet\Control\Power\PowerThrottling` `PowerThrottlingOff=1` | Energia |
| MMCSS priorizar jogos | `…\Multimedia\SystemProfile` `SystemResponsiveness=10`; `…\Tasks\Games` `GPU Priority=8`, `Priority=6`, `Scheduling Category=High` | FPS |
| Plano Desempenho Máximo | `powercfg -duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61` + ativar; revert reativa o anterior e remove | Energia |
| Desativar gravação em segundo plano (Game DVR) | `HKCU\System\GameConfigStore` `GameDVR_Enabled=0`; `HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR` `AppCaptureEnabled=0` | FPS |
| Desativar aceleração do mouse | `HKCU\Control Panel\Mouse` `MouseSpeed="0"`, `MouseThreshold1="0"`, `MouseThreshold2="0"` + `SystemParametersInfo(SPI_SETMOUSE)` para aplicar sem logoff | Entrada |

## 6. Métricas reais (oo-metrics)

| Métrica | Fonte | Frequência |
|---|---|---|
| CPU % | `GetSystemTimes` (delta) | 1 s, só com janela visível |
| Memória em uso / total | `GlobalMemoryStatusEx` | 1 s |
| Timer resolution atual | `NtQueryTimerResolution` | 1 s |
| Hardware (CPU, GPU, RAM) | WMI `Win32_Processor`, `Win32_VideoController`, `Win32_PhysicalMemory` | uma vez ao abrir |

Temperatura de GPU fica para o subprojeto 2 (exige NVAPI/ADLX). O card de GPU mostra apenas o modelo até lá.

**Pontuação:** `100 × (peso dos tweaks recomendados aplicados) / (peso dos tweaks recomendados suportados neste PC)`. Derivada 100% do `detect()` real — nunca um número fixo. O polling de métricas para quando a janela é minimizada.

## 7. Interface

- **Design aprovado:** direção "Glacier sem brilho" (mockup v3). Sem glow, blur ou degradê decorativo; profundidade por 3 superfícies (`#0B0C0E`, `#0F1013`, `#121317`), linhas de 1px (`#1C1E23`, `#25282E`), branco gelo `#EEF3F7` como único acento, fonte **Geist** + **Geist Mono** (arquivos embutidos, sem rede).
- **Movimento:** `--ease: cubic-bezier(.2,.8,.2,1)` e `--spring: cubic-bezier(.34,1.4,.64,1)`; só `transform`/`opacity` animam; entrada escalonada, indicador de navegação deslizante, anel com contagem, toggles com mola; `prefers-reduced-motion` desliga tudo.
- **Componente Switch:** `flex: none` obrigatório e deslocamento do knob calculado a partir de largura/altura do trilho (correção do bug do mockup, onde o knob saía do trilho quando o flexbox espremia o toggle).
- **Telas deste subprojeto:** Visão geral, Desempenho (5 tweaks com toggle + justificativa expansível), Histórico (journal com desfazer por item/lote/tudo), Configurações (idioma, verificar atualização, sobre). Demais itens da sidebar aparecem com selo "Em breve".
- **Janela:** sem moldura nativa (titlebar própria), tamanho mínimo 1100×700.

## 8. Privilégios

O app roda elevado (`requireAdministrator` no manifest), como o winutil — quase todo tweak exige admin e isso evita um serviço auxiliar neste subprojeto. O daemon do Modo Gaming (subprojeto 2) será um processo separado iniciado por Tarefa Agendada com "executar com privilégios mais altos", sem UAC a cada boot.

## 9. Auto-update via GitHub

1. Push de tag `vX.Y.Z` → `release.yml` (runner `windows-latest`) compila, gera instalador **NSIS**, assina o pacote de update com a chave **minisign** (`TAURI_SIGNING_PRIVATE_KEY` em GitHub Secrets) e publica Release com `latest.json`.
2. O app consulta `https://github.com/LucianoBarral/OpenOptimizer/releases/latest/download/latest.json` ao abrir e a cada 6 h.
3. Havendo versão nova: banner discreto "Atualização X.Y.Z disponível · Ver novidades · Atualizar". Baixa em segundo plano, **verifica a assinatura**, instala em modo passivo e reinicia. Nunca atualiza no meio de um batch de tweaks.
4. Falha de rede/assinatura → banner some silenciosamente, detalhe em Configurações; nunca bloqueia o uso.
5. Fonte única da versão: `app/tauri.conf.json`; um script de release sincroniza `Cargo.toml` e `ui/package.json`.

Assinatura Authenticode (remover aviso do SmartScreen no primeiro download) fica como melhoria futura via SignPath (gratuito para OSS); não bloqueia este subprojeto.

## 10. Tratamento de erros

- Erros do backend são um enum `OoError` serializável (`AccessDenied`, `NotSupported`, `Conflict`, `Io`, `Win32 { code }`, `Internal`), mapeado para mensagens i18n na UI — nunca string crua do Windows para o usuário.
- Tweak com `detect() = Unsupported` (ex.: chave inexistente nesta edição do Windows) aparece desabilitado com motivo e não conta na pontuação.
- Panic no backend é capturado no limite do comando Tauri e vira `OoError::Internal` + log em `%ProgramData%\OpenOptimizer\logs\`.

## 11. Testes e verificação

- **oo-core** — testes unitários com `FakeBackend` (registro em memória): apply → journal → revert restaura exatamente o `before`; valor inexistente volta a inexistir; conflito detectado; recuperação de `Pending` após "crash" simulado; falha no meio de um tweak reverte suas Actions.
- **oo-sys (Windows real)** — testes de integração em `HKCU\Software\OpenOptimizer\_test` no runner `windows-latest` do CI (cria, altera, reverte, verifica, limpa).
- **UI** — `svelte-check` + Vitest para lógica (i18n, formatação, pontuação).
- **Fim a fim (manual, antes de fechar o subprojeto):** instalar v0.1.0, aplicar os 5 tweaks, conferir no `regedit`/`powercfg /getactivescheme`, reverter tudo e conferir valores originais; publicar v0.1.1 e observar o app se atualizar sozinho.
- **Orçamento de leveza medido:** tamanho do instalador e RAM em repouso (janela aberta 60 s) registrados no PR de release.

## 12. Critérios de pronto

- [ ] CI verde (fmt, clippy `-D warnings`, testes Rust, svelte-check, vitest).
- [ ] Os 5 tweaks aplicam e revertem no Windows real com valores conferidos.
- [ ] Recuperação de journal `Pending` funciona após matar o processo no meio de um batch.
- [ ] Auto-update v0.1.0 → v0.1.1 observado de ponta a ponta.
- [x] Instalador ≤ 15 MB (medido: 2,0 MB).
- [x] CPU em repouso ≈ 0% (medido: 0,1–0,26% com métricas ao vivo).
- [x] RAM (v0.1.1, `scripts/measure-memory.ps1`): janela aberta 111,2 MB (meta ≤ 120 MB); **minimizada 23,5 MB** (−79%, WebView2 em memória baixa); processo Rust 5,8 MB.
- [ ] Interface em pt-BR e en; `prefers-reduced-motion` respeitado.
