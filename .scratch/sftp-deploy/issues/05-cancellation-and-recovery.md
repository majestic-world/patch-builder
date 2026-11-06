# 05: Cancelar e recuperar Deploys com falha

**What to build:** Tornar Deploy controlável e recuperável em redes lentas ou defeituosas, com cancelamento cooperativo, timeouts, limpeza segura, erros por estágio, retry e métricas úteis sem comprometer o Manifest publicado.

**Blocked by:** 02: Reutilizar Archives por hashes do Manifest.

**Status:** ready-for-agent

- [ ] O publisher pode cancelar uma operação ativa a partir da modal.
- [ ] Cancelamento é observado durante streaming e entre operações SFTP; uma chamada bloqueada retorna no máximo após seu timeout configurado.
- [ ] Cancelar fecha a conexão, evita publicar o Manifest local e remove temporários em melhor esforço.
- [ ] Falha de limpeza não substitui nem oculta a causa principal, mas aparece como contexto não sensível quando útil.
- [ ] Conexão TCP, handshake SSH, autenticação e operações SFTP têm timeouts finitos.
- [ ] Erros distinguem validação, conexão, host key, autenticação, leitura do Manifest, diretório, Archive, concorrência e publicação final.
- [ ] Mensagens incluem somente host e caminho não sensíveis necessários para agir; credenciais e conteúdo de chave nunca aparecem.
- [ ] Falha durante Archive ou Manifest mantém o Manifest remoto anterior visível.
- [ ] A modal preserva campos não sensíveis após falha e oferece retry sem reutilizar silenciosamente senha ou passphrase descartada.
- [ ] Progresso é monotônico e informa estágio, Archive atual, contagens, bytes concluídos e totais, percentual e taxa baseada em tempo decorrido.
- [ ] O relatório final informa versões anterior e publicada, novos, modificados, reparados e reutilizados, bytes enviados, duração e taxa média.
- [ ] Uma operação cancelada ou falha não produz relatório de sucesso enganoso.
- [ ] Testes com servidor SFTP descartável interrompem conexão e rename em pontos controlados e verificam Manifest anterior, ausência de arquivo público parcial e comportamento de retry.
- [ ] Testes cobrem endpoint inalcançável e servidor travado sem depender de espera ilimitada.
- [ ] Um smoke da aplicação real demonstra cancelamento durante um Archive grande, falha acionável e nova tentativa bem-sucedida.
