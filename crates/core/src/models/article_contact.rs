//! Join model representing many-to-many relationships between articles and contacts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Join entity associating an `Article` with a `Contact`.
///
/// # Examples
///
/// ```
/// use uuid::Uuid;
/// use newsjournal_core::ArticleContact;
///
/// let article_id = Uuid::new_v4();
/// let contact_id = Uuid::new_v4();
/// let link = ArticleContact::new(article_id, contact_id);
///
/// assert_eq!(link.article_id, article_id);
/// assert_eq!(link.contact_id, contact_id);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ArticleContact {
    /// ID of the associated article.
    pub article_id: Uuid,
    /// ID of the associated contact.
    pub contact_id: Uuid,
    /// Timestamp when the contact was linked to the article.
    pub created_at: DateTime<Utc>,
}

impl ArticleContact {
    /// Creates a new `ArticleContact` linking the specified article and contact with the current UTC timestamp.
    #[must_use]
    pub fn new(article_id: Uuid, contact_id: Uuid) -> Self {
        Self {
            article_id,
            contact_id,
            created_at: Utc::now(),
        }
    }

    /// Creates an `ArticleContact` with a specific creation timestamp.
    #[must_use]
    pub const fn with_timestamp(
        article_id: Uuid,
        contact_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            article_id,
            contact_id,
            created_at,
        }
    }
}
