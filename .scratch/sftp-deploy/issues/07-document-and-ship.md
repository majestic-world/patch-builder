# 07: Documentar e validar o Deploy distribuído

**What to build:** Entregar o SFTP Deploy como parte documentada e verificável do Patch Builder distribuído, registrando seus contratos, limites e segurança e provando o fluxo completo no executável otimizado entregue ao publisher.

**Blocked by:** 06: Integrar Deploy ao fluxo desktop.

**Status:** ready-for-agent

- [ ] A documentação de uso explica configuração, autenticação por senha e chave, confiança no host, Deploy incremental, progresso, conflitos, cancelamento e resultado.
- [ ] A documentação deixa claro que Launchers continuam consumindo Manifest e Archives estáticos por HTTP ou CDN e que SFTP é somente o transporte do publisher.
- [ ] O glossário define Deploy e remote Update tree e mantém Build como a operação que produz a Update tree local.
- [ ] Um ADR registra comparação por hashes do Manifest, upload temporário, Manifest por último, confiança no índice remoto e a política deliberada de não excluir Archives remotos.
- [ ] A documentação alerta que confiança no primeiro fingerprint é uma decisão de segurança e que mudança de chave deve ser verificada fora do aplicativo.
- [ ] A documentação informa que um Archive remoto existente e listado com hash correspondente não é baixado e re-hashado.
- [ ] A documentação informa que Archives remotos obsoletos permanecem consumindo armazenamento mesmo quando saem do Manifest atual.
- [ ] Toda a suíte Rust passa, incluindo Scan, Plan, Build, Manifest, configurações e Deploy.
- [ ] O build otimizado é produzido pelo caminho oficial de distribuição sem etapas manuais adicionais.
- [ ] O executável distribuído abre a modal e conclui Deploys reais por senha e chave privada contra servidor SFTP descartável.
- [ ] O executável distribuído demonstra primeira publicação, repetição sem mudanças, atualização incremental, conflito, cancelamento e retry.
- [ ] O artefato distribuído inclui as dependências SSH necessárias e não exige componente específico do Patch Builder instalado separadamente.
- [ ] A inspeção final confirma que não existe senha, passphrase, chave privada ou conteúdo sensível em configurações, relatórios e logs produzidos pelos cenários.
- [ ] Não permanecem scripts descartáveis, servidores de teste ativos, credenciais de fixture ou arquivos temporários de validação fora da infraestrutura de teste intencional.
