# 03: Usar chave privada e lembrar a conexão

**What to build:** Permitir Deploy com chave privada protegida ou não por passphrase e lembrar os dados não sensíveis do destino, mantendo credenciais fora da persistência e oferecendo gestão explícita para mudanças legítimas da chave do servidor.

**Blocked by:** 01: Publicar a primeira Update tree com senha.

**Status:** ready-for-agent

- [ ] A modal permite escolher entre autenticação por senha e por chave privada.
- [ ] O modo de chave privada oferece seletor nativo de arquivo e passphrase opcional mascarada.
- [ ] Chave ausente, ilegível, incompatível ou com passphrase incorreta produz erro de autenticação acionável sem revelar conteúdo sensível.
- [ ] Um servidor SFTP real aceita Deploy com chave sem passphrase e com chave criptografada.
- [ ] Host, porta, usuário, destino remoto, método de autenticação e caminho da chave privada são lembrados entre reinicializações.
- [ ] Um documento de configurações anterior ao Deploy carrega com perfil vazio e porta `22`, preservando Source, Update tree, algoritmo de hash e aparência.
- [ ] Perfis de senha e de chave privada serializam e desserializam somente seus campos não sensíveis.
- [ ] Senha, passphrase e conteúdo da chave privada não aparecem no JSON salvo, hosts conhecidos, erros, progresso ou relatório.
- [ ] Credenciais efêmeras são limpas quando a modal fecha ou a operação termina.
- [ ] Falha ao salvar o perfil mantém a escolha válida na sessão atual e informa que ela não será lembrada.
- [ ] Uma chave de host persistida é reutilizada em conexões seguintes sem nova confirmação.
- [ ] Uma chave de host alterada continua bloqueada e mostra identidade esperada e recebida.
- [ ] Substituir uma chave de host exige ação explícita e específica; cancelar mantém a identidade anterior e não autentica.
- [ ] Testes no seam público de configurações cobrem compatibilidade legada, round-trip dos perfis e ausência de segredos.
- [ ] Um smoke com a aplicação real cobre alternância de método, seleção nativa da chave, reinicialização e restauração do perfil não sensível.
