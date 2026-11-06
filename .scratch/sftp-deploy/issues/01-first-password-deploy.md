# 01: Publicar a primeira Update tree com senha

**What to build:** Permitir que um publisher abra Deploy na barra de título, informe um destino SFTP e publique com senha uma Update tree completa em um servidor vazio. O fluxo deve validar a saída local, verificar a identidade SSH, transferir Archives sem expor arquivos parciais e instalar o Manifest por último.

**Blocked by:** None (can start immediately).

**Status:** ready-for-agent

- [ ] Um controle rotulado Deploy aparece ao lado do controle de aparência e abre uma modal com host, porta, usuário, destino remoto e senha mascarada.
- [ ] A porta começa em `22`, os campos obrigatórios são validados antes da conexão e o trabalho de rede não bloqueia a UI.
- [ ] O primeiro contato exibe algoritmo e fingerprint SHA-256 da chave do host e só continua após confiança explícita.
- [ ] A identidade confiada é armazenada em um arquivo de hosts conhecidos pertencente ao Patch Builder, sem alterar a configuração SSH global do usuário.
- [ ] Uma chave de host já conhecida e correspondente é aceita; uma chave divergente bloqueia o Deploy sem autenticar nem escrever no servidor.
- [ ] A senha permanece apenas em memória, aparece mascarada e não entra em configurações, hosts conhecidos, progresso, relatório ou erros.
- [ ] O Manifest local é lido e validado antes de qualquer escrita remota; caminhos vazios, absolutos, com barra invertida, prefixo de plataforma, `.` ou `..` são rejeitados.
- [ ] Todos os Archives exigidos pelo Manifest são verificados localmente antes da publicação e um Archive ausente interrompe o fluxo sem escrita remota.
- [ ] Um destino sem Manifest é tratado como primeira publicação; um destino que já contém Manifest não é sobrescrito por este primeiro fluxo incompleto.
- [ ] Diretórios remotos aninhados são criados conforme necessário com semântica POSIX, independentemente do sistema operacional local.
- [ ] Archives são transmitidos sequencialmente por streaming para nomes temporários irmãos e instalados por rename, sem carregar um Archive inteiro em memória.
- [ ] O Manifest é enviado para um nome temporário e instalado como `manifest.json` somente depois de todos os Archives.
- [ ] Falha antes da publicação final não deixa um Manifest novo visível nem um Archive parcial sob o nome público.
- [ ] A modal mostra ao menos o estágio, o Archive atual e progresso por bytes, e o sucesso informa destino, versão e bytes enviados.
- [ ] Um teste pela operação completa usa um servidor SFTP descartável real com autenticação por senha e confirma os bytes dos Archives e do Manifest remoto.
- [ ] Um smoke na aplicação real prova abertura da modal, confiança no primeiro host, progresso e conclusão do primeiro Deploy.
