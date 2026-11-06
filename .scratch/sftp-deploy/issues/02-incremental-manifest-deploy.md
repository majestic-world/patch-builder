# 02: Reutilizar Archives por hashes do Manifest

**What to build:** Estender Deploy para comparar uma Update tree local com uma remote Update tree existente e transferir somente Archives novos, modificados ou ausentes, usando o contrato de hashes do Manifest e publicando apenas as mudanças necessárias.

**Blocked by:** 01: Publicar a primeira Update tree com senha.

**Status:** ready-for-agent

- [ ] Um Manifest remoto ausente continua produzindo a primeira publicação completa.
- [ ] Um Manifest remoto inválido, ilegível ou acima do limite aceito falha antes de qualquer escrita e nunca é tratado como destino vazio.
- [ ] Entradas locais e remotas são associadas por caminho exato do Manifest.
- [ ] BLAKE3 reutiliza um Archive somente quando o hash BLAKE3 corresponde.
- [ ] CRC32 reutiliza um Archive somente quando o hash CRC32 corresponde.
- [ ] A seleção combinada reutiliza um Archive somente quando BLAKE3 e CRC32 correspondem.
- [ ] Manifests com algoritmos diferentes são considerados incomparáveis e selecionam todos os Archives locais para envio.
- [ ] Tamanho, data de modificação e timestamp remoto não estabelecem igualdade de conteúdo.
- [ ] Uma entrada com hash correspondente é reutilizada somente quando o Archive remoto também existe como arquivo regular; ausência seleciona reparo por upload.
- [ ] Um caminho novo ou hash diferente seleciona exatamente o Archive correspondente para envio.
- [ ] Archives selecionados são instalados por arquivo temporário e substituição segura; incompatibilidade de rename preserva o arquivo público anterior e retorna erro acionável.
- [ ] Entradas e Archives existentes apenas no servidor não são apagados.
- [ ] Remover uma entrada local pode publicar somente o novo Manifest, deixando o Archive remoto obsoleto armazenado, mas fora do contrato atual do Launcher.
- [ ] Manifests semanticamente idênticos com todos os Archives presentes produzem resultado atualizado sem escrita remota.
- [ ] O progresso usa os bytes comprimidos reais dos Archives selecionados, não os tamanhos descomprimidos do Manifest.
- [ ] O resultado distingue Archives novos, modificados, reparados e reutilizados, além dos bytes efetivamente transferidos.
- [ ] Até existir a confirmação do ticket de conflitos, Manifest remoto mais novo ou divergente na mesma versão é recusado com segurança, sem sobrescrita silenciosa.
- [ ] Testes pela operação completa contra SFTP real cobrem primeira publicação, repetição sem mudanças, arquivo novo, arquivo alterado, Archive remoto ausente, remoção local e transição entre algoritmos.
