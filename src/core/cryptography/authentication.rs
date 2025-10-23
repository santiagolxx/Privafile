use chrono::{Duration, Utc};
use rusty_paseto::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: i64,
    pub iat: i64,
}

pub struct PasetoManager {
    key: PasetoSymmetricKey<V4, Local>,
}

impl PasetoManager {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.as_ref();

        if path.exists() {
            let key_bytes = fs::read(path)?;
            if key_bytes.len() != 32 {
                return Err("Invalid key length: must be 32 bytes".into());
            }

            let mut key_array = [0u8; 32];
            key_array.copy_from_slice(&key_bytes);
            let key = PasetoSymmetricKey::<V4, Local>::from(Key::from(&key_array));

            Ok(Self { key })
        } else {
            let key = Key::<32>::try_new_random()?;

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }

            fs::write(path, key.as_ref())?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
            }

            Ok(Self {
                key: PasetoSymmetricKey::<V4, Local>::from(key),
            })
        }
    }

    pub fn create_token(
        &self,
        user_id: &str,
        expires_in_hours: i64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let now = Utc::now();
        let exp = now + Duration::hours(expires_in_hours);

        let token = PasetoBuilder::<V4, Local>::default()
            .set_claim(SubjectClaim::from(user_id))
            .set_claim(ExpirationClaim::try_from(exp.to_rfc3339())?)
            .set_claim(IssuedAtClaim::try_from(now.to_rfc3339())?)
            .build(&self.key)?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, Box<dyn std::error::Error>> {
        let verified_token = PasetoParser::<V4, Local>::default().parse(token, &self.key)?;

        // Convertir las fechas RFC3339 a timestamps
        let exp_str = verified_token["exp"]
            .as_str()
            .ok_or("Missing 'exp' claim")?;
        let exp_ts = chrono::DateTime::parse_from_rfc3339(exp_str)?.timestamp();

        let iat_str = verified_token["iat"]
            .as_str()
            .ok_or("Missing 'iat' claim")?;
        let iat_ts = chrono::DateTime::parse_from_rfc3339(iat_str)?.timestamp();

        let claims = Claims {
            sub: verified_token["sub"]
                .as_str()
                .ok_or("Missing 'sub' claim")?
                .to_string(),
            exp: exp_ts,
            iat: iat_ts,
        };

        let now = chrono::Utc::now().timestamp();
        if claims.exp < now {
            return Err("Token expired".into());
        }

        Ok(claims)
    }
}
