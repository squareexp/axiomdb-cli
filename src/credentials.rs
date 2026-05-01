use anyhow::{bail, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Credentials {
    #[allow(dead_code)]
    pub project_id: String,
    pub branch_id: Option<String>,
    pub branch_name: Option<String>,
    pub database: String,
    #[allow(dead_code)]
    pub runtime_key: String,
    #[allow(dead_code)]
    pub direct_key: String,
    pub database_url: String,
    pub direct_url: String,
}

pub fn format_prisma_env_block(database_url: &str, direct_url: &str) -> String {
    format!("DATABASE_URL=\"{database_url}\"\nDIRECT_URL=\"{direct_url}\"")
}

pub fn ensure_prisma_contract(creds: &Credentials) -> Result<()> {
    ensure_url(&creds.database_url, "DATABASE_URL", 6432)?;
    ensure_url(&creds.direct_url, "DIRECT_URL", 5432)?;
    Ok(())
}

fn ensure_url(url: &str, label: &str, port: u16) -> Result<()> {
    let host_port = format!("@db.squareexp.com:{port}/");
    let has_sslmode = url.contains("?sslmode=require") || url.contains("&sslmode=require");
    if !url.starts_with("postgresql://") || !url.contains(&host_port) || !has_sslmode {
        bail!(
            "{label} from gateway is not Prisma-ready; expected db.squareexp.com:{port} with sslmode=require"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{ensure_prisma_contract, format_prisma_env_block, Credentials};

    fn fixture() -> Credentials {
        Credentials {
            project_id: "project-1".to_string(),
            branch_id: Some("branch-1".to_string()),
            branch_name: Some("feature-proof".to_string()),
            database: "sq_admin4_dev_br_feature-proof".to_string(),
            runtime_key: "DATABASE_URL_ADMIN4_DEV_BR_FEATURE_PROOF".to_string(),
            direct_key: "DIRECT_URL_ADMIN4_DEV_BR_FEATURE_PROOF".to_string(),
            database_url: "postgresql://app:secret@db.squareexp.com:6432/sq_admin4_dev_br_feature-proof?sslmode=require".to_string(),
            direct_url: "postgresql://owner:secret@db.squareexp.com:5432/sq_admin4_dev_br_feature-proof?sslmode=require".to_string(),
        }
    }

    #[test]
    fn formats_copy_paste_prisma_block() {
        let creds = fixture();
        assert_eq!(
            format_prisma_env_block(&creds.database_url, &creds.direct_url),
            "DATABASE_URL=\"postgresql://app:secret@db.squareexp.com:6432/sq_admin4_dev_br_feature-proof?sslmode=require\"\nDIRECT_URL=\"postgresql://owner:secret@db.squareexp.com:5432/sq_admin4_dev_br_feature-proof?sslmode=require\""
        );
    }

    #[test]
    fn validates_prisma_contract() {
        assert!(ensure_prisma_contract(&fixture()).is_ok());
    }
}
