use anyhow::Result;
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Production,
}

impl Environment {
    /// Détecte automatiquement l'environnement
    pub fn detect() -> Self {
        // Méthode 1: Vérifier si on est dans AWS Lambda
        if env::var("AWS_LAMBDA_FUNCTION_NAME").is_ok() {
            return Self::Production;
        }

        // Méthode 2: Vérifier la variable APP_ENV
        match env::var("APP_ENV").as_deref() {
            Ok("production") | Ok("prod") => Self::Production,
            _ => Self::Development,
        }
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Self::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Self::Development)
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub environment: Environment,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: i64,
    #[expect(dead_code, reason = "CORS origin is consumed at startup in app.rs; field retained for completeness")]
    pub frontend_url: String,
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
        Self::load_env_file(&environment)?;

        // Récupérer les variables avec fallbacks intelligents
        let database_url = Self::get_database_url(&environment)?;
        let jwt_secret = Self::get_jwt_secret(&environment)?;
        let jwt_expiration_hours = env::var("JWT_EXPIRATION_HOURS")
            .unwrap_or_else(|_| "1".to_string())
            .parse::<i64>()
            .unwrap_or(1);
        let frontend_url = Self::get_frontend_url(&environment);
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .unwrap_or(3000);

        tracing::info!("✅ Configuration loaded successfully");
        tracing::debug!("   Database: {}", Self::mask_credentials(&database_url));
        tracing::debug!("   Frontend: {}", frontend_url);
        tracing::debug!("   Server: {}:{}", server_host, server_port);

        Ok(Self {
            environment,
            database_url,
            jwt_secret,
            jwt_expiration_hours,
            frontend_url,
            server_host,
            server_port,
        })
    }

    /// Charge le bon fichier .env selon l'environnement
    fn load_env_file(environment: &Environment) -> Result<()> {
        // En production (Lambda), les variables sont déjà injectées
        if environment.is_production() {
            tracing::info!("📦 Production mode: using injected environment variables");
            return Ok(());
        }

        // En développement, charger .env
        tracing::info!("📦 Development mode: loading .env file");

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

        Ok(())
    }

    /// Récupère DATABASE_URL avec logique intelligente
    fn get_database_url(environment: &Environment) -> Result<String> {
        // Essayer DATABASE_URL directement (fonctionne dans tous les cas)
        if let Ok(url) = env::var("DATABASE_URL") {
            return Ok(url);
        }

        // Si en prod et DATABASE_URL manque, erreur critique
        if environment.is_production() {
            anyhow::bail!(
                "DATABASE_URL must be set in production! \
                 Configure it in Lambda environment variables."
            );
        }

        // En dev, construire l'URL depuis les composants
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string());
        let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "postgres".to_string());
        let host = env::var("DB_HOST").unwrap_or_else(|_| "localhost".to_string());
        let port = env::var("DB_PORT").unwrap_or_else(|_| "5432".to_string());
        let database = env::var("POSTGRES_DB").unwrap_or_else(|_| "auth_db".to_string());

        Ok(format!(
            "postgres://{}:{}@{}:{}/{}",
            user, password, host, port, database
        ))
    }

    /// Récupère JWT_SECRET avec validation
    fn get_jwt_secret(environment: &Environment) -> Result<String> {
        let secret = match env::var("JWT_SECRET") {
            Ok(s) => s,
            Err(_) if environment.is_production() => {
                tracing::error!("❌ JWT_SECRET not set in production!");
                anyhow::bail!("JWT_SECRET is required in production");
            }
            Err(_) => {
                tracing::warn!("⚠️  JWT_SECRET not set, using default (DEVELOPMENT ONLY!)");
                "dev_secret_key_change_in_production".to_string()
            }
        };

        // Valider la longueur du secret en production
        if environment.is_production() && secret.len() < 32 {
            anyhow::bail!(
                "JWT_SECRET must be at least 32 characters in production (current: {})",
                secret.len()
            );
        }

        Ok(secret)
    }

    /// Récupère FRONTEND_URL avec fallback
    fn get_frontend_url(environment: &Environment) -> String {
        env::var("FRONTEND_URL").unwrap_or_else(|_| {
            if environment.is_production() {
                "https://dofus-graal.eu".to_string()
            } else {
                "http://localhost:8080".to_string()
            }
        })
    }

    /// Masque les credentials dans les logs
    fn mask_credentials(url: &str) -> String {
        if let Some(at_pos) = url.find('@')
            && let Some(scheme_end) = url.find("://")
        {
            let scheme = &url[..scheme_end + 3];
            let after_at = &url[at_pos..];
            return format!("{}***:***{}", scheme, after_at);
        }
        url.to_string()
    }

    /// Retourne true si on est en mode production
    pub fn is_production(&self) -> bool {
        self.environment.is_production()
    }

    /// Retourne true si on est en mode développement
    #[expect(dead_code, reason = "Available for conditional behavior in request handlers")]
    pub fn is_development(&self) -> bool {
        self.environment.is_development()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn environment_detects_production_for_lambda() {
        unsafe {
            env::set_var("AWS_LAMBDA_FUNCTION_NAME", "test-function");
        }
        assert_eq!(Environment::detect(), Environment::Production);
        unsafe {
            env::remove_var("AWS_LAMBDA_FUNCTION_NAME");
        }
    }

    #[test]
    fn environment_respects_app_env_variable() {
        unsafe {
            env::set_var("APP_ENV", "production");
        }
        assert_eq!(Environment::detect(), Environment::Production);
        unsafe {
            env::remove_var("APP_ENV");
        }

        unsafe {
            env::set_var("APP_ENV", "development");
        }
        assert_eq!(Environment::detect(), Environment::Development);
        unsafe {
            env::remove_var("APP_ENV");
        }
    }

    #[test]
    fn environment_defaults_to_development() {
        unsafe {
            env::remove_var("AWS_LAMBDA_FUNCTION_NAME");
            env::remove_var("APP_ENV");
        }
        assert_eq!(Environment::detect(), Environment::Development);
    }

    #[test]
    fn mask_credentials_hides_password_in_url() {
        let url = "postgres://user:password@localhost:5432/db";
        let masked = Config::mask_credentials(url);
        assert_eq!(masked, "postgres://***:***@localhost:5432/db");
    }
}
