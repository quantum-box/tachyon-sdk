use super::*;

// --- Pages app target ---

#[derive(Debug, Clone, ValueEnum)]
pub enum PagesApp {
    /// Main Tachyon app (@cloudflare/next-on-pages)
    Tachyon,
    /// CMS app (@opennextjs/cloudflare)
    Cms,
    /// Documentation site (@cloudflare/next-on-pages)
    Docs,
}

impl PagesApp {
    fn name(&self) -> &str {
        match self {
            PagesApp::Tachyon => "tachyon",
            PagesApp::Cms => "cms",
            PagesApp::Docs => "docs",
        }
    }

    pub(crate) fn cf_project_name(&self) -> &str {
        match self {
            PagesApp::Tachyon => "tachyon-app",
            PagesApp::Cms => "tachyon-apps-cms-app",
            PagesApp::Docs => "tachyon-docs",
        }
    }

    /// Returns the pages build command and output directory.
    fn pages_build_info(&self) -> (&str, &str) {
        match self {
            PagesApp::Cms => ("npx opennextjs-cloudflare build", ".open-next/assets"),
            PagesApp::Tachyon | PagesApp::Docs => {
                ("npx @cloudflare/next-on-pages", ".vercel/output/static")
            }
        }
    }

    fn preview_command(&self, port: u16) -> String {
        match self {
            PagesApp::Cms => {
                format!("npx opennextjs-cloudflare preview --port {port}")
            }
            PagesApp::Tachyon | PagesApp::Docs => {
                format!("npx wrangler pages dev .vercel/output/static --port {port}")
            }
        }
    }
}

// --- Local build pipeline ---

fn run_shell(description: &str, cmd: &str, cwd: &std::path::Path) -> Result<()> {
    println!("  > {cmd}");
    let status = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .current_dir(cwd)
        .status()?;
    if !status.success() {
        return Err(anyhow!(
            "{description} failed with exit code: {:?}",
            status.code()
        ));
    }
    Ok(())
}

fn resolve_project_dir(project_dir: Option<&PathBuf>) -> Result<PathBuf> {
    match project_dir {
        Some(p) => {
            let abs = std::fs::canonicalize(p)?;
            Ok(abs)
        }
        None => Ok(std::env::current_dir()?),
    }
}

pub(super) fn run_local_build(
    app: &PagesApp,
    project_dir: Option<&PathBuf>,
    deploy: bool,
) -> Result<()> {
    let root = resolve_project_dir(project_dir)?;
    let app_dir = root.join("apps").join(app.name());
    if !app_dir.exists() {
        return Err(anyhow!(
            "App directory not found: {}. \
             Make sure you're in the tachyon-apps repository root \
             or specify --project-dir.",
            app_dir.display()
        ));
    }
    let (pages_build_cmd, output_dir) = app.pages_build_info();

    println!("=== Cloudflare Pages Build Pipeline ===");
    println!("  App:     {}", app.name());
    println!("  Root:    {}", root.display());
    println!("  Deploy:  {deploy}");
    println!();

    // Step 1: Install dependencies
    println!("[1/4] Installing dependencies...");
    run_shell("yarn install", "yarn install", &root)?;
    println!();

    // Step 2: Next.js build via turbo
    println!("[2/4] Building {} (turbo)...", app.name());
    run_shell(
        "turbo build",
        &format!("npx turbo run build --filter={}", app.name()),
        &root,
    )?;
    println!();

    // Step 3: Pages build
    println!("[3/4] Building for Cloudflare Pages...");
    run_shell("pages build", pages_build_cmd, &app_dir)?;
    println!();

    // Step 4: Deploy or finish
    if deploy {
        println!("[4/4] Deploying to Cloudflare Pages...");
        let deploy_cmd = match app {
            PagesApp::Cms => "npx opennextjs-cloudflare deploy".to_string(),
            _ => format!(
                "npx wrangler pages deploy {output_dir} \
                 --project-name {}",
                app.cf_project_name()
            ),
        };
        run_shell("pages deploy", &deploy_cmd, &app_dir)?;
    } else {
        println!("[4/4] Build complete.");
        println!("  Output: {}/{output_dir}", app_dir.display());
        println!();
        println!("  To preview: tachyon compute dev {}", app.name());
        println!(
            "  To deploy:  tachyon compute build {} --deploy",
            app.name()
        );
    }

    println!();
    println!("=== Done ===");
    Ok(())
}

pub(super) fn run_local_dev(
    app: &PagesApp,
    project_dir: Option<&PathBuf>,
    port: u16,
) -> Result<()> {
    let root = resolve_project_dir(project_dir)?;
    let app_dir = root.join("apps").join(app.name());
    if !app_dir.exists() {
        return Err(anyhow!(
            "App directory not found: {}. \
             Make sure you're in the tachyon-apps repository root \
             or specify --project-dir.",
            app_dir.display()
        ));
    }
    let (pages_build_cmd, _) = app.pages_build_info();

    println!("=== Cloudflare Pages Local Preview ===");
    println!("  App:  {}", app.name());
    println!("  Port: {port}");
    println!();

    // Step 1: Install
    println!("[1/3] Installing dependencies...");
    run_shell("yarn install", "yarn install", &root)?;
    println!();

    // Step 2: Build
    println!("[2/3] Building {} ...", app.name());
    run_shell(
        "turbo build",
        &format!("npx turbo run build --filter={}", app.name()),
        &root,
    )?;
    run_shell("pages build", pages_build_cmd, &app_dir)?;
    println!();

    // Step 3: Preview server
    println!("[3/3] Starting preview server on port {port}...");
    let preview_cmd = app.preview_command(port);
    run_shell("preview server", &preview_cmd, &app_dir)?;

    Ok(())
}
