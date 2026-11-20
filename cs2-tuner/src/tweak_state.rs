#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakStatus {
    Active,
    Disabled,
    Partial,
    Incompatible,
    RebootRequired,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compatibility {
    Supported,
    Unsupported,
    Unverified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    Known(String),
    Unavailable,
    Unreadable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TweakDefinition {
    pub enabled_value: String,
    pub disabled_value: String,
}

/// Classifies only values that were read reliably; a missing or failed read is never guessed.
pub fn classify(
    definition: &TweakDefinition,
    compatibility: Compatibility,
    reading: &Reading,
    reboot_pending: bool,
) -> TweakStatus {
    if compatibility == Compatibility::Unsupported {
        return TweakStatus::Incompatible;
    }
    if compatibility == Compatibility::Unverified {
        return TweakStatus::Unknown;
    }
    if reboot_pending {
        return TweakStatus::RebootRequired;
    }
    match reading {
        Reading::Known(value) if value == &definition.enabled_value => TweakStatus::Active,
        Reading::Known(value) if value == &definition.disabled_value => TweakStatus::Disabled,
        Reading::Known(_) => TweakStatus::Partial,
        Reading::Unavailable | Reading::Unreadable => TweakStatus::Unknown,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChangeRecord {
    /// `Some("")` means the known prior value was empty; `None` means it was unknown.
    pub previous_value: Option<String>,
    pub applied_value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangePlan {
    AlreadyAtTarget,
    Apply(ChangeRecord),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanError {
    CurrentValueUnknown,
}

/// Produces a plan only when the current value is known, and avoids reapplying the target.
pub fn plan_change(current: Option<&str>, target: &str) -> Result<ChangePlan, PlanError> {
    let current = current.ok_or(PlanError::CurrentValueUnknown)?;
    if current == target {
        Ok(ChangePlan::AlreadyAtTarget)
    } else {
        Ok(ChangePlan::Apply(ChangeRecord {
            previous_value: Some(current.to_owned()),
            applied_value: target.to_owned(),
        }))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RollbackError {
    PreviousValueUnknown,
    CurrentValueUnknown,
    CurrentValueChanged,
}

/// Allows a rollback only if the observed value is still the value this app applied.
pub fn rollback_target<'a>(
    record: &'a ChangeRecord,
    current: Option<&str>,
) -> Result<&'a str, RollbackError> {
    let previous = record
        .previous_value
        .as_deref()
        .ok_or(RollbackError::PreviousValueUnknown)?;
    let current = current.ok_or(RollbackError::CurrentValueUnknown)?;
    if current != record.applied_value {
        return Err(RollbackError::CurrentValueChanged);
    }
    Ok(previous)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn definition() -> TweakDefinition {
        TweakDefinition {
            enabled_value: "1".into(),
            disabled_value: "0".into(),
        }
    }

    #[test]
    fn classifies_all_read_states_without_guessing() {
        let d = definition();
        let supported = Compatibility::Supported;
        assert_eq!(
            classify(&d, supported, &Reading::Known("1".into()), false),
            TweakStatus::Active
        );
        assert_eq!(
            classify(&d, supported, &Reading::Known("0".into()), false),
            TweakStatus::Disabled
        );
        assert_eq!(
            classify(&d, supported, &Reading::Known("2".into()), false),
            TweakStatus::Partial
        );
        assert_eq!(
            classify(&d, supported, &Reading::Unavailable, false),
            TweakStatus::Unknown
        );
        assert_eq!(
            classify(&d, supported, &Reading::Unreadable, false),
            TweakStatus::Unknown
        );
        assert_eq!(
            classify(
                &d,
                Compatibility::Unsupported,
                &Reading::Known("1".into()),
                false
            ),
            TweakStatus::Incompatible
        );
        assert_eq!(
            classify(
                &d,
                Compatibility::Unverified,
                &Reading::Known("1".into()),
                false
            ),
            TweakStatus::Unknown
        );
        assert_eq!(
            classify(&d, supported, &Reading::Known("1".into()), true),
            TweakStatus::RebootRequired
        );
    }

    #[test]
    fn does_not_reapply_an_existing_target_and_requires_a_known_current_value() {
        assert_eq!(plan_change(Some("1"), "1"), Ok(ChangePlan::AlreadyAtTarget));
        assert_eq!(plan_change(None, "1"), Err(PlanError::CurrentValueUnknown));
        assert_eq!(
            plan_change(Some(""), "1"),
            Ok(ChangePlan::Apply(ChangeRecord {
                previous_value: Some(String::new()),
                applied_value: "1".into(),
            }))
        );
    }

    #[test]
    fn rollback_requires_known_prior_value_and_no_external_change() {
        let record = ChangeRecord {
            previous_value: Some("0".into()),
            applied_value: "1".into(),
        };
        assert_eq!(rollback_target(&record, Some("1")), Ok("0"));
        assert_eq!(
            rollback_target(&record, Some("2")),
            Err(RollbackError::CurrentValueChanged)
        );
        assert_eq!(
            rollback_target(&record, None),
            Err(RollbackError::CurrentValueUnknown)
        );

        let unknown_before = ChangeRecord {
            previous_value: None,
            applied_value: "1".into(),
        };
        assert_eq!(
            rollback_target(&unknown_before, Some("1")),
            Err(RollbackError::PreviousValueUnknown)
        );
    }

    #[test]
    fn legitimately_empty_previous_value_is_restorable() {
        let record = ChangeRecord {
            previous_value: Some(String::new()),
            applied_value: "1".into(),
        };
        assert_eq!(rollback_target(&record, Some("1")), Ok(""));
    }
}
