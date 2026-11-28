# Contribuindo com o Open Optimizer

Obrigado pelo interesse! Algumas regras mantêm o projeto confiável:

## Princípios (não negociáveis)

1. **Tudo reversível** — toda alteração no sistema passa pelo motor de reversão (`oo-core`), que grava o valor original antes de mudar.
2. **Nunca tocar no processo de um jogo** — sem `OpenProcess`, injeção, leitura ou escrita de memória. Detecção só por snapshot de nomes.
3. **Sem placebo** — toda otimização nova precisa de justificativa técnica (`rationale` no `tweaks.toml`) e, quando possível, de medição.
4. **Dados reais** — nenhum número fixo na interface.
5. **Executáveis do sistema por caminho absoluto** (`oo_sys::windows::system_exe`) — o app roda elevado.

## Fluxo

1. Abra uma issue descrevendo o problema ou a proposta.
2. Crie um branch a partir de `main` e escreva o teste antes do código.
3. Antes do PR, rode:
   ```powershell
   cargo fmt --all
   node scripts/build-daemon.mjs
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace
   npm --prefix ui run check ; npm --prefix ui test
   ```
4. Atualize o [CHANGELOG.md](CHANGELOG.md) e o [CHANGELOG.en.md](CHANGELOG.en.md) (seção da próxima versão, mesmos itens nos dois) e o README, se mudar algo visível.
5. Abra o PR — o CI precisa ficar verde.

## Adicionando uma otimização

- Registro/serviço: uma entrada em `crates/oo-catalog/tweaks.toml` (com `rationale`).
- Textos: chaves `tweak.<id>.title`, `.desc` e `.why` em `ui/src/lib/i18n/pt-BR.ts` **e** `en.ts` (o teste de paridade falha se faltar).
- Nada que exija reinício sem `requires_reboot = true`.
