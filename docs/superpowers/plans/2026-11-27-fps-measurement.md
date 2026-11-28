# FPS medido (2b) — plano de implementação

> Execução inline (escolha do usuário em subprojetos anteriores). Spec: `docs/superpowers/specs/2026-11-27-fps-measurement-design.md`.

**Objetivo:** gravar FPS médio / 1% low / 0,1% low / frametime de cada sessão do Modo Gaming via PresentMon e mostrar antes → depois por jogo.

## Restrições globais
- Nunca abrir handle, injetar ou ler memória do jogo; só PresentMon (ETW).
- PresentMon v2.6.0, SHA-256 `b2a706bc6ad475749e3b7e3409263aa1e6906d45bdcf993f6dbc0f660188f1af`, conferido no build.
- Aquecimento 15 s; sessão válida ≥ 60 s medidos; histórico podado em 600 → 500 linhas.
- Executáveis do sistema por caminho absoluto; daemon continua sem janela.

## Tarefas

1. **`crates/oo-fps`** — `CsvReader`, `FrameStats` (histograma 0,05 ms × 5000 + estouro), `Summary`, `FpsRecord`, `append_history`, `read_history`. Testes primeiro (`tests/fps.rs`): cabeçalho/colunas por nome, linha inválida, aquecimento, percentis sintéticos, swap chain dominante, `valid`, poda.
2. **Empacotamento** — `scripts/fetch-presentmon.mjs` (download do asset fixo, SHA-256, `app/binaries/presentmon-<triple>.exe`, cache local), chamado por `prebuild`/`predev` e pelo CI antes do clippy; `externalBin` += `binaries/presentmon`; `THIRD_PARTY_NOTICES.md` com a licença MIT da Intel.
3. **Captura no daemon** — `oo-gaming-win::fps::FpsCapture` (spawn `CREATE_NO_WINDOW`, stdout em pipe, thread leitora, `finish` com espera de 5 s e kill); daemon inicia/finaliza nas transições de `status.game` e no desligamento; grava `fps-history.jsonl`.
4. **App** — `FpsRecordView` em `oo-views` (ts-rs) + comando `fps_history`.
5. **UI** — `lib/fpsCompare.ts` (+ teste) e seção "Desempenho medido" na tela Modo Gaming (sparkline, última sessão, antes → depois); i18n PT/EN.
6. **Release 0.3.0** — CHANGELOG PT/EN, README (recurso + roadmap), bump de versão; verificação: `cargo test`, `clippy -D warnings`, `npm run check`, `npm test`, build.
