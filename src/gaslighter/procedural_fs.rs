use rand::Rng;

pub struct FakeEntry {
    pub name: String,
    pub size: u64,
}

pub fn list_directory(path: &str) -> Vec<FakeEntry> {
    let mut rng = rand::thread_rng();
    let count = rng.gen_range(5..=15usize);

    let stems = ["config", "data", "log", "cache", "blob", "index", "meta", "shard", "tmp", "manifest"];
    let exts  = ["rs", "toml", "json", "bin", "log", "dat", "cfg", "idx"];

    (0..count)
        .map(|_| {
            let stem = stems[rng.gen_range(0..stems.len())];
            let ext  = exts[rng.gen_range(0..exts.len())];
            let id   = rng.gen_range(1000u32..9999);
            FakeEntry {
                name: format!("{path}/{stem}_{id}.{ext}"),
                size: rng.gen_range(128u64..=10_485_760),
            }
        })
        .collect()
}
