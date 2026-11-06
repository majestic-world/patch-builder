# 04: Proteger conflitos de publicação

**What to build:** Impedir que Deploy substitua silenciosamente uma publicação remota mais nova, divergente ou alterada por outro publisher, permitindo substituição apenas após uma confirmação explícita baseada no conflito observado.

**Blocked by:** 02: Reutilizar Archives por hashes do Manifest.

**Status:** ready-for-agent

- [ ] Um Manifest remoto com versão maior que a local produz um conflito de possível rollback antes de qualquer substituição.
- [ ] Um Manifest remoto com a mesma versão e conteúdo diferente produz um conflito de divergência antes de qualquer substituição.
- [ ] A modal mostra versões local e remota, explica o risco e oferece cancelar ou substituir explicitamente.
- [ ] Silêncio, fechamento da modal ou confirmação de outro tipo nunca autorizam a substituição.
- [ ] Confirmar executa novamente a operação com autorização limitada ao conflito apresentado; uma mudança posterior exige nova decisão.
- [ ] O plano de Deploy mantém uma identidade do Manifest remoto inicialmente observado.
- [ ] Imediatamente antes de instalar o Manifest local, Deploy relê o remoto e exige que ele ainda corresponda à identidade observada.
- [ ] Mudança, criação ou remoção concorrente do Manifest aborta a publicação final e preserva o Manifest então visível.
- [ ] Archives já instalados antes de detectar concorrência podem permanecer, mas não são anunciados por um novo Manifest local.
- [ ] Um Manifest remoto mais antigo segue o fluxo incremental normal sem confirmação de rollback.
- [ ] Uma publicação idêntica continua retornando atualizada sem apresentar confirmação.
- [ ] Testes contra SFTP real cobrem versão remota maior, mesma versão divergente, cancelamento, substituição confirmada e alteração concorrente antes da publicação final.
- [ ] Um smoke da aplicação real prova que os diálogos de conflito são claros, acessíveis e não confundidos com confiança de chave SSH.
