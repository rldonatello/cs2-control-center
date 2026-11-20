# Fronteiras de segurança

## Windows

O núcleo recebe leituras explícitas e nunca deduz um valor quando a leitura falha. A camada futura de sistema deverá implementar adaptadores pequenos e auditáveis por ajuste, com leitura antes de qualquer proposta de escrita. Um plano só pode ser criado com estado anterior conhecido; se o valor já estiver no alvo, a operação é no-op. No registro de reversão, `Some("")` representa um valor anterior conhecido e vazio; `None` representa valor desconhecido e não pode ser revertido. A reversão também só pode ocorrer quando o estado lido ainda corresponde ao valor aplicado pelo app. Se outra ferramenta ou o usuário mudou o valor depois, há conflito e nada é sobrescrito.

O catálogo real fica fora do protótipo até cada tweak ter suporte, impacto, método de leitura/escrita, condição de reinicialização e estratégia de rollback revisados e testados. Não há dependência de mudanças de terceiros ou de perfis de instalação do Windows.

## BIOS / UEFI

A experiência é uma aba própria. Nesta entrega é diagnóstico-only, sem leitura de variáveis de firmware e sem escrita. No futuro, um item só pode ser oferecido mediante perfil validado para correspondência exata de fabricante/modelo do sistema; fabricante/produto/revisão da placa; fornecedor/versão do BIOS; modelo e configuração do processador (quantidade de pacotes, núcleos e processadores lógicos); e a lista exata dos módulos de memória. A identidade/configuração de cada módulo considera slot, capacidade, velocidade nominal e configurada, tipo SMBIOS, fabricante e part number. A ordem da lista é normalizada, mas diferenças em qualquer módulo falham a correspondência. Números de série não são coletados nem usados.

Campos ausentes, genéricos ou inválidos, versão divergente ou ausência de evidência de validação devem falhar fechada e manter diagnóstico somente. Cada alteração BIOS permanece individual e ligada à combinação exata. O coletor `docs/coletar-hardware.ps1` usa consultas CIM locais somente leitura e não grava configurações.

SCEWIN/SCEHUB não são redistribuídos, incorporados nem invocados. Qualquer uso futuro depende de comprovação de licença e autorização independente.
