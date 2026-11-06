# 06: Integrar Deploy ao fluxo desktop

**What to build:** Completar a experiência desktop de Deploy, coordenando-a com Scan, Plan e Build e tornando todos os estados da modal operáveis por ponteiro, teclado e tecnologia assistiva em Light e Dark.

**Blocked by:** 03: Usar chave privada e lembrar a conexão; 04: Proteger conflitos de publicação; 05: Cancelar e recuperar Deploys com falha.

**Status:** ready-for-agent

- [ ] Scan, Build e Deploy são mutuamente exclusivos enquanto qualquer um executa trabalho que possa alterar o estado de publicação.
- [ ] Um Scan bem-sucedido cujo Plan contém trabalho bloqueia Deploy e orienta o publisher a executar Build primeiro.
- [ ] Sem Scan atual bem-sucedido, uma Update tree existente com Manifest e Archives válidos ainda pode ser publicada.
- [ ] Deploy nunca reconstrói a Source nem transforma Archives; Build permanece o único produtor da Update tree local.
- [ ] A modal cobre configuração, confiança de host, conexão, comparação, transferência, publicação, conflito, falha e sucesso sem expor o conteúdo subjacente a interações acidentais.
- [ ] O controle Deploy possui nome, descrição, foco e ativação por teclado, além de consumir o gesto sem iniciar drag ou maximização da barra de título.
- [ ] Campos, escolha de autenticação, seletor de chave, confiança, conflito, progresso, cancelamento, retry e fechamento têm ordem de foco e semântica acessível.
- [ ] A modal permanece legível em Light e Dark e usa os tokens semânticos existentes, adicionando somente estados realmente ausentes.
- [ ] Aparência pode continuar sendo alterada durante Deploy somente se isso não fechar, reiniciar ou mutar o trabalho ativo.
- [ ] Minimizar, maximizar, restaurar, fechar e arrastar a janela continuam funcionando fora da área consumida pelo novo controle.
- [ ] O estado concluído apresenta fatos da publicação e ações de fechar ou iniciar nova tentativa, sem botão Open e sem abrir navegador ou cliente SFTP.
- [ ] A janela no tamanho mínimo não sobrepõe Deploy, aparência, título do produto ou controles nativos.
- [ ] Um smoke na aplicação real percorre senha, chave privada, confiança, conflito, progresso, cancelamento, erro, retry e sucesso nos dois temas.
- [ ] O smoke valida navegação por teclado, rótulos acessíveis, título arrastável, double-click para maximizar e controles de janela.
- [ ] A suíte Slint compila todos os componentes, recursos e contratos de callback sem testes de texto-fonte ou bindings incidentais.
