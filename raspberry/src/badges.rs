use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use log::{debug, info};
use rusqlite::{params, Connection, OptionalExtension};

#[derive(Clone, Debug)]
pub struct Badge {
    pub id: i32,
    pub uid: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub enabled: bool,
}

fn parse_datetime(s: &str) -> anyhow::Result<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Utc))
        .map_err(|e| anyhow!("Failed to parse datetime: {}", e))
}

pub struct BadgeManager {
    conn: Connection,
}

impl BadgeManager {
    /// Create a new BadgeManager and initialize the database
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        let manager = BadgeManager { conn };
        manager.init_db()?;
        Ok(manager)
    }

    /// Initialize the database schema
    fn init_db(&self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS badges (
                id INTEGER PRIMARY KEY,
                uid TEXT NOT NULL UNIQUE,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                last_used TEXT,
                expires_at TEXT,
                enabled INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )?;
        debug!("Badge database initialized");
        Ok(())
    }

    /// Add a new badge with optional expiry date (for temporary badges)
    pub fn add_badge_with_expiry(&self, uid: &str, name: &str, expires_at: Option<DateTime<Utc>>) -> Result<Badge> {
        if uid.is_empty() {
            return Err(anyhow!("Badge UID cannot be empty"));
        }
        if name.is_empty() {
            return Err(anyhow!("Badge name cannot be empty"));
        }

        let now = Utc::now();
        let created_at = now.to_rfc3339();
        let expires_at_str = expires_at.map(|dt| dt.to_rfc3339());

        self.conn.execute(
            "INSERT INTO badges (uid, name, created_at, expires_at, enabled) VALUES (?, ?, ?, ?, 1)",
            params![uid, name, created_at, expires_at_str],
        )?;

        let badge_type = if expires_at.is_some() { "temporary" } else { "permanent" };
        info!("Added {} badge: {} ({})", badge_type, name, uid);

        Ok(Badge {
            id: self.conn.last_insert_rowid() as i32,
            uid: uid.to_string(),
            name: name.to_string(),
            created_at: now,
            last_used: None,
            expires_at,
            enabled: true,
        })
    }

    /// Remove a badge from the database
    pub fn remove_badge(&self, uid: &str) -> Result<()> {
        let rows = self.conn.execute(
            "DELETE FROM badges WHERE uid = ?",
            params![uid],
        )?;

        if rows == 0 {
            return Err(anyhow!("Badge with UID '{}' not found", uid));
        }

        info!("Removed badge with UID: {}", uid);
        Ok(())
    }

    /// Get a badge by UID
    pub fn get_badge(&self, uid: &str) -> Result<Option<Badge>> {
        let result = self.conn
            .query_row(
                "SELECT id, uid, name, created_at, last_used, expires_at, enabled FROM badges WHERE uid = ?",
                params![uid],
                |row| {
                    let created_at_str: String = row.get(3)?;
                    let created_at = parse_datetime(&created_at_str)
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
                    
                    let last_used = row.get::<_, Option<String>>(4)?
                        .map(|s| parse_datetime(&s))
                        .transpose()
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
                    
                    let expires_at = row.get::<_, Option<String>>(5)?
                        .map(|s| parse_datetime(&s))
                        .transpose()
                        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
                    
                    Ok(Badge {
                        id: row.get(0)?,
                        uid: row.get(1)?,
                        name: row.get(2)?,
                        created_at,
                        last_used,
                        expires_at,
                        enabled: row.get::<_, i32>(6)? != 0,
                    })
                },
            )
            .optional()?;

        Ok(result)
    }

    /// Check if a badge UID is valid, enabled, and not expired
    pub fn is_valid_badge(&self, uid: &str) -> Result<bool> {
        match self.get_badge(uid)? {
            Some(badge) => {
                if !badge.enabled {
                    return Ok(false);
                }
                // Check if badge has expired
                if let Some(expires_at) = badge.expires_at {
                    Ok(Utc::now() <= expires_at)
                } else {
                    Ok(true)
                }
            },
            None => Ok(false),
        }
    }

    /// Get all badges
    pub fn get_all_badges(&self) -> Result<Vec<Badge>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, uid, name, created_at, last_used, expires_at, enabled FROM badges ORDER BY created_at DESC"
        )?;

        let badges = stmt.query_map([], |row| {
            let created_at_str: String = row.get(3)?;
            let created_at = parse_datetime(&created_at_str)
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            
            let last_used = row.get::<_, Option<String>>(4)?
                .map(|s| parse_datetime(&s))
                .transpose()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            
            let expires_at = row.get::<_, Option<String>>(5)?
                .map(|s| parse_datetime(&s))
                .transpose()
                .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;
            
            Ok(Badge {
                id: row.get(0)?,
                uid: row.get(1)?,
                name: row.get(2)?,
                created_at,
                last_used,
                expires_at,
                enabled: row.get::<_, i32>(6)? != 0,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(badges)
    }

    /// Enable a badge
    pub fn enable_badge(&self, uid: &str) -> Result<()> {
        let rows = self.conn.execute(
            "UPDATE badges SET enabled = 1 WHERE uid = ?",
            params![uid],
        )?;

        if rows == 0 {
            return Err(anyhow!("Badge with UID '{}' not found", uid));
        }

        info!("Enabled badge: {}", uid);
        Ok(())
    }

    /// Disable a badge
    pub fn disable_badge(&self, uid: &str) -> Result<()> {
        let rows = self.conn.execute(
            "UPDATE badges SET enabled = 0 WHERE uid = ?",
            params![uid],
        )?;

        if rows == 0 {
            return Err(anyhow!("Badge with UID '{}' not found", uid));
        }

        info!("Disabled badge: {}", uid);
        Ok(())
    }

    /// Update the last_used timestamp for a badge
    pub fn update_last_used(&self, uid: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE badges SET last_used = ? WHERE uid = ?",
            params![now, uid],
        )?;
        Ok(())
    }

    /// Clean up expired temporary badges from database
    pub fn cleanup_expired_badges(&self) -> Result<usize> {
        let rows = self.conn.execute(
            "DELETE FROM badges WHERE expires_at IS NOT NULL AND expires_at < ?",
            params![Utc::now().to_rfc3339()],
        )?;
        if rows > 0 {
            info!("Cleaned up {} expired temporary badges", rows);
        }
        Ok(rows)
    }
}
