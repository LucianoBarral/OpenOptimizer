# Política de segurança

O Open Optimizer roda com privilégios de administrador, então segurança é prioridade.

## Versões suportadas

Apenas a versão mais recente (atualização automática) recebe correções.

## Reportar uma vulnerabilidade

**Não abra uma issue pública.** Use o [aviso privado de segurança do GitHub](https://github.com/LucianoBarral/OpenOptimizer/security/advisories/new) com:

- descrição do problema e impacto;
- passos para reproduzir;
- versão afetada.

Respondemos assim que possível e creditamos quem reportar (se desejar).

## Garantias do projeto

- Atualizações são **assinadas** (minisign) e verificadas antes de instalar.
- O app **nunca** abre handle, injeta ou lê/escreve memória de processos de jogos.
- Executáveis do sistema são chamados por caminho absoluto do System32 (sem *binary planting*).
- Toda alteração no sistema é registrada e reversível.
