#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemoryModuleFingerprint {
    pub capacity_bytes: u64,
    pub speed_mhz: u32,
    pub configured_speed_mhz: u32,
    pub smbios_memory_type: u16,
    pub manufacturer: String,
    pub part_number: String,
    pub device_locator: String,
}

impl MemoryModuleFingerprint {
    fn complete(&self) -> bool {
        self.capacity_bytes > 0
            && self.speed_mhz > 0
            && self.configured_speed_mhz > 0
            && self.smbios_memory_type > 0
            && required_text(&self.manufacturer)
            && required_text(&self.part_number)
            && required_text(&self.device_locator)
    }

    fn key(&self) -> MemoryModuleKey {
        MemoryModuleKey {
            capacity_bytes: self.capacity_bytes,
            speed_mhz: self.speed_mhz,
            configured_speed_mhz: self.configured_speed_mhz,
            smbios_memory_type: self.smbios_memory_type,
            manufacturer: canonical_text(&self.manufacturer),
            part_number: canonical_text(&self.part_number),
            device_locator: canonical_text(&self.device_locator),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct MemoryModuleKey {
    capacity_bytes: u64,
    speed_mhz: u32,
    configured_speed_mhz: u32,
    smbios_memory_type: u16,
    manufacturer: String,
    part_number: String,
    device_locator: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FirmwareFingerprint {
    pub system_manufacturer: String,
    pub system_model: String,
    pub board_manufacturer: String,
    pub board_product: String,
    pub board_revision: String,
    pub bios_vendor: String,
    pub bios_version: String,
    pub processor_model: String,
    pub processor_package_count: u32,
    pub processor_cores: u32,
    pub processor_logical_processors: u32,
    pub memory_modules: Vec<MemoryModuleFingerprint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FirmwareFingerprintKey {
    system_manufacturer: String,
    system_model: String,
    board_manufacturer: String,
    board_product: String,
    board_revision: String,
    bios_vendor: String,
    bios_version: String,
    processor_model: String,
    processor_package_count: u32,
    processor_cores: u32,
    processor_logical_processors: u32,
    memory_modules: Vec<MemoryModuleKey>,
}

impl FirmwareFingerprint {
    fn complete(&self) -> bool {
        [
            &self.system_manufacturer,
            &self.system_model,
            &self.board_manufacturer,
            &self.board_product,
            &self.board_revision,
            &self.bios_vendor,
            &self.bios_version,
        ]
        .iter()
        .all(|value| required_text(value))
            && required_text(&self.processor_model)
            && self.processor_package_count > 0
            && self.processor_cores > 0
            && self.processor_logical_processors > 0
            && !self.memory_modules.is_empty()
            && self
                .memory_modules
                .iter()
                .all(MemoryModuleFingerprint::complete)
    }

    fn key(&self) -> FirmwareFingerprintKey {
        let mut memory_modules: Vec<_> = self
            .memory_modules
            .iter()
            .map(MemoryModuleFingerprint::key)
            .collect();
        memory_modules.sort();
        FirmwareFingerprintKey {
            system_manufacturer: canonical_text(&self.system_manufacturer),
            system_model: canonical_text(&self.system_model),
            board_manufacturer: canonical_text(&self.board_manufacturer),
            board_product: canonical_text(&self.board_product),
            board_revision: canonical_text(&self.board_revision),
            bios_vendor: canonical_text(&self.bios_vendor),
            bios_version: canonical_text(&self.bios_version),
            processor_model: canonical_text(&self.processor_model),
            processor_package_count: self.processor_package_count,
            processor_cores: self.processor_cores,
            processor_logical_processors: self.processor_logical_processors,
            memory_modules,
        }
    }

    /// Requires complete system, board, firmware, CPU and RAM identity; no wildcard or partial match.
    pub fn exact_match(&self, other: &Self) -> bool {
        self.complete() && other.complete() && self.key() == other.key()
    }
}

fn canonical_text(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn required_text(value: &str) -> bool {
    let normalized = canonical_text(value);
    !normalized.is_empty()
        && ![
            "unknown",
            "not specified",
            "not available",
            "n/a",
            "none",
            "0",
            "to be filled by o.e.m.",
            "default string",
        ]
        .contains(&normalized.as_str())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedBiosProfile {
    pub fingerprint: FirmwareFingerprint,
    pub validation_record: String,
    pub validated: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BiosMode {
    DiagnosticOnly,
    ExactValidatedProfileAvailable,
}

/// This only reports whether a validated exact profile could be offered later.
/// It does not read, scan, or write firmware settings.
pub fn mode_for(
    detected: Option<&FirmwareFingerprint>,
    profile: Option<&ValidatedBiosProfile>,
) -> BiosMode {
    let (Some(detected), Some(profile)) = (detected, profile) else {
        return BiosMode::DiagnosticOnly;
    };
    if profile.validated
        && !profile.validation_record.trim().is_empty()
        && detected.exact_match(&profile.fingerprint)
    {
        BiosMode::ExactValidatedProfileAvailable
    } else {
        BiosMode::DiagnosticOnly
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn module(locator: &str, part_number: &str) -> MemoryModuleFingerprint {
        MemoryModuleFingerprint {
            capacity_bytes: 16 * 1024 * 1024 * 1024,
            speed_mhz: 5600,
            configured_speed_mhz: 5600,
            smbios_memory_type: 34,
            manufacturer: "Example Memory Co".into(),
            part_number: part_number.into(),
            device_locator: locator.into(),
        }
    }

    fn fingerprint() -> FirmwareFingerprint {
        FirmwareFingerprint {
            system_manufacturer: "Example OEM".into(),
            system_model: "Boarded PC A".into(),
            board_manufacturer: "Example Board Co".into(),
            board_product: "X-100".into(),
            board_revision: "1.0".into(),
            bios_vendor: "Example Firmware".into(),
            bios_version: "2.4.1".into(),
            processor_model: "Example CPU 8-Core".into(),
            processor_package_count: 1,
            processor_cores: 8,
            processor_logical_processors: 16,
            memory_modules: vec![
                module("DIMM_A1", "MEM-16G-A1"),
                module("DIMM_B1", "MEM-16G-B1"),
            ],
        }
    }

    fn validated_profile() -> ValidatedBiosProfile {
        ValidatedBiosProfile {
            fingerprint: fingerprint(),
            validation_record: "test record".into(),
            validated: true,
        }
    }

    #[test]
    fn requires_complete_exact_hardware_and_firmware_match() {
        let expected = fingerprint();
        assert!(expected.exact_match(&fingerprint()));

        let mut different_firmware = fingerprint();
        different_firmware.bios_version = "2.4.2".into();
        assert!(!expected.exact_match(&different_firmware));

        let mut incomplete = fingerprint();
        incomplete.board_revision.clear();
        assert!(!expected.exact_match(&incomplete));

        let mut incomplete_memory = fingerprint();
        incomplete_memory.memory_modules[0].part_number = "Unknown".into();
        assert!(!expected.exact_match(&incomplete_memory));
    }

    #[test]
    fn cpu_or_memory_difference_keeps_bios_in_diagnostic_mode() {
        let profile = validated_profile();

        let mut different_cpu = fingerprint();
        different_cpu.processor_model = "Different CPU Model".into();
        assert_eq!(
            mode_for(Some(&different_cpu), Some(&profile)),
            BiosMode::DiagnosticOnly
        );

        let mut different_memory = fingerprint();
        different_memory.memory_modules[0].capacity_bytes *= 2;
        assert_eq!(
            mode_for(Some(&different_memory), Some(&profile)),
            BiosMode::DiagnosticOnly
        );

        let mut different_memory_kit = fingerprint();
        different_memory_kit.memory_modules[0].part_number = "OTHER-KIT-16G".into();
        assert_eq!(
            mode_for(Some(&different_memory_kit), Some(&profile)),
            BiosMode::DiagnosticOnly
        );
    }

    #[test]
    fn memory_module_order_does_not_change_the_exact_match() {
        let expected = fingerprint();
        let mut reordered = fingerprint();
        reordered.memory_modules.reverse();
        assert!(expected.exact_match(&reordered));
    }

    #[test]
    fn unvalidated_or_unmatched_profiles_remain_diagnostic_only() {
        let detected = fingerprint();
        let mut profile = validated_profile();
        profile.validated = false;
        assert_eq!(
            mode_for(Some(&detected), Some(&profile)),
            BiosMode::DiagnosticOnly
        );

        profile.validated = true;
        assert_eq!(
            mode_for(Some(&detected), Some(&profile)),
            BiosMode::ExactValidatedProfileAvailable
        );
        profile.fingerprint.bios_version = "different".into();
        assert_eq!(
            mode_for(Some(&detected), Some(&profile)),
            BiosMode::DiagnosticOnly
        );
    }
}
