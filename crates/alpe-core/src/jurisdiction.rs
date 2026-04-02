//! EU jurisdiction types and sovereignty-aware replication rules.
//!
//! Sovereignty is a first-class concept in the Alpe platform. Every resource
//! is tagged with a jurisdiction that determines where it can be stored
//! and replicated. This module encodes the replication rules as pure functions.
//!
//! # Replication Rules
//!
//! - A resource in country X can only replicate within country X
//! - A resource under the EU umbrella can replicate to any member state
//! - Cross-country replication (e.g. DE → FR) is **never** allowed
//! - EU umbrella cannot replicate to EU umbrella (it's not a physical location)
//!
//! # Examples
//!
//! ```
//! use alpe_core::jurisdiction::Jurisdiction;
//!
//! let de = Jurisdiction::DE;
//! assert!(de.is_country());
//! assert!(Jurisdiction::can_replicate(de, de));
//! assert!(!Jurisdiction::can_replicate(de, Jurisdiction::FR));
//! ```

use crate::error::SovereigntyError;

/// EU jurisdictions: the EU umbrella plus all 27 member states.
///
/// Each variant represents either the EU umbrella (for resources that may
/// be placed in any member state) or a specific country.
///
/// # Serialization
///
/// Jurisdictions serialize to and deserialize from their uppercase ISO 3166-1
/// alpha-2 code (e.g. `"DE"`, `"FR"`), except for the EU umbrella which uses `"EU"`.
///
/// # Examples
///
/// ```
/// use alpe_core::jurisdiction::Jurisdiction;
/// use std::str::FromStr;
///
/// let j = Jurisdiction::from_str("DE").unwrap();
/// assert_eq!(j, Jurisdiction::DE);
/// assert_eq!(j.to_string(), "DE");
/// ```
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde::Serialize,
    serde::Deserialize,
    strum::Display,
    strum::EnumString,
)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Jurisdiction {
    /// European Union umbrella — resource may be placed in any member state.
    EU,
    /// Austria
    AT,
    /// Belgium
    BE,
    /// Bulgaria
    BG,
    /// Croatia
    HR,
    /// Cyprus
    CY,
    /// Czech Republic
    CZ,
    /// Denmark
    DK,
    /// Estonia
    EE,
    /// Finland
    FI,
    /// France
    FR,
    /// Germany
    DE,
    /// Greece
    GR,
    /// Hungary
    HU,
    /// Ireland
    IE,
    /// Italy
    IT,
    /// Latvia
    LV,
    /// Lithuania
    LT,
    /// Luxembourg
    LU,
    /// Malta
    MT,
    /// Netherlands
    NL,
    /// Poland
    PL,
    /// Portugal
    PT,
    /// Romania
    RO,
    /// Slovakia
    SK,
    /// Slovenia
    SI,
    /// Spain
    ES,
    /// Sweden
    SE,
}

impl Jurisdiction {
    /// Returns `true` if this jurisdiction represents a specific country
    /// (i.e. not the EU umbrella).
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::jurisdiction::Jurisdiction;
    ///
    /// assert!(Jurisdiction::DE.is_country());
    /// assert!(!Jurisdiction::EU.is_country());
    /// ```
    #[must_use]
    pub const fn is_country(&self) -> bool {
        !matches!(self, Self::EU)
    }

