use cs2_tuner_prototype::bios::mode_for;
use cs2_tuner_prototype::tweak_state::{
    ChangePlan, ChangeRecord, Compatibility, Reading, RollbackError, TweakDefinition, TweakStatus,
    classify, plan_change, rollback_target,
};
use eframe::egui;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Windows,
    Bios,
}

struct PrototypeApp {
    tab: Tab,
    demo_current: String,
    demo_record: Option<ChangeRecord>,
    message: String,
}

impl Default for PrototypeApp {
    fn default() -> Self {
        Self {
            tab: Tab::Windows,
            demo_current: "0".into(),
            demo_record: None,
            message: "Demonstração local: nenhuma configuração do computador foi lida ou alterada."
                .into(),
        }
    }
}

impl PrototypeApp {
    fn status(&self) -> TweakStatus {
        let definition = TweakDefinition {
            enabled_value: "1".into(),
            disabled_value: "0".into(),
        };
        classify(
            &definition,
            Compatibility::Supported,
            &Reading::Known(self.demo_current.clone()),
            false,
        )
    }

    fn simulate_enable(&mut self) {
        match plan_change(Some(&self.demo_current), "1") {
            Ok(ChangePlan::AlreadyAtTarget) => {
                self.message =
                    "Já estava no valor-alvo; a simulação não reaplicou a mudança.".into();
            }
            Ok(ChangePlan::Apply(record)) => {
                self.demo_current = record.applied_value.clone();
                self.demo_record = Some(record);
                self.message = "Ativação simulada somente na memória desta janela.".into();
            }
            Err(_) => self.message = "Valor anterior desconhecido: operação bloqueada.".into(),
        }
    }

    fn simulate_disable(&mut self) {
        match plan_change(Some(&self.demo_current), "0") {
            Ok(ChangePlan::AlreadyAtTarget) => {
                self.message =
                    "Já estava no valor-alvo; a simulação não reaplicou a mudança.".into();
            }
            Ok(ChangePlan::Apply(record)) => {
                self.demo_current = record.applied_value.clone();
                self.demo_record = Some(record);
                self.message = "Desativação simulada somente na memória desta janela.".into();
            }
            Err(_) => self.message = "Valor anterior desconhecido: operação bloqueada.".into(),
        }
    }

    fn simulate_rollback(&mut self) {
        let Some(record) = self.demo_record.as_ref() else {
            self.message = "Não há alteração simulada para reverter.".into();
            return;
        };
        match rollback_target(record, Some(&self.demo_current)) {
            Ok(previous) => {
                self.demo_current = previous.to_owned();
                self.demo_record = None;
                self.message = "Reversão simulada: valor anterior restaurado na memória.".into();
            }
            Err(RollbackError::CurrentValueChanged) => {
                self.message =
                    "Conflito: o valor mudou desde a aplicação; reversão bloqueada.".into();
            }
            Err(_) => self.message = "Não há dados suficientes para reverter.".into(),
        }
    }
}

impl eframe::App for PrototypeApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.heading("CS2 Control Center");
            ui.label("Protótipo Rust • Windows 10/11");
            ui.colored_label(
                egui::Color32::from_rgb(190, 100, 40),
                "DEMONSTRAÇÃO — não lê nem altera configurações reais",
            );
            ui.separator();
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(self.tab == Tab::Windows, "Ajustes do Windows")
                    .clicked()
                {
                    self.tab = Tab::Windows;
                }
                if ui
                    .selectable_label(self.tab == Tab::Bios, "BIOS / UEFI")
                    .clicked()
                {
                    self.tab = Tab::Bios;
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| match self.tab {
            Tab::Windows => self.windows_panel(ui),
            Tab::Bios => self.bios_panel(ui),
        });
    }
}

impl PrototypeApp {
    fn windows_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Ajustes reversíveis do Windows");
        ui.label("A leitura e aplicação reais estão desativadas nesta entrega; não existe catálogo de tweaks validado fornecido ainda.");
        ui.add_space(10.0);
        ui.group(|ui| {
            ui.strong("Demonstração em memória — não é um ajuste do Windows");
            ui.label(format!(
                "Estado simulado: {} • valor: {}",
                status_label(self.status()),
                self.demo_current
            ));
            ui.horizontal(|ui| {
                if ui.button("Simular ativação").clicked() {
                    self.simulate_enable();
                }
                if ui.button("Simular desativação").clicked() {
                    self.simulate_disable();
                }
                if ui
                    .add_enabled(
                        self.demo_record.is_some(),
                        egui::Button::new("Simular reversão"),
                    )
                    .clicked()
                {
                    self.simulate_rollback();
                }
            });
            ui.label(&self.message);
        });
        ui.add_space(14.0);
        ui.heading("Regras de segurança implementadas e testadas");
        ui.label("• Só classifica um estado quando a leitura é confiável; leitura ausente ou falha vira “Desconhecido”.");
        ui.label(
            "• Valor já no alvo não é reaplicado; valor anterior é preservado no plano da mudança.",
        );
        ui.label("• A reversão só prossegue se o valor atual ainda for o que o app aplicou; mudanças externas causam bloqueio.");
        ui.label("• Estados previstos: ativo, desativado, parcial, incompatível, reinicialização necessária e desconhecido.");
    }

    fn bios_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading("Diagnóstico de BIOS / UEFI");
        ui.colored_label(
            egui::Color32::from_rgb(190, 100, 40),
            "SOMENTE LEITURA — nenhuma varredura ou escrita de firmware nesta entrega",
        );
        ui.add_space(8.0);
        ui.label("A interface de BIOS fica separada. Este protótipo não chama SCEWIN/SCEHUB nem inclui esses componentes.");
        ui.label(format!(
            "Modo atual do motor de correspondência: {:?}",
            mode_for(None, None)
        ));
        ui.add_space(8.0);
        ui.strong("Para habilitar futuramente um item BIOS, o perfil precisa validar exatamente:");
        ui.label("Fabricante/modelo do sistema; fabricante/produto/revisão da placa; fornecedor/versão do BIOS.");
        ui.label("Campos ausentes, versão diferente ou perfil sem registro de validação mantêm o modo diagnóstico.");
        ui.add_space(8.0);
        ui.label("Nenhum perfil de hardware foi aprovado ou carregado. Não há controles de edição nesta aba.");
    }
}

fn status_label(status: TweakStatus) -> &'static str {
    match status {
        TweakStatus::Active => "Ativo",
        TweakStatus::Disabled => "Desativado",
        TweakStatus::Partial => "Parcial",
        TweakStatus::Incompatible => "Incompatível",
        TweakStatus::RebootRequired => "Reinicialização necessária",
        TweakStatus::Unknown => "Desconhecido",
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([860.0, 620.0])
            .with_min_inner_size([680.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "CS2 Control Center — protótipo",
        options,
        Box::new(|_creation_context| Ok(Box::new(PrototypeApp::default()))),
    )
}
