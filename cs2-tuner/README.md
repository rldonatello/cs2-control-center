# CS2 Control Center — protótipo

Aplicativo desktop experimental em Rust para Windows 10/11. Esta entrega comprova a estrutura da interface e as regras essenciais de segurança; **não altera o Windows, não lê configurações do computador e não escreve no BIOS/UEFI**.

## O que está pronto

- Interface nativa com abas independentes para **Ajustes do Windows** e **BIOS / UEFI**.
- Demonstração interativa inteiramente em memória, identificada como simulação.
- Modelo de estados: ativo, desativado, parcial, incompatível, reinicialização necessária e desconhecido.
- Regras testadas para não reaplicar valores já configurados e bloquear reversão quando o estado atual diverge. O rollback distingue `Some("")` (valor anterior conhecido e vazio) de `None` (valor anterior desconhecido).
- Fingerprint BIOS com correspondência exata de sistema, placa, BIOS, modelo/configuração do processador e configuração de memória por módulo; correspondência incompleta mantém diagnóstico somente.
- Coletor PowerShell somente leitura; não coleta números de série de CPU/RAM/placa, UUIDs ou identificadores de rede.
- Não incorpora nem invoca SCEWIN ou SCEHUB.

## Executar e testar

Com Rust instalado:

```powershell
cargo test
cargo run
```

Para compilar para Windows, execute `cargo build --release` no Windows com a toolchain Rust apropriada. O protótipo usa `eframe/egui` para a janela nativa.

## Limites desta entrega

Ainda não existe catálogo de tweaks Windows validado nem leitor do Registro/serviços. Os botões da aba Windows só mudam o estado da demonstração local. A aba BIOS não faz varredura: não há perfil de máquina validado e não existe caminho de escrita. A licença/proveniência do repositório de referência e o direito de redistribuir ferramentas externas continuam pendentes, portanto nada disso foi copiado.

## Próxima entrada necessária

Para definir e testar a primeira combinação de hardware/firmware, execute no próprio Windows o coletor somente leitura `docs/coletar-hardware.ps1` e compartilhe o JSON resultante. Ele inclui versão/build do Windows; fabricante/modelo do sistema; fabricante/produto/revisão da placa; fornecedor/versão/data do BIOS; modelo(s), quantidade de processadores, núcleos e processadores lógicos; e, por módulo de memória, slot, capacidade, velocidade nominal/configurada, tipo SMBIOS, fabricante e part number. Números de série dos módulos, UUIDs e identificadores de rede são omitidos. Se o firmware não informar algum campo necessário, a combinação não será considerada exata e a aba continua em diagnóstico.