    /// Returns all 27 EU member states (excluding the EU umbrella).
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::jurisdiction::Jurisdiction;
    ///
    /// assert_eq!(Jurisdiction::member_states().len(), 27);
    /// ```
    #[must_use]
    pub const fn member_states() -> &'static [Self] {
        &[
            Self::AT,
            Self::BE,
            Self::BG,
            Self::HR,
            Self::CY,
            Self::CZ,
            Self::DK,
            Self::EE,
            Self::FI,
            Self::FR,
            Self::DE,
            Self::GR,
            Self::HU,
            Self::IE,
            Self::IT,
            Self::LV,
            Self::LT,
            Self::LU,
            Self::MT,
            Self::NL,
            Self::PL,
            Self::PT,
            Self::RO,
            Self::SK,
            Self::SI,
            Self::ES,
            Self::SE,
        ]
    }

    /// Returns `true` if data in `from` jurisdiction can be replicated to `to`.
    ///
    /// # Rules
    ///
    /// - Same country → allowed
    /// - EU umbrella → any member state is allowed
    /// - Cross-country → never allowed
    /// - EU → EU → not allowed (EU is not a physical location)
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::jurisdiction::Jurisdiction;
    ///
    /// // Same country is always OK
    /// assert!(Jurisdiction::can_replicate(Jurisdiction::DE, Jurisdiction::DE));
    ///
    /// // EU umbrella can replicate to any member state
    /// assert!(Jurisdiction::can_replicate(Jurisdiction::EU, Jurisdiction::FR));
    ///
    /// // Cross-country is never OK
    /// assert!(!Jurisdiction::can_replicate(Jurisdiction::DE, Jurisdiction::FR));
    /// ```
    #[must_use]
    pub const fn can_replicate(from: Self, to: Self) -> bool {
        // EU → EU is not allowed (not a physical location)
        if matches!(from, Self::EU) && matches!(to, Self::EU) {
            return false;
        }

        // Country → EU umbrella is not allowed
        if from.is_country() && !to.is_country() {
            return false;
        }

        // EU umbrella → any member state is allowed
        if matches!(from, Self::EU) && to.is_country() {
            return true;
        }

        // Same country → allowed, different country → blocked
        from as u8 == to as u8
    }

    /// Validates that replication from `from` to `to` is allowed.
    ///
    /// Returns `Ok(())` if replication is permitted, or a [`SovereigntyError`]
    /// describing the violation.
    ///
    /// # Errors
    ///
    /// Returns [`SovereigntyError`] if replication between the given jurisdictions
    /// is not permitted by the sovereignty rules.
    ///
    /// # Examples
    ///
    /// ```
    /// use alpe_core::jurisdiction::Jurisdiction;
    ///
    /// assert!(Jurisdiction::validate_replication(Jurisdiction::DE, Jurisdiction::DE).is_ok());
    /// assert!(Jurisdiction::validate_replication(Jurisdiction::DE, Jurisdiction::FR).is_err());
    /// ```
    pub fn validate_replication(from: Self, to: Self) -> Result<(), SovereigntyError> {
        if Self::can_replicate(from, to) {
            Ok(())
        } else {
            Err(SovereigntyError::new(format!(
                "{from} cannot replicate to {to}"
            )))
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn all_27_member_states_exist() {
        assert_eq!(Jurisdiction::member_states().len(), 27);
    }

    #[test]
    fn eu_umbrella_is_not_a_country() {
        assert!(!Jurisdiction::EU.is_country());
    }

    #[test]
    fn every_member_state_is_a_country() {
        for member in Jurisdiction::member_states() {
            assert!(member.is_country(), "{member} should be a country");
        }
    }

    #[test]
    fn jurisdiction_parses_from_uppercase_string() {
        let j = Jurisdiction::from_str("DE").expect("should parse DE");
        assert_eq!(j, Jurisdiction::DE);
    }

    #[test]
    fn jurisdiction_rejects_invalid_string() {
        assert!(Jurisdiction::from_str("US").is_err());
        assert!(Jurisdiction::from_str("").is_err());
        assert!(Jurisdiction::from_str("de").is_err());
    }

    #[test]
    fn jurisdiction_displays_as_uppercase() {
        assert_eq!(Jurisdiction::DE.to_string(), "DE");
        assert_eq!(Jurisdiction::FR.to_string(), "FR");
        assert_eq!(Jurisdiction::EU.to_string(), "EU");
    }

    #[test]
    fn jurisdiction_roundtrips_through_display_and_parse() {
        for member in Jurisdiction::member_states() {
            let s = member.to_string();
            let roundtrip = Jurisdiction::from_str(&s).expect("should roundtrip");
            assert_eq!(*member, roundtrip);
        }

        // EU umbrella also roundtrips
        let s = Jurisdiction::EU.to_string();
        let roundtrip = Jurisdiction::from_str(&s).expect("EU roundtrip");
        assert_eq!(Jurisdiction::EU, roundtrip);
    }

    #[test]
    fn country_replicates_to_same_country() {
        assert!(Jurisdiction::can_replicate(
            Jurisdiction::DE,
            Jurisdiction::DE
        ));
        assert!(Jurisdiction::can_replicate(
            Jurisdiction::FR,
            Jurisdiction::FR
        ));
    }

    #[test]
    fn country_cannot_replicate_to_different_country() {
        assert!(!Jurisdiction::can_replicate(
            Jurisdiction::DE,
            Jurisdiction::FR
        ));
    }

    #[test]
    fn country_cannot_replicate_to_eu_umbrella() {
        assert!(!Jurisdiction::can_replicate(
            Jurisdiction::DE,
            Jurisdiction::EU
        ));
    }

    #[test]
    fn eu_umbrella_replicates_to_any_member_state() {
        for member in Jurisdiction::member_states() {
            assert!(
                Jurisdiction::can_replicate(Jurisdiction::EU, *member),
                "EU should replicate to {member}"
            );
        }
    }

    #[test]
    fn eu_umbrella_cannot_replicate_to_eu_umbrella() {
        assert!(!Jurisdiction::can_replicate(
            Jurisdiction::EU,
            Jurisdiction::EU
        ));
    }

    #[test]
    fn no_country_replicates_to_any_other_country_exhaustive() {
        let states = Jurisdiction::member_states();
        for from in states {
            for to in states {
                if from != to {
                    assert!(
                        !Jurisdiction::can_replicate(*from, *to),
                        "{from} should not replicate to {to}"
                    );
                }
            }
        }
    }

    #[test]
    fn validate_replication_ok_for_same_country() {
        assert!(Jurisdiction::validate_replication(Jurisdiction::DE, Jurisdiction::DE).is_ok());
    }

    #[test]
    fn validate_replication_err_for_cross_country() {
        let err = Jurisdiction::validate_replication(Jurisdiction::DE, Jurisdiction::FR)
            .expect_err("should fail");
        let msg = err.to_string();
        assert!(msg.contains("DE"), "error should mention source");
        assert!(msg.contains("FR"), "error should mention target");
    }
}
