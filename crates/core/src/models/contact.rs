//! Contact entity representing sources, interviewees, and collaborators.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::validation::{validate_email, validate_name, validate_phone, ValidationError};

/// Core domain entity representing a contact or source.
///
/// # Examples
///
/// ```
/// use newsjournal_core::Contact;
///
/// let contact = Contact::new("Jane Doe")
///     .with_organization("City Transit Authority")
///     .with_role("Spokesperson")
///     .with_email("jane.doe@transit.example.gov");
///
/// assert_eq!(contact.name, "Jane Doe");
/// assert_eq!(contact.organization.as_deref(), Some("City Transit Authority"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Contact {
    /// Unique identifier for the contact.
    pub id: Uuid,
    /// Full name of the contact (required).
    pub name: String,
    /// Organization, company, or institution the contact is affiliated with.
    pub organization: Option<String>,
    /// Job title, role, or relationship (e.g. "Spokesperson", "Lead Investigator").
    pub role: Option<String>,
    /// Phone number.
    pub phone: Option<String>,
    /// Email address.
    pub email: Option<String>,
    /// Background notes, secure contact details, or beats.
    pub notes: Option<String>,
    /// Timestamp when the contact record was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the contact record was last modified.
    pub updated_at: DateTime<Utc>,
}

impl Contact {
    /// Creates a new `Contact` with required name and default timestamps.
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            organization: None,
            role: None,
            phone: None,
            email: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Initializes a fluent builder for constructing a `Contact`.
    pub fn builder(name: impl Into<String>) -> ContactBuilder {
        ContactBuilder::new(name)
    }

    /// Sets the organization.
    pub fn with_organization(mut self, org: impl Into<String>) -> Self {
        self.organization = Some(org.into());
        self
    }

    /// Sets the role or title.
    pub fn with_role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Sets the phone number.
    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    /// Sets the email address.
    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets contact notes.
    pub fn with_notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Updates the `updated_at` timestamp to current UTC time.
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Validates all field invariants of the contact.
    ///
    /// Checks:
    /// - Name is non-empty and within length limits.
    /// - Email (if present) is a valid email format.
    /// - Phone (if present) is a valid phone format.
    pub fn validate(&self) -> Result<(), ValidationError> {
        validate_name(&self.name)?;
        if let Some(ref email) = self.email {
            validate_email(email)?;
        }
        if let Some(ref phone) = self.phone {
            validate_phone(phone)?;
        }
        Ok(())
    }
}

/// Fluent builder for creating a `Contact`.
#[derive(Debug, Clone)]
pub struct ContactBuilder {
    id: Option<Uuid>,
    name: String,
    organization: Option<String>,
    role: Option<String>,
    phone: Option<String>,
    email: Option<String>,
    notes: Option<String>,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
}

impl ContactBuilder {
    /// Creates a new builder with required contact `name`.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: None,
            name: name.into(),
            organization: None,
            role: None,
            phone: None,
            email: None,
            notes: None,
            created_at: None,
            updated_at: None,
        }
    }

    /// Overrides the generated UUID.
    pub fn id(mut self, id: Uuid) -> Self {
        self.id = Some(id);
        self
    }

    /// Sets organization.
    pub fn organization(mut self, organization: impl Into<String>) -> Self {
        self.organization = Some(organization.into());
        self
    }

    /// Sets role.
    pub fn role(mut self, role: impl Into<String>) -> Self {
        self.role = Some(role.into());
        self
    }

    /// Sets phone number.
    pub fn phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    /// Sets email address.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Sets background notes.
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self
    }

    /// Overrides created_at timestamp.
    pub fn created_at(mut self, created_at: DateTime<Utc>) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Overrides updated_at timestamp.
    pub fn updated_at(mut self, updated_at: DateTime<Utc>) -> Self {
        self.updated_at = Some(updated_at);
        self
    }

    /// Builds the `Contact` entity without validating invariants.
    #[must_use]
    pub fn build(self) -> Contact {
        let now = Utc::now();
        Contact {
            id: self.id.unwrap_or_else(Uuid::new_v4),
            name: self.name,
            organization: self.organization,
            role: self.role,
            phone: self.phone,
            email: self.email,
            notes: self.notes,
            created_at: self.created_at.unwrap_or(now),
            updated_at: self.updated_at.unwrap_or(now),
        }
    }

    /// Builds and validates the `Contact` entity.
    pub fn build_validated(self) -> Result<Contact, ValidationError> {
        let contact = self.build();
        contact.validate()?;
        Ok(contact)
    }
}
