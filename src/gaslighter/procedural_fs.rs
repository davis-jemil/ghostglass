use crate::logger::SharedLogger;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

const HONEYTOKEN_PATH_PATTERNS: &[&str] = &["secrets", "certs", "private"];

pub struct FakeEntry {
    pub name: String,
    pub size: u64,
}

const DIR_SIZE: u64 = 4096;

const EXTENSIONS: &[&str] = &["rs", "cfg", "log", "json", "db", "sh", "env", "key"];

const FILE_STEMS: &[&str] = &[
    "auth_service",
    "db_config",
    "session_store",
    "deploy",
    "backup_2024",
    "user_manager",
    "api_gateway",
    "cache_layer",
    "metrics_collector",
    "worker_pool",
    "request_router",
    "token_validator",
    "rate_limiter",
    "audit_log",
    "schema_migration",
    "connection_pool",
    "health_check",
    "load_balancer",
    "queue_consumer",
    "event_dispatcher",
    "config_loader",
    "billing_service",
    "notification_hub",
    "build_artifact",
    "webhook_handler",
    "cron_runner",
    "feature_flags",
    "search_indexer",
];

const SUBDIR_NAMES: &[&str] = &[
    "config", "logs", "scripts", "backups", "src", "data", "tmp", "secrets", "keys", "deploy",
    "bin", "migrations", "certs", "vendor",
];

const JUICY_FILES: &[&str] = &[
    ".env",
    ".env.production",
    ".env.local",
    "id_rsa",
    "id_rsa.pub",
    "private_key.key",
    "master.key",
    "credentials.json",
    "aws_credentials",
    "secrets.env",
    "shadow_backup.db",
    "deploy_key.key",
];

fn seed_from_path(path: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    path.hash(&mut hasher);
    hasher.finish()
}

fn realistic_size(rng: &mut StdRng) -> u64 {
    rng.gen_range(198u64..=2_847_331)
}

fn random_filename(rng: &mut StdRng, used: &mut HashSet<String>) -> String {
    loop {
        let stem = FILE_STEMS[rng.gen_range(0..FILE_STEMS.len())];
        let ext = EXTENSIONS[rng.gen_range(0..EXTENSIONS.len())];
        let name = format!("{stem}.{ext}");
        if used.insert(name.clone()) {
            return name;
        }
    }
}

fn push_files(rng: &mut StdRng, prefix: &str, count: usize, out: &mut Vec<FakeEntry>) {
    let mut used = HashSet::new();
    for _ in 0..count {
        let name = random_filename(rng, &mut used);
        out.push(FakeEntry {
            name: format!("{prefix}/{name}"),
            size: realistic_size(rng),
        });
    }
}

fn push_juicy(rng: &mut StdRng, prefix: &str, out: &mut Vec<FakeEntry>) {
    let juicy_count = rng.gen_range(0..=2usize);
    let mut used = HashSet::new();
    for _ in 0..juicy_count {
        let mut name = JUICY_FILES[rng.gen_range(0..JUICY_FILES.len())];
        while !used.insert(name) {
            name = JUICY_FILES[rng.gen_range(0..JUICY_FILES.len())];
        }
        out.push(FakeEntry {
            name: format!("{prefix}/{name}"),
            size: realistic_size(rng),
        });
    }
}

/// Procedurally generates a believable fake directory listing for any input path.
/// Generation is seeded from the path itself, so repeated listings of the same
/// path stay consistent. Each call surfaces two levels of nesting; descending
/// into one of the returned subdirectories and listing it again seeds a fresh,
/// equally consistent tree underneath it — the "infinite hallway" never ends.
pub fn list_directory(path: &str, logger: &SharedLogger) -> Vec<FakeEntry> {
    if HONEYTOKEN_PATH_PATTERNS.iter().any(|p| path.contains(p)) {
        if let Ok(mut log) = logger.lock() {
            log.log_honeytoken(path);
        }
    }

    let mut rng = StdRng::seed_from_u64(seed_from_path(path));
    let clean = path.trim_end_matches('/');
    let mut entries = Vec::new();

    let file_count = rng.gen_range(5..=9usize);
    push_files(&mut rng, clean, file_count, &mut entries);
    push_juicy(&mut rng, clean, &mut entries);

    let mut used_dirs = HashSet::new();
    let subdir_count = rng.gen_range(2..=3usize);
    for _ in 0..subdir_count {
        let mut dir = SUBDIR_NAMES[rng.gen_range(0..SUBDIR_NAMES.len())];
        while !used_dirs.insert(dir) {
            dir = SUBDIR_NAMES[rng.gen_range(0..SUBDIR_NAMES.len())];
        }
        let sub_prefix = format!("{clean}/{dir}");
        entries.push(FakeEntry {
            name: format!("{sub_prefix}/"),
            size: DIR_SIZE,
        });

        let sub_file_count = rng.gen_range(3..=6usize);
        push_files(&mut rng, &sub_prefix, sub_file_count, &mut entries);
        push_juicy(&mut rng, &sub_prefix, &mut entries);
    }

    entries
}
