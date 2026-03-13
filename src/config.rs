use anyhow::Result;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    /// Local development (no Lambda, localhost frontend)
    Local,
    /// Dev Lambda deployment (dev.dofus-graal.eu frontend)
    Dev,
    /// Production Lambda deployment (dofus-graal.eu frontend)
    Production,
}

impl Environment {
    /// Détecte automatiquement l'environnement.
    /// Local   → pas de Lambda
    /// Dev     → Lambda + APP_ENV=dev
    /// Prod    → Lambda + APP_ENV absent/autre
    pub fn detect() -> Self {
        if env::var("AWS_LAMBDA_FUNCTION_NAME").is_err() {
            return Self::Local;
        }

        match env::var("APP_ENV").as_deref() {
            Ok("dev" | "development") => Self::Dev,
            _ => Self::Production,
        }
    }

    pub fn is_local(&self) -> bool {
        matches!(self, Self::Local)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Local => "local",
            Self::Dev => "dev",
            Self::Production => "production",
        }
    }

    /// Retourne les origins CORS autorisées pour cet environnement.
    pub fn cors_origins(&self) -> &'static [&'static str] {
        match self {
            Self::Local => &[
                "http://localhost:8080",
                "http://127.0.0.1:8080",
                "http://0.0.0.0:8080",
            ],
            Self::Dev => &["https://dev.dofus-graal.eu", "http://127.0.0.1:8080"],
            Self::Production => &["https://dofus-graal.eu"],
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub environment: Environment,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    pub server_host: String,
    pub server_port: u16,
}

impl Config {
    /// Charge la configuration depuis les variables d'environnement
    /// avec détection automatique de l'environnement
    pub fn from_env() -> Result<Self> {
        let environment = Environment::detect();

        tracing::info!(
            "🌍 Environment detected: {}",
            environment.as_str().to_uppercase()
        );

        // Charger le fichier .env approprié
        Self::load_env_file(&environment);

        // Récupérer les variables avec fallbacks intelligents
        let database_url = Self::get_database_url(&environment)?;
        let jwt_secret = Self::get_jwt_secret(&environment)?;
        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<i64>()
            .unwrap_or(1);
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        tracing::info!("✅ Configuration loaded successfully");
        tracing::debug!("   Database: {}", Self::mask_credentials(&database_url));
        tracing::debug!("   CORS origins: {:?}", environment.cors_origins());
        tracing::debug!("   Server: {}:{}", server_host, server_port);

        Ok(Self {
            environment,
            database_url,
            jwt_secret,
            jwt_expiration_hours,
            server_host,
            server_port,
        })
    }

    /// Charge le bon fichier .env selon l'environnement
    fn load_env_file(environment: &Environment) {
        // Sur Lambda (Dev ou Production), les variables sont injectées par AWS
        if !environment.is_local() {
            tracing::info!("📦 Lambda mode: using injected environment variables");
            return;
        }

        // En local, charger .env
        tracing::info!("📦 Local mode: loading .env file");

        // Essayer de charger .env (optionnel)
        if let Ok(path) = env::current_dir() {
            let env_path = path.join(".env");
            if env_path.exists() {
                tracing::debug!("   Loading: {}", env_path.display());
                // Note: On ne peut pas utiliser dotenvy sans l'ajouter aux dépendances
                // Les variables doivent être chargées via docker-compose ou export
            } else {
                tracing::warn!("   .env file not found, using environment variables");
            }
        }
    }

    /// Récupère `DATABASE_URL` avec logique intelligente
    fn get_database_url(environment: &Environment) -> Result<String> {
        // Essayer DATABASE_URL directement (fonctionne dans tous les cas)
        if let Ok(url) = env::var("DATABASE_URL") {
            return Ok(url);
        }

        // Sur Lambda, DATABASE_URL est obligatoire
        if !environment.is_local() {
            anyhow::bail!(
                "DATABASE_URL must be set on Lambda! \
                 Configure it in Lambda environment variables."
            );
        }

        // En local, construire l'URL depuis les composants
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
        let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let database = env::var("POSTGRES_DB").unwrap_or_else(|_| "auth_db".to_string());

        Ok(format!(
            "postgres://{user}:{password}@{host}:{port}/{database}"
        ))
    }

    /// Récupère `JWT_SECRET` avec validation
    fn get_jwt_secret(environment: &Environment) -> Result<String> {
        let secret = match env::var("JWT_SECRET") {
            Ok(s) => s,
            Err(_) if !environment.is_local() => {
                tracing::error!("❌ JWT_SECRET not set on Lambda!");
                anyhow::bail!("JWT_SECRET is required on Lambda");
            }
            Err(_) => {
                tracing::warn!("⚠️  JWT_SECRET not set, using default (LOCAL ONLY!)");
                "dev_secret_key_change_in_production".to_string()
            }
        };

        // Valider la longueur du secret hors local
        if !environment.is_local() && secret.len() < 32 {
            anyhow::bail!(
                "JWT_SECRET must be at least 32 characters on Lambda (current: {})",
                secret.len()
            );
        }

        Ok(secret)
    }

    /// Masque les credentials dans les logs
    fn mask_credentials(url: &str) -> String {
        if let Some(at_pos) = url.find('@')
            && let Some(scheme_end) = url.find("://")
        {
            let scheme = &url[..scheme_end + 3];
            let after_at = &url[at_pos..];
            return format!("{scheme}***:***{after_at}");
        }
        url.to_string()
    }

    pub fn is_local(&self) -> bool {
        self.environment.is_local()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Sérialise les tests qui modifient des variables d'environnement globales
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn environment_detects_production_for_lambda_without_app_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            env::set_var("AWS_LAMBDA_FUNCTION_NAME", "test-function");
            env::remove_var("APP_ENV");
        }
        assert_eq!(Environment::detect(), Environment::Production);
        unsafe {
            env::remove_var("AWS_LAMBDA_FUNCTION_NAME");
        }
    }

    #[test]
    fn environment_detects_dev_for_lambda_with_dev_app_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            env::set_var("AWS_LAMBDA_FUNCTION_NAME", "test-function");
            env::set_var("APP_ENV", "dev");
        }
        assert_eq!(Environment::detect(), Environment::Dev);
        unsafe {
            env::remove_var("AWS_LAMBDA_FUNCTION_NAME");
            env::remove_var("APP_ENV");
        }
    }

    #[test]
    fn environment_defaults_to_local_without_lambda() {
        let _lock = ENV_LOCK.lock().unwrap();
        unsafe {
            env::remove_var("AWS_LAMBDA_FUNCTION_NAME");
            env::remove_var("APP_ENV");
        }
        assert_eq!(Environment::detect(), Environment::Local);
    }

    #[test]
    fn mask_credentials_hides_password_in_url() {
        let url = "postgres://user:password@localhost:5432/db";
        let masked = Config::mask_credentials(url);
        assert_eq!(masked, "postgres://***:***@localhost:5432/db");
    }
}
