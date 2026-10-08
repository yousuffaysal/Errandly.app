fn main() {
    // Pin Google sign-in to this build's own Supabase project: the host from
    // VITE_SUPABASE_URL (environment or ../.env.local) is compiled in, and the
    // app refuses to open sign-in pages anywhere else.
    println!("cargo:rerun-if-env-changed=VITE_SUPABASE_URL");
    println!("cargo:rerun-if-changed=../.env.local");
    let url = std::env::var("VITE_SUPABASE_URL").ok().or_else(|| {
        std::fs::read_to_string("../.env.local").ok()?.lines().find_map(|l| {
            l.trim().strip_prefix("VITE_SUPABASE_URL=").map(|v| v.trim().trim_matches('"').to_string())
        })
    });
    if let Some(host) = url.as_deref().and_then(|u| u.strip_prefix("https://")).map(|r| r.trim_end_matches('/')) {
        if !host.is_empty() {
            println!("cargo:rustc-env=ERRANDLY_SUPABASE_HOST={host}");
        }
    }
    tauri_build::build()
}
